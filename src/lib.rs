//! `rs-llama` library API for loading GGUF models and running local inference.
//!
//! The CLI binary is `rs-llama`. Import the library as `rs_llama`.

mod download;
mod engine;
mod runtime_backend;

pub use download::{
    download_huggingface_model, download_huggingface_model_bundle, resolve_model_files,
    resolve_model_path, HfDownload, ResolvedModel,
};
pub use engine::{EngineConfig, GenerateRequest, LlamaEngine, DEFAULT_GPU_LAYERS};
pub use runtime_backend::{runtime_devices, RuntimeBackend, RuntimeDevice};

/// Backend metadata from the build. Dynamic release builds return `dynamic`.
/// Use `LlamaEngine::active_backend()` for the runtime-selected backend.
pub fn compiled_backend() -> &'static str {
    llama_sys::COMPILED_BACKEND
}

/// Why the build backend metadata was selected.
/// Dynamic release builds return `runtime`.
pub fn backend_selection_source() -> &'static str {
    llama_sys::BACKEND_SELECTION_SOURCE
}
