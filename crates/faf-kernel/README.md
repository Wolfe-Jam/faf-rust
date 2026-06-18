# faf-kernel

**The FAF kernel — parse, validate, and score `.faf` files.** The engine every FAF shell runs on.

`.faf` is the **Foundational AI-context Format**, an open IANA-registered standard
(`application/vnd.faf+yaml`): one small, portable YAML file that says what a project is,
how it's built, and why. `faf-kernel` is the single source of truth consumed by every FAF
shell — CLI, MCP server, WASM, edge worker. Score it here, and the CLI, the browser, and
the edge all agree **by construction** — parity is a property of the build, not a test.

**IANA media type:** `application/vnd.faf+yaml`

## Install

```toml
[dependencies]
faf-kernel = "1.0"
```

## Quick start

```rust
use faf_kernel::{parse, score};

let faf = parse("faf_version: 2.5.0\nproject:\n  name: my-project\n").unwrap();
assert_eq!(faf.project_name(), "my-project");

let result = score("project:\n  name: x\n").unwrap();
assert!(result.score <= 100); // Mk4 33-slot scoring, 0–100
```

## Scoring (Mk4, always-33)

`.faf` files score **0–100** against a fixed **33-slot** model. Each slot is populated,
empty, or `slotignored`; score = populated ÷ active (33 − slotignored). The kernel knows
nothing about `app_type`, owner, or intent — a 600-engineer monorepo and a one-file script
are the same object: a fill pattern over 33 slots. `app_type` decides which slots are
written `slotignored` at generation time; the kernel only reads the markers. The score is
**deterministic** — the same file always scores the same.

**Tiers:** Trophy 🏆 is the only emoji; sub-Trophy tiers are clean Unicode (★ ◆ ◇ ● ○ ♡).

## Testing

**62 WJTTC tests** (Brake / Engine / Aero). As the kernel beneath every shell, this is the
code those shells are tested against too — **233 tests pass across the FAF Rust workspace.**

```bash
cargo test -p faf-kernel
```

## Part of the FAF Rust workspace

One kernel, many shells:

- [`faf-fafb`](https://crates.io/crates/faf-fafb) — the FAFb v2 binary format (the compiled brick)
- [`faf-rust-sdk`](https://crates.io/crates/faf-rust-sdk) — the high-level SDK facade (kernel + fafb + Axum)
- [`faf-wasm-sdk`](https://crates.io/crates/faf-wasm-sdk) — the same engine for the browser and edge (WASM)

## Links

- [faf.one](https://faf.one) — project home · [the format spec](https://faf.one/spec)
- [IANA registration](https://www.iana.org/assignments/media-types/application/vnd.faf+yaml) — `application/vnd.faf+yaml`
- [FAF on Zenodo](https://doi.org/10.5281/zenodo.18251362) — academic paper
- [FAF on Grokipedia](https://grokipedia.com/page/faf-file-format)

## License

MIT

---

### Get the CLI

> **faf-cli** — the original AI-context CLI. A must-have for every builder.

```bash
npx faf-cli auto
```

**Anthropic MCP [#2759](https://github.com/modelcontextprotocol/servers/pull/2759)** · **IANA:** `application/vnd.faf+yaml` · [faf.one](https://faf.one) · [npm](https://www.npmjs.com/package/faf-cli)
