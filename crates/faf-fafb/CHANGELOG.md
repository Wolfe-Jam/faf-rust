# Changelog

## 1.0.6 — 2026-09-23

**The serializer is pinned and the payload contract is written down. No wire change.**

- `serde_yaml_ng` pinned to `=0.10.0`. The payload bytes this crate emits *are* the
  Content ID, so a caret range let a dependent's resolution move them and break every
  published ID with no spec change.
- `BINARY-FORMAT.md` § Payload, new: a payload is `name:\n` + the value serialized at
  column 0, so it is not a one-key mapping. Readers drop the first line and parse the
  remainder, which round-trips with nesting intact. Payloads are YAML 1.2 core — `yes`,
  `no`, `on` and `off` are strings, emitted unquoted; a YAML 1.1 parser reads them as
  booleans.
- `tests/parity/serializer-edge.faf` + `.fafb`: a second golden carrying the quoting
  cases a serializer upgrade actually moves (bool-like and number-like strings, block
  and folded scalars, embedded colons, quotes, tabs, newlines, unicode, deep nesting).
  Byte-pinned, Content ID `63ea6d8a…`.
- Three tests: the edge-fixture byte pin, the payload round-trip, and the negative half
  (a whole payload is not a YAML document).
- The sibling list no longer says the FAFb CLI writes v1 ROMs — it compiles wire v2
  through this crate as of 0.9.5.

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
