#!/data/data/com.termux/files/usr/bin/bash
# On-device Termux smoke: text LED lamp + vision image.
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

python3 scripts/make-vision-image.py || true
if [ ! -s testdata/led-lamp.png ]; then
  mkdir -p testdata
  python3 - <<'PY'
from pathlib import Path
Path('testdata').mkdir(exist_ok=True)
Path('testdata/led-lamp.png').write_bytes(bytes.fromhex(
    '89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de'
    '0000000c4944415408d763f8ff33c00000030101d5d45f8b0000000049454e44ae426082'
))
PY
fi

"$BIN" \
  --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF \
  --hf-file SmolVLM-256M-Instruct-Q8_0.gguf \
  --image testdata/led-lamp.png \
  --chat \
  --no-echo-prompt \
  --prompt "Read the text in the image. What does it say?" \
  --max-tokens 48 \
  --ctx-size 2048 \
  --threads 2 \
  >vision-termux-output.txt 2>vision-termux-stderr.txt

cat vision-termux-stderr.txt
cat vision-termux-output.txt
grep -qiE 'mmproj|vision' vision-termux-stderr.txt
test -s vision-termux-output.txt
ls models | grep -qi mmproj

echo "Termux text + vision smoke passed"
