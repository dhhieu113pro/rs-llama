use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;

#[derive(Parser, Debug)]
#[command(author, version, about = "Run a GGUF model with llama.cpp from Rust")]
struct Args {
    /// Path to a local GGUF model.
    #[arg(short, long)]
    model: Option<PathBuf>,

    /// Hugging Face repository, for example: mradermacher/SmolLM2-135M-Instruct-GGUF.
    #[arg(long)]
    hf_repo: Option<String>,

    /// GGUF file inside the Hugging Face repository.
    #[arg(long)]
    hf_file: Option<String>,

    /// Hugging Face revision/branch/tag.
    #[arg(long, default_value = "main")]
    hf_revision: String,

    /// Directory used to cache downloaded Hugging Face models.
    #[arg(long, default_value = "models")]
    model_dir: PathBuf,

    /// Download the Hugging Face model again even when it already exists locally.
    #[arg(long, default_value_t = false)]
    hf_force_download: bool,

    /// Prompt to generate from.
    #[arg(short, long, default_value = "Hello from Rust!")]
    prompt: String,

    /// Maximum number of new tokens to generate.
    #[arg(short = 'n', long, default_value_t = 128)]
    max_tokens: i32,

    /// Context size.
    #[arg(short = 'c', long, default_value_t = 2048)]
    ctx_size: u32,

    /// Number of GPU layers to offload when built with cuda/vulkan/metal.
    #[arg(long, default_value_t = 0)]
    gpu_layers: u32,

    /// Number of CPU threads. 0 lets llama.cpp choose.
    #[arg(short = 't', long, default_value_t = 0)]
    threads: i32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let model_path = resolve_model_path(&args)?;

    let backend = LlamaBackend::init().context("failed to initialize llama.cpp backend")?;

    let model_params = make_model_params(args.gpu_layers);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
        .with_context(|| format!("failed to load model: {}", model_path.display()))?;

    let n_ctx = NonZeroU32::new(args.ctx_size).context("ctx-size must be greater than zero")?;
    let mut ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));

    if args.threads > 0 {
        ctx_params = ctx_params
            .with_n_threads(args.threads)
            .with_n_threads_batch(args.threads);
    }

    let mut ctx = model
        .new_context(&backend, ctx_params)
        .context("failed to create llama.cpp context")?;

    let prompt_tokens = model
        .str_to_token(&args.prompt, AddBos::Always)
        .context("failed to tokenize prompt")?;

    if prompt_tokens.is_empty() {
        bail!("prompt produced no tokens");
    }

    let max_total_tokens = prompt_tokens.len() as i32 + args.max_tokens;
    if max_total_tokens > ctx.n_ctx() as i32 {
        bail!(
            "prompt + max-tokens ({max_total_tokens}) exceeds context size ({})",
            ctx.n_ctx()
        );
    }

    let mut batch = LlamaBatch::new(512, 1);
    let last_prompt_index = prompt_tokens.len() - 1;

    for (i, token) in prompt_tokens.iter().copied().enumerate() {
        batch.add(token, i as i32, &[0], i == last_prompt_index)?;
    }

    ctx.decode(&mut batch).context("failed to decode prompt")?;

    print!("{}", args.prompt);
    io::stdout().flush()?;

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(0.8),
        LlamaSampler::dist(42),
    ]);

    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut position = prompt_tokens.len() as i32;

    for _ in 0..args.max_tokens {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if model.is_eog_token(token) {
            break;
        }

        let piece = model.token_to_piece(token, &mut decoder, true, None)?;
        print!("{piece}");
        io::stdout().flush()?;

        batch.clear();
        batch.add(token, position, &[0], true)?;
        ctx.decode(&mut batch).context("failed to decode generated token")?;
        position += 1;
    }

    println!();
    Ok(())
}

fn resolve_model_path(args: &Args) -> Result<PathBuf> {
    match (&args.model, &args.hf_repo, &args.hf_file) {
        (Some(model), None, None) => {
            if !model.exists() {
                bail!("model does not exist: {}", model.display());
            }
            Ok(model.clone())
        }
        (None, Some(repo), Some(file)) => download_huggingface_model(
            repo,
            file,
            &args.hf_revision,
            &args.model_dir,
            args.hf_force_download,
        ),
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            bail!("use either --model or --hf-repo/--hf-file, not both")
        }
        (None, Some(_), None) => bail!("--hf-file is required when --hf-repo is used"),
        (None, None, Some(_)) => bail!("--hf-repo is required when --hf-file is used"),
        (None, None, None) => bail!("provide --model or --hf-repo with --hf-file"),
    }
}

fn download_huggingface_model(
    repo: &str,
    file: &str,
    revision: &str,
    model_dir: &Path,
    force: bool,
) -> Result<PathBuf> {
    let file_name = Path::new(file)
        .file_name()
        .context("invalid --hf-file path")?;

    fs::create_dir_all(model_dir)
        .with_context(|| format!("failed to create model directory: {}", model_dir.display()))?;

    let destination = model_dir.join(file_name);
    if destination.exists() && !force {
        eprintln!("Using cached model: {}", destination.display());
        return Ok(destination);
    }

    let url = format!(
        "https://huggingface.co/{repo}/resolve/{revision}/{file}?download=true"
    );

    eprintln!("Downloading {repo}/{file} ...");

    let client = Client::builder()
        .user_agent(concat!("llama-rust/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to create HTTP client")?;

    let mut request = client.get(&url);
    if let Some(token) = huggingface_token() {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    let mut response = request.send().context("Hugging Face download request failed")?;
    if !response.status().is_success() {
        bail!(
            "Hugging Face download failed with HTTP {} for {repo}/{file}",
            response.status()
        );
    }

    let total = response.content_length();
    let temporary = destination.with_extension("gguf.part");
    let mut output = File::create(&temporary)
        .with_context(|| format!("failed to create: {}", temporary.display()))?;

    let mut downloaded = 0_u64;
    let mut buffer = [0_u8; 1024 * 1024];

    loop {
        let read = response.read(&mut buffer).context("failed while downloading model")?;
        if read == 0 {
            break;
        }

        output
            .write_all(&buffer[..read])
            .context("failed while writing downloaded model")?;
        downloaded += read as u64;

        if let Some(total) = total {
            let percent = downloaded as f64 / total as f64 * 100.0;
            eprint!("\r{:.1}% ({}/{})", percent, format_bytes(downloaded), format_bytes(total));
        } else {
            eprint!("\r{} downloaded", format_bytes(downloaded));
        }
        io::stderr().flush()?;
    }

    output.flush()?;
    drop(output);
    eprintln!();

    if destination.exists() {
        fs::remove_file(&destination)
            .with_context(|| format!("failed to replace: {}", destination.display()))?;
    }
    fs::rename(&temporary, &destination).with_context(|| {
        format!(
            "failed to move downloaded model from {} to {}",
            temporary.display(),
            destination.display()
        )
    })?;

    eprintln!("Saved model: {}", destination.display());
    Ok(destination)
}

fn huggingface_token() -> Option<String> {
    env::var("HF_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("HUGGING_FACE_HUB_TOKEN")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * KIB;
    const GIB: f64 = 1024.0 * MIB;

    let bytes_f = bytes as f64;
    if bytes_f >= GIB {
        format!("{:.2} GiB", bytes_f / GIB)
    } else if bytes_f >= MIB {
        format!("{:.2} MiB", bytes_f / MIB)
    } else if bytes_f >= KIB {
        format!("{:.2} KiB", bytes_f / KIB)
    } else {
        format!("{bytes} B")
    }
}

fn make_model_params(gpu_layers: u32) -> LlamaModelParams {
    let params = LlamaModelParams::default();

    #[cfg(any(feature = "cuda", feature = "vulkan", feature = "metal"))]
    {
        if gpu_layers > 0 {
            return params.with_n_gpu_layers(gpu_layers);
        }
    }

    #[cfg(not(any(feature = "cuda", feature = "vulkan", feature = "metal")))]
    let _ = gpu_layers;

    params
}
