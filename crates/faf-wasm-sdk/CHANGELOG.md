# Changelog

## 3.1.0 — 2026-09-25 — The Always33 Edition

One engine, one number: `score_faf` is always-33 again, so a repo scores the same in every FAF app.

- **`score_faf` is always-33 again** — it returns `faf_kernel::score` unchanged. The 21-slot split added in `d53e489` (2026-09-18, npm `faf-scoring-kernel` 2.1.0) is gone: a file without its 12 enterprise `slotignored` markers scored 100 there and 64 in the kernel. One engine, one number, in every FAF app.
- `score_faf_enterprise` stays as an alias of `score_faf`, so existing callers keep working.
- `score_fafb` now agrees with `score_faf` (both always-33).
- Requires faf-kernel 1.1.1 (`tbd` / `todo` are placeholders).
- Parity: `score_faf` = the always-33 reference on all 80 FAF repo `project.faf` files (node + web builds).

## 3.0.1 — 2026-06-18

- Docs: elevated README to a landing page (npm + cargo install, quick start, sibling cross-links, links).
- Meta: added `documentation = "https://docs.rs/faf-wasm-sdk"`.

No code or behavior changes.

## 3.0.0 — 2026-06-16

- Thin `wasm-bindgen` shell over faf-kernel (scoring) + faf-fafb (binary v2). Removed this crate's own `mk4`/`fafb` copies — parity by construction.
