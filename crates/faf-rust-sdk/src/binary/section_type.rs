//! FAFB Section Types
//!
//! Identifiers for different section types in .fafb files.
//! Readers MUST skip unknown section types gracefully.

/// Core section: metadata (faf_version, name, score)
pub const SECTION_META: u8 = 0x01;

/// Core section: tech stack (languages, frameworks)
pub const SECTION_TECH_STACK: u8 = 0x02;

/// Core section: key files with descriptions
pub const SECTION_KEY_FILES: u8 = 0x03;

/// Core section: system architecture/design
pub const SECTION_ARCHITECTURE: u8 = 0x04;

/// Core section: build/test/run commands
pub const SECTION_COMMANDS: u8 = 0x05;

/// Core section: additional context
pub const SECTION_CONTEXT: u8 = 0x06;

/// Core section: bi-sync metadata
pub const SECTION_BISYNC: u8 = 0x07;

/// Extended section: pre-computed embedding vectors
pub const SECTION_EMBEDDINGS: u8 = 0x10;

/// Extended section: token boundary markers
pub const SECTION_TOKEN_MAP: u8 = 0x11;

/// Extended section: model-specific optimization hints
pub const SECTION_MODEL_HINTS: u8 = 0x12;

/// Custom user-defined section
pub const SECTION_CUSTOM: u8 = 0xFF;

/// Section type with display name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Meta,
    TechStack,
    KeyFiles,
    Architecture,
    Commands,
    Context,
    BiSync,
    Embeddings,
    TokenMap,
    ModelHints,
    Custom,
    Unknown(u8),
}

impl SectionType {
    /// Get the byte identifier for this section type
    pub const fn id(&self) -> u8 {
        match self {
            Self::Meta => SECTION_META,
            Self::TechStack => SECTION_TECH_STACK,
            Self::KeyFiles => SECTION_KEY_FILES,
            Self::Architecture => SECTION_ARCHITECTURE,
            Self::Commands => SECTION_COMMANDS,
            Self::Context => SECTION_CONTEXT,
            Self::BiSync => SECTION_BISYNC,
            Self::Embeddings => SECTION_EMBEDDINGS,
            Self::TokenMap => SECTION_TOKEN_MAP,
            Self::ModelHints => SECTION_MODEL_HINTS,
            Self::Custom => SECTION_CUSTOM,
            Self::Unknown(id) => *id,
        }
    }

    /// Get human-readable name
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Meta => "META",
            Self::TechStack => "TECH_STACK",
            Self::KeyFiles => "KEY_FILES",
            Self::Architecture => "ARCHITECTURE",
            Self::Commands => "COMMANDS",
            Self::Context => "CONTEXT",
            Self::BiSync => "BISYNC",
            Self::Embeddings => "EMBEDDINGS",
            Self::TokenMap => "TOKEN_MAP",
            Self::ModelHints => "MODEL_HINTS",
            Self::Custom => "CUSTOM",
            Self::Unknown(_) => "UNKNOWN",
        }
    }

    /// Check if this is a core section (0x01-0x0F)
    pub const fn is_core(&self) -> bool {
        matches!(
            self,
            Self::Meta
                | Self::TechStack
                | Self::KeyFiles
                | Self::Architecture
                | Self::Commands
                | Self::Context
                | Self::BiSync
        )
    }

    /// Check if this is an extended section (0x10-0xFE)
    pub const fn is_extended(&self) -> bool {
        matches!(self, Self::Embeddings | Self::TokenMap | Self::ModelHints)
    }

    /// Get default priority for this section type
    pub const fn default_priority(&self) -> u8 {
        match self {
            Self::Meta => 255,         // Critical - never truncate
            Self::TechStack => 200,    // High
            Self::KeyFiles => 200,     // High
            Self::Commands => 180,     // High
            Self::Architecture => 128, // Medium
            Self::Context => 64,       // Low
            Self::BiSync => 32,        // Low
            Self::Embeddings => 16,    // Optional
            Self::TokenMap => 16,      // Optional
            Self::ModelHints => 16,    // Optional
            Self::Custom => 64,        // Low
            Self::Unknown(_) => 0,     // Optional
        }
    }
}

impl From<u8> for SectionType {
    fn from(id: u8) -> Self {
        match id {
            SECTION_META => Self::Meta,
            SECTION_TECH_STACK => Self::TechStack,
            SECTION_KEY_FILES => Self::KeyFiles,
            SECTION_ARCHITECTURE => Self::Architecture,
            SECTION_COMMANDS => Self::Commands,
            SECTION_CONTEXT => Self::Context,
            SECTION_BISYNC => Self::BiSync,
            SECTION_EMBEDDINGS => Self::Embeddings,
            SECTION_TOKEN_MAP => Self::TokenMap,
            SECTION_MODEL_HINTS => Self::ModelHints,
            SECTION_CUSTOM => Self::Custom,
            other => Self::Unknown(other),
        }
    }
}

impl From<SectionType> for u8 {
    fn from(section_type: SectionType) -> Self {
        section_type.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_type_ids() {
        assert_eq!(SectionType::Meta.id(), 0x01);
        assert_eq!(SectionType::TechStack.id(), 0x02);
        assert_eq!(SectionType::KeyFiles.id(), 0x03);
        assert_eq!(SectionType::Commands.id(), 0x05);
        assert_eq!(SectionType::Embeddings.id(), 0x10);
        assert_eq!(SectionType::Custom.id(), 0xFF);
    }

    #[test]
    fn test_section_type_roundtrip() {
        for id in 0x01..=0x07 {
            let section_type = SectionType::from(id);
            assert_eq!(section_type.id(), id);
        }
    }

    #[test]
    fn test_unknown_section_preserved() {
        let unknown = SectionType::from(0x99);
        assert!(matches!(unknown, SectionType::Unknown(0x99)));
        assert_eq!(unknown.id(), 0x99);
        assert_eq!(unknown.name(), "UNKNOWN");
    }

    #[test]
    fn test_is_core() {
        assert!(SectionType::Meta.is_core());
        assert!(SectionType::TechStack.is_core());
        assert!(!SectionType::Embeddings.is_core());
        assert!(!SectionType::Custom.is_core());
    }

    #[test]
    fn test_is_extended() {
        assert!(!SectionType::Meta.is_extended());
        assert!(SectionType::Embeddings.is_extended());
        assert!(SectionType::TokenMap.is_extended());
    }

    #[test]
    fn test_default_priorities() {
        assert_eq!(SectionType::Meta.default_priority(), 255);
        assert_eq!(SectionType::TechStack.default_priority(), 200);
        assert_eq!(SectionType::Context.default_priority(), 64);
        assert_eq!(SectionType::Unknown(0x99).default_priority(), 0);
    }
}
