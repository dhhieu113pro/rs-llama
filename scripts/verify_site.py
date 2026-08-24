from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
paths = {name: SITE / name for name in ("index.html", "tokens.css", "styles.css", "app.js")}
missing = [str(p.relative_to(ROOT)) for p in paths.values() if not p.exists()]
if missing:
    raise AssertionError(f"missing site files: {', '.join(missing)}")

html = paths["index.html"].read_text(encoding="utf-8")
tokens = paths["tokens.css"].read_text(encoding="utf-8")
css = paths["styles.css"].read_text(encoding="utf-8")
js = paths["app.js"].read_text(encoding="utf-8")

for needle in (
    '<main id="main">', 'id="examples"', 'id="platforms"', 'data-copy-target=',
    'aria-label="Toggle color theme"', 'https://github.com/dhhieu113pro/rs-llama',
    'https://github.com/dhhieu113pro/rs-llama/releases',
    'rs-llama = { git = &quot;https://github.com/dhhieu113pro/rs-llama&quot; }',
    '--features cuda', '--features vulkan', '--features metal', '--image ./photo.jpg'
):
    assert needle in html, f"missing HTML contract: {needle}"

for needle in (
    "Hallmark · genre: modern-minimal · macrostructure: Component Playground",
    "Hallmark · pre-emit critique:", "overflow-x: clip", "prefers-reduced-motion: reduce",
    ":focus-visible", "overflow-wrap: anywhere", "min-width: 0"
):
    assert needle in css, f"missing CSS contract: {needle}"

for width in (320, 375, 414, 768):
    assert str(width) in css, f"missing responsive marker: {width}px"

for needle in ("--color-accent:", "--font-display:", "--font-mono:", "--space-", "--ease-", "oklch("):
    assert needle in tokens, f"missing token: {needle}"

raw_color = re.compile(r"#[0-9a-fA-F]{3,8}|(?<!var\()\b(?:oklch|rgb|hsl)\(")
in_comment = False
for line in css.splitlines():
    stripped = line.strip()
    if stripped.startswith("/*"):
        in_comment = True
    if not in_comment:
        assert not raw_color.search(line), f"raw color outside tokens.css: {line}"
    if "*/" in stripped:
        in_comment = False

for needle in ("localStorage", "matchMedia", "navigator.clipboard", "1400"):
    assert needle in js, f"missing JS behavior: {needle}"

print("site verification passed")
