# llama-rust

A small Rust CLI for running GGUF language models through `llama.cpp` using the maintained [`llama-cpp-2`](https://crates.io/crates/llama-cpp-2) bindings.

The application layer and orchestration are written in Rust, while `llama.cpp` provides the low-level model loading and inference backend.

## Features

- Run local GGUF models
- Download GGUF models directly from Hugging Face
- Cache downloaded models locally
- Support Hugging Face branches, tags, and revisions
- Support private or gated Hugging Face repositories with a token
- CPU inference
- Optional CUDA, Vulkan, and Metal builds
- Configurable context size, CPU threads, GPU layers, and generation length

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

## Architecture

```text
Rust CLI / API
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

1. Extract inference into a reusable `LlamaEngine` Rust library.
2. Add chat-template support from GGUF metadata.
3. Add streaming callbacks instead of writing directly to stdout.
4. Add configurable samplers: top-k, top-p, min-p, temperature, and seed.
5. Add model/device information commands.
6. Add an OpenAI-compatible HTTP server in Rust with Axum.
7. Add embeddings.
8. Add multimodal/vision support through llama.cpp `mtmd`.
9. Add Android/Termux build presets.
10. If the goal becomes a fully pure-Rust rewrite, replace llama.cpp/ggml components incrementally: GGUF reader -> tokenizer -> tensor ops -> quantized matmul -> transformer graph -> GPU backends.
