# Changelog

## 1.1.0 — 2026-08-25

- Added: `FafData.commands` (top-level build/test/lint/dev map), `FafData.security`
  (secrets/example/never), `FafData.ai_instructions` (warnings/working_style),
  `FafData.conventions` — closing the gap against what AGENTS.md generators
  (faf-cli's `faf export --agents`) actually read from a `.faf` file.
- Added: `FafFile::commands()` — reads top-level `commands` first, falls back to
  `instant_context.commands` for older `.faf` files that nest them there.
- Additive, backward compatible — no existing field renamed or removed.

## 1.0.1 — 2026-06-18

- Docs: elevated README to a landing page (install, quick start, scoring, testing, sibling cross-links, links).
- Docs: full API doc coverage — documented the public data-model types and their fields, the `Mk4Result` fields, and the `FafError` variants.
- Meta: added `documentation = "https://docs.rs/faf-kernel"`.

No code or behavior changes.

## 1.0.0 — 2026-06-16

- Initial release. The FAF kernel — parse, validate, and score `.faf` files (Mk4, 33-slot model).
