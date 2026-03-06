# faf-sdk (Rust)

High-performance Rust SDK for **FAF (Foundational AI-context Format)** - optimized for inference workloads.

**IANA Media Type:** `application/vnd.faf+yaml`

## Installation

```toml
[dependencies]
faf-rust-sdk = "1.2"
```

## Quick Start

```rust
use faf_rust_sdk::{parse, validate, compress, CompressionLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"
faf_version: 2.5.0
ai_score: "85%"
project:
  name: my-app
  goal: Build something great
instant_context:
  what_building: CLI tool
  tech_stack: Rust, Python
  key_files:
    - src/main.rs
stack:
  backend: Rust
"#;

    // Parse
    let faf = parse(content)?;

    // Access
    println!("Project: {}", faf.project_name());
    println!("Stack: {:?}", faf.tech_stack());
    println!("Score: {:?}", faf.score());

    // Validate
    let result = validate(&faf);
    println!("Valid: {}, Score: {}%", result.valid, result.score);

    // Compress for token optimization
    let minimal = compress(&faf, CompressionLevel::Minimal);

    Ok(())
}
```

## Features

### Zero-Copy Parsing

Designed for high-throughput inference:

```rust
use faf_rust_sdk::parse;

// Parse in ~1ms
let faf = parse(content)?;

// Direct field access - no allocation
let name = faf.project_name();
let stack = faf.tech_stack();
```

### Compression Levels

Optimize for context window constraints:

```rust
use faf_rust_sdk::{compress, CompressionLevel};

// Level 1: ~150 tokens
let minimal = compress(&faf, CompressionLevel::Minimal);

// Level 2: ~400 tokens
let standard = compress(&faf, CompressionLevel::Standard);

// Level 3: ~800 tokens
let full = compress(&faf, CompressionLevel::Full);
```

### Validation

Check structure and completeness:

```rust
use faf_rust_sdk::validate;

let result = validate(&faf);
if result.valid {
    println!("Score: {}%", result.score);
} else {
    println!("Errors: {:?}", result.errors);
}
```

## API

### Core Functions

| Function | Description |
|----------|-------------|
| `parse(content)` | Parse YAML string |
| `parse_file(path)` | Parse from file |
| `validate(&faf)` | Validate structure |
| `compress(&faf, level)` | Compress for tokens |
| `stringify(&faf)` | Convert back to YAML |

### FafFile Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `project_name()` | `&str` | Project name |
| `goal()` | `Option<&str>` | Project goal |
| `score()` | `Option<u8>` | AI score (0-100) |
| `tech_stack()` | `Option<&str>` | Technology stack |
| `what_building()` | `Option<&str>` | What's being built |
| `key_files()` | `&[String]` | Key file paths |
| `is_high_quality()` | `bool` | Score >= 70% |

## Testing

**137/137 passing** — WJTTC Championship-Grade 3-Tier coverage:

| Tier | Tests | What |
|------|-------|------|
| T1 BRAKES | 16 | Security — corruption, validation, type safety |
| T2 ENGINE | 22 | Core — parsing, scoring, compression, discovery |
| T3 AERO | 20 | Polish — unicode, large inputs, YAML quirks |
| Unit | 17 | Inline |
| Doc | 7 | Doctests |

```bash
cargo test
```

## Performance

Optimized for inference workloads:

| Operation | Time |
|-----------|------|
| Parse | <1ms |
| Validate | <0.1ms |
| Compress | <0.1ms |

## Why Rust?

For native AI inference embedding:

- **Zero-copy** where possible
- **No GC** pauses
- **Predictable** latency
- **Easy FFI** to Python/C++

## See Also

- **[mcpaas](https://crates.io/crates/mcpaas)** — Connect to MCPaaS Radio Protocol in Rust. Broadcast context once, every AI receives. faf-rust-sdk reads the format; mcpaas streams it live.

**Do I need both?** Yes. `faf-rust-sdk` parses your .faf project DNA. `mcpaas` streams it live to every AI. One reads, the other delivers.

## Links

- **Spec:** [github.com/Wolfe-Jam/faf](https://github.com/Wolfe-Jam/faf)
- **Site:** [faf.one](https://faf.one)
- **Python SDK:** [faf-python-sdk](https://github.com/Wolfe-Jam/faf-python-sdk)

## License

MIT
