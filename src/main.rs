use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use rs_llama::{resolve_model_files, EngineConfig, GenerateRequest, HfDownload, LlamaEngine};

#[derive(Parser, Debug)]
#[command(author, version, about = "Run a GGUF model with llama.cpp from Rust")]
struct Args {
    /// Path to a local GGUF model.
    #[arg(short, long)]
    model: Option<PathBuf>,

    /// Hugging Face repository, for example: ggml-org/SmolVLM-Instruct-GGUF.
    #[arg(long)]
    hf_repo: Option<String>,

    /// GGUF file inside the Hugging Face repository.
    #[arg(long)]
    hf_file: Option<String>,

    /// Explicit mmproj filename inside the Hugging Face repository.
    #[arg(long)]
    hf_mmproj: Option<String>,

    /// Do not auto-download mmproj from the Hugging Face repo.
    #[arg(long, default_value_t = false)]
    no_mmproj: bool,

    /// Local multimodal projector GGUF (mmproj).
    #[arg(long)]
    mmproj: Option<PathBuf>,

    /// Optional image for vision models.
    #[arg(long)]
    image: Option<PathBuf>,

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
    // llama.cpp uses large stack frames. Windows defaults to 1MB and overflows.
    std::thread::Builder::new()
        .name("rs-llama".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(run)
        .context("failed to start inference thread")?
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

fn run() -> Result<()> {
    let args = Args::parse();

    let hf = match (&args.hf_repo, &args.hf_file) {
        (Some(repo), Some(file)) => Some(HfDownload {
            repo: repo.clone(),
            file: file.clone(),
            revision: args.hf_revision.clone(),
            model_dir: args.model_dir.clone(),
            force: args.hf_force_download,
            show_progress: true,
            mmproj_file: args.hf_mmproj.clone(),
            auto_mmproj: !args.no_mmproj,
        }),
        _ => None,
    };

    let resolved = resolve_model_files(args.model.as_deref(), hf.as_ref(), args.mmproj.as_deref())?;
    if let Some(mmproj) = &resolved.mmproj_path {
        eprintln!("Vision mmproj: {}", mmproj.display());
    }

    let mut config = EngineConfig::new(resolved.model_path)
        .with_ctx_size(args.ctx_size)
        .with_threads(args.threads)
        .with_gpu_layers(args.gpu_layers);
    if let Some(mmproj) = resolved.mmproj_path {
        config = config.with_mmproj(mmproj);
    }
    let engine = LlamaEngine::load(config)?;

    let mut request = GenerateRequest::new(&args.prompt).with_max_tokens(args.max_tokens);
    if let Some(image) = args.image {
        request = request.with_image(image);
    }

    print!("{}", args.prompt);
    io::stdout().flush()?;
    engine.generate_to_writer(&request, &mut io::stdout())?;
    println!();
    Ok(())
}
