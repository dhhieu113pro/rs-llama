# Android and Termux

GitHub-hosted runners cannot boot Termux. CI does two things:

1. **Cross-compile** `aarch64-linux-android` with the NDK (`cargo ndk -t arm64-v8a`). That is the same target Termux uses.
2. **Syntax-check** `scripts/termux-build.sh` and `scripts/termux-smoke.sh`.

Real GGUF inference on a phone is still on-device.

## Termux (on the phone)

```bash
pkg update -y
pkg install -y rust cmake clang git make binutils
git clone https://github.com/dhhieu113pro/rs-llama
cd rs-llama
bash scripts/termux-build.sh
bash scripts/termux-smoke.sh
```

Smoke asks **What is an LED lamp?** and fails if the answer has no led/light/diode/lamp/electric/bulb.

First build compiles llama.cpp. Give Termux plenty of RAM; close other apps.

## NDK cross-compile (PC / CI)

```bash
rustup target add aarch64-linux-android
export ANDROID_NDK_HOME=/path/to/ndk
cargo install cargo-ndk
cargo ndk -t arm64-v8a -p 28 build --release --bin rs-llama
```

Copy `target/aarch64-linux-android/release/rs-llama` into Termux if you prefer not to compile on the phone.
