# rs-llama landing page design

## Summary
Build a developer-first GitHub Pages landing page for `rs-llama` that feels related to the `local-coding-mcp` site but has its own Rust + local inference identity. The site should present `rs-llama` as a Rust library and CLI for `llama.cpp`, highlight real capabilities from the repo, and drive visitors toward install, GitHub, docs, and releases.

## Goals
- Replace a README-like homepage with a polished product-style landing page.
- Communicate the value proposition immediately: Rust + llama.cpp + local GGUF inference.
- Showcase real capabilities only: Hugging Face downloads, vision/mmproj, CPU/CUDA/Vulkan/Metal support, and platform coverage.
- Keep the site static, lightweight, responsive, and easy to deploy via GitHub Pages.

## Audience
- Rust developers building local AI tools.
- Developers comparing `rs-llama` with direct `llama.cpp` usage.
- Users landing from GitHub releases who want quick install and platform information.

## Structure
1. **Header** — project brand, nav links, icon-only theme toggle.
2. **Hero** — eyebrow `Rust binding + CLI for llama.cpp`, H1 `Run llama.cpp from Rust.`, short support copy, primary CTA `Get started`, secondary CTA `View on GitHub`, and a code panel.
3. **Capability rail** — CPU, CUDA, Vulkan, Metal, Vision, Hugging Face, Windows, Linux, macOS, Android / Termux.
4. **Architecture section** — simple flow: `Rust app / CLI -> rs-llama -> LlamaEngine -> llama.cpp / ggml`.
5. **Feature cards** — Cargo install snippet, first model example, vision/mmproj example, backend portability.
6. **Platform / CI support** — polished matrix based on README support table.
7. **Final CTA** — buttons for GitHub and releases.
8. **Footer** — project links and attribution.

## Visual direction
- Same quality bar and clean composition as `local-coding-mcp`, but not a clone.
- Warm rust/copper accent, graphite/slate dark palette, warm off-white light palette.
- Strong sans-serif UI typography with monospace for code.
- Rounded cards, thin borders, restrained shadow/glow, subtle motion only.
- Auto light/dark mode plus manual icon toggle.

## Hallmark constraints
- Use structural variety rather than a generic hero → three-features → CTA template.
- Keep all colours and font declarations behind named design tokens.
- Do not draw fake browser, IDE, terminal, or code-window chrome around code examples.
- Do not invent metrics, testimonials, usage numbers, or compatibility claims.
- Verify mobile layouts at 320, 375, 414, and 768 px.
- Use upright headings; no italic display type.
- Include a Hallmark pre-emit critique stamp in the final CSS artifact after self-review.

## Technical plan
Create a static site under `site/`:
- `site/index.html`
- `site/styles.css`
- `site/app.js`
- optional small assets under `site/assets/`

Add a GitHub Actions Pages workflow that deploys the static site directory without unnecessary build tooling.

## Constraints
- Use only facts already present in the repository README and metadata.
- Do not invent benchmarks, usage stats, or unsupported claims.
- Support 320, 375, 414, 768, and desktop widths.
- Include semantic HTML, focus states, contrast-safe themes, and `prefers-reduced-motion` handling.

## Deliverable
A production-ready single-page landing site for `https://dhhieu113pro.github.io/rs-llama/` with polished light/dark UI, code-first content, responsive layout, and automated GitHub Pages deployment.