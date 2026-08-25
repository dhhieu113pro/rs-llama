use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Once;

unsafe extern "C" {
    fn ggml_backend_dev_count() -> usize;
    fn ggml_backend_dev_get(index: usize) -> llama_sys::ggml_backend_dev_t;
    fn ggml_backend_dev_name(device: llama_sys::ggml_backend_dev_t) -> *const c_char;
    fn ggml_backend_dev_description(device: llama_sys::ggml_backend_dev_t) -> *const c_char;
}

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

pub(crate) struct DeviceCandidate {
    pub backend: RuntimeBackend,
    pub devices: Vec<llama_sys::ggml_backend_dev_t>,
}

pub(crate) struct DevicePlan {
    pub candidates: Vec<DeviceCandidate>,
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

pub(crate) fn device_plan_for_model(gpu_layers: u32) -> DevicePlan {
    ensure_backend_initialized();

    let raw = unsafe { enumerate_raw_devices() };
    let snapshot = raw.iter().map(|(_, device)| device.clone()).collect();
    let mut candidates = Vec::new();

    if gpu_layers > 0 {
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
                candidates.push(DeviceCandidate {
                    backend: preferred,
                    devices,
                });
            }
        }
    }

    candidates.push(DeviceCandidate {
        backend: RuntimeBackend::Cpu,
        devices: Vec::new(),
    });

    DevicePlan {
        candidates,
        snapshot,
    }
}

unsafe fn enumerate_raw_devices() -> Vec<(llama_sys::ggml_backend_dev_t, RuntimeDevice)> {
    let mut result = Vec::new();
    let count = ggml_backend_dev_count();

    for index in 0..count {
        let device = ggml_backend_dev_get(index);
        if device.is_null() {
            continue;
        }

        let name = c_string(ggml_backend_dev_name(device));
        let description = c_string(ggml_backend_dev_description(device));
        let backend = classify_backend(&name);
        let is_gpu = matches!(
            backend,
            RuntimeBackend::Cuda | RuntimeBackend::Vulkan | RuntimeBackend::Metal
        );

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
