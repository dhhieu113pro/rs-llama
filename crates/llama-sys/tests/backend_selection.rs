#[path = "../src/backend.rs"]
mod backend;

use backend::{
    select_backend, vulkan_toolchain_ready, Backend, SelectionInput, SelectionSource,
};

fn input(target: &str) -> SelectionInput<'_> {
    SelectionInput {
        target,
        requested: None,
        feature_cuda: false,
        feature_vulkan: false,
        feature_metal: false,
        cuda_available: false,
        vulkan_available: false,
    }
}

#[test]
fn auto_prefers_metal_on_apple() {
    let result = select_backend(input("aarch64-apple-darwin")).unwrap();
    assert_eq!(result.backend, Backend::Metal);
    assert_eq!(result.source, SelectionSource::Auto);
}

#[test]
fn auto_prefers_cuda_over_vulkan() {
    let mut input = input("x86_64-pc-windows-msvc");
    input.cuda_available = true;
    input.vulkan_available = true;

    let result = select_backend(input).unwrap();

    assert_eq!(result.backend, Backend::Cuda);
    assert_eq!(result.source, SelectionSource::Auto);
}

#[test]
fn auto_uses_vulkan_when_cuda_is_unavailable() {
    let mut input = input("x86_64-unknown-linux-gnu");
    input.vulkan_available = true;

    let result = select_backend(input).unwrap();

    assert_eq!(result.backend, Backend::Vulkan);
}

#[test]
fn auto_falls_back_to_cpu() {
    let result = select_backend(input("x86_64-unknown-linux-gnu")).unwrap();
    assert_eq!(result.backend, Backend::Cpu);
}

#[test]
fn auto_keeps_android_on_cpu() {
    let mut input = input("aarch64-linux-android");
    input.cuda_available = true;
    input.vulkan_available = true;

    let result = select_backend(input).unwrap();

    assert_eq!(result.backend, Backend::Cpu);
}

#[test]
fn environment_override_forces_backend() {
    let mut input = input("x86_64-unknown-linux-gnu");
    input.requested = Some("vulkan");

    let result = select_backend(input).unwrap();

    assert_eq!(result.backend, Backend::Vulkan);
    assert_eq!(result.source, SelectionSource::Environment);
}

#[test]
fn cargo_feature_forces_backend() {
    let mut input = input("x86_64-unknown-linux-gnu");
    input.feature_cuda = true;

    let result = select_backend(input).unwrap();

    assert_eq!(result.backend, Backend::Cuda);
    assert_eq!(result.source, SelectionSource::Feature);
}

#[test]
fn conflicting_features_are_rejected() {
    let mut input = input("x86_64-unknown-linux-gnu");
    input.feature_cuda = true;
    input.feature_vulkan = true;

    assert!(select_backend(input).is_err());
}

#[test]
fn feature_and_non_auto_environment_override_are_rejected() {
    let mut input = input("x86_64-unknown-linux-gnu");
    input.feature_cuda = true;
    input.requested = Some("cpu");

    assert!(select_backend(input).is_err());
}

#[test]
fn invalid_environment_override_is_rejected() {
    let mut input = input("x86_64-unknown-linux-gnu");
    input.requested = Some("directml");

    assert!(select_backend(input).is_err());
}

#[test]
fn vulkan_toolchain_requires_glslc() {
    assert!(!vulkan_toolchain_ready(true, false, true));
}

#[test]
fn vulkan_toolchain_requires_spirv_headers() {
    assert!(!vulkan_toolchain_ready(true, true, false));
}

#[test]
fn vulkan_toolchain_is_ready_when_all_requirements_exist() {
    assert!(vulkan_toolchain_ready(true, true, true));
}

#[test]
fn dynamic_mode_rejects_static_gpu_features() {
    assert!(backend::validate_build_mode(true, true, false, false).is_err());
    assert!(backend::validate_build_mode(true, false, true, false).is_err());
    assert!(backend::validate_build_mode(true, false, false, true).is_err());
}

#[test]
fn dynamic_mode_accepts_no_static_gpu_feature() {
    assert!(backend::validate_build_mode(true, false, false, false).is_ok());
}

#[test]
fn dynamic_desktop_backend_sets_match_release_contract() {
    assert_eq!(
        backend::required_dynamic_backends("x86_64-pc-windows-msvc"),
        &[Backend::Cpu, Backend::Cuda, Backend::Vulkan]
    );
    assert_eq!(
        backend::required_dynamic_backends("x86_64-unknown-linux-gnu"),
        &[Backend::Cpu, Backend::Cuda, Backend::Vulkan]
    );
    assert_eq!(
        backend::required_dynamic_backends("aarch64-apple-darwin"),
        &[Backend::Cpu, Backend::Metal]
    );
    assert_eq!(
        backend::required_dynamic_backends("aarch64-linux-android"),
        &[Backend::Cpu]
    );
}
