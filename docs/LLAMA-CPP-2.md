# What llama-cpp-2 does

`rs-llama` does **not** run the model itself. It talks to this stack:

```text
rs-llama (our CLI + library)
    |
    v
llama-cpp-2          Rust safe API  (model load, context, sampler, mtmd)
    |
    v
llama-cpp-sys-2      bindgen + CMake build of llama.cpp
    |
    v
llama.cpp / ggml     C/C++ inference engine (GGUF, CPU/GPU, vision CLIP/mtmd)
```

## Each crate

### llama-cpp-2

Rust wrappers around llama.cpp C API:

- `LlamaBackend` — init ggml/llama
- `LlamaModel` — load GGUF
- `LlamaContext` — KV cache + decode
- `LlamaBatch` / `LlamaSampler` — generate tokens
- `mtmd` — vision/audio projector (`MtmdContext`, `MtmdBitmap`)

It stays close to llama.cpp on purpose. Not a high-level chat SDK.

Repo: https://github.com/utilityai/llama-cpp-rs  
Published crate: `llama-cpp-2` 0.1.154 on crates.io (older snapshot).  
**GitHub `main` already has `llama-cpp-2/src/mtmd.rs`.**

### llama-cpp-sys-2

- Compiles llama.cpp with CMake
- Runs bindgen on `llama.h` / `mtmd.h`
- Feature `mtmd` builds `tools/mtmd` (CLIP + projector)

### llama.cpp

The real engine. Lives as a git submodule inside `llama-cpp-sys-2/llama.cpp`.
Copying that whole tree into `rs-llama` is ~17MB+ of C/CUDA/Metal. Do not vendor it unless we fork llama.cpp itself.

## Why we should not copy llama.cpp into this repo

- Huge binary history and GPU backends
- bindgen/CMake paths assume their layout
- Updates become merge hell

Better: depend on their git repo (or a path checkout) and patch only Rust.

## Use GitHub source (current Cargo.toml)

```toml
llama-cpp-2 = { git = "https://github.com/utilityai/llama-cpp-rs", package = "llama-cpp-2", features = ["mtmd"] }
```

Cargo clones it into `~/.cargo/git` and builds `llama-cpp-2` + `llama-cpp-sys-2` + llama.cpp.

## Clone locally so we can edit easily

```bash
git clone --recursive https://github.com/utilityai/llama-cpp-rs.git vendor/llama-cpp-rs
```

Then in `Cargo.toml` switch to:

```toml
llama-cpp-2 = { path = "vendor/llama-cpp-rs/llama-cpp-2", features = ["mtmd"] }
```

Edit:

- `vendor/llama-cpp-rs/llama-cpp-2/src/mtmd.rs` — vision API
- `vendor/llama-cpp-rs/llama-cpp-2/src/model.rs` — load GGUF
- `vendor/llama-cpp-rs/llama-cpp-sys-2/` — C bindings / CMake

Do **not** commit `vendor/llama-cpp-rs` unless we decide to submodule it.

```bash
git submodule add https://github.com/utilityai/llama-cpp-rs vendor/llama-cpp-rs
cd vendor/llama-cpp-rs && git submodule update --init --recursive
```

## Feature flags we pass through

| rs-llama feature | llama-cpp-2 feature | meaning |
|---|---|---|
| `mtmd` (default) | `mtmd` | build CLIP/mmproj |
| `cuda` | `cuda` | NVIDIA |
| `vulkan` | `vulkan` | Vulkan |
| `metal` | `metal` | Apple GPU |
