<!-- latest=v3.0.0 -->
# Changelog

## [3.0.0] — 2026-09-25 — The Always33 Edition

One engine, one number: `score_faf` is always-33, so a repo scores the same in every FAF app.

- **Breaking:** `score_faf` scores all 33 slots — it returns faf-kernel's score unchanged. A file without its 12 enterprise `slotignored` markers now scores against 33 (e.g. 100 → 64 in 2.1.0 terms); a file that carries them scores the same as before.
- `score_faf_enterprise` is an alias of `score_faf` — existing callers keep working.
- `score_fafb` agrees with `score_faf`.
- `tbd` / `todo` (any case) are placeholders — they count as empty.
- Built from `faf-wasm-sdk` **3.1.0** (`sdk_version()`) over `faf-kernel` **1.1.1**.
- Parity: `score_faf` equals the always-33 reference on all 80 FAF repo `project.faf` files.

## [2.1.0] — 2026-09-20 — The Brick Edition

`compile_fafb` emits FAFb wire v2 — same bytes as the faf-fafb golden.

- `compile_fafb` emits FAFb **v2** (`version_major = 2`). Built from `faf-wasm-sdk` **3.0.1** against `faf-fafb` 1.0.4 (1.0.5 is docs-only; wire unchanged).
- `score_faf` is 21-slot base; `score_faf_enterprise` is 33-slot Mk4.
- First npm release since 2.0.3 (2026-03-15).

## [2.0.3] — 2026-03-15

README / docs patches on 2.0.0.

## [2.0.0] — 2026-03-15 — The Definitive Edition

First npm `faf-scoring-kernel` as a named WASM package.
