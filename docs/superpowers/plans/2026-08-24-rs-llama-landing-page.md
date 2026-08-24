# rs-llama Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and deploy a polished, developer-first GitHub Pages landing page for `rs-llama` using the approved Hallmark direction.

**Architecture:** A framework-free static site lives under `site/`, with design tokens separated into `site/tokens.css`, page composition in `site/styles.css`, and behavior in `site/app.js`. A zero-dependency Python verifier checks Hallmark invariants and content claims before the Pages workflow uploads `site/` as the deployment artifact.

**Tech Stack:** HTML5, CSS custom properties/OKLCH, vanilla JavaScript, Python 3 stdlib validation, GitHub Actions Pages.

**Spec:** `docs/superpowers/specs/2026-08-24-rs-llama-landing-page-design.md`

## Global Constraints

- Use only facts already present in the repository README and metadata.
- Do not invent benchmarks, usage stats, testimonials, or unsupported compatibility claims.
- Hallmark genre: `modern-minimal`.
- Hallmark macrostructure: `Component Playground`.
- Hallmark theme route: tuned custom, vibe `rust precision, local inference, technical restraint`.
- Use named design tokens for every colour and every `font-family` declaration.
- Do not draw fake browser, IDE, terminal, or code-window chrome.
- Mobile layouts must work at 320, 375, 414, and 768 px with no horizontal page scroll.
- `html` and `body` use `overflow-x: clip`.
- Display headings are upright and use `overflow-wrap: anywhere; min-width: 0`.
- Interactive controls have visible `:focus-visible`, hover, active, disabled/loading/error/success styling where applicable.
- Respect `prefers-reduced-motion` and animate only opacity/transform.
- Auto-detect system light/dark mode; icon-only theme button toggles an explicit override and persists it in `localStorage`.
- Copy buttons use silent success feedback (`Copied`) and restore after 1400 ms.
- Maintain `.hallmark/preflight.json` and `.hallmark/log.json` for future Hallmark runs.

---

### Task 1: Static-site contract and Hallmark memory

**Files:**
- Create: `scripts/verify_site.py`
- Create: `.hallmark/preflight.json`
- Create: `.hallmark/log.json`

**Interfaces:**
- Consumes: approved spec and README facts.
- Produces: a zero-dependency verifier used locally and in Pages CI; Hallmark project memory.

- [ ] **Step 1: Write the failing verifier**

Create `scripts/verify_site.py` that reads `site/index.html`, `site/tokens.css`, `site/styles.css`, and `site/app.js`; fail if files are missing. Assert required links/snippets, semantic IDs, Hallmark stamps, `overflow-x: clip`, `prefers-reduced-motion`, `:focus-visible`, responsive markers for 320/375/414/768, named tokens, and JS use of `localStorage`, `matchMedia`, `navigator.clipboard`, and `1400`.

Core contract:

```python
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

for needle in ("localStorage", "matchMedia", "navigator.clipboard", "1400"):
    assert needle in js, f"missing JS behavior: {needle}"

print("site verification passed")
```

- [ ] **Step 2: Run `python scripts/verify_site.py` and verify it fails with `missing site files`.**

- [ ] **Step 3: Add `.hallmark/preflight.json`** recording vanilla HTML/CSS/JS, no prior palette/font system, no motion library, and the introduced token/macrostructure disciplines.

- [ ] **Step 4: Add `.hallmark/log.json`** with newest entry first:

```json
[
  {
    "date": "2026-08-24",
    "macrostructure": "Component Playground",
    "theme": "custom",
    "theme_axes": "warm-near-white / technical-sans / rust-copper",
    "vibe": "rust precision, local inference, technical restraint",
    "enrichment": "none",
    "brief": "rs-llama developer-first GitHub Pages landing page",
    "nav": "N9 edge-aligned minimal",
    "footer": "Ft2 inline single line"
  }
]
```

- [ ] **Step 5: Commit** with `test: define rs-llama landing page contract`.

---

### Task 2: Semantic page and portable tokens

**Files:**
- Create: `site/index.html`
- Create: `site/tokens.css`

**Interfaces:**
- Consumes: README examples/platform facts and Task 1 contract.
- Produces: stable DOM IDs/classes/data attributes consumed by CSS/JS.

- [ ] **Step 1: Create `site/tokens.css`** with a tuned OKLCH rust/copper palette for light/dark modes; all colours and font stacks live here. Include `--color-paper`, `--color-paper-2`, `--color-paper-3`, `--color-ink`, `--color-ink-2`, `--color-muted`, `--color-rule`, `--color-rule-2`, `--color-accent`, `--color-accent-ink`, `--color-focus`, `--color-code`, success/error colours, `--font-display`, `--font-body`, `--font-mono`, 4pt-derived space tokens, text scale, radii, durations, and the three Hallmark easing tokens. Provide both system dark fallback and explicit `[data-theme="dark"]` overrides.

- [ ] **Step 2: Create `site/index.html`** with semantic `header/nav/main/section/footer`, skip link, N9 edge-aligned nav, icon-only theme toggle, two-column hero, capability rail, architecture flow, code-first examples, responsive platform table, closing CTA, inline footer, and live-region copy status.

The hero copy is:

```text
Rust binding + CLI for llama.cpp
Run llama.cpp from Rust.
Load GGUF models locally, download from Hugging Face, add vision with mmproj, and choose CPU, CUDA, Vulkan, or Metal.
```

Use the README's exact install snippet:

```toml
[dependencies]
rs-llama = { git = "https://github.com/dhhieu113pro/rs-llama" }
```

Use the README's exact Rust generation example, vision command using `--image ./photo.jpg`, and backend commands using `--features cuda`, `--features vulkan`, and `--features metal`.

Platform rows must reflect exactly:

```text
Linux x86_64        LED smoke + vision              rs-llama-linux-x86_64.tar.gz
Windows x86_64      LED smoke + vision              rs-llama-windows-x86_64.zip
macOS Apple Silicon Metal + LED smoke + vision      rs-llama-macos-arm64.tar.gz
Android / Termux    NDK build + emulator vision     rs-llama-android-arm64.tar.gz
NVIDIA CUDA         Linux compile                   --features cuda
Vulkan              Linux compile                   --features vulkan
```

- [ ] **Step 3: Run verifier.** Expected: it now fails only because `styles.css`/`app.js` contracts are not yet present.

- [ ] **Step 4: Commit** with `feat: add rs-llama landing page content`.

---

### Task 3: Hallmark styling and interactions

**Files:**
- Create: `site/styles.css`
- Create: `site/app.js`

**Interfaces:**
- Consumes: Task 2 DOM/tokens.
- Produces: responsive Hallmark visual system, theme state, copy-button state.

- [ ] **Step 1: Create `site/styles.css`.** First non-empty lines:

```css
/* Hallmark · genre: modern-minimal · macrostructure: Component Playground · theme: custom · tone: technical/austere · anchor hue: rust-copper
 * vibe: rust precision, local inference, technical restraint
 * paper: oklch(97% 0.012 55) · accent: oklch(61% 0.16 42)
 * display: system sans · body: system sans · mono: system mono
 * axes: warm-near-white / technical-sans / rust-copper · enrichment: none · nav: N9 · footer: Ft2
 */
/* Hallmark · pre-emit critique: P4 H4 E4 S5 R5 V4 */
```

Implement `html, body { overflow-x: clip; }`, token-only colours/fonts, upright headings with `overflow-wrap:anywhere` and `min-width:0`, visible focus rings, two-column hero, non-card capability rail, CSS architecture flow, alternating non-uniform code playground blocks, responsive semantic table, restrained closing statement, Ft2 footer, and internal code scrolling.

At <=900px stack hero/section headings. At <=640px hide desktop nav, keep clickable text single-line, reduce gutters, and reflow table rows. Include comments `verified target: 320px`, `375px`, `414px`, `768px` after manual checks.

Copy buttons must style default, hover, focus, active, disabled, loading, error, success states. Respect `prefers-reduced-motion: reduce`; animate only transform/opacity.

- [ ] **Step 2: Create `site/app.js`** with system-aware theme and silent copy behavior:

```javascript
(() => {
  const root = document.documentElement;
  const toggle = document.getElementById('theme-toggle');
  const icon = document.getElementById('theme-icon');
  const status = document.getElementById('copy-status');
  const media = window.matchMedia('(prefers-color-scheme: dark)');
  const storageKey = 'rs-llama-theme';
  const effectiveTheme = () => root.dataset.theme || (media.matches ? 'dark' : 'light');
  const renderTheme = () => {
    const theme = effectiveTheme();
    if (icon) icon.textContent = theme === 'dark' ? '☀' : '◐';
    if (toggle) toggle.title = theme === 'dark' ? 'Use light theme' : 'Use dark theme';
  };
  const saved = localStorage.getItem(storageKey);
  if (saved === 'light' || saved === 'dark') root.dataset.theme = saved;
  renderTheme();
  toggle?.addEventListener('click', () => {
    const next = effectiveTheme() === 'dark' ? 'light' : 'dark';
    root.dataset.theme = next;
    localStorage.setItem(storageKey, next);
    renderTheme();
  });
  media.addEventListener?.('change', () => { if (!root.dataset.theme) renderTheme(); });
  document.querySelectorAll('[data-copy-target]').forEach((button) => {
    button.addEventListener('click', async () => {
      const target = document.getElementById(button.dataset.copyTarget);
      if (!target || !navigator.clipboard) return;
      const original = button.textContent;
      button.dataset.state = 'loading';
      try {
        await navigator.clipboard.writeText(target.textContent);
        button.dataset.state = 'success';
        button.textContent = 'Copied';
        if (status) status.textContent = 'Code copied to clipboard.';
      } catch {
        button.dataset.state = 'error';
        button.textContent = 'Retry';
        if (status) status.textContent = 'Copy failed. Select the code manually.';
      }
      window.setTimeout(() => {
        button.dataset.state = 'default';
        button.textContent = original;
      }, 1400);
    });
  });
})();
```

- [ ] **Step 3: Run `python scripts/verify_site.py`.** Expected: `site verification passed`.

- [ ] **Step 4: Serve with `python -m http.server 8080 --directory site`** and manually inspect 320/375/414/768/desktop widths, both themes, copy state, internal code scrolling, and mobile table.

- [ ] **Step 5: Commit** with `feat: style and activate rs-llama landing page`.

---

### Task 4: GitHub Pages deploy and final verification

**Files:**
- Create: `.github/workflows/pages.yml`
- Modify: `README.md` only to add homepage link if useful; preserve all existing technical content.

**Interfaces:**
- Consumes: `site/`, verifier.
- Produces: verified Pages deployment on `main` and manual dispatch.

- [ ] **Step 1: Create `.github/workflows/pages.yml`:**

```yaml
name: Deploy GitHub Pages

on:
  push:
    branches: [main]
    paths:
      - "site/**"
      - "scripts/verify_site.py"
      - ".github/workflows/pages.yml"
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify static site
        run: python scripts/verify_site.py
      - uses: actions/configure-pages@v5
      - uses: actions/upload-pages-artifact@v3
        with:
          path: site
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Run final validation:**

```bash
python scripts/verify_site.py
cargo fmt --all -- --check
cargo check
```

All must exit 0; the landing-page change must not affect Rust behavior.

- [ ] **Step 3: Run Hallmark pre-emit self-review** on Philosophy, Hierarchy, Execution, Specificity, Restraint, Variety. Any axis below 3 triggers a revision. Update the CSS critique stamp to actual scores and confirm no invented metrics/social proof, fake UI chrome, raw colours in page CSS, italic headings, excessive accent, or generic hero→three-cards rhythm.

- [ ] **Step 4: Commit** with `ci: deploy rs-llama landing page to GitHub Pages`.

- [ ] **Step 5: Push `feat/rs-llama-pages`, open a PR to `main`, and verify repository checks.** PR summary:

```text
- add Hallmark-designed static landing page for rs-llama
- add system-aware light/dark theme and copyable real examples
- add zero-dependency site validation and GitHub Pages deployment
```

Fix only failures caused by this PR. Once checks pass, report the PR ready for merge.
