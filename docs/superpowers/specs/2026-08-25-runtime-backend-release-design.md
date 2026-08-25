# Runtime GPU Backend Release Design

Date: 2026-08-25
Status: Approved design

## Goal

Publish one normal `rs-llama` release artifact per supported platform that automatically uses the best usable llama.cpp backend at runtime and safely falls back to CPU when GPU acceleration is unavailable or unusable.

The user experience should be:

- Windows/Linux: CUDA when usable, otherwise Vulkan when usable, otherwise CPU.
- macOS: Metal when usable, otherwise CPU.
- Android: CPU in the first implementation.
- No separate CPU/CUDA/Vulkan downloads for normal users.
- `--gpu-layers 0` remains an explicit CPU-only inference path.

## Problem

The current release workflow runs a normal `cargo build --release` on GitHub-hosted Windows/Linux/macOS runners. Backend selection currently happens while `llama-sys` is built, so the resulting executable is tied to the backend chosen by the build environment. A release runner without CUDA/Vulkan development tooling therefore produces a CPU-oriented Windows/Linux artifact even if the end user's machine has a supported GPU.

This does not satisfy the release requirement: the downloaded artifact must choose acceleration based on the end user's machine, not the GitHub Actions runner that built it.

## Chosen Architecture

Use llama.cpp dynamic backend loading for release builds.

The release build enables llama.cpp's dynamic backend mechanism (`GGML_BACKEND_DL`) and packages the executable together with backend modules that can be loaded at runtime. The core binary must remain runnable with only the CPU backend available.

Runtime priority is platform-specific:

### Windows and Linux

1. CUDA backend, if its module loads and llama.cpp reports a usable CUDA device.
2. Vulkan backend, if its module loads and llama.cpp reports a usable Vulkan device.
3. CPU backend.

### macOS

1. Metal backend, if usable.
2. CPU backend.

### Android

CPU only for this scope. Android GPU backend packaging can be designed separately after the desktop release path is stable.

The fallback decision is runtime capability-based. The mere presence of a backend library is not considered success; the backend must load and expose a usable device.

## Release Artifact Layout

Each platform keeps one normal archive name and contains everything needed for the supported runtime path except large vendor runtimes that are intentionally external.

Representative Windows archive:

```text
rs-llama-windows-x86_64/
  rs-llama.exe
  ggml-base.dll / llama core runtime dependencies as required
  ggml-cpu.dll
  ggml-vulkan.dll
  ggml-cuda.dll
```

Representative Linux archive:

```text
rs-llama-linux-x86_64/
  rs-llama
  libggml-base.so / llama core runtime dependencies as required
  libggml-cpu.so
  libggml-vulkan.so
  libggml-cuda.so
```

Representative macOS archive:

```text
rs-llama-macos-arm64/
  rs-llama
  required llama/ggml dynamic libraries
  CPU backend module
  Metal backend module
```

Exact library names are taken from the pinned/current llama.cpp build output rather than hard-coded from this document.

## CUDA Runtime Policy

Do not bundle the full CUDA runtime/toolkit in the normal rs-llama release archive.

The CUDA backend module is included when CI can build it, but CUDA activation at runtime depends on the compatible NVIDIA driver/runtime libraries being present on the user's machine. If CUDA cannot load, runtime selection continues to Vulkan and then CPU.

This keeps the standard release reasonably sized and avoids turning every rs-llama download into a several-hundred-megabyte CUDA distribution.

A separate CUDA-runtime convenience package is explicitly out of scope for this implementation and can be added later if user demand justifies it.

## Build-System Changes

`llama-sys` gains a release-oriented dynamic-backend build mode rather than replacing the existing developer/static modes.

In dynamic release mode:

- Build llama.cpp core/shared runtime in the form required by `GGML_BACKEND_DL`.
- Enable the CPU backend unconditionally.
- Build CUDA and Vulkan backend modules for Windows/Linux release artifacts.
- Build Metal plus CPU for macOS.
- Do not statically link one GPU backend into the Rust executable as the selected backend.
- Preserve existing explicit static backend Cargo features for developers and deterministic builds unless they conflict with dynamic-backend mode.

The dynamic mode must be explicit in the build system so normal crates.io consumers are not unexpectedly forced into a multi-library distribution model.

## Runtime Initialization

`LlamaEngine` continues to call llama.cpp backend initialization once per process.

In dynamic release mode, initialization must ensure llama.cpp discovers backend modules next to the executable or in the platform-appropriate packaged backend directory. Backend loading follows llama.cpp's supported loader behavior rather than implementing a custom Rust `dlopen`/`LoadLibrary` backend ABI.

After backend loading, model loading uses the existing default `gpu_layers = 999`, allowing llama.cpp to offload as much as the selected usable device supports.

If no GPU device is usable, CPU must remain available and model loading must continue without requiring the user to rebuild or download another artifact.

## Backend Reporting

The current build-time backend label is insufficient for dynamic release artifacts.

The CLI/API should report runtime reality. At minimum, startup diagnostics must distinguish:

- loaded/available backend modules,
- selected device/backend used for model offload when this information is exposed by the pinned llama.cpp API,
- CPU fallback when no GPU device is usable.

The exact public API can be small, but it must not report `CUDA (auto)` merely because a CUDA module was packaged. It should report CUDA only when CUDA is actually usable/selected.

`RS_LLAMA_BACKEND` remains useful for build/developer workflows. A runtime `--device`/backend override may be added if it can be implemented directly with llama.cpp device selection, but it is not required to satisfy the first release milestone. Auto runtime selection is the default and required path.

## Failure and Fallback Behavior

Expected non-fatal conditions include:

- CUDA backend library cannot load because NVIDIA/CUDA runtime dependencies are absent.
- CUDA backend loads but no supported NVIDIA device is usable.
- Vulkan loader/backend is unavailable.
- Vulkan backend loads but no suitable Vulkan device is usable.

These conditions must not terminate the process while CPU remains usable. They should produce concise diagnostics and continue down the fallback chain.

Fatal conditions remain actual application/model errors such as an unreadable GGUF, incompatible model data, or failure to initialize even the CPU backend.

## GitHub Actions Release Changes

The release workflow must build and test the distributable package, not only `target/release/rs-llama`.

### Windows/Linux release jobs

Install the build dependencies required to compile both Vulkan and CUDA backend modules. CUDA compilation may use the toolkit available/provisioned in CI; the resulting package does not include the full toolkit/runtime.

Build dynamic-backend release output and stage the executable plus required llama/ggml runtime libraries and backend modules into a package directory.

### macOS release job

Build dynamic core/runtime with CPU + Metal modules and stage them together.

### Android

Keep the current CPU release path for this phase.

## Verification Strategy

Verification is layered so GPU-less GitHub runners still validate the critical fallback contract.

1. Pure policy/unit tests verify platform priority: CUDA -> Vulkan -> CPU, Metal -> CPU, and explicit CPU behavior.
2. Package-layout tests verify every desktop archive contains the executable, CPU backend, and expected platform GPU backend modules.
3. CPU-fallback smoke test runs the packaged executable in an environment where GPU backends are unavailable/unusable and performs real text inference. This proves that shipping GPU modules does not make GPU availability mandatory.
4. Existing vision smoke testing should run against the packaged executable when practical, so dynamic library discovery is exercised for multimodal paths too.
5. Backend loader diagnostics are asserted sufficiently to show that CPU fallback occurred rather than silently testing a different build type.
6. CUDA/Vulkan runtime GPU execution on real hardware is desirable but not required to gate every PR because GitHub-hosted runners do not guarantee those devices. Dedicated/self-hosted GPU verification can be added later without weakening CPU fallback CI.

## Compatibility

- Existing Rust library usage remains source-compatible where possible.
- `EngineConfig::new()` keeps default GPU offload behavior.
- `with_gpu_layers(0)` remains a supported CPU-only request.
- Existing static Cargo backend features remain available for consumers that want a backend-specific build.
- crates.io publishing must not require consumers to redistribute all release backend modules unless they opt into the dynamic-backend mode.

## Out of Scope

- Bundling the complete CUDA runtime/toolkit in the standard archive.
- Android Vulkan/GPU release support.
- A GUI/backend chooser.
- Downloading backend modules dynamically from the network at application startup.
- Supporting every llama.cpp accelerator in the first implementation (ROCm, SYCL, OpenCL, etc.).
- Maintaining separate normal CPU/CUDA/Vulkan release binaries.

## Success Criteria

The implementation is complete when all of the following are true:

1. A user downloads one standard archive for Windows, Linux, or macOS.
2. The same Windows/Linux archive can run on a CPU-only machine and on a machine where a packaged GPU backend is usable.
3. Runtime selection prefers CUDA over Vulkan over CPU on Windows/Linux, and Metal over CPU on macOS.
4. Missing or unusable GPU dependencies do not prevent CPU inference.
5. The packaged executable, not merely the build-tree executable, passes release smoke tests.
6. CLI/runtime diagnostics reflect the backend/device actually available/selected rather than the backend chosen on the CI build host.
7. Normal release archives do not bundle the full CUDA runtime.
