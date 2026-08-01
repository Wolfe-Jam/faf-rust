# faf-rust

The FAF Rust workspace — **one kernel, many shells.**

**.faf** — Foundational AI-context Format. IANA-registered as `application/vnd.faf+yaml`.

**This repo is the source of truth** for foundation crates on crates.io.  
Full name map (live · superseded · reserved): **[docs/CRATE-SUPERSESSION.md](docs/CRATE-SUPERSESSION.md)**.

## Crates (live)

| Crate | crates.io | What |
|-------|-----------|------|
| [`faf-kernel`](crates/faf-kernel) | [faf-kernel](https://crates.io/crates/faf-kernel) | Parse, validate, Mk4 score |
| [`faf-fafb`](crates/faf-fafb) | [faf-fafb](https://crates.io/crates/faf-fafb) | FAFb binary brick |
| [`faf-rust-sdk`](crates/faf-rust-sdk) | [faf-rust-sdk](https://crates.io/crates/faf-rust-sdk) **3.x** | Facade → kernel + fafb |
| [`faf-wasm-sdk`](crates/faf-wasm-sdk) | [faf-wasm-sdk](https://crates.io/crates/faf-wasm-sdk) **3.x** | WASM shell → kernel + fafb |

```toml
[dependencies]
faf-rust-sdk = "3"
```

Related products (separate repos): [`rust-faf-mcp`](https://crates.io/crates/rust-faf-mcp) · [`mcp-better`](https://crates.io/crates/mcp-better).  
Older crate names stay under FAF ownership — see the [supersession map](docs/CRATE-SUPERSESSION.md) for “use instead.”

## Build

```bash
cargo build --workspace
cargo test --workspace

# Full ship bar (same gates as GitHub CI — run before push)
bash scripts/ci.sh
# Optional: block push on red CI twin
bash scripts/install-hooks.sh
```

## Links

- [faf.one](https://faf.one) — project home
- [FAF specification](https://github.com/Wolfe-Jam/faf)
- [Crate supersession map](docs/CRATE-SUPERSESSION.md) — namespace stewardship

If `faf-rust` has been useful, consider starring the repo — it helps others find it.

## License

MIT
