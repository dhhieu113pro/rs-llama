# rs-llama

A Rust **library and CLI** for running GGUF language models through `llama.cpp` using the maintained [`llama-cpp-2`](https://crates.io/crates/llama-cpp-2) bindings.

GitHub repo: [dhhieu113pro/rs-llama](https://github.com/dhhieu113pro/rs-llama)  
Crates.io name: **`rs-llama`**

## Install

After the first publish:

```bash
cargo add rs-llama
cargo install rs-llama
```

Until it is published, use Git:

```toml
[dependencies]
rs-llama = { git = "https://github.com/dhhieu113pro/rs-llama" }
```

Optional GPU features:

```toml
rs-llama = { git = "https://github.com/dhhieu113pro/rs-llama", features = ["cuda"] }
```

Example:

```rust
use rs_llama::{download_huggingface_model_bundle, EngineConfig, GenerateRequest, HfDownload, LlamaEngine};

fn main() -> anyhow::Result<()> {
    let bundle = download_huggingface_model_bundle(&HfDownload::new(
        "ggml-org/SmolVLM-256M-Instruct-GGUF",
        "SmolVLM-256M-Instruct-Q8_0.gguf",
    ))?;

    let mut config = EngineConfig::new(bundle.model_path).with_ctx_size(1024);
    if let Some(mmproj) = bundle.mmproj_path {
        config = config.with_mmproj(mmproj);
    }

    let engine = LlamaEngine::load(config)?;
    let text = engine.generate(&GenerateRequest::new("What is in this picture?"))?;
    println!("{text}");
    Ok(())
}
```

Public API:

- `LlamaEngine::load` / `generate` / `generate_to_writer` / `generate_with_callback`
- `EngineConfig` and `GenerateRequest`
- `HfDownload`, `download_huggingface_model`, `download_huggingface_model_bundle`
- `resolve_model_path`, `resolve_model_files`, `ResolvedModel`

The consuming project still needs a C/C++ compiler, CMake, and Clang/libclang because `llama.cpp` is compiled as part of the build.

## Vision / mmproj

When you download from Hugging Face, `rs-llama` lists the repo and **automatically downloads `mmproj*.gguf`** if one exists next to the language model.

```bash
cargo run --release -- \
  --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF \
  --hf-file SmolVLM-256M-Instruct-Q8_0.gguf \
  --image ./photo.jpg \
  --prompt "What is LED lamp in this image?"
```

It prefers `mmproj-F16` / `FP16` / `BF16` over quantized projectors, and prefers a projector in the same folder as the model file.

```bash
# explicit projector file in the repo
--hf-mmproj mmproj-F16.gguf

# local projector
--mmproj ./models/mmproj-F16.gguf

# skip auto download
--no-mmproj
```

For a local GGUF, it also looks in the same directory for `*mmproj*.gguf`.

`llama-cpp-2` 0.1.154 does not expose llama.cpp `mtmd` image encoding yet, so the projector is downloaded and attached. Pixel-level CLIP encode will land when the binding adds `mtmd`.

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

## Download and run a model from Hugging Face

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --prompt "Hello from Rust!" \
  --max-tokens 64
```

By default, downloaded models are stored in `./models/`.

## Private or gated Hugging Face models

The application checks `HF_TOKEN` then `HUGGING_FACE_HUB_TOKEN`.

```bash
export HF_TOKEN=hf_xxx
```

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
--hf-mmproj <HF_MMPROJ>
--mmproj <MMPROJ>
--no-mmproj
--image <IMAGE>
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

## Continuous integration

`.github/workflows/ci.yml` builds the `rs-llama` binary and smoke-tests real GGUF inference on Linux, Windows, and macOS.

## Architecture

```text
Your Rust project
    |
    v
rs_llama library  +  rs-llama CLI
    |
    +--> Hugging Face downloader/cache
    |      + auto mmproj detect/download
    v
llama-cpp-2
    |
    v
llama.cpp / ggml
```
