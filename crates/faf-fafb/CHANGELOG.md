# Changelog

## 1.0.5 — 2026-09-20

**Spec 2.0 docs: Registration section dropped. No wire change.**

- `BINARY-FORMAT.md` no longer has a Registration section. Wire v2 is unchanged (golden master frozen).

## 1.0.4 — 2026-09-17

- Spec 2.0 lock (no frozen-wire byte change). Flag table corrected to match the golden master: bit 6 is `STRING_TABLE`, bit 7 is `RESOLVED`.
- Two identities: Content ID (stored content-chunk payloads) and file digest (every byte). Stamps and `__` chunks are excluded from the Content ID.
- `entries_within_budget` is tier-tail prefix truncation (64, then 128, then 150–200). Critical (255) always remains.
- Readers pass unknown section names through, including unknown `__` names. Writers reject unknown `__` YAML keys.
- `__score__` is not on the structural closed list. `TOKENIZED` set without `__tokens__` does not reject.
- `SOURCE_DATE_EPOCH` is honoured when stamping `created_timestamp`.
- CHANGELOG: Spec corrected to match the wire: bit 6 is STRING_TABLE. No byte changed.

## 1.0.3 — 2026-09-17

- Docs: point devs at both CLIs, stamp test counts, note v1 ROMs.

## 1.0.2 — 2026-06-18

- Docs: elevated README to a landing page (install, quick start, how-it-works, stability, testing, sibling cross-links, links).
- Meta: added `documentation = "https://docs.rs/faf-fafb"`.

No code or behavior changes. FAFb wire v2 remains frozen (byte-exact golden-master).

## 1.0.1 — 2026-06-16

- FAFb v2 — the compiled binary form of `.faf`. Closed-canonical IFF-style chunks, byte-identical output.
