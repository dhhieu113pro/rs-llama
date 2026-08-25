<p align="center">
  <img src="assets/logo.svg" alt="rs-llama logo" width="160" height="160">
</p>

<h1 align="center">rs-llama</h1>

<p align="center">
  <strong>Rust library and CLI for running GGUF models through <a href="https://github.com/ggml-org/llama.cpp">llama.cpp</a></strong>
</p>

<p align="center">
  <a href="https://dhhieu113pro.github.io/rs-llama/">Website</a> ·
  <a href="https://github.com/dhhieu113pro/rs-llama/releases">Releases</a> ·
  <a href="https://github.com/dhhieu113pro/rs-llama">GitHub</a>
</p>

<p align="center">
  <a href="https://github.com/dhhieu113pro/rs-llama/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/dhhieu113pro/rs-llama/ci.yml?branch=main&style=flat-square&label=CI" alt="CI">
  </a>
  <img src="https://img.shields.io/github/v/release/dhhieu113pro/rs-llama?style=flat-square" alt="Release">
  <img src="https://img.shields.io/github/license/dhhieu113pro/rs-llama?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS%20%7C%20Android-blue?style=flat-square" alt="Platforms">
  <img src="https://img.shields.io/badge/GPU-CUDA%20%7C%20Vulkan%20%7C%20Metal-green?style=flat-square" alt="GPU">
  <img src="https://img.shields.io/github/last-commit/dhhieu113pro/rs-llama?style=flat-square" alt="Last commit">
</p>

---

## Runtime GPU auto-detection

Downloaded desktop releases contain runtime-loadable llama.cpp backends and automatically choose the best usable backend on the machine where you run them:

| Release platform | Runtime priority |
|---|---|
| Windows / Linux | CUDA → Vulkan → CPU |
| macOS | Metal → CPU |
| Android / Termux | CPU |

You download **one archive for your platform**. If CUDA is missing, cannot load, has no usable device, or cannot load the model, rs-llama tries Vulkan. If no GPU backend can load the model, it retries on CPU. macOS similarly falls back from Metal to CPU.

The standard Windows/Linux archives include the rs-llama executable plus the CPU, CUDA, and Vulkan llama.cpp backend modules. The full CUDA toolkit/runtime is **not** bundled; if the compatible NVIDIA runtime is absent, CUDA simply does not win selection and fallback continues.

The selected backend is reported after the model has successfully loaded:

```text
Backend: CUDA
Device: CUDA0 — NVIDIA GPU
```

`EngineConfig` and the CLI default to `999` GPU layers, asking llama.cpp to offload all model layers it can. To explicitly use CPU inference:

```bash
rs-llama --model model.gguf --gpu-layers 0 --prompt "Hello"
```

### Source builds and backend overrides

Normal Cargo/source builds retain the existing backend-specific behavior. A plain source build chooses a suitable backend from the build environment:

```bash
cargo build --release
```

`RS_LLAMA_BACKEND` is a **build-time** override:

```bash
RS_LLAMA_BACKEND=cpu cargo build --release
RS_LLAMA_BACKEND=cuda cargo build --release
RS_LLAMA_BACKEND=vulkan cargo build --release
RS_LLAMA_BACKEND=metal cargo build --release
```

Explicit static Cargo features remain supported:

```bash
cargo build --release --features cuda
cargo build --release --features vulkan
cargo build --release --features metal
```

The distributable desktop bundle mode is explicit:

```bash
cargo build --release --features dynamic-backends
```

`dynamic-backends` must not be combined with `cuda`, `vulkan`, or `metal`; dynamic releases build the platform backend set together instead of selecting one backend at compile time.

## Platforms

| Platform              | CI                                           | Release asset                        |
|-----------------------|----------------------------------------------|--------------------------------------|
| Linux x86_64          | CPU + CUDA + Vulkan package + text + vision | `rs-llama-linux-x86_64.tar.gz`       |
| Windows x86_64        | CPU + CUDA + Vulkan package + text + vision | `rs-llama-windows-x86_64.zip`        |
| macOS Apple Silicon   | CPU + Metal package + text + vision         | `rs-llama-macos-arm64.tar.gz`        |
| Android / Termux arm64| NDK CPU build                                | `rs-llama-android-arm64.tar.gz`      |

Tags `v*` publish the four archives plus `SHA256SUMS`.

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
    |
    +--> runtime device registry
    |      Windows/Linux: CUDA -> Vulkan -> CPU
    |      macOS:         Metal -> CPU
    v
llama.cpp / ggml
```

## Requirements

For downloaded releases, CPU fallback requires no GPU SDK. GPU backends use the corresponding runtime/driver available on the user's machine.

For source builds:

- Rust toolchain
- Git
- C/C++ compiler, CMake, Clang / libclang
- Optional CUDA toolkit for CUDA source builds
- Optional Vulkan SDK/development package for Vulkan source builds

```bash
export LLAMA_CPP_SRC=/path/to/llama.cpp
export LLAMA_CPP_REV=master   # or a pinned commit
```

## Install (library)

```toml
[dependencies]
rs-llama = { git = "https://github.com/dhhieu113pro/rs-llama" }
```

```rust
use rs_llama::{
    download_huggingface_model_bundle, EngineConfig, GenerateRequest, HfDownload, LlamaEngine,
};

fn main() -> anyhow::Result<()> {
    let bundle = download_huggingface_model_bundle(&HfDownload::new(
        "mradermacher/SmolLM2-135M-Instruct-GGUF",
        "SmolLM2-135M-Instruct.Q4_K_M.gguf",
    ))?;

    let engine = LlamaEngine::load(
        EngineConfig::new(bundle.model_path).with_ctx_size(1024),
    )?;

    println!("backend: {}", engine.active_backend());
    let out = engine.generate(
        &GenerateRequest::new("What is an LED lamp?").with_chat(true),
    )?;
    println!("{out}");
    Ok(())
}
```

## Build

For normal source builds:

```bash
cargo build --release
```

Android / Termux: see [docs/ANDROID.md](docs/ANDROID.md)

```bash
bash scripts/termux-build.sh
bash scripts/termux-smoke.sh
```

## Run (CLI)

Instruct models need `--chat`.

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --chat --no-echo-prompt \
  --prompt "What is an LED lamp?"
```

Force model layers to CPU:

```bash
cargo run --release -- --model ./model.gguf --gpu-layers 0
```

## Vision / mmproj

CI generates a text image (`LED LAMP`) and runs `--image` on the packaged Linux, Windows, and macOS desktop runtime bundles.

```bash
cargo run --release -- \
  --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF \
  --hf-file SmolVLM-256M-Instruct-Q8_0.gguf \
  --image ./photo.jpg \
  --chat \
  --prompt "What is in this image?"
```

## Continuous integration

| Check                                  | Where                          |
|----------------------------------------|--------------------------------|
| Backend selection policy               | Pure Rust, no GPU required     |
| Default/static regression tests        | Linux, Windows, macOS          |
| Dynamic runtime backend package layout | Linux, Windows, macOS          |
| Forced CPU fallback inference          | Packaged desktop executable    |
| Vision image + mmproj                  | Packaged desktop executable    |
| Android arm64 release binary           | NDK `arm64-v8a`                |
| Archive backend-content assertions     | Package job                    |

## License

MIT

---

<p align="center">
  <a href="https://dhhieu113pro.github.io/rs-llama/">dhhieu113pro.github.io/rs-llama</a>
</p>
