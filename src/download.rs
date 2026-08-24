use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;

/// Hugging Face download / cache options.
#[derive(Debug, Clone)]
pub struct HfDownload {
    pub repo: String,
    pub file: String,
    pub revision: String,
    pub model_dir: PathBuf,
    pub force: bool,
    pub show_progress: bool,
}

impl HfDownload {
    pub fn new(repo: impl Into<String>, file: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            file: file.into(),
            revision: "main".to_string(),
            model_dir: PathBuf::from("models"),
            force: false,
            show_progress: false,
        }
    }
}

/// Resolve a local GGUF path or download one from Hugging Face.
pub fn resolve_model_path(
    local_model: Option<&Path>,
    hf: Option<&HfDownload>,
) -> Result<PathBuf> {
    match (local_model, hf) {
        (Some(model), None) => {
            if !model.exists() {
                bail!("model does not exist: {}", model.display());
            }
            Ok(model.to_path_buf())
        }
        (None, Some(hf)) => download_huggingface_model(hf),
        (Some(_), Some(_)) => bail!("use either a local model path or a Hugging Face download, not both"),
        (None, None) => bail!("provide a local model path or Hugging Face repo/file"),
    }
}

/// Download a GGUF file from Hugging Face, or reuse the local cache.
pub fn download_huggingface_model(hf: &HfDownload) -> Result<PathBuf> {
    let file_name = Path::new(&hf.file)
        .file_name()
        .context("invalid Hugging Face file path")?;

    fs::create_dir_all(&hf.model_dir).with_context(|| {
        format!("failed to create model directory: {}", hf.model_dir.display())
    })?;

    let destination = hf.model_dir.join(file_name);
    if destination.exists() && !hf.force {
        if hf.show_progress {
            eprintln!("Using cached model: {}", destination.display());
        }
        return Ok(destination);
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/{}/{}?download=true",
        hf.repo, hf.revision, hf.file
    );

    if hf.show_progress {
        eprintln!("Downloading {}/{} ...", hf.repo, hf.file);
    }

    let client = Client::builder()
        .user_agent(concat!("llama-rust/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to create HTTP client")?;

    let mut request = client.get(&url);
    if let Some(token) = huggingface_token() {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    let mut response = request.send().context("Hugging Face download request failed")?;
    if !response.status().is_success() {
        bail!(
            "Hugging Face download failed with HTTP {} for {}/{}",
            response.status(),
            hf.repo,
            hf.file
        );
    }

    let total = response.content_length();
    let temporary = destination.with_extension("gguf.part");
    let mut output = File::create(&temporary)
        .with_context(|| format!("failed to create: {}", temporary.display()))?;

    let mut downloaded = 0_u64;
    let mut buffer = [0_u8; 1024 * 1024];

    loop {
        let read = response
            .read(&mut buffer)
            .context("failed while downloading model")?;
        if read == 0 {
            break;
        }

        output
            .write_all(&buffer[..read])
            .context("failed while writing downloaded model")?;
        downloaded += read as u64;

        if hf.show_progress {
            if let Some(total) = total {
                let percent = downloaded as f64 / total as f64 * 100.0;
                eprint!(
                    "\r{:.1}% ({}/{})",
                    percent,
                    format_bytes(downloaded),
                    format_bytes(total)
                );
            } else {
                eprint!("\r{} downloaded", format_bytes(downloaded));
            }
            io::stderr().flush()?;
        }
    }

    output.flush()?;
    drop(output);
    if hf.show_progress {
        eprintln!();
    }

    if destination.exists() {
        fs::remove_file(&destination)
            .with_context(|| format!("failed to replace: {}", destination.display()))?;
    }
    fs::rename(&temporary, &destination).with_context(|| {
        format!(
            "failed to move downloaded model from {} to {}",
            temporary.display(),
            destination.display()
        )
    })?;

    if hf.show_progress {
        eprintln!("Saved model: {}", destination.display());
    }
    Ok(destination)
}

fn huggingface_token() -> Option<String> {
    env::var("HF_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("HUGGING_FACE_HUB_TOKEN")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * KIB;
    const GIB: f64 = 1024.0 * MIB;

    let bytes_f = bytes as f64;
    if bytes_f >= GIB {
        format!("{:.2} GiB", bytes_f / GIB)
    } else if bytes_f >= MIB {
        format!("{:.2} MiB", bytes_f / MIB)
    } else if bytes_f >= KIB {
        format!("{:.2} KiB", bytes_f / KIB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::format_bytes;

    #[test]
    fn formats_byte_sizes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.00 KiB");
    }
}
