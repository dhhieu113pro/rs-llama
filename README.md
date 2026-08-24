# rs-llama

A Rust **library and CLI** for running GGUF language models through `llama.cpp` using the maintained [`llama-cpp-2`](https://crates.io/crates/llama-cpp-2) bindings.

GitHub repo: [dhhieu113pro/llama-rust](https://github.com/dhhieu113pro/llama-rust)  
Crates.io name: **`rs-llama`** (the old names `llama-rust` and `llama-cpp-rs` are already taken).

## Install

After the first publish:

```bash
cargo add rs-llama
cargo install rs-llama
```

Until it is published, use Git:

```toml
[dependencies]
rs-llama = { git = "https://github.com/dhhieu113pro/llama-rust" }
```

```rust
use rs_llama::{download_huggingface_model, EngineConfig, GenerateRequest, HfDownload, LlamaEngine};
```

Optional GPU features:

```toml
rs-llama = { git = "https://github.com/dhhieu113pro/llama-rust", features = ["cuda"] }
```

Example:

```rust
use rs_llama::{download_huggingface_model, EngineConfig, GenerateRequest, HfDownload, LlamaEngine};

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

## Publish to crates.io

Name `rs-llama` is currently free. From your machine:

```bash
cargo login
cargo publish --dry-run
cargo publish
```

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

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --prompt "Hello from Rust!" \
  --max-tokens 64
```

By default, downloaded models are stored in `./models/`.

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

```bash
cargo run --release -- \
  --hf-repo owner/model-GGUF \
  --hf-file model.Q4_K_M.gguf \
  --hf-revision main \
  --prompt "Hello"
```

## Private or gated Hugging Face models

The application checks `HF_TOKEN` then `HUGGING_FACE_HUB_TOKEN`.

```bash
export HF_TOKEN=hf_xxx
cargo run --release -- \
  --hf-repo owner/private-model-GGUF \
  --hf-file model.Q4_K_M.gguf \
  --prompt "Hello"
```

Do not commit your Hugging Face token to the repository.

## GPU offload

```bash
cargo run --release --features cuda -- \
  --model ./models/model.gguf \
  --prompt "Hello" \
  --max-tokens 128 \
  --gpu-layers 999
```

## CLI options

```text
-m, --model <MODEL>
--hf-repo <HF_REPO>
--hf-file <HF_FILE>
--hf-revision <HF_REVISION>
--model-dir <MODEL_DIR>
--hf-force-download
-p, --prompt <PROMPT>
-n, --max-tokens <MAX_TOKENS>
-c, --ctx-size <CTX_SIZE>
--gpu-layers <GPU_LAYERS>
-t, --threads <THREADS>
```

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

## Continuous integration

`.github/workflows/ci.yml` builds the `rs-llama` binary and smoke-tests real GGUF inference on Ubuntu.

## Architecture

```text
Your Rust project
    |
    v
rs_llama library  +  rs-llama CLI
    |
    +--> Hugging Face downloader/cache
    |
    v
llama-cpp-2
    |
    v
llama.cpp / ggml
```

## Next milestones

1. Publish `rs-llama` to crates.io.
2. Add chat-template support from GGUF metadata.
3. Add async/streaming helpers.
4. Add configurable samplers: top-k, top-p, min-p, temperature, and seed.
5. Add model/device information commands.
6. Add an OpenAI-compatible HTTP server in Rust with Axum.
7. Add embeddings.
8. Add multimodal/vision support through llama.cpp `mtmd`.
9. Add Android/Termux build presets.
10. If the goal becomes a fully pure-Rust rewrite, replace llama.cpp/ggml components incrementally.
