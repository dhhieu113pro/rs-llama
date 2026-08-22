use std::io::{self, Write};
use std::num::NonZeroU32;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

#[derive(Parser, Debug)]
#[command(author, version, about = "Run a GGUF model with llama.cpp from Rust")]
struct Args {
    /// Path to a GGUF model.
    #[arg(short, long)]
    model: PathBuf,

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

    let backend = LlamaBackend::init().context("failed to initialize llama.cpp backend")?;

    let model_params = make_model_params(args.gpu_layers);
    let model = LlamaModel::load_from_file(&backend, &args.model, &model_params)
        .with_context(|| format!("failed to load model: {}", args.model.display()))?;

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
