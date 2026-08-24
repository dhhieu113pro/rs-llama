# rs-llama

A Rust **library and CLI** for running GGUF models through [llama.cpp](https://github.com/ggml-org/llama.cpp).

It does **not** use `llama-cpp-2`. Inference goes through our crate `llama-sys`, which clones llama.cpp at build time, compiles it with CMake, and binds `llama.h`.

GitHub: [dhhieu113pro/rs-llama](https://github.com/dhhieu113pro/rs-llama)  
Releases: [github.com/dhhieu113pro/rs-llama/releases](https://github.com/dhhieu113pro/rs-llama/releases)  
Crate name: **`rs-llama`**

## Releases

Latest published tag: **[v0.1.0](https://github.com/dhhieu113pro/rs-llama/releases/tag/v0.1.0)** (22 Aug 2026).

| Platform | In v0.1.0 | Asset |
| --- | --- | --- |
| Linux x86_64 CPU | yes | `llama-rust-linux-x86_64.tar.gz` |
| Windows x86_64 CPU | yes | `llama-rust-windows-x86_64.zip` |
| macOS (GitHub `macos-latest`, Apple Silicon) | yes | `llama-rust-macos.tar.gz` |
| Linux aarch64 | no | — |
| Android / Termux | no binary on the release | cross-compile in CI only |
| CUDA / Vulkan prebuilt | no | compile from source |

Those v0.1.0 files still use the old `llama-rust-*` names. The next `v*` tag will upload:

- `rs-llama-linux-x86_64.tar.gz`
- `rs-llama-windows-x86_64.zip`
- `rs-llama-macos.tar.gz`

A GitHub Release is created **only on tags `v*`**, and only after Linux + Windows + macOS tests pass (text smoke + vision mmproj smoke).

```bash
# Linux
curl -L -o rs-llama.tar.gz https://github.com/dhhieu113pro/rs-llama/releases/download/v0.1.0/llama-rust-linux-x86_64.tar.gz
tar -xzf rs-llama.tar.gz

# macOS
curl -L -o rs-llama.tar.gz https://github.com/dhhieu113pro/rs-llama/releases/download/v0.1.0/llama-rust-macos.tar.gz
tar -xzf rs-llama.tar.gz

# Windows
# download llama-rust-windows-x86_64.zip from the release page
```

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

## Requirements

- Rust (`rustup`, `cargo`, `rustc`)
- Git (build clones `ggml-org/llama.cpp`)
- C/C++ compiler, CMake, Clang / libclang

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

### NVIDIA CUDA

```bash
cargo build --release --features cuda
cargo run --release --features cuda -- --model ./models/model.gguf --gpu-layers 999 --prompt "Hello"
```

### Vulkan

```bash
cargo build --release --features vulkan
```

### Apple Metal

```bash
cargo build --release --features metal
```

### Android / Termux

See [docs/ANDROID.md](docs/ANDROID.md). CI cross-compiles `aarch64-linux-android`. On a phone:

```bash
bash scripts/termux-build.sh
bash scripts/termux-smoke.sh
```

## Run

Instruct models need `--chat`.

```bash
cargo run --release -- --model ./models/model.gguf --chat --prompt "What is an LED lamp?"

cargo run --release -- --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf --chat --no-echo-prompt --prompt "What is an LED lamp?"
```

```bash
export HF_TOKEN=hf_xxx
```

## Vision / mmproj

HF downloads auto-fetch `mmproj*.gguf` when present. CI runs this on Linux, Windows, and macOS.

```bash
cargo run --release -- --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF --hf-file SmolVLM-256M-Instruct-Q8_0.gguf --image ./photo.jpg --chat --prompt "What is in this image?"
```

`mtmd` pixel encode is not wired yet.

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
| CPU + GGUF smoke | Linux, Windows, macOS | **What is an LED lamp?** |
| Vision mmproj + `--image` | Linux, Windows, macOS | SmolVLM + tiny PNG |
| Metal | macOS | `--features metal` + smoke |
| Vulkan | Linux | compile |
| CUDA | Linux | compile nvcc only |
| Android arm64 | Linux NDK | `cargo ndk -t arm64-v8a` |
| Termux scripts | Linux | shellcheck |

Release packages the three desktop CPU binaries only after those desktop tests pass.
