# faf-core

The FAF kernel — parse, validate, and score `.faf` files.

`.faf` is the **Foundational AI-context Format**, IANA-registered as
`application/vnd.faf+yaml`. This crate is the single source of truth consumed
by every FAF shell: CLI, MCP server, WASM, edge worker. Score it here, and the
CLI, the browser, and the edge all agree by construction — parity is a property
of the build, not a test.

```rust
use faf_core::{parse, score};

let faf = parse("faf_version: 2.5.0\nproject:\n  name: my-project\n").unwrap();
assert_eq!(faf.project_name(), "my-project");

let result = score("project:\n  name: x\n").unwrap();
assert!(result.score <= 100); // Mk4 33-slot scoring, 0–100
```

- **Scoring:** Mk4 engine. The slot universe (21 vs 33) is derived from the
  document's `app_type` — no license logic in the kernel.
- **Tiers:** Trophy 🏆 is the only emoji; sub-Trophy tiers are clean Unicode
  (★ ◆ ◇ ● ○ ♡).

Part of the [`faf-rust`](https://github.com/Wolfe-Jam/faf-rust) workspace.

## License

MIT
