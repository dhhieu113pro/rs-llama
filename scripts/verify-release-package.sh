#!/usr/bin/env bash
set -euo pipefail

package_dir=${1:?package directory required}
platform=${2:?platform required}
mode=${3:-full}

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
  *)
    echo "unsupported platform: $platform" >&2
    exit 2
    ;;
esac

test -f "$exe"

for stem in "${required[@]}"; do
  find "$package_dir" -maxdepth 1 -type f -iname "*${stem}*" -print -quit | grep -q . || {
    echo "missing required backend module: $stem" >&2
    exit 1
  }
done

if [[ "$mode" == "--layout-only" ]]; then
  exit 0
fi

(
  cd "$package_dir"
  bin="./$(basename "$exe")"
  "$bin" \
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
if ! grep -qiE 'led|light|diode|lamp|electric|bulb' "$package_dir/smoke-output.txt"; then
  echo "Packaged smoke test did not answer the LED lamp question" >&2
  exit 1
fi
