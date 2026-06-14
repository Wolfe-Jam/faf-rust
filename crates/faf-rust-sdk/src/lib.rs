//! FAF Rust SDK — the facade over the FAF kernel.
//!
//! `faf-rust-sdk` re-exports two crates so downstream code has one dependency
//! for the whole Rust FAF surface:
//!
//! - [`faf-kernel`](https://docs.rs/faf-kernel) — parse, validate, and score
//!   `.faf` files (the kernel).
//! - [`faf-fafb`](https://docs.rs/faf-fafb) — the compiled binary form,
//!   re-exported here under the [`binary`] module.
//!
//! As of 3.0 the SDK contains no logic of its own; it is a stable import
//! surface. The kernel is the single source of truth — the same `faf-kernel`
//! object scores in the CLI, the MCP server, WASM, and the edge worker, so
//! parity is a property of the build, not a test that has to be re-run.
//!
//! # Example
//!
//! ```rust
//! use faf_rust_sdk::{parse, score};
//!
//! let content = r#"
//! faf_version: 2.5.0
//! project:
//!   name: my-project
//!   goal: Build something great
//! "#;
//!
//! let faf = parse(content).unwrap();
//! assert_eq!(faf.project_name(), "my-project");
//!
//! let result = score(content).unwrap();
//! assert!(result.score <= 100);
//! ```

#[cfg(feature = "axum")]
pub mod axum;

/// The compiled binary form of `.faf` — FAFb v2 (re-export of `faf-fafb`).
pub mod binary {
    pub use faf_fafb::*;
}

// Kernel re-exports — parse, validate, score, compress, discover, plus all
// FAF data types.
pub use faf_kernel::*;

// Binary-format types at the crate root (back-compat with pre-3.0 paths).
pub use faf_fafb::{FafbError, FafbHeader, Flags, Priority, SectionEntry, SectionTable};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
