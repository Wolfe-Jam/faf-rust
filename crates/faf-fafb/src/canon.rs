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
/// names, in serialization order. **This IS the format: there is no chunk 14.**
///
/// The set mirrors `faf-cli`'s `FafData` (the single source of truth for the
/// `.faf` structure — `src/core/types.ts`): 11 DNA chunks + 2 Context. The
/// metastamp is NOT a chunk — it's the FAFb header (`created_timestamp`),
/// which is why `.faf`'s `generated:` key lives there, not here. Non-canonical
/// top-level keys (anything tools add) fold losslessly into `context`.
pub const CANONICAL_CHUNKS: &[CanonicalChunk] = &[
    // ── DNA — core identity (11) ──
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
        name: "app_type",
        classification: ChunkClassification::Dna,
        priority: 200,
    },
    CanonicalChunk {
        name: "about",
        classification: ChunkClassification::Dna,
        priority: 150,
    },
    CanonicalChunk {
        name: "stack",
        classification: ChunkClassification::Dna,
        priority: 200,
    },
    CanonicalChunk {
        name: "human_context",
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
    // ── Context (2) — derived output + the fold target ──
    CanonicalChunk {
        name: "scores",
        classification: ChunkClassification::Context,
        priority: 64,
    },
    CanonicalChunk {
        name: "context",
        classification: ChunkClassification::Context,
        priority: 64,
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
    fn table_is_closed_at_13() {
        // Mirrors faf-cli FafData: 11 DNA + 2 Context. The one-way door.
        assert_eq!(CANONICAL_CHUNKS.len(), 13);
        let dna = CANONICAL_CHUNKS
            .iter()
            .filter(|c| c.classification == ChunkClassification::Dna)
            .count();
        let ctx = CANONICAL_CHUNKS
            .iter()
            .filter(|c| c.classification == ChunkClassification::Context)
            .count();
        assert_eq!(dna, 11);
        assert_eq!(ctx, 2);
    }

    #[test]
    fn mirrors_faf_cli_fafdata() {
        // The canonical set IS faf-cli's FafData top-level keys (the truth).
        for key in [
            "faf_version",
            "project",
            "app_type",
            "about",
            "stack",
            "human_context",
            "monorepo",
            "scores",
            "tech_stack",
            "key_files",
            "commands",
            "architecture",
            "context",
        ] {
            assert!(is_canonical(key), "FafData key '{}' must be canonical", key);
        }
        // Keys that diverged from the old faf-rust-sdk model are NOT canonical.
        for key in [
            "instant_context",
            "ai_score",
            "ai_confidence",
            "ai_tldr",
            "context_quality",
            "preferences",
            "state",
            "tags",
            "meta",
            "bi_sync",
            "docs",
            "generated",
        ] {
            assert!(!is_canonical(key), "'{}' must fold, not be a chunk", key);
        }
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
    fn context_is_the_fold_target() {
        // `context` is canonical (Context class) and is where non-canonical
        // keys fold. No Pointer-class chunk exists in the FafData truth.
        assert_eq!(
            canonical_chunk("context").unwrap().classification,
            ChunkClassification::Context
        );
        assert!(
            !CANONICAL_CHUNKS
                .iter()
                .any(|c| c.classification == ChunkClassification::Pointer)
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
