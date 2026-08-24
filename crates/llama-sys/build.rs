use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=LLAMA_CPP_SRC");
    println!("cargo:rerun-if-env-changed=LLAMA_CPP_REV");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_dir = llama_src_dir(&out_dir);
    let dst = build_llama(&src_dir);

    generate_bindings(&src_dir, &out_dir);
    emit_link_flags(&dst);
}

fn llama_src_dir(out_dir: &Path) -> PathBuf {
    if let Ok(path) = env::var("LLAMA_CPP_SRC") {
        let path = PathBuf::from(path);
        assert!(path.join("include/llama.h").exists(), "LLAMA_CPP_SRC missing include/llama.h");
        return path;
    }

    let src = out_dir.join("llama.cpp");
    let rev = env::var("LLAMA_CPP_REV").unwrap_or_else(|_| "master".to_string());
    if !src.join("include/llama.h").exists() {
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                &rev,
                "https://github.com/ggml-org/llama.cpp.git",
                src.to_str().unwrap(),
            ])
            .status()
            .expect("failed to spawn git");
        assert!(status.success(), "git clone llama.cpp failed");
    }
    src
}

fn build_llama(src: &Path) -> PathBuf {
    let mut config = cmake::Config::new(src);
    config
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("GGML_NATIVE", "OFF")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_TOOLS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF")
        .define("LLAMA_BUILD_COMMON", "OFF")
        .define("LLAMA_CURL", "OFF");

    if cfg!(feature = "cuda") {
        config.define("GGML_CUDA", "ON");
    }
    if cfg!(feature = "vulkan") {
        config.define("GGML_VULKAN", "ON");
    }
    if cfg!(feature = "metal") || cfg!(target_os = "macos") {
        config.define("GGML_METAL", "ON");
    }

    config.build()
}

fn generate_bindings(src: &Path, out_dir: &Path) {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", src.join("include").display()))
        .clang_arg(format!("-I{}", src.join("ggml/include").display()))
        .allowlist_function("llama_.*")
        .allowlist_type("llama_.*")
        .allowlist_var("LLAMA_.*")
        .blocklist_function("llama_log_set")
        .generate()
        .expect("bindgen failed");
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write bindings.rs");
}

fn emit_link_flags(dst: &Path) {
    let search_dirs = [
        dst.join("lib"),
        dst.join("lib64"),
        dst.join("build/src"),
        dst.join("build/ggml/src"),
        dst.join("build/ggml/src/ggml-cpu"),
    ];

    for dir in &search_dirs {
        if dir.exists() {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }

    let mut linked = std::collections::BTreeSet::new();
    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if let Some(lib) = static_lib_name(&name) {
                    if linked.insert(lib.to_string()) {
                        println!("cargo:rustc-link-lib=static={lib}");
                    }
                }
            }
        }
    }

    if linked.is_empty() {
        println!("cargo:rustc-link-lib=static=llama");
        println!("cargo:rustc-link-lib=static=ggml");
        println!("cargo:rustc-link-lib=static=ggml-base");
        println!("cargo:rustc-link-lib=static=ggml-cpu");
    }

    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=shell32");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalKit");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=gomp");
    }
}

fn static_lib_name(file: &str) -> Option<&str> {
    if let Some(stem) = file.strip_suffix(".a") {
        return Some(stem.strip_prefix("lib").unwrap_or(stem));
    }
    if let Some(stem) = file.strip_suffix(".lib") {
        if stem.ends_with("dll") {
            return None;
        }
        return Some(stem.strip_prefix("lib").unwrap_or(stem));
    }
    None
}
