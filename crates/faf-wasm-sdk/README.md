# faf-wasm-sdk

The FAF kernel, for the edge. A thin `wasm-bindgen` shell over
[`faf-kernel`](https://crates.io/crates/faf-kernel) (scoring) +
[`faf-fafb`](https://crates.io/crates/faf-fafb) (binary v2).

**No scoring or format logic lives here.** The same engine that runs in the
CLI and the MCP server runs in the browser and at the edge — so there is
nothing to drift. (v3 replaced this crate's own `mk4.rs`/`fafb` copies with
the workspace kernel.)

8 pure-function exports, JSON / bytes in and out:

```js
import init, { sdk_version, score_faf, validate_faf,
  compile_fafb, decompile_fafb, score_fafb, fafb_info } from 'faf-wasm-sdk';

await init();
const result = score_faf(yaml);    // JSON — Mk4, always-33 slots
const bytes  = compile_fafb(yaml); // Uint8Array — FAFb v2
```

> **v3 behavior:** `score_faf` scores against the always-33 model (no separate
> Base/21); `score_faf_enterprise` is a deprecated alias. Tiers are canonical
> (🏆 ★ ◆ ◇ ● ○ ♡ — no medals). `compile_fafb` emits FAFb **v2**.

Part of the [`faf-rust`](https://github.com/Wolfe-Jam/faf-rust) workspace.

## License

MIT
