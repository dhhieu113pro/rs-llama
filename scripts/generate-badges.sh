#!/usr/bin/env bash
# Generate simple static SVG badges for rs-llama
# Usage: ./scripts/generate-badges.sh [output-dir]
set -euo pipefail

OUT="${1:-assets/badges}"
mkdir -p "$OUT"

# Color palette matching the logo
BG="#0d1117"
FG="#e6edf3"
ACCENT="#00e5ff"
GREEN="#3fb950"
BLUE="#58a6ff"

badge() {
  local label="$1"
  local value="$2"
  local color="${3:-$ACCENT}"
  local filename="$4"
  local label_w=$(( ${#label} * 7 + 12 ))
  local value_w=$(( ${#value} * 7 + 12 ))
  local total=$(( label_w + value_w ))

  cat > "$OUT/$filename" << SVG
<svg xmlns="http://www.w3.org/2000/svg" width="$total" height="20" role="img" aria-label="$label: $value">
  <title>$label: $value</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#fff" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r"><rect width="$total" height="20" rx="3" fill="#fff"/></clipPath>
  <g clip-path="url(#r)">
    <rect width="$label_w" height="20" fill="#21262d"/>
    <rect x="$label_w" width="$value_w" height="20" fill="$color"/>
    <rect width="$total" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="11">
    <text x="$(( label_w / 2 ))" y="14" fill="#fff">$label</text>
    <text x="$(( label_w + value_w / 2 ))" y="14" fill="#fff">$value</text>
  </g>
</svg>
SVG
  echo "Wrote $OUT/$filename"
}

badge "platforms" "4" "$BLUE" "platforms.svg"
badge "GPU" "CUDA | Vulkan | Metal" "$GREEN" "gpu.svg"
badge "vision" "mmproj" "$ACCENT" "vision.svg"
badge "android" "Termux arm64" "$BLUE" "android.svg"
badge "license" "MIT" "#8b949e" "license.svg"

echo "Done. Badges in $OUT/"
