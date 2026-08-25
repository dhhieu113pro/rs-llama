#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    unused_imports,
    clippy::all
)]

/// llama.cpp acceleration backend selected when this crate was built.
pub const COMPILED_BACKEND: &str = env!("RS_LLAMA_COMPILED_BACKEND");

/// Why the compiled backend was selected: `auto`, `environment`, or `feature`.
pub const BACKEND_SELECTION_SOURCE: &str = env!("RS_LLAMA_BACKEND_SOURCE");

/// Whether this build uses llama.cpp runtime-loadable backend modules.
pub const DYNAMIC_BACKENDS: bool = cfg!(feature = "dynamic-backends");

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn dynamic_backend_metadata_matches_feature() {
        assert_eq!(super::DYNAMIC_BACKENDS, cfg!(feature = "dynamic-backends"));
    }
}
