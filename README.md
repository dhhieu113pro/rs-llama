# rs-llama

A Rust **library and CLI** for running GGUF models through [llama.cpp](https://github.com/ggml-org/llama.cpp).

Website: [dhhieu113pro.github.io/rs-llama](https://dhhieu113pro.github.io/rs-llama/)  
GitHub: [dhhieu113pro/rs-llama](https://github.com/dhhieu113pro/rs-llama)  
Releases: [github.com/dhhieu113pro/rs-llama/releases](https://github.com/dhhieu113pro/rs-llama/releases)

## Platforms

| Platform | CI | Release asset |
| --- | --- | --- |
| Linux x86_64 | LED smoke + vision | `rs-llama-linux-x86_64.tar.gz` |
| Windows x86_64 | LED smoke + vision | `rs-llama-windows-x86_64.zip` |
| macOS Apple Silicon | Metal + LED smoke + vision | `rs-llama-macos-arm64.tar.gz` |
| Android / Termux arm64 | NDK build + emulator vision (x86_64) | `rs-llama-android-arm64.tar.gz` |
| NVIDIA CUDA | Linux compile | `--features cuda` |
| Vulkan | Linux compile | `--features vulkan` |

Tags `v*` publish those four binaries plus `SHA256SUMS`.

```bash
curl -L -O https://github.com/dhhieu113pro/rs-llama/releases/latest/download/rs-llama-linux-x86_64.tar.gz
tar -xzf rs-llama-linux-x86_64.tar.gz
./rs-llama-linux-x86_64/rs-llama --help
```

## Architecture

```text
rs-llama CLI + library
    |
    +--> Hugging Face download / cache + mmproj
    v
LlamaEngine
    v
llama.cpp / ggml
```

## Requirements

Rust, Git, C/C++ compiler, CMake, Clang / libclang.

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
    println!("{}", engine.generate(&GenerateRequest::new("What is an LED lamp?").with_chat(true))?);
    Ok(())
}
```

## Build

```bash
cargo build --release
cargo build --release --features cuda
cargo build --release --features vulkan
cargo build --release --features metal
```

Android / Termux: [docs/ANDROID.md](docs/ANDROID.md)

```bash
bash scripts/termux-build.sh
bash scripts/termux-smoke.sh
```

## Run

Instruct models need `--chat`.

```bash
cargo run --release -- --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf --chat --no-echo-prompt --prompt "What is an LED lamp?"
```

## Vision / mmproj

CI generates a text image (`LED LAMP`) and runs `--image` on Linux, Windows, macOS, and an Android x86_64 emulator.

```bash
cargo run --release -- --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF --hf-file SmolVLM-256M-Instruct-Q8_0.gguf --image ./photo.jpg --chat --prompt "What is in this image?"
```

## Continuous integration

| Check | Where |
| --- | --- |
| CPU + LED lamp smoke | Linux, Windows, macOS |
| Vision image + mmproj | Linux, Windows, macOS |
| Vision image + mmproj | Android emulator x86_64 |
| Metal | macOS |
| Vulkan / CUDA compile | Linux |
| Android arm64 release binary | NDK `arm64-v8a` |
