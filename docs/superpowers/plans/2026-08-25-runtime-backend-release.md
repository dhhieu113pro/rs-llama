# Runtime GPU Backend Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish one rs-llama desktop archive per platform that loads the best usable llama.cpp GPU backend at runtime and falls back to CPU without requiring another download.

**Architecture:** Keep the existing static/backend-specific build path for crates.io and developer builds, and add an explicit `dynamic-backends` release mode in `rs-llama-sys`. In that mode llama.cpp builds a shared core with `GGML_BACKEND_DL=ON`; release CI stages the executable, core libraries, CPU backend, and required GPU backend modules next to each other so llama.cpp's native loader performs runtime discovery. Rust reports runtime devices after `llama_backend_init()` rather than treating the build-time backend label as runtime truth.

**Tech Stack:** Rust 2021, Cargo features, llama.cpp CMake, bindgen, GitHub Actions, Bash/PowerShell packaging, clap CLI.

**Spec:** `docs/superpowers/specs/2026-08-25-runtime-backend-release-design.md`

## Global Constraints

- Windows/Linux runtime priority: CUDA when usable, otherwise Vulkan when usable, otherwise CPU.
- macOS runtime priority: Metal when usable, otherwise CPU.
- Android remains CPU-only in this implementation.
- Keep one normal archive per supported platform; do not publish separate normal CPU/CUDA/Vulkan binaries.
- `EngineConfig::new()` keeps `gpu_layers = 999`; `with_gpu_layers(0)` remains explicit CPU-only inference.
- Do not bundle the full CUDA runtime/toolkit in the standard release archive.
- Windows/Linux desktop release packaging must fail if CPU, Vulkan, or CUDA backend modules are missing.
- macOS desktop release packaging must fail if CPU or Metal backend support is missing.
- Keep existing static Cargo features (`cuda`, `vulkan`, `metal`) for developer/deterministic builds.
- Dynamic backend distribution is opt-in and must not become mandatory for normal crates.io consumers.
- Runtime backend loading must use llama.cpp's supported loader; do not implement a custom Rust backend ABI loader.

---

## File Structure

- `Cargo.toml` — expose the top-level `dynamic-backends` feature and forward it to `rs-llama-sys`.
- `crates/llama-sys/Cargo.toml` — define the `dynamic-backends` feature independently from static GPU features.
- `crates/llama-sys/src/backend.rs` — keep static build-selection policy and add small pure policy helpers for validating dynamic-mode feature combinations/platform backend requirements.
- `crates/llama-sys/tests/backend_selection.rs` — pure Rust tests for dynamic-mode compatibility and required release backend sets.
- `crates/llama-sys/wrapper.h` — include the ggml backend/device declarations needed for runtime reporting if they are not already transitively exposed by `llama.h`.
- `crates/llama-sys/build.rs` — branch static vs dynamic llama.cpp configuration, enable `GGML_BACKEND_DL`, emit dynamic link search/rpath directives, and expose build mode metadata.
- `crates/llama-sys/src/lib.rs` — expose `DYNAMIC_BACKENDS` build metadata while preserving current static metadata.
- `src/runtime_backend.rs` — focused runtime device/backend inspection and classification logic built on llama.cpp/ggml APIs.
- `src/engine.rs` — initialize backend loading once and expose the runtime backend snapshot from an initialized engine.
- `src/lib.rs` — export runtime backend types/functions and deprecate build-time backend reporting as runtime information without breaking source compatibility.
- `src/main.rs` — print actual runtime backend/device information after engine initialization.
- `scripts/verify-release-package.sh` — validate staged archive contents and run the packaged binary in forced CPU mode for a real inference smoke test.
- `.github/workflows/release.yml` — provision CUDA/Vulkan build prerequisites, build dynamic desktop packages, stage modules/core libraries, verify the package, and publish the verified archive.
- `README.md` — document release auto-detection/fallback and distinguish it from source/static feature builds.

---

### Task 1: Define Dynamic Build Policy

**Files:**
- Modify: `crates/llama-sys/src/backend.rs`
- Modify: `crates/llama-sys/tests/backend_selection.rs`

**Interfaces:**
- Consumes: existing `Backend`, `SelectionInput`, and `select_backend` static-build policy.
- Produces: `pub fn validate_build_mode(dynamic: bool, feature_cuda: bool, feature_vulkan: bool, feature_metal: bool) -> Result<(), String>` and `pub fn required_dynamic_backends(target: &str) -> &'static [Backend]`.

- [ ] **Step 1: Write failing dynamic-mode policy tests**

Add these tests to `crates/llama-sys/tests/backend_selection.rs`:

```rust
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
```

- [ ] **Step 2: Run the pure policy suite and verify RED**

Run:

```bash
rustc --edition=2021 --test crates/llama-sys/tests/backend_selection.rs -o /tmp/backend-selection-tests
/tmp/backend-selection-tests
```

Expected: compilation fails because `validate_build_mode` and `required_dynamic_backends` do not exist.

- [ ] **Step 3: Implement the minimal policy helpers**

Add to `crates/llama-sys/src/backend.rs`:

```rust
pub fn validate_build_mode(
    dynamic: bool,
    feature_cuda: bool,
    feature_vulkan: bool,
    feature_metal: bool,
) -> Result<(), String> {
    if dynamic && (feature_cuda || feature_vulkan || feature_metal) {
        return Err(
            "dynamic-backends cannot be combined with cuda, vulkan, or metal features".to_string(),
        );
    }
    Ok(())
}

pub fn required_dynamic_backends(target: &str) -> &'static [Backend] {
    const DESKTOP: &[Backend] = &[Backend::Cpu, Backend::Cuda, Backend::Vulkan];
    const APPLE: &[Backend] = &[Backend::Cpu, Backend::Metal];
    const CPU: &[Backend] = &[Backend::Cpu];

    if target.contains("android") {
        CPU
    } else if target.contains("apple") {
        APPLE
    } else {
        DESKTOP
    }
}
```

- [ ] **Step 4: Run policy tests and verify GREEN**

Run the same `rustc --test` command.

Expected: all existing and new backend policy tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/llama-sys/src/backend.rs crates/llama-sys/tests/backend_selection.rs
git commit -m "test: define dynamic backend release policy"
```

---

### Task 2: Add an Explicit Dynamic-Backends Cargo Mode

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/llama-sys/Cargo.toml`
- Modify: `crates/llama-sys/src/lib.rs`
- Modify: `crates/llama-sys/build.rs`

**Interfaces:**
- Consumes: `validate_build_mode(...)` from Task 1.
- Produces: Cargo feature `dynamic-backends`; `llama_sys::DYNAMIC_BACKENDS: bool`; dynamic CMake configuration with `GGML_BACKEND_DL=ON` and shared runtime output.

- [ ] **Step 1: Add a failing compile-time metadata assertion**

In `crates/llama-sys/src/lib.rs`, add a unit-test module:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn dynamic_backend_metadata_matches_feature() {
        assert_eq!(super::DYNAMIC_BACKENDS, cfg!(feature = "dynamic-backends"));
    }
}
```

- [ ] **Step 2: Run the crate test and verify RED**

Run:

```bash
cargo test -p rs-llama-sys dynamic_backend_metadata_matches_feature
```

Expected: compile failure because `DYNAMIC_BACKENDS` does not exist.

- [ ] **Step 3: Add the feature forwarding and metadata**

Add to root `Cargo.toml`:

```toml
dynamic-backends = ["llama-sys/dynamic-backends"]
```

Add to `crates/llama-sys/Cargo.toml`:

```toml
dynamic-backends = []
```

Add to `crates/llama-sys/src/lib.rs`:

```rust
/// Whether this build uses llama.cpp runtime-loadable backend modules.
pub const DYNAMIC_BACKENDS: bool = cfg!(feature = "dynamic-backends");
```

At the beginning of backend selection in `build.rs`, validate the mode:

```rust
let dynamic_backends = cfg!(feature = "dynamic-backends");
backend::validate_build_mode(
    dynamic_backends,
    cfg!(feature = "cuda"),
    cfg!(feature = "vulkan"),
    cfg!(feature = "metal"),
)
.unwrap_or_else(|err| panic!("{err}"));
```

- [ ] **Step 4: Split CMake configuration by build mode**

Keep the current static path unchanged when `dynamic_backends == false`. For dynamic mode configure llama.cpp with:

```rust
config
    .define("BUILD_SHARED_LIBS", "ON")
    .define("GGML_BACKEND_DL", "ON")
    .define("GGML_NATIVE", "OFF")
    .define("GGML_CCACHE", "OFF")
    .define("LLAMA_BUILD_TESTS", "OFF")
    .define("LLAMA_BUILD_TOOLS", "OFF")
    .define("LLAMA_BUILD_EXAMPLES", "OFF")
    .define("LLAMA_BUILD_SERVER", "OFF")
    .define("LLAMA_BUILD_COMMON", "OFF")
    .define("LLAMA_BUILD_APP", "OFF")
    .define("LLAMA_CURL", "OFF");
```

For Windows/Linux dynamic mode define all required desktop modules:

```rust
config
    .define("GGML_CUDA", "ON")
    .define("GGML_VULKAN", "ON")
    .define("GGML_METAL", "OFF");
```

For macOS dynamic mode:

```rust
config
    .define("GGML_CUDA", "OFF")
    .define("GGML_VULKAN", "OFF")
    .define("GGML_METAL", "ON");
```

For Android retain CPU-only configuration. Do not call `detect_backend()` to choose one GPU backend in dynamic mode.

- [ ] **Step 5: Link only the dynamic core from Rust in dynamic mode**

Refactor `emit_link_flags` so static builds keep existing archive enumeration and vendor link flags. Dynamic builds should add llama.cpp install/build library directories and link the core shared libraries needed by the FFI (`llama`, `ggml`, `ggml-base`, `ggml-cpu` as produced by the pinned llama.cpp revision), but must not directly link CUDA/Vulkan vendor libraries from Rust. Add platform runtime search behavior:

```rust
if dynamic_backends && !target.contains("windows") {
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
}
```

For macOS use loader-relative rpath instead:

```rust
println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
```

Do not hard-code backend module filenames in Rust; packaging discovers the actual CMake output names.

- [ ] **Step 6: Run static regression tests**

Run:

```bash
cargo test --workspace --release
```

Expected: existing static/default build remains green.

- [ ] **Step 7: Build dynamic mode on the current desktop platform**

Run:

```bash
cargo build --release --features dynamic-backends
```

Expected: CMake enables `GGML_BACKEND_DL`; build succeeds when CUDA/Vulkan development prerequisites required for that platform are installed. On a developer machine lacking those prerequisites, use CI from Task 5 as the authoritative dynamic build gate rather than weakening the required release backend set.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml crates/llama-sys/Cargo.toml crates/llama-sys/src/lib.rs crates/llama-sys/build.rs
git commit -m "feat: add dynamic llama backend build mode"
```

---

### Task 3: Report Runtime Backend Devices Instead of Build-Time Guessing

**Files:**
- Create: `src/runtime_backend.rs`
- Modify: `crates/llama-sys/wrapper.h`
- Modify: `src/engine.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: llama.cpp backend initialization and ggml device enumeration bindings.
- Produces:
  - `pub enum RuntimeBackend { Cpu, Cuda, Vulkan, Metal, Other(String) }`
  - `pub struct RuntimeDevice { pub name: String, pub description: String, pub backend: RuntimeBackend, pub is_gpu: bool }`
  - `pub fn runtime_devices() -> Vec<RuntimeDevice>`
  - `pub fn active_backend(&self) -> RuntimeBackend` on `LlamaEngine`, derived after model load from usable enumerated devices and actual offload intent.

- [ ] **Step 1: Write pure classification tests before FFI code**

Create `src/runtime_backend.rs` with tests first:

```rust
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
```

Define the expected classifier signature above the tests only after observing RED:

```rust
fn classify_backend(name: &str) -> RuntimeBackend
```

- [ ] **Step 2: Run classification tests and verify RED**

Run:

```bash
cargo test -p rs-llama classifies_known_backend_names keeps_unknown_backend_name
```

Expected: compile failure because runtime backend types/classifier do not exist.

- [ ] **Step 3: Implement classification and runtime device enumeration**

Include ggml backend headers in `crates/llama-sys/wrapper.h`:

```c
#include "llama.h"
#include "ggml-backend.h"
```

Extend bindgen allowlists in `build.rs` to include the exact `ggml_backend_dev_*`, `ggml_backend_reg_*`, and device-type constants used by the implementation.

Implement `RuntimeBackend`, `RuntimeDevice`, and `classify_backend` in `src/runtime_backend.rs`. Enumerate devices with llama.cpp/ggml's current registry/device API after `llama_backend_init()`. Convert C strings lossily and classify using case-insensitive backend/device names. Set `is_gpu` from the ggml device type API rather than name matching.

- [ ] **Step 4: Make initialization and reporting share one initialized state**

Move the `Once`-guarded backend initialization behind a crate-visible helper in `src/runtime_backend.rs`:

```rust
pub(crate) fn ensure_backend_initialized() {
    static BACKEND: Once = Once::new();
    BACKEND.call_once(|| unsafe { llama_sys::llama_backend_init() });
}
```

Use it from both `runtime_devices()` and `LlamaEngine::load()`.

Store a runtime snapshot in `LlamaEngine` after successful model load:

```rust
runtime_devices: Vec<RuntimeDevice>,
active_backend: RuntimeBackend,
```

If `config.gpu_layers == 0`, set `active_backend` to `RuntimeBackend::Cpu`. Otherwise choose the highest-priority usable GPU reported by llama.cpp (`Cuda`, then `Vulkan`, then `Metal` as platform appropriate), falling back to CPU. Do not infer CUDA merely from the presence of a packaged DLL/SO.

- [ ] **Step 5: Preserve compatibility but stop presenting build metadata as runtime truth**

Keep `compiled_backend()` and `backend_selection_source()` exported for static-build compatibility. Add:

```rust
pub use runtime_backend::{runtime_devices, RuntimeBackend, RuntimeDevice};
```

Document `compiled_backend()` as build metadata only. Add `LlamaEngine::active_backend()` and `LlamaEngine::runtime_devices()` accessors.

- [ ] **Step 6: Change CLI diagnostics to print after model initialization**

Remove the pre-load:

```rust
eprintln!("Backend: {} ({})", ...);
```

After `LlamaEngine::load(config)?`, print:

```rust
eprintln!("Backend: {}", engine.active_backend());
for device in engine.runtime_devices() {
    eprintln!("Device: {} — {}", device.name, device.description);
}
```

Implement `Display` for `RuntimeBackend` so output is `CUDA`, `Vulkan`, `Metal`, `CPU`, or the unknown backend label.

- [ ] **Step 7: Run unit and smoke tests**

Run:

```bash
cargo test --workspace --release
cargo run --release -- --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf --chat --no-echo-prompt --prompt "What is an LED lamp?" --max-tokens 40 --ctx-size 1024 --threads 2 --gpu-layers 0
```

Expected: tests pass; CLI prints `Backend: CPU` for the forced CPU run and generates a non-empty answer.

- [ ] **Step 8: Commit**

```bash
git add src/runtime_backend.rs src/engine.rs src/lib.rs src/main.rs crates/llama-sys/wrapper.h crates/llama-sys/build.rs
git commit -m "feat: report runtime llama backend devices"
```

---

### Task 4: Add Release Package Verification

**Files:**
- Create: `scripts/verify-release-package.sh`

**Interfaces:**
- Consumes: staged package directory, platform identifier, packaged `rs-llama` executable.
- Produces: deterministic nonzero exit on missing required runtime files or failed packaged CPU inference.

- [ ] **Step 1: Create the verifier with an intentionally failing fixture mode**

Create `scripts/verify-release-package.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

package_dir=${1:?package directory required}
platform=${2:?platform required}

case "$platform" in
  windows)
    exe="$package_dir/rs-llama.exe"
    required=("ggml-cpu" "ggml-cuda" "ggml-vulkan")
    ;;
  linux)
    exe="$package_dir/rs-llama"
    required=("ggml-cpu" "ggml-cuda" "ggml-vulkan")
    ;;
  macos)
    exe="$package_dir/rs-llama"
    required=("ggml-cpu" "ggml-metal")
    ;;
  *) echo "unsupported platform: $platform" >&2; exit 2 ;;
esac

test -f "$exe"
for stem in "${required[@]}"; do
  find "$package_dir" -maxdepth 1 -type f -iname "*${stem}*" -print -quit | grep -q . || {
    echo "missing required backend module: $stem" >&2
    exit 1
  }
done
```

Add a `--layout-only` third argument path so CI/unit-like checks can validate a synthetic fixture without downloading a model; without that flag the script proceeds to the real smoke command in Step 3.

- [ ] **Step 2: Verify the layout check fails on a missing backend**

Run:

```bash
rm -rf /tmp/rs-llama-package-test
mkdir -p /tmp/rs-llama-package-test
touch /tmp/rs-llama-package-test/rs-llama
touch /tmp/rs-llama-package-test/libggml-cpu.so
touch /tmp/rs-llama-package-test/libggml-vulkan.so
bash scripts/verify-release-package.sh /tmp/rs-llama-package-test linux --layout-only
```

Expected: FAIL with `missing required backend module: ggml-cuda`.

- [ ] **Step 3: Add the packaged CPU inference smoke path**

For non-layout-only execution, run the executable from inside `package_dir` so loader-relative discovery is exercised:

```bash
(
  cd "$package_dir"
  "./$(basename "$exe")" \
    --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
    --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
    --chat \
    --no-echo-prompt \
    --prompt "What is an LED lamp?" \
    --max-tokens 40 \
    --ctx-size 1024 \
    --threads 2 \
    --gpu-layers 0 \
    2>backend-output.txt \
    | tee smoke-output.txt
)
grep -q "Backend: CPU" "$package_dir/backend-output.txt"
test -s "$package_dir/smoke-output.txt"
```

On Windows Git Bash, invoke `./rs-llama.exe`; on Unix use `./rs-llama`.

- [ ] **Step 4: Verify the layout check passes with a complete synthetic fixture**

Run:

```bash
touch /tmp/rs-llama-package-test/libggml-cuda.so
bash scripts/verify-release-package.sh /tmp/rs-llama-package-test linux --layout-only
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add scripts/verify-release-package.sh
git commit -m "test: verify dynamic release packages"
```

---

### Task 5: Build and Test Dynamic Desktop Packages in GitHub Actions

**Files:**
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `dynamic-backends` Cargo feature and `scripts/verify-release-package.sh`.
- Produces: tested package directories/artifacts containing the executable, shared core libraries, CPU backend, and all required desktop GPU modules.

- [ ] **Step 1: Make the current release matrix prove it is not yet packaging dynamic modules**

Before changing build dependencies, add a temporary package-layout validation step after the current build/staging logic that invokes:

```yaml
- name: Verify dynamic backend package layout
  if: runner.os != 'Android'
  shell: bash
  run: bash scripts/verify-release-package.sh "${{ runner.temp }}/rs-llama-package" "${{ matrix.platform }}" --layout-only
```

Stage the current binary alone into that directory first. Push the test-only commit and run the workflow.

Expected: Windows/Linux/macOS desktop jobs fail because required backend modules are absent. This is the CI RED proof.

- [ ] **Step 2: Provision Linux dynamic backend build dependencies**

Update the Linux dependency step to install Vulkan shader/compiler/header dependencies and a CUDA toolkit suitable for llama.cpp compilation. Use the GitHub runner/NVIDIA CUDA repository setup supported by the selected Ubuntu image; verify tools explicitly:

```bash
nvcc --version
glslc --version
pkg-config --exists vulkan
```

Do not add CUDA runtime files to the staged package merely because the toolkit exists on CI.

- [ ] **Step 3: Provision Windows dynamic backend build dependencies**

Install Vulkan SDK/tooling and CUDA toolkit before the build. Verify in PowerShell:

```powershell
nvcc --version
glslc --version
```

Ensure `CUDA_PATH` and `VULKAN_SDK` are exported into `GITHUB_ENV` if installers do not already provide them to later steps.

- [ ] **Step 4: Build desktop release jobs with dynamic mode**

Change desktop build command to:

```bash
cargo build --release --features dynamic-backends
```

Keep Android on the existing CPU-only build path.

- [ ] **Step 5: Stage the executable and runtime libraries from actual build output**

Add a platform-specific staging step that creates `${RUNNER_TEMP}/rs-llama-package` and copies:

- `rs-llama` / `rs-llama.exe`
- llama/ggml shared core libraries required by the executable
- CPU backend module
- Windows/Linux CUDA backend module
- Windows/Linux Vulkan backend module
- macOS Metal backend/runtime files

Discover files from the CMake install/build directories under Cargo `OUT_DIR`/`target` rather than assuming names from the design document. Fail the step if more than one ambiguous candidate exists for a required role.

- [ ] **Step 6: Run package layout verification and forced-CPU inference**

Invoke:

```yaml
- name: Verify packaged runtime backends and CPU fallback
  shell: bash
  run: bash scripts/verify-release-package.sh "${{ runner.temp }}/rs-llama-package" "${{ matrix.platform }}"
```

Expected: package includes every required module and performs real inference with `--gpu-layers 0`, printing `Backend: CPU`.

- [ ] **Step 7: Run vision smoke against the packaged executable**

Modify `scripts/ci-vision-smoke.sh` to accept an optional first argument for the executable path, defaulting to the current target path for normal CI. In release CI call it with the staged executable so dynamic library discovery is exercised by the vision path as well.

Expected: vision smoke passes from the staged package directory.

- [ ] **Step 8: Upload the staged package directory, not the naked executable**

Change each desktop `upload-artifact` step to upload `${RUNNER_TEMP}/rs-llama-package/**`. Leave Android upload behavior unchanged.

- [ ] **Step 9: Verify the CI GREEN proof**

Run the updated workflow on the feature branch.

Expected: Windows, Linux, and macOS dynamic build jobs pass package layout, forced-CPU text inference, and packaged vision smoke; Android remains green on CPU-only build.

- [ ] **Step 10: Commit**

```bash
git add .github/workflows/release.yml scripts/ci-vision-smoke.sh
git commit -m "ci: package runtime GPU backends"
```

---

### Task 6: Package and Publish the Verified Runtime Bundle

**Files:**
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: tested staged directories uploaded by Task 5.
- Produces: `rs-llama-windows-x86_64.zip`, `rs-llama-linux-x86_64.tar.gz`, `rs-llama-macos-arm64.tar.gz`, Android archive, and `SHA256SUMS` with runtime modules included.

- [ ] **Step 1: Add an archive-content assertion before changing packaging**

After archives are created, add commands that list each archive and require backend stems:

```bash
unzip -l dist/rs-llama-windows-x86_64.zip | grep -Ei 'ggml-cpu'
unzip -l dist/rs-llama-windows-x86_64.zip | grep -Ei 'ggml-cuda'
unzip -l dist/rs-llama-windows-x86_64.zip | grep -Ei 'ggml-vulkan'
tar -tzf dist/rs-llama-linux-x86_64.tar.gz | grep -Ei 'ggml-cpu'
tar -tzf dist/rs-llama-linux-x86_64.tar.gz | grep -Ei 'ggml-cuda'
tar -tzf dist/rs-llama-linux-x86_64.tar.gz | grep -Ei 'ggml-vulkan'
tar -tzf dist/rs-llama-macos-arm64.tar.gz | grep -Ei 'ggml-cpu'
tar -tzf dist/rs-llama-macos-arm64.tar.gz | grep -Ei 'ggml-metal|metal'
```

Expected before packaging rewrite: FAIL because current archives contain only the executable.

- [ ] **Step 2: Replace single-binary packaging with directory packaging**

Package each downloaded tested desktop artifact directory as-is under its existing top-level archive folder name. Do not selectively copy only `rs-llama`; the tested runtime libraries/modules are part of the product.

Android continues to package its CPU-only executable as today.

- [ ] **Step 3: Regenerate checksums after complete archives exist**

Keep:

```bash
(cd dist && sha256sum * > SHA256SUMS)
```

Run it only after all archives have been finalized.

- [ ] **Step 4: Verify archive assertions GREEN**

Run the package job via Actions.

Expected: every required backend stem is present in its platform archive; `SHA256SUMS` contains all four archives.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: publish runtime backend bundles"
```

---

### Task 7: Document Runtime Auto-Detection and Preserve Source-Build Semantics

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: final runtime release behavior from Tasks 2-6.
- Produces: user-facing instructions that distinguish downloaded release bundles from Cargo source builds.

- [ ] **Step 1: Add release behavior documentation**

Add a `Runtime GPU auto-detection` section containing this contract:

```markdown
Downloaded desktop releases automatically load the best usable llama.cpp backend at runtime:

- Windows/Linux: CUDA -> Vulkan -> CPU
- macOS: Metal -> CPU
- Android: CPU

You download one archive for your platform. If CUDA is unavailable or cannot load, rs-llama continues with Vulkan; if no GPU backend is usable, it continues on CPU. The standard archive does not bundle the full CUDA runtime/toolkit.
```

- [ ] **Step 2: Document explicit CPU inference**

Add:

```bash
rs-llama --model model.gguf --gpu-layers 0 --prompt "Hello"
```

Explain that this forces model layers to CPU even when a GPU backend is available.

- [ ] **Step 3: Clarify Cargo/source builds**

Document that normal library consumers keep the existing source/static semantics, while `dynamic-backends` is intended for distributable desktop bundles:

```bash
cargo build --release --features dynamic-backends
```

Also retain examples for `cuda`, `vulkan`, and `metal` static feature builds and state that they must not be combined with `dynamic-backends`.

- [ ] **Step 4: Run documentation-sensitive CLI checks**

Run:

```bash
cargo run --release -- --help
cargo test --workspace --release
```

Expected: `--gpu-layers` help still states that `0` is CPU inference; workspace tests pass.

- [ ] **Step 5: Commit**

```bash
git add README.md
git commit -m "docs: explain runtime GPU fallback releases"
```

---

### Task 8: Final Release Verification

**Files:**
- Review only: all files changed in Tasks 1-7.

**Interfaces:**
- Consumes: complete implementation.
- Produces: evidence that the exact PR head satisfies the spec before merge/tagging.

- [ ] **Step 1: Run the pure backend policy gate**

Run:

```bash
rustc --edition=2021 --test crates/llama-sys/tests/backend_selection.rs -o /tmp/backend-selection-tests
/tmp/backend-selection-tests
```

Expected: all tests pass.

- [ ] **Step 2: Run the complete Rust regression suite**

Run:

```bash
cargo test --workspace --release
```

Expected: all tests pass in default/static mode.

- [ ] **Step 3: Verify dynamic mode rejects conflicting static features**

Run:

```bash
cargo check --features dynamic-backends,cuda
```

Expected: FAIL with `dynamic-backends cannot be combined with cuda, vulkan, or metal features`.

- [ ] **Step 4: Verify exact-head GitHub Actions**

Check the workflow runs for the final commit SHA.

Expected:

- Backend selection policy: success.
- Windows desktop dynamic build/package/text smoke/vision smoke: success.
- Linux desktop dynamic build/package/text smoke/vision smoke: success.
- macOS desktop dynamic build/package/text smoke/vision smoke: success.
- Android CPU build: success.
- Package job archive-content assertions: success.

Do not treat queued, skipped in a blocking way, cancelled, or earlier-SHA runs as success.

- [ ] **Step 5: Inspect produced package artifacts**

Download the exact-head `rs-llama-packages` artifact and inspect archive listings.

Expected:

```text
Windows: executable + core runtime + CPU + CUDA + Vulkan modules
Linux: executable + core runtime + CPU + CUDA + Vulkan modules
macOS: executable + core runtime + CPU + Metal support
Android: CPU executable
```

Confirm no full CUDA toolkit/runtime directory has been accidentally bundled.

- [ ] **Step 6: Review the final diff against the spec**

Confirm all success criteria in `docs/superpowers/specs/2026-08-25-runtime-backend-release-design.md` have corresponding passing evidence, and confirm no unrelated files changed.

- [ ] **Step 7: Request code review**

Use the Superpowers requesting-code-review workflow against the exact final head. Resolve High/Medium findings before merge.

- [ ] **Step 8: Merge/tag only after verification**

After the PR is green and reviewed, merge it. Create the next `v*` tag only if a release is desired immediately; the tag-triggered workflow must publish the already-verified runtime bundles.
