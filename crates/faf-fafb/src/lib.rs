//! faf-fafb — FAFb v2, the compiled binary form of `.faf`.
//!
//! IFF-inspired chunked binary: a string table for section names, a section
//! table at the end for O(1) random access, classification bits (DNA /
//! Context / Pointer), priority-based truncation, and a CRC32 seal over the
//! source `.faf`.
//!
//! **Closed canonical.** The writer emits exactly the canonical chunk set
//! (see [`canon`]), in canonical order; non-canonical top-level keys fold into
//! the `context` chunk. Identical content produces byte-identical output
//! regardless of input key order — so a `.fafb` is content-addressable: the
//! same project context yields the same hash, everywhere. The reader keeps the
//! IFF rule (skip unknown section names gracefully), so a future minor version
//! can add a chunk without breaking deployed readers.
//!
//! **v2 only.** FAFb v1 is pre-release history and is rejected on read
//! (`IncompatibleVersion`) — re-compile from the `.faf` source, which is always
//! the source of truth.
//!
//! ## Usage
//!
//! ```rust
//! use faf_fafb::{compile, decompile, CompileOptions};
//!
//! let yaml = "faf_version: 2.5.0\nproject:\n  name: my-project\n";
//! let opts = CompileOptions { use_timestamp: false };
//! let bytes = compile(yaml, &opts).unwrap();
//! let result = decompile(&bytes).unwrap();
//! let name = result.get_section_string_by_name("project").unwrap();
//! assert!(name.contains("my-project"));
//! ```

pub mod canon;
pub mod compile;
pub mod error;
pub mod flags;
pub mod header;
pub mod priority;
pub mod section;
pub mod string_table;

// Re-exports for convenience
pub use canon::{
    CANONICAL_CHUNKS, CLASSIFICATION_MASK, CanonicalChunk, ChunkClassification, canonical_chunk,
    is_canonical,
};
pub use compile::{CompileOptions, DecompiledFafb, compile, decompile};
pub use error::{FafbError, FafbResult};
pub use flags::{
    FLAG_COMPRESSED, FLAG_EMBEDDINGS, FLAG_MODEL_HINTS, FLAG_RESOLVED, FLAG_SIGNED,
    FLAG_STRING_TABLE, FLAG_TOKENIZED, FLAG_WEIGHTED, Flags,
};
pub use header::{
    FafbHeader, HEADER_SIZE, MAGIC, MAGIC_U32, MAX_FILE_SIZE, MAX_SECTIONS, VERSION_MAJOR,
    VERSION_MINOR,
};
pub use priority::{
    PRIORITY_CRITICAL, PRIORITY_HIGH, PRIORITY_LOW, PRIORITY_MEDIUM, PRIORITY_OPTIONAL, Priority,
};
pub use section::{SECTION_ENTRY_SIZE, SectionEntry, SectionTable};
pub use string_table::StringTable;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
