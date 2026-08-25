use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "src/backend.rs"]
mod backend;

use backend::{select_backend, Backend, Selection, SelectionInput};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/backend.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=LLAMA_CPP_SRC");
    println!("cargo:rerun-if-env-changed=LLAMA_CPP_REV");
    println!("cargo:rerun-if-env-changed=RS_LLAMA_BACKEND");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");
    println!("cargo:rerun-if-env-changed=CUDA_HOME");
    println!("cargo:rerun-if-env-changed=CUDA_ROOT");
    println!("cargo:rerun-if-env-changed=VULKAN_SDK");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-env-changed=ANDROID_NDK_HOME");
    println!("cargo:rerun-if-env-changed=ANDROID_NDK");
    println!("cargo:rerun-if-env-changed=NDK_HOME");
    println!("cargo:rerun-if-env-changed=INCLUDE");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap_or_default();
    let selection = detect_backend(&target).unwrap_or_else(|err| panic!("{err}"));

    println!(
        "cargo:warning=rs-llama backend: {} ({})",
        selection.backend.as_str(),
        selection.source.as_str()
    );
    println!(
        "cargo:rustc-env=RS_LLAMA_COMPILED_BACKEND={}",
        selection.backend.as_str()
    );
    println!(
        "cargo:rustc-env=RS_LLAMA_BACKEND_SOURCE={}",
        selection.source.as_str()
    );

    let src_dir = llama_src_dir(&out_dir);
    let dst = build_llama(&src_dir, &target, selection.backend);

    generate_bindings(&src_dir, &out_dir, &target);
    emit_link_flags(&dst, &target, selection.backend);
}

fn detect_backend(target: &str) -> Result<Selection, String> {
    let host = env::var("HOST").unwrap_or_default();
    let native_build = same_platform_family(&host, target);

    select_backend(SelectionInput {
        target,
        requested: env::var("RS_LLAMA_BACKEND").ok().as_deref(),
        feature_cuda: cfg!(feature = "cuda"),
        feature_vulkan: cfg!(feature = "vulkan"),
        feature_metal: cfg!(feature = "metal"),
        cuda_available: native_build && cuda_toolkit_available(target),
        vulkan_available: native_build && vulkan_toolkit_available(target),
    })
}

fn same_platform_family(host: &str, target: &str) -> bool {
    fn family(triple: &str) -> &'static str {
        if triple.contains("windows") {
            "windows"
        } else if triple.contains("apple") {
            "apple"
        } else if triple.contains("android") {
            "android"
        } else {
            "unix"
        }
    }

    family(host) == family(target)
}

fn cuda_toolkit_available(target: &str) -> bool {
    if target.contains("android") || target.contains("apple") {
        return false;
    }

    cuda_root(target).is_some()
}

fn cuda_root(target: &str) -> Option<PathBuf> {
    let nvcc_name = if target.contains("windows") {
        "nvcc.exe"
    } else {
        "nvcc"
    };

    let valid_root = |root: &Path| {
        root.join("bin").join(nvcc_name).exists() || root.join("include/cuda.h").exists()
    };

    if let Some(root) = ["CUDA_PATH", "CUDA_HOME", "CUDA_ROOT"]
        .into_iter()
        .filter_map(|key| env::var_os(key).map(PathBuf::from))
        .find(|root| valid_root(root))
    {
        return Some(root);
    }

    if !target.contains("windows") {
        let root = PathBuf::from("/usr/local/cuda");
        if valid_root(&root) {
            return Some(root);
        }
    }

    command_path(nvcc_name, target)
        .and_then(|nvcc| nvcc.parent().and_then(Path::parent).map(Path::to_path_buf))
        .filter(|root| valid_root(root))
}

fn vulkan_toolkit_available(target: &str) -> bool {
    if target.contains("android") || target.contains("apple") {
        return false;
    }

    if vulkan_sdk_root().is_some() {
        return true;
    }

    if target.contains("windows") {
        return false;
    }

    command_succeeds("pkg-config", &["--exists", "vulkan"])
        || (Path::new("/usr/include/vulkan/vulkan.h").exists()
            && (Path::new("/usr/lib/libvulkan.so").exists()
                || Path::new("/usr/lib64/libvulkan.so").exists()
                || Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so").exists()
                || Path::new("/usr/lib/aarch64-linux-gnu/libvulkan.so").exists()))
}

fn vulkan_sdk_root() -> Option<PathBuf> {
    env::var_os("VULKAN_SDK")
        .map(PathBuf::from)
        .filter(|root| {
            root.join("include/vulkan/vulkan.h").exists()
                || root.join("Include/vulkan/vulkan.h").exists()
        })
}

fn command_path(program: &str, target: &str) -> Option<PathBuf> {
    let locator = if target.contains("windows") {
        "where"
    } else {
        "which"
    };
    let output = Command::new(locator).arg(program).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
}

fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn llama_src_dir(out_dir: &Path) -> PathBuf {
    if let Ok(path) = env::var("LLAMA_CPP_SRC") {
        let path = PathBuf::from(path);
        assert!(
            path.join("include/llama.h").exists(),
            "LLAMA_CPP_SRC missing include/llama.h"
        );
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

fn android_ndk() -> Option<PathBuf> {
    ["ANDROID_NDK_HOME", "ANDROID_NDK", "NDK_HOME"]
        .into_iter()
        .find_map(|key| env::var_os(key).map(PathBuf::from))
}

fn android_abi(target: &str) -> &'static str {
    if target.starts_with("aarch64-") {
        "arm64-v8a"
    } else if target.starts_with("armv7-") {
        "armeabi-v7a"
    } else if target.starts_with("x86_64-") {
        "x86_64"
    } else {
        "x86"
    }
}

fn build_llama(src: &Path, target: &str, backend: Backend) -> PathBuf {
    let mut config = cmake::Config::new(src);
    config
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("GGML_NATIVE", "OFF")
        .define("GGML_CCACHE", "OFF")
        .define("GGML_CUDA", "OFF")
        .define("GGML_VULKAN", "OFF")
        .define("GGML_METAL", "OFF")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_TOOLS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF")
        .define("LLAMA_BUILD_COMMON", "OFF")
        .define("LLAMA_BUILD_APP", "OFF")
        .define("LLAMA_CURL", "OFF");

    match backend {
        Backend::Cuda => {
            config.define("GGML_CUDA", "ON");
        }
        Backend::Vulkan => {
            config.define("GGML_VULKAN", "ON");
        }
        Backend::Metal if !target.contains("android") => {
            config.define("GGML_METAL", "ON");
        }
        Backend::Cpu | Backend::Metal => {}
    }

    if target.contains("android") {
        let ndk = android_ndk()
            .expect("ANDROID_NDK_HOME / ANDROID_NDK / NDK_HOME required for Android");
        let toolchain = ndk.join("build/cmake/android.toolchain.cmake");
        config
            .define("CMAKE_TOOLCHAIN_FILE", toolchain.to_string_lossy().as_ref())
            .define("ANDROID_ABI", android_abi(target))
            .define("ANDROID_PLATFORM", "android-28")
            .define("GGML_OPENMP", "OFF");
    }

    // Windows (especially ARM64) often fails to link OpenMP runtime (__kmpc_* symbols).
    // Disable it for reliability, matching the Android approach.
    if target.contains("windows") {
        config.define("GGML_OPENMP", "OFF");
    }

    config.build()
}

/// Discover MSVC INCLUDE paths so bindgen's libclang can find stdbool.h / stddef.h.
/// Matches the approach used by utilityai/llama-cpp-rs#839.
fn msvc_include_paths(out_dir: &Path) -> Vec<String> {
    let mut paths = Vec::new();

    // Prefer INCLUDE already set (Developer Prompt / vcvars / CI).
    if let Ok(include) = env::var("INCLUDE") {
        for p in include
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            paths.push(p.to_string());
        }
        if !paths.is_empty() {
            return paths;
        }
    }

    // Otherwise ask the `cc` crate — it bootstraps the MSVC environment.
    let dummy = out_dir.join("bindgen_dummy.c");
    if fs::write(&dummy, "int main(void) { return 0; }").is_ok() {
        let mut build = cc::Build::new();
        build.file(&dummy);
        if let Ok(compiler) = build.try_get_compiler() {
            if let Some((_, value)) = compiler
                .env()
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("INCLUDE"))
            {
                for p in value
                    .to_string_lossy()
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    paths.push(p.to_string());
                }
            }
        }
    }

    paths
}

fn generate_bindings(src: &Path, out_dir: &Path, target: &str) {
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", src.join("include").display()))
        .clang_arg(format!("-I{}", src.join("ggml/include").display()))
        .allowlist_function("llama_.*")
        .allowlist_type("llama_.*")
        .allowlist_var("LLAMA_.*")
        .blocklist_function("llama_log_set");

    if target.contains("android") {
        if let Some(ndk) = android_ndk() {
            let host = if cfg!(target_os = "macos") {
                "darwin-x86_64"
            } else {
                "linux-x86_64"
            };
            let prebuilt = ndk.join("toolchains/llvm/prebuilt").join(host);
            let sysroot = prebuilt.join("sysroot");
            if sysroot.exists() {
                builder = builder.clang_arg(format!("--sysroot={}", sysroot.display()));
            }
            builder = builder.clang_arg(format!("--target={target}"));
        }
    }

    // Windows MSVC: libclang does not inherit MSVC system includes by default,
    // which causes fatal errors like "stdbool.h file not found" in ggml.h.
    if target.contains("windows") && target.contains("msvc") {
        for include in msvc_include_paths(out_dir) {
            builder = builder.clang_arg("-isystem").clang_arg(include);
        }
        builder = builder
            .clang_arg(format!("--target={target}"))
            .clang_arg("-fms-compatibility")
            .clang_arg("-fms-extensions");
    }

    builder
        .generate()
        .expect("bindgen failed")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write bindings.rs");
}

fn emit_link_flags(dst: &Path, target: &str, backend: Backend) {
    let search_dirs = [
        dst.join("lib"),
        dst.join("lib64"),
        dst.join("build/src"),
        dst.join("build/ggml/src"),
        dst.join("build/ggml/src/ggml-cpu"),
        dst.join("build/ggml/src/ggml-cuda"),
        dst.join("build/ggml/src/ggml-vulkan"),
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

    if backend == Backend::Vulkan {
        emit_vulkan_link_flags(target);
    }

    if backend == Backend::Cuda {
        emit_cuda_link_flags(target);
    }

    if target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=shell32");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=Foundation");
        if backend == Backend::Metal {
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=MetalKit");
        }
    } else if target.contains("android") {
        println!("cargo:rustc-link-lib=dylib=c++_shared");
        println!("cargo:rustc-link-lib=dylib=log");
        println!("cargo:rustc-link-lib=dylib=android");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=m");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=gomp");
    }
}

fn emit_vulkan_link_flags(target: &str) {
    if let Some(root) = vulkan_sdk_root() {
        let candidates = if target.contains("windows") {
            if target.starts_with("i686-") {
                vec![root.join("Lib32")]
            } else {
                vec![root.join("Lib")]
            }
        } else {
            vec![
                root.join("lib"),
                root.join("lib64"),
                root.join("x86_64/lib"),
                root.join("aarch64/lib"),
            ]
        };

        for dir in candidates {
            if dir.exists() {
                println!("cargo:rustc-link-search=native={}", dir.display());
            }
        }
    }

    if target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=vulkan-1");
    } else if !target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=vulkan");
    }
}

fn emit_cuda_link_flags(target: &str) {
    if let Some(root) = cuda_root(target) {
        let mut search_dirs = Vec::new();
        if target.contains("windows") {
            search_dirs.push(root.join("lib/x64"));
        } else {
            search_dirs.push(root.join("lib64"));
            if target.starts_with("aarch64-") {
                search_dirs.push(root.join("targets/aarch64-linux/lib"));
            } else {
                search_dirs.push(root.join("targets/x86_64-linux/lib"));
            }

            for dir in search_dirs.clone() {
                search_dirs.push(dir.join("stubs"));
            }
        }

        for dir in search_dirs {
            if dir.exists() {
                println!("cargo:rustc-link-search=native={}", dir.display());
            }
        }
    }

    if target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=cudart");
        println!("cargo:rustc-link-lib=dylib=cublas");
        println!("cargo:rustc-link-lib=dylib=cublasLt");
        println!("cargo:rustc-link-lib=dylib=cuda");
    } else {
        println!("cargo:rustc-link-lib=static=cudart_static");
        println!("cargo:rustc-link-lib=static=cublas_static");
        println!("cargo:rustc-link-lib=static=cublasLt_static");
        println!("cargo:rustc-link-lib=dylib=cuda");
        println!("cargo:rustc-link-lib=static=culibos");
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
