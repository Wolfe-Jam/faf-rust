# faf-kernel

The FAF kernel — parse, validate, and score `.faf` files.

`.faf` is the **Foundational AI-context Format**, IANA-registered as
`application/vnd.faf+yaml`. This crate is the single source of truth consumed
by every FAF shell: CLI, MCP server, WASM, edge worker. Score it here, and the
CLI, the browser, and the edge all agree by construction — parity is a property
of the build, not a test.

```rust
use faf_kernel::{parse, score};

let faf = parse("faf_version: 2.5.0\nproject:\n  name: my-project\n").unwrap();
assert_eq!(faf.project_name(), "my-project");

let result = score("project:\n  name: x\n").unwrap();
assert!(result.score <= 100); // Mk4 33-slot scoring, 0–100
```

- **Scoring:** Mk4 engine, always against the 33-slot model. Each slot is
  populated, empty, or `slotignored`; score = populated ÷ active (33 −
  slotignored). The kernel knows nothing about `app_type`, owner, or intent —
  a complex repo and a minimal profile are the same object: a fill pattern over
  33 slots. `app_type` decides which slots are written `slotignored` at
  generation time; the kernel only reads the markers.
- **Tiers:** Trophy 🏆 is the only emoji; sub-Trophy tiers are clean Unicode
  (★ ◆ ◇ ● ○ ♡).

Part of the [`faf-rust`](https://github.com/Wolfe-Jam/faf-rust) workspace.

## License

MIT
