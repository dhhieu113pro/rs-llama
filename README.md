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

Text generation follows llama.cpp `examples/simple/simple.cpp`:

`load model → tokenize → llama_decode → llama_sampler_sample → token_to_piece`

## Requirements

- Rust (`rustup`, `cargo`, `rustc`)
- Git (build clones `ggml-org/llama.cpp`)
- C/C++ compiler
- CMake
- Clang / libclang (`bindgen`)

Optional: pin a local tree

```bash
export LLAMA_CPP_SRC=/path/to/llama.cpp
# or
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

Public API:

- `LlamaEngine::load` / `generate` / `generate_to_writer` / `generate_with_callback`
- `EngineConfig`, `GenerateRequest` (`with_chat`, `with_image`)
- `HfDownload`, `download_huggingface_model`, `download_huggingface_model_bundle`
- `resolve_model_path`, `resolve_model_files`, `ResolvedModel`

## Build

```bash
cargo build --release
cargo build --release --features cuda
cargo build --release --features vulkan
cargo build --release --features metal
```

The first build is slow: it clones and compiles llama.cpp.

## Run

### Local GGUF

```bash
cargo run --release -- \
  --model ./models/model.gguf \
  --prompt "Explain why Rust is useful for local LLM inference." \
  --max-tokens 128
```

### Hugging Face + instruct chat (recommended)

Instruct models need `--chat`. Without it they often continue the question instead of answering.

```bash
cargo run --release -- \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --chat \
  --no-echo-prompt \
  --prompt "What is an LED lamp?" \
  --max-tokens 80
```

`--chat` wraps ChatML:

```text
<|im_start|>system
You are a helpful assistant. Answer the question in one or two short sentences.
<|im_end|>
<|im_start|>user
What is an LED lamp?
<|im_end|>
<|im_start|>assistant
```

`--no-echo-prompt` prints only the model answer.

Downloaded files go to `./models/`.

### Private or gated repos

```bash
export HF_TOKEN=hf_xxx
```

Also accepts `HUGGING_FACE_HUB_TOKEN`.

### GPU offload

```bash
cargo run --release --features cuda -- \
  --model ./models/model.gguf \
  --prompt "Hello" \
  --gpu-layers 999
```

## Vision / mmproj

Hugging Face downloads list the repo and **auto-download `mmproj*.gguf`** when present (prefers F16/FP16/BF16, same folder as the model).

```bash
cargo run --release -- \
  --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF \
  --hf-file SmolVLM-256M-Instruct-Q8_0.gguf \
  --image ./photo.jpg \
  --chat \
  --prompt "What is an LED lamp in this image?"
```

```bash
--hf-mmproj mmproj-F16.gguf     # file inside the repo
--mmproj ./models/mmproj-F16.gguf
--no-mmproj                     # skip auto download
```

Local `--model` also scans the same directory for `*mmproj*.gguf`.

Pixel encode through llama.cpp `mtmd` is not wired yet. The projector is downloaded and attached; CLIP encode is the next step.

## CLI flags

```text
-m, --model <MODEL>
--hf-repo <HF_REPO>
--hf-file <HF_FILE>
--hf-mmproj <HF_MMPROJ>
--mmproj <MMPROJ>
--no-mmproj
--image <IMAGE>
--chat
--no-echo-prompt
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

CI and Release run on **Linux, Windows, and macOS**:

1. Build `rs-llama` (compiles llama.cpp via `llama-sys`)
2. `cargo test --release`
3. Smoke test: download SmolLM2-135M-Instruct Q4_K_M and ask **What is an LED lamp?** with `--chat --no-echo-prompt`
4. Fail if the answer does not mention led / light / diode / lamp / electric / bulb

Release packages binaries only after those tests pass. A GitHub Release is created only on tags `v*`.

## Workspace

```text
rs-llama/
  src/                 CLI + library
  crates/llama-sys/    FFI to llama.cpp
  .github/workflows/   CI + gated release
```

See [docs/LLAMA-CPP-2.md](docs/LLAMA-CPP-2.md) for the old wrapper notes and why we build llama.cpp directly.
