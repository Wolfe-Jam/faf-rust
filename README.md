# faf-rust

The FAF Rust workspace — one kernel, many shells.

**.faf** — Foundational AI-context Format. IANA-registered as `application/vnd.faf+yaml`.

## Crates

| Crate | What |
|-------|------|
| [`faf-rust-sdk`](crates/faf-rust-sdk) | Parse, validate, score, and compile `.faf` files |

More crates join as the workspace consolidates the FAF Rust estate. Each published crate keeps its existing name on crates.io — same packages, same installs.

## Build

```bash
cargo build --workspace
cargo test --workspace
```

## Links

- [faf.one](https://faf.one) — project home
- [FAF specification](https://github.com/Wolfe-Jam/faf)

If `faf-rust` has been useful, consider starring the repo — it helps others find it.

## License

MIT
