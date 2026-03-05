//! FAFB Binary Format
//!
//! Implementation of the .fafb binary format specification.
//! Compiles human-readable .faf (YAML) to AI-optimized binary.
//!
//! ## Features
//!
//! - **O(1) section lookup** - Section table at end for instant access
//! - **Priority truncation** - Smart context window management
//! - **Pre-computed tokens** - No runtime estimation
//! - **Memory mapping ready** - Zero-copy loading design
//!
//! ## Usage
//!
//! ```ignore
//! use faf_rust_sdk::binary::{FafbHeader, Flags, SectionEntry, SectionTable, SectionType, Priority};
//!
//! // Create a new header
//! let mut header = FafbHeader::with_timestamp();
//! header.set_source_checksum(yaml_content.as_bytes());
//! header.section_count = 3;
//!
//! // Create section table
//! let mut table = SectionTable::new();
//! table.push(SectionEntry::new(SectionType::Meta, 32, 100));
//! table.push(SectionEntry::new(SectionType::TechStack, 132, 200)
//!     .with_priority(Priority::high()));
//!
//! // Budget-aware loading
//! let sections = table.entries_within_budget(1000);
//! ```
//!
//! ## Format Version
//!
//! Current: v1.0
//!
//! See FAFB-BINARY-SPEC.md for full specification.

pub mod compile;
pub mod error;
pub mod flags;
pub mod header;
pub mod priority;
pub mod section;
pub mod section_type;

// Re-exports for convenience
pub use compile::{compile, decompile, DecompiledFafb};
pub use error::{FafbError, FafbResult};
pub use flags::{
    Flags, FLAG_COMPRESSED, FLAG_EMBEDDINGS, FLAG_MODEL_HINTS, FLAG_SIGNED, FLAG_TOKENIZED,
    FLAG_WEIGHTED,
};
pub use header::{
    FafbHeader, HEADER_SIZE, MAGIC, MAGIC_U32, MAX_FILE_SIZE, MAX_SECTIONS, VERSION_MAJOR,
    VERSION_MINOR,
};
pub use priority::{
    Priority, PRIORITY_CRITICAL, PRIORITY_HIGH, PRIORITY_LOW, PRIORITY_MEDIUM, PRIORITY_OPTIONAL,
};
pub use section::{SectionEntry, SectionTable, SECTION_ENTRY_SIZE};
pub use section_type::{
    SectionType, SECTION_ARCHITECTURE, SECTION_BISYNC, SECTION_COMMANDS, SECTION_CONTEXT,
    SECTION_CUSTOM, SECTION_EMBEDDINGS, SECTION_KEY_FILES, SECTION_META, SECTION_MODEL_HINTS,
    SECTION_TECH_STACK, SECTION_TOKEN_MAP,
};
