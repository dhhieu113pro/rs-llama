use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Once;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeBackend {
    Cpu,
    Cuda,
    Vulkan,
    Metal,
    Other(String),
}

impl fmt::Display for RuntimeBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cpu => write!(f, "CPU"),
            Self::Cuda => write!(f, "CUDA"),
            Self::Vulkan => write!(f, "Vulkan"),
            Self::Metal => write!(f, "Metal"),
            Self::Other(name) => write!(f, "{name}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDevice {
    pub name: String,
    pub description: String,
    pub backend: RuntimeBackend,
    pub is_gpu: bool,
}

pub(crate) struct SelectedDevices {
    pub backend: RuntimeBackend,
    pub devices: Vec<llama_sys::ggml_backend_dev_t>,
    pub snapshot: Vec<RuntimeDevice>,
}

pub(crate) fn ensure_backend_initialized() {
    static BACKEND: Once = Once::new();
    BACKEND.call_once(|| unsafe {
        llama_sys::llama_backend_init();
    });
}

pub fn runtime_devices() -> Vec<RuntimeDevice> {
    ensure_backend_initialized();
    unsafe {
        enumerate_raw_devices()
            .into_iter()
            .map(|(_, device)| device)
            .collect()
    }
}

pub(crate) fn select_devices_for_model(gpu_layers: u32) -> SelectedDevices {
    ensure_backend_initialized();

    let raw = unsafe { enumerate_raw_devices() };
    let snapshot = raw.iter().map(|(_, device)| device.clone()).collect();

    if gpu_layers == 0 {
        return SelectedDevices {
            backend: RuntimeBackend::Cpu,
            devices: Vec::new(),
            snapshot,
        };
    }

    #[cfg(target_os = "macos")]
    let priorities = [RuntimeBackend::Metal];
    #[cfg(not(target_os = "macos"))]
    let priorities = [RuntimeBackend::Cuda, RuntimeBackend::Vulkan];

    for preferred in priorities {
        let mut devices: Vec<llama_sys::ggml_backend_dev_t> = raw
            .iter()
            .filter(|(_, device)| device.is_gpu && device.backend == preferred)
            .map(|(raw, _)| *raw)
            .collect();

        if !devices.is_empty() {
            devices.push(ptr::null_mut());
            return SelectedDevices {
                backend: preferred,
                devices,
                snapshot,
            };
        }
    }

    SelectedDevices {
        backend: RuntimeBackend::Cpu,
        devices: Vec::new(),
        snapshot,
    }
}

unsafe fn enumerate_raw_devices() -> Vec<(llama_sys::ggml_backend_dev_t, RuntimeDevice)> {
    let mut result = Vec::new();
    let count = llama_sys::ggml_backend_dev_count();

    for index in 0..count {
        let device = llama_sys::ggml_backend_dev_get(index);
        if device.is_null() {
            continue;
        }

        let name = c_string(llama_sys::ggml_backend_dev_name(device));
        let description = c_string(llama_sys::ggml_backend_dev_description(device));
        let reg = llama_sys::ggml_backend_dev_backend_reg(device);
        let registry_name = if reg.is_null() {
            String::new()
        } else {
            c_string(llama_sys::ggml_backend_reg_name(reg))
        };
        let backend = classify_backend(if registry_name.is_empty() {
            &name
        } else {
            &registry_name
        });
        let device_type = llama_sys::ggml_backend_dev_type(device);
        let is_gpu = device_type == llama_sys::GGML_BACKEND_DEVICE_TYPE_GPU
            || device_type == llama_sys::GGML_BACKEND_DEVICE_TYPE_IGPU;

        result.push((
            device,
            RuntimeDevice {
                name,
                description,
                backend,
                is_gpu,
            },
        ));
    }

    result
}

unsafe fn c_string(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    CStr::from_ptr(value).to_string_lossy().into_owned()
}

fn classify_backend(name: &str) -> RuntimeBackend {
    let upper = name.to_ascii_uppercase();
    if upper.contains("CUDA") {
        RuntimeBackend::Cuda
    } else if upper.contains("VULKAN") || upper == "VK" {
        RuntimeBackend::Vulkan
    } else if upper.contains("METAL") {
        RuntimeBackend::Metal
    } else if upper.contains("CPU") {
        RuntimeBackend::Cpu
    } else {
        RuntimeBackend::Other(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_backend_names() {
        assert_eq!(classify_backend("CUDA0"), RuntimeBackend::Cuda);
        assert_eq!(classify_backend("Vulkan0"), RuntimeBackend::Vulkan);
        assert_eq!(classify_backend("Metal"), RuntimeBackend::Metal);
        assert_eq!(classify_backend("CPU"), RuntimeBackend::Cpu);
    }

    #[test]
    fn keeps_unknown_backend_name() {
        assert_eq!(
            classify_backend("SYCL0"),
            RuntimeBackend::Other("SYCL0".to_string())
        );
    }
}
