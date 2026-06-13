# faf-fafb

FAFb v2 — the compiled binary form of `.faf`. The brick.

IFF-inspired chunked binary: string table, section table at the end for O(1)
random access, classification bits (DNA / Context / Pointer), priority-based
truncation, CRC32 seal over the source.

**Closed canonical.** The writer emits exactly the canonical chunk set in
canonical order; non-canonical keys fold into the `context` chunk. Identical
content compiles to **identical bytes** regardless of input key order — so a
`.fafb` is content-addressable: the same project context, the same hash,
everywhere. The reader keeps the IFF rule (skip unknown names), so a future
minor version can add a chunk without breaking deployed readers.

```rust
use faf_fafb::{compile, decompile, CompileOptions};

let yaml = "faf_version: 2.5.0\nproject:\n  name: my-project\n";
let bytes = compile(yaml, &CompileOptions { use_timestamp: false }).unwrap();
assert_eq!(&bytes[0..4], b"FAFB");
assert_eq!(bytes[4], 2); // FAFb v2

let result = decompile(&bytes).unwrap();
assert!(result.get_section_string_by_name("project").unwrap().contains("my-project"));
```

**v2 only** — FAFb v1 is pre-release history and is rejected on read;
re-compile from the `.faf` source. Full spec: [`BINARY-FORMAT.md`](BINARY-FORMAT.md).

Part of the [`faf-rust`](https://github.com/Wolfe-Jam/faf-rust) workspace.

## License

MIT
