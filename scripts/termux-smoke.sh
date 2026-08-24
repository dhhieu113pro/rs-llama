#!/data/data/com.termux/files/usr/bin/bash
# On-device Termux smoke test. Needs network for the first GGUF download.
set -euo pipefail

cd "$(dirname "$0")/.."
BIN=target/release/rs-llama
test -x "$BIN" || { echo "Build first: bash scripts/termux-build.sh"; exit 1; }

"$BIN" \
  --hf-repo mradermacher/SmolLM2-135M-Instruct-GGUF \
  --hf-file SmolLM2-135M-Instruct.Q4_K_M.gguf \
  --chat \
  --no-echo-prompt \
  --prompt "What is an LED lamp?" \
  --max-tokens 64 \
  --ctx-size 1024 \
  --threads 2 \
  | tee smoke-termux.txt

test -s smoke-termux.txt
grep -qiE 'led|light|diode|lamp|electric|bulb' smoke-termux.txt
echo "Termux smoke passed"
