# llama-rust

A small Rust CLI that runs GGUF language models through `llama.cpp` using the maintained `llama-cpp-2` bindings.

This is the practical first step toward a Rust-native llama.cpp-style runtime: the application API and orchestration are Rust, while the low-level tensor/inference backend is llama.cpp.

## Requirements

- Rust toolchain (`rustup`, `cargo`, `rustc`)
- C/C++ compiler
- CMake
- Clang/libclang (required by bindgen)

## Build

CPU:

```bash
cargo build --release
```

NVIDIA CUDA:

```bash
cargo build --release --features cuda
```

Vulkan:

```bash
cargo build --release --features vulkan
```

Apple Metal:

```bash
cargo build --release --features metal
```

## Run

CPU:

```bash
cargo run --release -- \
  --model ./models/model.gguf \
  --prompt "Explain why Rust is useful for local LLM inference." \
  --max-tokens 128
```

CUDA with GPU offload:

```bash
cargo run --release --features cuda -- \
  --model ./models/model.gguf \
  --prompt "Hello" \
  --max-tokens 128 \
  --gpu-layers 999
```

Useful options:

```text
-m, --model <MODEL>             GGUF model path
-p, --prompt <PROMPT>           Prompt text
-n, --max-tokens <N>            Maximum generated tokens
-c, --ctx-size <N>              Context size
-t, --threads <N>               CPU thread count
    --gpu-layers <N>            Layers to offload to GPU
```

## Architecture

```text
Rust CLI / API
    |
    v
llama-cpp-2 safe-ish Rust wrappers
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
4. Add configurable samplers: top-k, top-p, min-p, temperature, seed.
5. Add model/device information commands.
6. Add an OpenAI-compatible HTTP server in Rust with Axum.
7. Add embeddings.
8. Add multimodal/vision support through llama.cpp `mtmd`.
9. Add Android/Termux build presets.
10. If the goal is a fully pure-Rust rewrite, replace llama.cpp/ggml components incrementally: GGUF reader -> tokenizer -> tensor ops -> quantized matmul -> transformer graph -> GPU backends.
