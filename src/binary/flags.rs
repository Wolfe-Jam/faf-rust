//! FAFB Feature Flags
//!
//! Bit flags for optional features in .fafb files.
//! Readers MUST ignore unknown flags and continue processing.

/// Content is zstd compressed
pub const FLAG_COMPRESSED: u16 = 0b0000_0000_0000_0001;

/// Contains pre-computed embeddings
pub const FLAG_EMBEDDINGS: u16 = 0b0000_0000_0000_0010;

/// Contains token boundaries
pub const FLAG_TOKENIZED: u16 = 0b0000_0000_0000_0100;

/// Contains attention weights
pub const FLAG_WEIGHTED: u16 = 0b0000_0000_0000_1000;

/// Contains model-specific hints
pub const FLAG_MODEL_HINTS: u16 = 0b0000_0000_0001_0000;

/// Contains cryptographic signature
pub const FLAG_SIGNED: u16 = 0b0000_0000_0010_0000;

// Reserved: bits 6-15 for future use

/// Helper struct for working with flags
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags(pub u16);

impl Flags {
    /// Create new empty flags
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create flags from raw u16
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// Get raw u16 value
    pub const fn raw(&self) -> u16 {
        self.0
    }

    /// Check if compressed flag is set
    pub const fn is_compressed(&self) -> bool {
        self.0 & FLAG_COMPRESSED != 0
    }

    /// Check if embeddings flag is set
    pub const fn has_embeddings(&self) -> bool {
        self.0 & FLAG_EMBEDDINGS != 0
    }

    /// Check if tokenized flag is set
    pub const fn is_tokenized(&self) -> bool {
        self.0 & FLAG_TOKENIZED != 0
    }

    /// Check if weighted flag is set
    pub const fn has_weights(&self) -> bool {
        self.0 & FLAG_WEIGHTED != 0
    }

    /// Check if model hints flag is set
    pub const fn has_model_hints(&self) -> bool {
        self.0 & FLAG_MODEL_HINTS != 0
    }

    /// Check if signed flag is set
    pub const fn is_signed(&self) -> bool {
        self.0 & FLAG_SIGNED != 0
    }

    /// Set compressed flag
    pub fn set_compressed(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_COMPRESSED;
        } else {
            self.0 &= !FLAG_COMPRESSED;
        }
    }

    /// Set embeddings flag
    pub fn set_embeddings(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_EMBEDDINGS;
        } else {
            self.0 &= !FLAG_EMBEDDINGS;
        }
    }

    /// Set tokenized flag
    pub fn set_tokenized(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_TOKENIZED;
        } else {
            self.0 &= !FLAG_TOKENIZED;
        }
    }

    /// Set weighted flag
    pub fn set_weighted(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_WEIGHTED;
        } else {
            self.0 &= !FLAG_WEIGHTED;
        }
    }

    /// Set model hints flag
    pub fn set_model_hints(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_MODEL_HINTS;
        } else {
            self.0 &= !FLAG_MODEL_HINTS;
        }
    }

    /// Set signed flag
    pub fn set_signed(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_SIGNED;
        } else {
            self.0 &= !FLAG_SIGNED;
        }
    }
}

impl From<u16> for Flags {
    fn from(raw: u16) -> Self {
        Self(raw)
    }
}

impl From<Flags> for u16 {
    fn from(flags: Flags) -> Self {
        flags.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_bits() {
        assert_eq!(FLAG_COMPRESSED, 1);
        assert_eq!(FLAG_EMBEDDINGS, 2);
        assert_eq!(FLAG_TOKENIZED, 4);
        assert_eq!(FLAG_WEIGHTED, 8);
        assert_eq!(FLAG_MODEL_HINTS, 16);
        assert_eq!(FLAG_SIGNED, 32);
    }

    #[test]
    fn test_flags_default() {
        let flags = Flags::new();
        assert_eq!(flags.raw(), 0);
        assert!(!flags.is_compressed());
        assert!(!flags.has_embeddings());
    }

    #[test]
    fn test_flags_set_get() {
        let mut flags = Flags::new();
        flags.set_compressed(true);
        flags.set_embeddings(true);

        assert!(flags.is_compressed());
        assert!(flags.has_embeddings());
        assert!(!flags.is_tokenized());

        assert_eq!(flags.raw(), FLAG_COMPRESSED | FLAG_EMBEDDINGS);
    }

    #[test]
    fn test_flags_unset() {
        let mut flags = Flags::from_raw(FLAG_COMPRESSED | FLAG_EMBEDDINGS);
        flags.set_compressed(false);

        assert!(!flags.is_compressed());
        assert!(flags.has_embeddings());
    }
}
