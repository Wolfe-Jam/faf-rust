//! FAFb v2 Chunk Registry
//!
//! Classifies YAML keys into DNA, Context, or Pointer chunks.
//! Ported from the v3 FAF specification.

/// Chunk classification for v2 binary format
///
/// Stored in bits 0-1 of `SectionEntry.flags`:
/// - `0b00` = DNA (core project identity)
/// - `0b01` = Context (runtime/supplementary)
/// - `0b10` = Pointer (documentation references)
/// - `0b11` = Reserved
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkClassification {
    /// Core project identity (project, tech_stack, commands, etc.)
    Dna = 0b00,
    /// Runtime/supplementary context
    Context = 0b01,
    /// Documentation pointer (e.g., "docs")
    Pointer = 0b10,
    /// Reserved for future use
    Reserved = 0b11,
}

impl ChunkClassification {
    /// Get the 2-bit value for encoding into flags
    pub const fn bits(&self) -> u32 {
        *self as u32
    }

    /// Decode from the low 2 bits of a flags value
    pub fn from_bits(bits: u32) -> Self {
        match bits & 0b11 {
            0b00 => Self::Dna,
            0b01 => Self::Context,
            0b10 => Self::Pointer,
            _ => Self::Reserved,
        }
    }

    /// Human-readable name
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dna => "DNA",
            Self::Context => "Context",
            Self::Pointer => "Pointer",
            Self::Reserved => "Reserved",
        }
    }
}

/// Classification mask for the low 2 bits of section flags
pub const CLASSIFICATION_MASK: u32 = 0b11;

/// Known DNA keys — core project identity fields
pub const DNA_KEYS: &[&str] = &[
    "faf_version",
    "project",
    "instant_context",
    "tech_stack",
    "key_files",
    "commands",
    "architecture",
    "context",
    "bi_sync",
    "meta",
];

/// The pointer key — documentation references
pub const POINTER_KEY: &str = "docs";

/// Classify a YAML key into its chunk type
pub fn classify_key(key: &str) -> ChunkClassification {
    if key == POINTER_KEY {
        ChunkClassification::Pointer
    } else if DNA_KEYS.contains(&key) {
        ChunkClassification::Dna
    } else {
        ChunkClassification::Context
    }
}

/// Get the default priority for a classified chunk
pub fn default_priority_for_classification(classification: ChunkClassification) -> u8 {
    match classification {
        ChunkClassification::Dna => 200,     // High
        ChunkClassification::Context => 64,  // Low
        ChunkClassification::Pointer => 128, // Medium
        ChunkClassification::Reserved => 0,  // Optional
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_dna_keys_classified() {
        for key in DNA_KEYS {
            assert_eq!(
                classify_key(key),
                ChunkClassification::Dna,
                "Expected '{}' to be DNA",
                key
            );
        }
    }

    #[test]
    fn test_pointer_key() {
        assert_eq!(classify_key("docs"), ChunkClassification::Pointer);
    }

    #[test]
    fn test_unknown_keys_are_context() {
        assert_eq!(classify_key("custom_field"), ChunkClassification::Context);
        assert_eq!(classify_key("my_data"), ChunkClassification::Context);
        assert_eq!(classify_key("anything"), ChunkClassification::Context);
    }

    #[test]
    fn test_case_sensitivity() {
        // Keys are case-sensitive (YAML convention)
        assert_eq!(classify_key("Project"), ChunkClassification::Context);
        assert_eq!(classify_key("DOCS"), ChunkClassification::Context);
        assert_eq!(classify_key("project"), ChunkClassification::Dna);
    }

    #[test]
    fn test_bits_roundtrip() {
        for class in &[
            ChunkClassification::Dna,
            ChunkClassification::Context,
            ChunkClassification::Pointer,
            ChunkClassification::Reserved,
        ] {
            assert_eq!(ChunkClassification::from_bits(class.bits()), *class);
        }
    }

    #[test]
    fn test_bit_values() {
        assert_eq!(ChunkClassification::Dna.bits(), 0b00);
        assert_eq!(ChunkClassification::Context.bits(), 0b01);
        assert_eq!(ChunkClassification::Pointer.bits(), 0b10);
        assert_eq!(ChunkClassification::Reserved.bits(), 0b11);
    }

    #[test]
    fn test_from_bits_masked() {
        // Higher bits should be ignored
        assert_eq!(
            ChunkClassification::from_bits(0xFF00_0000),
            ChunkClassification::Dna
        );
        assert_eq!(
            ChunkClassification::from_bits(0xFF00_0001),
            ChunkClassification::Context
        );
    }

    #[test]
    fn test_classification_names() {
        assert_eq!(ChunkClassification::Dna.name(), "DNA");
        assert_eq!(ChunkClassification::Context.name(), "Context");
        assert_eq!(ChunkClassification::Pointer.name(), "Pointer");
        assert_eq!(ChunkClassification::Reserved.name(), "Reserved");
    }

    #[test]
    fn test_default_priorities() {
        assert_eq!(
            default_priority_for_classification(ChunkClassification::Dna),
            200
        );
        assert_eq!(
            default_priority_for_classification(ChunkClassification::Context),
            64
        );
        assert_eq!(
            default_priority_for_classification(ChunkClassification::Pointer),
            128
        );
    }
}
