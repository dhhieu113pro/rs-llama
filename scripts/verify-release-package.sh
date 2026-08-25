#!/usr/bin/env bash
set -euo pipefail

package_dir=${1:?package directory required}
platform=${2:?platform required}
mode=${3:-full}
repo_dir=$(pwd)
model_dir=${RS_LLAMA_VERIFY_MODEL_DIR:-$repo_dir/models}
verify_dir=${RUNNER_TEMP:-/tmp}/rs-llama-package-verify-${platform}

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

rm -rf "$verify_dir"
mkdir -p "$verify_dir" "$model_dir"

(
  cd "$package_dir"
  bin="./$(basename "$exe")"
  "$bin" \
    --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
    --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
    --model-dir "$model_dir" \
    --chat \
    --no-echo-prompt \
    --prompt "What is an LED lamp?" \
    --max-tokens 40 \
    --ctx-size 1024 \
    --threads 2 \
    --gpu-layers 0 \
    2>"$verify_dir/backend-output.txt" \
    | tee "$verify_dir/smoke-output.txt"
)

grep -q "Backend: CPU" "$verify_dir/backend-output.txt"
test -s "$verify_dir/smoke-output.txt"
if ! grep -qiE 'led|light|diode|lamp|electric|bulb' "$verify_dir/smoke-output.txt"; then
  echo "Packaged smoke test did not answer the LED lamp question" >&2
  exit 1
fi
