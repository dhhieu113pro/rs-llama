#!/usr/bin/env bash
# Vision smoke: generate a text image, download SmolVLM + mmproj, run --image.
set -euo pipefail

if [ "${1:-}" != "" ]; then
  BIN="$1"
elif [ -f target/release/rs-llama.exe ]; then
  BIN=target/release/rs-llama.exe
elif [ -f target/release/rs-llama ]; then
  BIN=target/release/rs-llama
else
  echo "rs-llama binary not found"
  exit 1
fi

if [[ "$BIN" != /* && "$BIN" != [A-Za-z]:* ]]; then
  BIN="$(pwd)/$BIN"
fi

test -f "$BIN"

python3 -m pip install --user --quiet pillow || pip3 install --user --quiet pillow || true
python3 scripts/make-vision-image.py
test -s testdata/led-lamp.png

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
  >vision-output.txt 2>vision-stderr.txt

echo "----- vision image -----"
ls -lah testdata/led-lamp.png
echo "----- vision stderr -----"
cat vision-stderr.txt
echo "----- vision stdout -----"
cat vision-output.txt

if ! grep -qiE 'mmproj|vision' vision-stderr.txt; then
  echo "Vision test did not download or attach mmproj"
  exit 1
fi

test -s vision-output.txt

MMPROJ_PATH="$(find models -maxdepth 1 -type f -iname '*mmproj*' -print -quit 2>/dev/null || true)"
if [ -z "$MMPROJ_PATH" ]; then
  echo "No mmproj file cached in models/"
  ls -lah models || true
  exit 1
fi

echo "Vision smoke passed on $(uname -s)"
