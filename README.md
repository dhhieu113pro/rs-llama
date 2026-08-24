# rs-llama

A Rust **library and CLI** for running GGUF models through [llama.cpp](https://github.com/ggml-org/llama.cpp).

It does **not** use `llama-cpp-2`. Inference goes through our crate `llama-sys`, which clones llama.cpp at build time, compiles it with CMake, and binds `llama.h`.

GitHub: [dhhieu113pro/rs-llama](https://github.com/dhhieu113pro/rs-llama)  
Crate name: **`rs-llama`**

## Architecture

```text
rs-llama CLI + library
    |
    +--> Hugging Face download / cache
    |      + auto-detect mmproj
    v
LlamaEngine
    |
    v
llama-sys          bindgen + CMake (this repo)
    |
    v
llama.cpp / ggml   cloned at build time
```

Text generation follows llama.cpp `examples/simple/simple.cpp`.

## Requirements

- Rust (`rustup`, `cargo`, `rustc`)
- Git (build clones `ggml-org/llama.cpp`)
- C/C++ compiler
- CMake
- Clang / libclang (`bindgen`)

```bash
export LLAMA_CPP_SRC=/path/to/llama.cpp
export LLAMA_CPP_REV=master
```

## Install from Git

```toml
[dependencies]
rs-llama = { git = "https://github.com/dhhieu113pro/rs-llama" }
```

```rust
use rs_llama::{download_huggingface_model_bundle, EngineConfig, GenerateRequest, HfDownload, LlamaEngine};

fn main() -> anyhow::Result<()> {
    let bundle = download_huggingface_model_bundle(&HfDownload::new(
        "mradermacher/SmolLM2-135M-Instruct-GGUF",
        "SmolLM2-135M-Instruct.Q4_K_M.gguf",
    ))?;

    let engine = LlamaEngine::load(EngineConfig::new(bundle.model_path).with_ctx_size(1024))?;
    let text = engine.generate(
        &GenerateRequest::new("What is an LED lamp?").with_chat(true).with_max_tokens(64),
    )?;
    println!("{text}");
    Ok(())
}
```

## Build

### CPU

```bash
cargo build --release
```

CI smoke-tests this on Linux, Windows, and macOS.

### NVIDIA CUDA

```bash
cargo build --release --features cuda
```

CI compiles this on Linux. GitHub-hosted runners have no NVIDIA GPU, so CUDA inference is not run there.

```bash
cargo run --release --features cuda -- --model ./models/model.gguf --gpu-layers 999 --prompt "Hello"
```

### Vulkan

```bash
cargo build --release --features vulkan
```

CI compiles this on Linux (`libvulkan-dev`).

### Apple Metal

```bash
cargo build --release --features metal
```

CI builds and smoke-tests this on `macos-latest`. `llama-sys` enables `GGML_METAL=ON` on macOS.

## Run a local GGUF model

```bash
cargo run --release -- --model ./models/model.gguf --chat --prompt "What is an LED lamp?"
```

## Download from Hugging Face

Instruct models need `--chat` or they continue the question instead of answering.

```bash
cargo run --release -- --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf --chat --no-echo-prompt --prompt "What is an LED lamp?"
```

### Private or gated repos

```bash
export HF_TOKEN=hf_xxx
```

## Vision / mmproj

Hugging Face downloads auto-fetch `mmproj*.gguf` when present.

```bash
cargo run --release -- --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF --hf-file SmolVLM-256M-Instruct-Q8_0.gguf --image ./photo.jpg --chat --prompt "What is in this image?"
```

```bash
--hf-mmproj mmproj-F16.gguf
--mmproj ./models/mmproj-F16.gguf
--no-mmproj
```

`mtmd` pixel encode is not wired yet. The projector is downloaded and attached.

## CLI flags

```text
-m, --model
--hf-repo --hf-file --hf-mmproj --mmproj --no-mmproj --image
--chat --no-echo-prompt
--hf-revision --model-dir --hf-force-download
-p, --prompt  -n, --max-tokens  -c, --ctx-size
--gpu-layers  -t, --threads
```

## Continuous integration

| Check | Where | What |
| --- | --- | --- |
| CPU + real GGUF smoke | Linux, Windows, macOS | Ask **What is an LED lamp?** |
| Metal | macOS | `--features metal` + smoke |
| Vulkan | Linux | compile `--features vulkan` |
| CUDA | Linux | compile `--features cuda` |

Smoke fails if the answer does not mention led / light / diode / lamp / electric / bulb.

Release packages binaries only after CPU tests pass. GitHub Release only on tags `v*`.

## Workspace

```text
rs-llama/
  src/                 CLI + library
  crates/llama-sys/    FFI to llama.cpp
  .github/workflows/   CI + gated release
```
