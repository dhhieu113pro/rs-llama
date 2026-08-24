//! Library API for loading GGUF models and running local inference.
//!
//! The CLI in `src/main.rs` is a thin wrapper around this crate.

mod download;
mod engine;

pub use download::{download_huggingface_model, resolve_model_path, HfDownload};
pub use engine::{EngineConfig, GenerateRequest, LlamaEngine};
