# rs-llama

A Rust **library and CLI** for running GGUF models through [llama.cpp](https://github.com/ggml-org/llama.cpp).

GitHub: [dhhieu113pro/rs-llama](https://github.com/dhhieu113pro/rs-llama)  
Releases: [github.com/dhhieu113pro/rs-llama/releases](https://github.com/dhhieu113pro/rs-llama/releases)

## Releases

Asset names follow [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw/releases) rustc triples. Android is attached like [NullClaw](https://github.com/nullclaw/nullclaw/releases).

**Published today (`v0.1.0`)** still uses the old names:

| Asset | Platform |
| --- | --- |
| `llama-rust-linux-x86_64.tar.gz` | Linux x86_64 |
| `llama-rust-windows-x86_64.zip` | Windows x86_64 |
| `llama-rust-macos.tar.gz` | macOS |

**Next `v*` tag** uploads:

| Asset | Platform |
| --- | --- |
| `rs-llama-x86_64-unknown-linux-gnu.tar.gz` | Linux x86_64 |
| `rs-llama-x86_64-pc-windows-msvc.zip` | Windows x86_64 |
| `rs-llama-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon |
| `rs-llama-aarch64-linux-android.tar.gz` | Android / Termux arm64 |
| `SHA256SUMS` | checksums |

Not in the release zip (compile in CI only): CUDA, Vulkan, Linux musl, Linux aarch64, Android armv7.

A GitHub Release is created only on tags `v*`, after desktop smoke + vision smoke + Android compile succeed.

```bash
# after the next tag
curl -L -O https://github.com/dhhieu113pro/rs-llama/releases/download/v0.2.0/rs-llama-x86_64-unknown-linux-gnu.tar.gz
tar -xzf rs-llama-x86_64-unknown-linux-gnu.tar.gz
./rs-llama-x86_64-unknown-linux-gnu/rs-llama --help
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

CI runs this on Linux, Windows, and macOS. `mtmd` pixel encode is not wired yet.

```bash
cargo run --release -- --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF --hf-file SmolVLM-256M-Instruct-Q8_0.gguf --image ./photo.jpg --chat --prompt "What is in this image?"
```

## Continuous integration

| Check | Where |
| --- | --- |
| CPU + LED lamp smoke | Linux, Windows, macOS |
| Vision mmproj + `--image` | Linux, Windows, macOS |
| Metal | macOS |
| Vulkan / CUDA compile | Linux |
| Android arm64 | NDK `cargo ndk -t arm64-v8a` |
