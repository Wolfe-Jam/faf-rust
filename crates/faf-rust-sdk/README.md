# faf-rust-sdk

**`.faf` is to context what `package.json` is to dependencies.**

One small, portable file that says what your project is, how it's built, and why —
readable at a glance by a teammate, your own code, or an AI assistant. `.faf` is the
**Foundational AI-context Format**: plain, human-readable YAML, an open IANA-registered
standard. `faf-rust-sdk` is the Rust way to read, validate, score, and compile it.

**IANA media type:** `application/vnd.faf+yaml`

## Why it exists

AI assistants start cold every session — they re-learn your stack, your layout, your
intent, every time. A `.faf` file is the portable context that fixes that: write it
once, and any AI tool (or your own code) can load a complete, structured picture of the
project in a single read. The score tells you how complete that picture is.

## Install

```toml
[dependencies]
faf-rust-sdk = "3.0"
```

## Quick start

```rust
use faf_rust_sdk::{parse, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"
faf_version: 2.5.0
project:
  name: my-app
  goal: Ship a fast CLI
  main_language: Rust
human_context:
  who: Rust developers
  what: A command-line tool
  why: Speed without ceremony
stack:
  build: cargo
tech_stack:
  - Rust
key_files:
  - src/main.rs
"#;

    let faf = parse(content)?;
    println!("Project: {}", faf.project_name());

    let result = validate(&faf);
    println!("Valid: {} — Score: {}%", result.valid, result.score);
    Ok(())
}
```

## Scoring

`.faf` files score **0–100** by how complete the context is, measured against a fixed
33-slot model. A higher score means an AI has more of what it needs to work without
guessing. The score is deterministic — the same file always scores the same.

## Compression

Trim context to fit a model's window:

```rust
use faf_rust_sdk::{compress, CompressionLevel};

let minimal  = compress(&faf, CompressionLevel::Minimal);   // ~150 tokens
let standard = compress(&faf, CompressionLevel::Standard);  // ~400 tokens
let full     = compress(&faf, CompressionLevel::Full);      // ~800 tokens
```

## FAFb — the binary form

`.faf` is the source; **FAFb** is the compiled output — a small, sealed binary with a
checksum, for shipping or caching context fast.

```rust
use faf_rust_sdk::binary::{compile, decompile, CompileOptions};

let bytes  = compile(yaml, &CompileOptions { use_timestamp: false })?;
let result = decompile(&bytes)?;
let name   = result.get_section_string_by_name("project");
```

### How it works (for the curious)

FAFb is modeled on **IFF** — the chunked format Commodore created for the Amiga in the
'80s (Microsoft's RIFF and the ELF executable format use the same idea). Every YAML key
becomes a named section via a **string table**, so there's no fixed enum and no "Unknown"
ceiling. Sections are classified **DNA** (core identity) or **Context** (supplementary),
ordered by priority, and sealed with a CRC32 checksum. The section table uses 16 bytes
per entry for O(1) lookup. Same format for a one-file script or a 600-engineer monorepo.

## Axum integration

Add FAF project context to any Axum server — parsed once at startup, then a single
`Arc::clone` per request:

```toml
[dependencies]
faf-rust-sdk = { version = "3.0", features = ["axum"] }
```

```rust
use axum::{Router, routing::get};
use faf_rust_sdk::axum::{FafLayer, FafContext};

let app: Router = Router::new()
    .route("/", get(handler))
    .layer(FafLayer::new());

async fn handler(faf: FafContext) -> String {
    format!("Project: {}", faf.project_name())
}
```

## Testing

faf-rust-sdk's own suite is **58 WJTTC tests** — 16 Brake (safety), 22 Engine (core),
20 Aero (edge). And because it's a thin facade, every API it exposes is already covered
by the crates beneath it — faf-kernel (62) and faf-fafb (103). **224 tests pass across
the three crates.**

```bash
cargo test -p faf-rust-sdk -p faf-kernel -p faf-fafb   # 224 passing
```

## Part of the FAF Rust workspace

One kernel, many shells:

- [`faf-kernel`](https://crates.io/crates/faf-kernel) — parse, validate, score (the engine)
- [`faf-fafb`](https://crates.io/crates/faf-fafb) — the FAFb v2 binary format
- [`faf-wasm-sdk`](https://crates.io/crates/faf-wasm-sdk) — the same engine for the browser and edge (WASM)

## Links

- [faf.one](https://faf.one) — project home
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
