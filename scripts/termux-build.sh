#!/data/data/com.termux/files/usr/bin/bash
# Native Termux build. Run this on the phone/tablet, not on GitHub Actions.
set -euo pipefail

cd "$(dirname "$0")/.."

pkg update -y
pkg install -y rust cmake clang git make binutils

rustc --version
cargo --version
cmake --version

cargo generate-lockfile
cargo build --release
cargo test --release --offline || cargo test --release

echo
echo "Binary: $(pwd)/target/release/rs-llama"
echo "Try:"
echo "  ./target/release/rs-llama --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf --chat --no-echo-prompt --prompt 'What is an LED lamp?'"
