# llama-rust

A Rust **library and CLI** for running GGUF language models through `llama.cpp` using the maintained [`llama-cpp-2`](https://crates.io/crates/llama-cpp-2) bindings.

The application layer and orchestration are written in Rust, while `llama.cpp` provides the low-level model loading and inference backend.

## Features

- Reusable `LlamaEngine` library for other Rust projects
- Run local GGUF models
- Download GGUF models directly from Hugging Face
- Cache downloaded models locally
- Support Hugging Face branches, tags, and revisions
- Support private or gated Hugging Face repositories with a token
- CPU inference
- Optional CUDA, Vulkan, and Metal builds
- Configurable context size, CPU threads, GPU layers, and generation length
- GitHub Actions CI that builds on Ubuntu and smoke-tests real GGUF inference

## Use as a library

This crate is both a binary (`llama-rust`) and a library (`llama_rust`).

Add it from Git (not published to crates.io yet):

```toml
[dependencies]
llama-rust = { git = "https://github.com/dhhieu113pro/llama-rust" }
```

Optional GPU features:

```toml
llama-rust = { git = "https://github.com/dhhieu113pro/llama-rust", features = ["cuda"] }
```

Example:

```rust
use llama_rust::{download_huggingface_model, EngineConfig, GenerateRequest, HfDownload, LlamaEngine};

fn main() -> anyhow::Result<()> {
    let model_path = download_huggingface_model(&HfDownload::new(
        "mradermacher/SmolLM2-135M-Instruct-GGUF",
        "SmolLM2-135M-Instruct.Q4_K_M.gguf",
    ))?;

    let engine = LlamaEngine::load(
        EngineConfig::new(model_path)
            .with_ctx_size(512)
            .with_threads(2),
    )?;

    let text = engine.generate(
        &GenerateRequest::new("Write one sentence about Rust.").with_max_tokens(32),
    )?;
    println!("{text}");
    Ok(())
}
```

Public API:

- `LlamaEngine::load` / `generate` / `generate_to_writer` / `generate_with_callback`
- `EngineConfig` and `GenerateRequest`
- `HfDownload`, `download_huggingface_model`, `resolve_model_path`

The consuming project still needs a C/C++ compiler, CMake, and Clang/libclang because `llama.cpp` is compiled as part of the build.

## Requirements

- Rust toolchain (`rustup`, `cargo`, `rustc`)
- C/C++ compiler
- CMake
- Clang/libclang for `bindgen`

## Build

### CPU

```bash
cargo build --release
```

### NVIDIA CUDA

```bash
cargo build --release --features cuda
```

### Vulkan

```bash
cargo build --release --features vulkan
```

### Apple Metal

```bash
cargo build --release --features metal
```

## Run a local GGUF model

```bash
cargo run --release -- \
  --model ./models/model.gguf \
  --prompt "Explain why Rust is useful for local LLM inference." \
  --max-tokens 128
```

You can also configure the context size and CPU thread count:

```bash
cargo run --release -- \
  --model ./models/model.gguf \
  --prompt "Hello" \
  --ctx-size 4096 \
  --threads 8 \
  --max-tokens 128
```

## Download and run a model from Hugging Face

Use `--hf-repo` and `--hf-file` instead of `--model`:

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --prompt "Hello from Rust!" \
  --max-tokens 64
```

By default, downloaded models are stored in:

```text
./models/
```

If the file already exists, the CLI reuses the cached copy instead of downloading it again.

### Change the model cache directory

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --model-dir ./my-models \
  --prompt "Hello"
```

### Force a new download

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --hf-force-download \
  --prompt "Hello"
```

### Use a branch, tag, or commit

The default Hugging Face revision is `main`.

```bash
cargo run --release -- \
  --hf-repo owner/model-GGUF \
  --hf-file model.Q4_K_M.gguf \
  --hf-revision main \
  --prompt "Hello"
```

## Private or gated Hugging Face models

Set a Hugging Face access token before running the CLI.

The application checks these environment variables in order:

1. `HF_TOKEN`
2. `HUGGING_FACE_HUB_TOKEN`

Linux/macOS:

```bash
export HF_TOKEN=hf_xxx
cargo run --release -- \
  --hf-repo owner/private-model-GGUF \
  --hf-file model.Q4_K_M.gguf \
  --prompt "Hello"
```

PowerShell:

```powershell
$env:HF_TOKEN = "hf_xxx"
cargo run --release -- `
  --hf-repo owner/private-model-GGUF `
  --hf-file model.Q4_K_M.gguf `
  --prompt "Hello"
```

Do not commit your Hugging Face token to the repository.

## GPU offload

Build with the backend you want, then use `--gpu-layers` to control how many model layers are offloaded to the GPU.

### CUDA example

```bash
cargo run --release --features cuda -- \
  --model ./models/model.gguf \
  --prompt "Hello" \
  --max-tokens 128 \
  --gpu-layers 999
```

The same option can be used with a Hugging Face model:

```bash
cargo run --release --features cuda -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --gpu-layers 999 \
  --prompt "Hello"
```

Use a smaller value for `--gpu-layers` if the model does not fit in available VRAM.

## CLI options

```text
-m, --model <MODEL>
    Path to a local GGUF model

--hf-repo <HF_REPO>
    Hugging Face repository, for example:
    mradermacher/SmolLM2-135M-Instruct-GGUF

--hf-file <HF_FILE>
    GGUF file inside the Hugging Face repository

--hf-revision <HF_REVISION>
    Hugging Face revision, branch, tag, or commit
    Default: main

--model-dir <MODEL_DIR>
    Directory used to cache downloaded Hugging Face models
    Default: models

--hf-force-download
    Download the Hugging Face model again even if it is already cached

-p, --prompt <PROMPT>
    Prompt to generate from
    Default: "Hello from Rust!"

-n, --max-tokens <MAX_TOKENS>
    Maximum number of new tokens to generate
    Default: 128

-c, --ctx-size <CTX_SIZE>
    Context size
    Default: 2048

--gpu-layers <GPU_LAYERS>
    Number of GPU layers to offload when built with CUDA, Vulkan, or Metal
    Default: 0

-t, --threads <THREADS>
    Number of CPU threads
    0 lets llama.cpp choose
    Default: 0
```

Show the current command-line help with:

```bash
cargo run --release -- --help
```

## Example: small model smoke test

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --prompt "Write one sentence about Rust." \
  --max-tokens 32
```

On the first run, the model is downloaded into `models/`. Later runs reuse the downloaded GGUF file.

## Continuous integration

`.github/workflows/ci.yml` runs on push and pull request:

1. Install CMake/Clang and a stable Rust toolchain
2. `cargo build --release --locked`
3. `cargo test --release --locked`
4. Download `SmolLM2-135M-Instruct.Q4_K_M.gguf` (~105 MB) and generate a few tokens with the built binary

The GGUF file is cached between CI runs. This is a CPU-only smoke test on `ubuntu-latest` (no CUDA/Vulkan/Metal runner).

You can also trigger it manually from the Actions tab (`workflow_dispatch`).

## Architecture

```text
Your Rust project
    |
    v
llama_rust library  +  llama-rust CLI
    |
    +--> Hugging Face downloader/cache
    |
    v
llama-cpp-2 Rust wrappers
    |
    v
llama-cpp-sys-2 FFI
    |
    v
llama.cpp / ggml
    |
    +--> CPU
    +--> CUDA
    +--> Vulkan
    +--> Metal
```

## Next milestones

1. Publish to crates.io and slim the public API.
2. Add chat-template support from GGUF metadata.
3. Add async/streaming helpers.
4. Add configurable samplers: top-k, top-p, min-p, temperature, and seed.
5. Add model/device information commands.
6. Add an OpenAI-compatible HTTP server in Rust with Axum.
7. Add embeddings.
8. Add multimodal/vision support through llama.cpp `mtmd`.
9. Add Android/Termux build presets.
10. If the goal becomes a fully pure-Rust rewrite, replace llama.cpp/ggml components incrementally: GGUF reader -> tokenizer -> tensor ops -> quantized matmul -> transformer graph -> GPU backends.
