# faf-wasm-sdk

**The FAF kernel, for the edge.** A thin `wasm-bindgen` shell over
[`faf-kernel`](https://crates.io/crates/faf-kernel) (scoring) +
[`faf-fafb`](https://crates.io/crates/faf-fafb) (binary v2) — so the same engine that runs in
the CLI and the MCP server runs in the browser and at the edge, with **nothing to drift.**

`.faf` is the **Foundational AI-context Format**, IANA-registered as `application/vnd.faf+yaml`.

## Install

From npm (browser / edge):

```bash
npm install faf-wasm-sdk
```

Or as a Rust dependency:

```toml
[dependencies]
faf-wasm-sdk = "3.0"
```

## Quick start (JS / WASM)

7 pure-function exports — JSON / bytes in and out:

```js
import init, { sdk_version, score_faf, validate_faf,
  compile_fafb, decompile_fafb, score_fafb, fafb_info } from 'faf-wasm-sdk';

await init();
const result = score_faf(yaml);    // JSON — Mk4, always-33 slots
const bytes  = compile_fafb(yaml); // Uint8Array — FAFb v2
```

> **No scoring or format logic lives here.** v3 replaced this crate's own `mk4.rs` / `fafb`
> copies with the workspace kernel. `score_faf` scores against the **always-33** model — one
> scorer for every project, solo script to enterprise monorepo. Tiers are canonical
> (🏆 ★ ◆ ◇ ● ○ ♡ — no medals). `compile_fafb` emits FAFb **v2**.

## Part of the FAF Rust workspace

One kernel, many shells:

- [`faf-kernel`](https://crates.io/crates/faf-kernel) — parse, validate, score (the engine)
- [`faf-fafb`](https://crates.io/crates/faf-fafb) — the FAFb v2 binary format
- [`faf-rust-sdk`](https://crates.io/crates/faf-rust-sdk) — the high-level SDK facade (native)

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
