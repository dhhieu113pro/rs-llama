//! `rs-llama` library API for loading GGUF models and running local inference.
//!
//! The CLI binary is `rs-llama`. Import the library as `rs_llama`.

mod download;
mod engine;

pub use download::{
    download_huggingface_model, download_huggingface_model_bundle, resolve_model_files,
    resolve_model_path, HfDownload, ResolvedModel,
};
pub use engine::{EngineConfig, GenerateRequest, LlamaEngine, DEFAULT_GPU_LAYERS};

/// llama.cpp acceleration backend selected when `rs-llama` was built.
pub fn compiled_backend() -> &'static str {
    llama_sys::COMPILED_BACKEND
}

/// Why the compiled backend was selected: `auto`, `environment`, or `feature`.
pub fn backend_selection_source() -> &'static str {
    llama_sys::BACKEND_SELECTION_SOURCE
}
