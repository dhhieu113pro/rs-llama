#!/usr/bin/env python3
"""Create testdata/led-lamp.png with visible text for vision smoke tests."""
from __future__ import annotations

import sys
from pathlib import Path

OUT = Path("testdata/led-lamp.png")


def font_candidates() -> list[str]:
    return [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/Library/Fonts/Arial.ttf",
        "C:/Windows/Fonts/arialbd.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/segoeui.ttf",
    ]


def render_with_pillow() -> None:
    from PIL import Image, ImageDraw, ImageFont

    width, height = 640, 320
    image = Image.new("RGB", (width, height), (18, 18, 22))
    draw = ImageDraw.Draw(image)
    draw.rounded_rectangle((32, 32, width - 32, height - 32), radius=24, fill=(255, 204, 0))
    draw.rounded_rectangle((48, 48, width - 48, height - 48), radius=18, fill=(255, 230, 80))

    font = ImageFont.load_default()
    for path in font_candidates():
        if Path(path).exists():
            try:
                font = ImageFont.truetype(path, 72)
                break
            except OSError:
                continue

    text = "LED LAMP"
    try:
        left, top, right, bottom = draw.textbbox((0, 0), text, font=font)
        tw, th = right - left, bottom - top
        xy = ((width - tw) / 2 - left, (height - th) / 2 - top)
    except Exception:
        xy = (160, 130)
    draw.text(xy, text, fill=(20, 20, 20), font=font)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    image.save(OUT)
    print(f"wrote {OUT.resolve()} ({OUT.stat().st_size} bytes)")


def render_fallback_ppm_png() -> None:
    """Stdlib-only PNG: yellow card + dark pixels forming block letters."""
    import struct
    import zlib

    w, h = 320, 160
    bg = (255, 210, 0)
    fg = (20, 20, 20)
    pixels = [bg] * (w * h)

    def plot(x: int, y: int) -> None:
        if 0 <= x < w and 0 <= y < h:
            pixels[y * w + x] = fg

    def fill(x0: int, y0: int, x1: int, y1: int) -> None:
        for y in range(y0, y1):
            for x in range(x0, x1):
                plot(x, y)

    # Very small 5x7 glyphs for L E D   L A M P
    glyphs = {
        "L": ["10000", "10000", "10000", "10000", "10000", "10000", "11111"],
        "E": ["11111", "10000", "10000", "11110", "10000", "10000", "11111"],
        "D": ["11110", "10001", "10001", "10001", "10001", "10001", "11110"],
        "A": ["01110", "10001", "10001", "11111", "10001", "10001", "10001"],
        "M": ["10001", "11011", "10101", "10001", "10001", "10001", "10001"],
        "P": ["11110", "10001", "10001", "11110", "10000", "10000", "10000"],
        " ": ["00000", "00000", "00000", "00000", "00000", "00000", "00000"],
    }
    text = "LED LAMP"
    scale = 6
    glyph_w, glyph_h = 5 * scale + 8, 7 * scale
    total_w = len(text) * glyph_w
    ox = (w - total_w) // 2
    oy = (h - glyph_h) // 2
    for i, ch in enumerate(text):
        rows = glyphs[ch]
        for ry, row in enumerate(rows):
            for rx, bit in enumerate(row):
                if bit == "1":
                    fill(
                        ox + i * glyph_w + rx * scale,
                        oy + ry * scale,
                        ox + i * glyph_w + (rx + 1) * scale,
                        oy + ry * scale + scale,
                    )

    def png_chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw = b""
    for y in range(h):
        raw += b"\x00"
        for x in range(w):
            raw += bytes(pixels[y * w + x])
    png = b"\x89PNG\r\n\x1a\n"
    png += png_chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
    png += png_chunk(b"IDAT", zlib.compress(raw, 9))
    png += png_chunk(b"IEND", b"")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(png)
    print(f"wrote fallback {OUT.resolve()} ({OUT.stat().st_size} bytes)")


def main() -> int:
    try:
        render_with_pillow()
        return 0
    except Exception as exc:
        print(f"pillow render failed ({exc}); using stdlib fallback", file=sys.stderr)
        render_fallback_ppm_png()
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
