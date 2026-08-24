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
    /// Explicit mmproj filename inside the repo. If empty, auto-detect.
    pub mmproj_file: Option<String>,
    /// Download a matching mmproj when the repo contains one.
    pub auto_mmproj: bool,
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
            mmproj_file: None,
            auto_mmproj: true,
        }
    }
}

/// Local model file plus optional multimodal projector.
#[derive(Debug, Clone)]
pub struct ResolvedModel {
    pub model_path: PathBuf,
    pub mmproj_path: Option<PathBuf>,
}

/// Resolve a local GGUF path or download one from Hugging Face.
pub fn resolve_model_path(
    local_model: Option<&Path>,
    hf: Option<&HfDownload>,
) -> Result<PathBuf> {
    Ok(resolve_model_files(local_model, hf, None)?.model_path)
}

/// Resolve the language model and an optional mmproj file.
pub fn resolve_model_files(
    local_model: Option<&Path>,
    hf: Option<&HfDownload>,
    local_mmproj: Option<&Path>,
) -> Result<ResolvedModel> {
    match (local_model, hf) {
        (Some(model), None) => {
            if !model.exists() {
                bail!("model does not exist: {}", model.display());
            }
            let mmproj_path = match local_mmproj {
                Some(path) => {
                    if !path.exists() {
                        bail!("mmproj does not exist: {}", path.display());
                    }
                    Some(path.to_path_buf())
                }
                None => find_local_mmproj(model),
            };
            Ok(ResolvedModel {
                model_path: model.to_path_buf(),
                mmproj_path,
            })
        }
        (None, Some(hf)) => {
            let mut bundle = download_huggingface_model_bundle(hf)?;
            if let Some(path) = local_mmproj {
                if !path.exists() {
                    bail!("mmproj does not exist: {}", path.display());
                }
                bundle.mmproj_path = Some(path.to_path_buf());
            }
            Ok(bundle)
        }
        (Some(_), Some(_)) => {
            bail!("use either a local model path or a Hugging Face download, not both")
        }
        (None, None) => bail!("provide a local model path or Hugging Face repo/file"),
    }
}

/// Download a GGUF file from Hugging Face, or reuse the local cache.
pub fn download_huggingface_model(hf: &HfDownload) -> Result<PathBuf> {
    Ok(download_huggingface_model_bundle(hf)?.model_path)
}

/// Download the language model and auto-detect/download mmproj when present.
pub fn download_huggingface_model_bundle(hf: &HfDownload) -> Result<ResolvedModel> {
    let model_path = download_hf_file(hf, &hf.file)?;
    let mmproj_path = resolve_hf_mmproj(hf)?;
    Ok(ResolvedModel {
        model_path,
        mmproj_path,
    })
}

fn resolve_hf_mmproj(hf: &HfDownload) -> Result<Option<PathBuf>> {
    if !hf.auto_mmproj && hf.mmproj_file.is_none() {
        return Ok(None);
    }

    let mmproj_file = if let Some(file) = &hf.mmproj_file {
        Some(file.clone())
    } else {
        find_hf_mmproj(hf)?
    };

    match mmproj_file {
        Some(file) => {
            if hf.show_progress {
                eprintln!("Found vision projector: {file}");
            }
            Ok(Some(download_hf_file(hf, &file)?))
        }
        None => {
            if hf.show_progress {
                eprintln!("No mmproj file found in {}.", hf.repo);
            }
            Ok(None)
        }
    }
}

fn find_hf_mmproj(hf: &HfDownload) -> Result<Option<String>> {
    let files = list_hf_files(hf)?;
    Ok(pick_mmproj_file(&files, &hf.file))
}

fn list_hf_files(hf: &HfDownload) -> Result<Vec<String>> {
    let url = format!(
        "https://huggingface.co/api/models/{}/tree/{}?recursive=1",
        hf.repo, hf.revision
    );
    let client = hf_client()?;
    let mut request = client.get(&url);
    if let Some(token) = huggingface_token() {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = request.send().context("failed to list Hugging Face repo files")?;
    if !response.status().is_success() {
        bail!(
            "failed to list files in {} (HTTP {})",
            hf.repo,
            response.status()
        );
    }
    let value: serde_json::Value = response.json().context("invalid Hugging Face tree JSON")?;
    let mut files = Vec::new();
    if let Some(items) = value.as_array() {
        for item in items {
            let is_file = item.get("type").and_then(|v| v.as_str()) == Some("file");
            if let Some(path) = item.get("path").and_then(|v| v.as_str()) {
                if is_file || path.ends_with(".gguf") {
                    files.push(path.to_string());
                }
            }
        }
    }
    Ok(files)
}

fn pick_mmproj_file(files: &[String], model_file: &str) -> Option<String> {
    let model_dir = Path::new(model_file)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|p| !p.is_empty() && p != ".");

    let mut candidates: Vec<&String> = files
        .iter()
        .filter(|path| {
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_ascii_lowercase();
            name.contains("mmproj") && name.ends_with(".gguf")
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by_key(|path| {
        let lower = path.to_ascii_lowercase();
        let same_dir = match &model_dir {
            Some(dir) => path.replace('\\', "/").starts_with(&format!("{dir}/")),
            None => !path.contains('/'),
        };
        let quality = if lower.contains("f16") || lower.contains("fp16") {
            0
        } else if lower.contains("bf16") {
            1
        } else if lower.contains("f32") || lower.contains("fp32") {
            2
        } else if lower.contains("q8") {
            3
        } else {
            4
        };
        (if same_dir { 0 } else { 1 }, quality, path.len())
    });

    candidates.first().copied().cloned()
}

fn find_local_mmproj(model: &Path) -> Option<PathBuf> {
    let dir = model.parent()?;
    let mut matches = Vec::new();
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if name.contains("mmproj") && name.ends_with(".gguf") {
            matches.push(path);
        }
    }
    pick_mmproj_file(
        &matches
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        model.to_string_lossy().as_ref(),
    )
    .map(PathBuf::from)
    .or_else(|| matches.into_iter().next())
}

fn download_hf_file(hf: &HfDownload, file: &str) -> Result<PathBuf> {
    let file_name = Path::new(file)
        .file_name()
        .context("invalid Hugging Face file path")?;

    fs::create_dir_all(&hf.model_dir).with_context(|| {
        format!("failed to create model directory: {}", hf.model_dir.display())
    })?;

    let destination = hf.model_dir.join(file_name);
    if destination.exists() && !hf.force {
        if hf.show_progress {
            eprintln!("Using cached file: {}", destination.display());
        }
        return Ok(destination);
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/{}/{}?download=true",
        hf.repo, hf.revision, file
    );

    if hf.show_progress {
        eprintln!("Downloading {}/{} ...", hf.repo, file);
    }

    let client = hf_client()?;
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
            file
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
            "failed to move downloaded file from {} to {}",
            temporary.display(),
            destination.display()
        )
    })?;

    if hf.show_progress {
        eprintln!("Saved: {}", destination.display());
    }
    Ok(destination)
}

fn hf_client() -> Result<Client> {
    Client::builder()
        .user_agent(concat!("rs-llama/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to create HTTP client")
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
    use super::{format_bytes, pick_mmproj_file};

    #[test]
    fn formats_byte_sizes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.00 KiB");
    }

    #[test]
    fn picks_mmproj_next_to_model() {
        let files = vec![
            "model-Q4_K_M.gguf".to_string(),
            "mmproj-F16.gguf".to_string(),
            "other/mmproj-Q4_0.gguf".to_string(),
        ];
        assert_eq!(
            pick_mmproj_file(&files, "model-Q4_K_M.gguf").as_deref(),
            Some("mmproj-F16.gguf")
        );
    }
}
