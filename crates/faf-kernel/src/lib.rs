//! faf-kernel — the FAF kernel.
//!
//! Parse, validate, and score `.faf` files (IANA-registered
//! `application/vnd.faf+yaml`). This crate is the single source of truth
//! consumed by every FAF shell — CLI, MCP server, WASM, edge worker.
//!
//! # Example
//!
//! ```rust
//! use faf_kernel::{parse, score};
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

mod compress;
mod discovery;
mod parser;
mod score;
mod types;
mod validator;

pub use compress::{CompressionLevel, compress, estimate_tokens};
pub use discovery::{FindError, find_and_parse, find_faf_file};
pub use parser::{FafError, FafFile, parse, parse_file, stringify};
pub use score::{Mk4Result, Mk4Scorer, SlotState, Universe, score, tier_name, tier_symbol};
pub use types::*;
pub use validator::{ValidationResult, validate};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
