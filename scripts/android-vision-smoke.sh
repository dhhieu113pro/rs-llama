#!/usr/bin/env bash
# Run vision smoke on a connected Android device/emulator (adb).
set -euo pipefail

BIN=${1:-target/x86_64-linux-android/release/rs-llama}
test -f "$BIN" || { echo "Android binary not found: $BIN"; exit 1; }

python3 scripts/make-vision-image.py
ADB=${ADB:-adb}
REMOTE=/data/local/tmp/rs-llama-smoke

NDK_ROOT=${ANDROID_NDK_HOME:-${ANDROID_NDK:-${ANDROID_NDK_ROOT:-}}}
if [ -z "$NDK_ROOT" ] && [ -d /opt/hostedtoolcache/ndk ]; then
  NDK_ROOT=$(find /opt/hostedtoolcache/ndk -mindepth 2 -maxdepth 2 -type d -name x64 -print -quit 2>/dev/null || true)
fi

CXX_SHARED=""
if [ -n "$NDK_ROOT" ]; then
  CXX_SHARED=$(find "$NDK_ROOT/toolchains/llvm/prebuilt" -type f -path '*/sysroot/usr/lib/x86_64-linux-android/libc++_shared.so' -print -quit 2>/dev/null || true)
fi

test -f "$CXX_SHARED" || {
  echo "Android NDK libc++_shared.so not found (NDK root: ${NDK_ROOT:-unset})"
  exit 1
}

"$ADB" shell mkdir -p "$REMOTE/models"
"$ADB" push "$BIN" "$REMOTE/rs-llama"
"$ADB" push "$CXX_SHARED" "$REMOTE/libc++_shared.so"
"$ADB" push testdata/led-lamp.png "$REMOTE/led-lamp.png"
"$ADB" shell chmod 755 "$REMOTE/rs-llama"

"$ADB" shell "cd $REMOTE && LD_LIBRARY_PATH=$REMOTE ./rs-llama \
  --hf-repo ggml-org/SmolVLM-256M-Instruct-GGUF \
  --hf-file SmolVLM-256M-Instruct-Q8_0.gguf \
  --model-dir $REMOTE/models \
  --image $REMOTE/led-lamp.png \
  --chat \
  --no-echo-prompt \
  --prompt 'Read the text in the image. What does it say?' \
  --max-tokens 48 \
  --ctx-size 2048 \
  --threads 2" \
  >vision-android-output.txt 2>vision-android-stderr.txt || true

echo "----- android vision stderr -----"
cat vision-android-stderr.txt || true
echo "----- android vision stdout -----"
cat vision-android-output.txt || true

if ! grep -qiE 'mmproj|vision' vision-android-stderr.txt vision-android-output.txt; then
  echo "Android vision test did not attach mmproj"
  exit 1
fi
test -s vision-android-output.txt
echo "Android vision smoke passed"
