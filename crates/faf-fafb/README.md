# faf-fafb

**FAFb v2 — the compiled binary form of `.faf`. The brick.**

`.faf` is the source (human-readable YAML, IANA `application/vnd.faf+yaml`); **FAFb** is the
compiled output — a small, sealed binary with a checksum, for shipping or caching project
context fast. Context, compiled.

## Install

```toml
[dependencies]
faf-fafb = "1.0"
```

## Quick start

```rust
use faf_fafb::{compile, decompile, CompileOptions};

let yaml = "faf_version: 2.5.0\nproject:\n  name: my-project\n";
let bytes = compile(yaml, &CompileOptions { use_timestamp: false }).unwrap();
assert_eq!(&bytes[0..4], b"FAFB");
assert_eq!(bytes[4], 2); // FAFb v2

let result = decompile(&bytes).unwrap();
assert!(result.get_section_string_by_name("project").unwrap().contains("my-project"));
```

## How it works

IFF-inspired chunked binary: a string table, a section table at the end for O(1) random
access, classification bits (DNA / Context / Pointer), priority-based truncation, and a
CRC32 seal over the source.

**Closed canonical.** The writer emits exactly the canonical chunk set in canonical order;
non-canonical keys fold into the `context` chunk. Identical content compiles to **identical
bytes** regardless of input key order — so a `.fafb` is content-addressable: the same project
context, the same hash, everywhere. The reader keeps the IFF rule (skip unknown names), so a
future minor version can add a chunk without breaking deployed readers.

**v2 only** — FAFb v1 is pre-release history and is rejected on read; re-compile from the
`.faf` source. Full spec: [`BINARY-FORMAT.md`](BINARY-FORMAT.md) · [faf.one/spec](https://faf.one/spec).

## Stability — wire v2 is frozen

The byte layout is **immutable**, enforced by a byte-exact golden-master test in the crate:
`compile()` must reproduce the vendored `.fafb` byte-for-byte; any structural change is caught
immediately. New capabilities ship only as forward-compatible additions — new chunks or flag
bits older readers skip. We do not break v2.

Because the `.faf` source is always authoritative, you **recompile, never migrate** — nothing
gets trapped in an old binary.

## Testing

**103 WJTTC tests**, including the byte-exact golden-master seal — **233 tests pass across the
FAF Rust workspace.**

```bash
cargo test -p faf-fafb
```

## Part of the FAF Rust workspace

One kernel, many shells:

- [`faf-kernel`](https://crates.io/crates/faf-kernel) — parse, validate, score (the engine)
- [`faf-rust-sdk`](https://crates.io/crates/faf-rust-sdk) — the high-level SDK facade
- [`faf-wasm-sdk`](https://crates.io/crates/faf-wasm-sdk) — the same engine for the browser and edge (WASM)

## Links

- [faf.one](https://faf.one) — project home · [the format spec](https://faf.one/spec)
- [IANA registration](https://www.iana.org/assignments/media-types/application/vnd.faf+yaml) — `application/vnd.faf+yaml`
- [FAF on Zenodo](https://doi.org/10.5281/zenodo.18251362) · [Grokipedia](https://grokipedia.com/page/faf-file-format)

## License

MIT

---

### Get the CLI

> **faf-cli** — the original AI-context CLI. A must-have for every builder.

```bash
npx faf-cli auto
```

**Anthropic MCP [#2759](https://github.com/modelcontextprotocol/servers/pull/2759)** · **IANA:** `application/vnd.faf+yaml` · [faf.one](https://faf.one) · [npm](https://www.npmjs.com/package/faf-cli)
