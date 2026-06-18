# Changelog

## 3.0.1 — 2026-06-18

- Docs: elevated README to a landing page (npm + cargo install, quick start, sibling cross-links, links).
- Meta: added `documentation = "https://docs.rs/faf-wasm-sdk"`.

No code or behavior changes.

## 3.0.0 — 2026-06-16

- Thin `wasm-bindgen` shell over faf-kernel (scoring) + faf-fafb (binary v2). Removed this crate's own `mk4`/`fafb` copies — parity by construction.
