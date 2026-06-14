//! The canonical chunk table — FAFb v2 is CLOSED CANONICAL.
//!
//! The writer emits exactly the chunks in this table, in this order.
//! Non-canonical top-level YAML keys are folded into the `context` chunk —
//! nothing is lost, and nothing grows a new section name. The reader keeps
//! the IFF rule (skip unknown names gracefully) so future minor versions can
//! add a chunk without breaking deployed readers: writer closed, reader
//! graceful.
//!
//! Closed canonical is what makes the brick deterministic: identical content
//! produces identical bytes regardless of input key order, so a `.fafb` is
//! content-addressable — same project context, same hash, everywhere.

/// Chunk classification, stored in bits 0–1 of `SectionEntry.flags`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkClassification {
    /// Core project identity (project, stack, human_context, …).
    Dna = 0b00,
    /// Runtime/supplementary context.
    Context = 0b01,
    /// Documentation references (docs).
    Pointer = 0b10,
    /// Reserved for future use.
    Reserved = 0b11,
}

impl ChunkClassification {
    /// The 2-bit value for encoding into flags.
    pub const fn bits(&self) -> u32 {
        *self as u32
    }

    /// Decode from the low 2 bits of a flags value.
    pub fn from_bits(bits: u32) -> Self {
        match bits & 0b11 {
            0b00 => Self::Dna,
            0b01 => Self::Context,
            0b10 => Self::Pointer,
            _ => Self::Reserved,
        }
    }

    /// Human-readable name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dna => "DNA",
            Self::Context => "Context",
            Self::Pointer => "Pointer",
            Self::Reserved => "Reserved",
        }
    }
}

/// Classification mask for the low 2 bits of section flags.
pub const CLASSIFICATION_MASK: u32 = 0b11;

/// One canonical chunk: name, classification, default priority.
#[derive(Debug, Clone, Copy)]
pub struct CanonicalChunk {
    pub name: &'static str,
    pub classification: ChunkClassification,
    pub priority: u8,
}

/// The canonical chunk table — the complete, closed set of FAFb v2 section
/// names, in serialization order. This IS the format: there is no chunk 24.
pub const CANONICAL_CHUNKS: &[CanonicalChunk] = &[
    CanonicalChunk {
        name: "faf_version",
        classification: ChunkClassification::Dna,
        priority: 255,
    },
    CanonicalChunk {
        name: "project",
        classification: ChunkClassification::Dna,
        priority: 255,
    },
    CanonicalChunk {
        name: "instant_context",
        classification: ChunkClassification::Dna,
        priority: 200,
    },
    CanonicalChunk {
        name: "human_context",
        classification: ChunkClassification::Dna,
        priority: 200,
    },
    CanonicalChunk {
        name: "stack",
        classification: ChunkClassification::Dna,
        priority: 200,
    },
    CanonicalChunk {
        name: "tech_stack",
        classification: ChunkClassification::Dna,
        priority: 200,
    },
    CanonicalChunk {
        name: "key_files",
        classification: ChunkClassification::Dna,
        priority: 200,
    },
    CanonicalChunk {
        name: "commands",
        classification: ChunkClassification::Dna,
        priority: 180,
    },
    CanonicalChunk {
        name: "monorepo",
        classification: ChunkClassification::Dna,
        priority: 150,
    },
    CanonicalChunk {
        name: "architecture",
        classification: ChunkClassification::Dna,
        priority: 128,
    },
    CanonicalChunk {
        name: "context",
        classification: ChunkClassification::Dna,
        priority: 64,
    },
    CanonicalChunk {
        name: "bi_sync",
        classification: ChunkClassification::Dna,
        priority: 32,
    },
    CanonicalChunk {
        name: "meta",
        classification: ChunkClassification::Dna,
        priority: 64,
    },
    CanonicalChunk {
        name: "ai_score",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "ai_confidence",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "ai_tldr",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "context_quality",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "preferences",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "state",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "tags",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "scores",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "generated",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "docs",
        classification: ChunkClassification::Pointer,
        priority: 128,
    },
];

/// Look up a canonical chunk by name.
pub fn canonical_chunk(name: &str) -> Option<&'static CanonicalChunk> {
    CANONICAL_CHUNKS.iter().find(|c| c.name == name)
}

/// Is this top-level key part of the canonical chunk set?
pub fn is_canonical(name: &str) -> bool {
    canonical_chunk(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_closed_at_23() {
        assert_eq!(CANONICAL_CHUNKS.len(), 23);
    }

    #[test]
    fn names_are_unique() {
        for (i, a) in CANONICAL_CHUNKS.iter().enumerate() {
            for b in &CANONICAL_CHUNKS[i + 1..] {
                assert_ne!(a.name, b.name);
            }
        }
    }

    #[test]
    fn identity_chunks_are_critical() {
        assert_eq!(canonical_chunk("faf_version").unwrap().priority, 255);
        assert_eq!(canonical_chunk("project").unwrap().priority, 255);
    }

    #[test]
    fn docs_is_the_pointer() {
        assert_eq!(
            canonical_chunk("docs").unwrap().classification,
            ChunkClassification::Pointer
        );
    }

    #[test]
    fn unknown_keys_are_not_canonical() {
        assert!(!is_canonical("custom_field"));
        assert!(!is_canonical("my_exotic_field"));
        // Case-sensitive (YAML convention)
        assert!(!is_canonical("Project"));
        assert!(is_canonical("project"));
    }

    #[test]
    fn classification_bits_roundtrip() {
        for class in &[
            ChunkClassification::Dna,
            ChunkClassification::Context,
            ChunkClassification::Pointer,
            ChunkClassification::Reserved,
        ] {
            assert_eq!(ChunkClassification::from_bits(class.bits()), *class);
        }
        // Higher bits ignored
        assert_eq!(
            ChunkClassification::from_bits(0xFF00_0001),
            ChunkClassification::Context
        );
    }
}
