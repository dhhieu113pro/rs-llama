#!/usr/bin/env bash
# Vision smoke: download SmolVLM + mmproj and run --image on any OS.
set -euo pipefail

if [ -f target/release/rs-llama.exe ]; then
  BIN=target/release/rs-llama.exe
elif [ -f target/release/rs-llama ]; then
  BIN=target/release/rs-llama
else
  echo "rs-llama binary not found"
  exit 1
fi

mkdir -p testdata
python3 - <<'PY'
from pathlib import Path
Path('testdata').mkdir(exist_ok=True)
Path('testdata/led-lamp.png').write_bytes(bytes.fromhex(
    '89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de'
    '0000000c4944415408d763f8ff33c00000030101d5d45f8b0000000049454e44ae426082'
))
PY

"$BIN" \
  --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF \
  --hf-file SmolVLM-256M-Instruct-Q8_0.gguf \
  --image testdata/led-lamp.png \
  --chat \
  --no-echo-prompt \
  --prompt "What is an LED lamp?" \
  --max-tokens 48 \
  --ctx-size 2048 \
  --threads 2 \
  >vision-output.txt 2>vision-stderr.txt

echo "----- vision stderr -----"
cat vision-stderr.txt
echo "----- vision stdout -----"
cat vision-output.txt

if ! grep -qiE 'mmproj|vision' vision-stderr.txt; then
  echo "Vision test did not download or attach mmproj"
  exit 1
fi

test -s vision-output.txt

if ! ls models | grep -qi mmproj; then
  echo "No mmproj file cached in models/"
  ls -lah models || true
  exit 1
fi

echo "Vision smoke passed on $(uname -s)"
