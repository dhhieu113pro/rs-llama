//! `rs-llama` library API for loading GGUF models and running local inference.
//!
//! The CLI binary is `rs-llama`. Import the library as `rs_llama`.

mod download;
mod engine;

pub use download::{download_huggingface_model, resolve_model_path, HfDownload};
pub use engine::{EngineConfig, GenerateRequest, LlamaEngine};
