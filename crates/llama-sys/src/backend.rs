#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Cpu,
    Cuda,
    Vulkan,
    Metal,
}

impl Backend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Cuda => "cuda",
            Self::Vulkan => "vulkan",
            Self::Metal => "metal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSource {
    Auto,
    Environment,
    Feature,
}

impl SelectionSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Environment => "environment",
            Self::Feature => "feature",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub backend: Backend,
    pub source: SelectionSource,
}

#[derive(Debug, Clone, Copy)]
pub struct SelectionInput<'a> {
    pub target: &'a str,
    pub requested: Option<&'a str>,
    pub feature_cuda: bool,
    pub feature_vulkan: bool,
    pub feature_metal: bool,
    pub cuda_available: bool,
    pub vulkan_available: bool,
}

pub fn select_backend(input: SelectionInput<'_>) -> Result<Selection, String> {
    let feature_count = [
        input.feature_cuda,
        input.feature_vulkan,
        input.feature_metal,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();

    if feature_count > 1 {
        return Err("enable only one of the cuda, vulkan, or metal Cargo features".to_string());
    }

    let requested = input.requested.map(str::trim).filter(|value| !value.is_empty());
    if let Some(value) = requested {
        if !matches!(value, "auto" | "cpu" | "cuda" | "vulkan" | "metal") {
            return Err(format!(
                "invalid RS_LLAMA_BACKEND '{value}'; expected auto, cpu, cuda, vulkan, or metal"
            ));
        }
        if value != "auto" && feature_count != 0 {
            return Err(
                "RS_LLAMA_BACKEND cannot force a backend when a GPU Cargo feature is enabled"
                    .to_string(),
            );
        }
        if value != "auto" {
            return Ok(Selection {
                backend: match value {
                    "cpu" => Backend::Cpu,
                    "cuda" => Backend::Cuda,
                    "vulkan" => Backend::Vulkan,
                    "metal" => Backend::Metal,
                    _ => unreachable!(),
                },
                source: SelectionSource::Environment,
            });
        }
    }

    if input.feature_cuda {
        return Ok(Selection {
            backend: Backend::Cuda,
            source: SelectionSource::Feature,
        });
    }
    if input.feature_vulkan {
        return Ok(Selection {
            backend: Backend::Vulkan,
            source: SelectionSource::Feature,
        });
    }
    if input.feature_metal {
        return Ok(Selection {
            backend: Backend::Metal,
            source: SelectionSource::Feature,
        });
    }

    let backend = if input.target.contains("android") {
        Backend::Cpu
    } else if input.target.contains("apple") {
        Backend::Metal
    } else if input.cuda_available {
        Backend::Cuda
    } else if input.vulkan_available {
        Backend::Vulkan
    } else {
        Backend::Cpu
    };

    Ok(Selection {
        backend,
        source: SelectionSource::Auto,
    })
}
