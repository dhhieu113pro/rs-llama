//! `rs-llama` library API for loading GGUF models and running local inference.
//!
//! The CLI binary is `rs-llama`. Import the library as `rs_llama`.

mod download;
mod engine;

pub use download::{
    download_huggingface_model, download_huggingface_model_bundle, resolve_model_files,
    resolve_model_path, HfDownload, ResolvedModel,
};
pub use engine::{EngineConfig, GenerateRequest, LlamaEngine};
