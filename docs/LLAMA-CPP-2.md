# llama.cpp integration

`rs-llama` no longer depends on `llama-cpp-2`.

```text
rs-llama (CLI + download + LlamaEngine)
    |
    v
llama-sys          our crate: bindgen + CMake
    |
    v
llama.cpp / ggml   cloned at build time from ggml-org/llama.cpp
```

`crates/llama-sys/build.rs`:

1. `git clone --depth 1` llama.cpp (or use `LLAMA_CPP_SRC`)
2. CMake static library build
3. bindgen on `include/llama.h`
4. link `llama` + `ggml*`

Override source:

```bash
export LLAMA_CPP_SRC=/path/to/llama.cpp
export LLAMA_CPP_REV=b1234   # git branch/tag if cloning
```
