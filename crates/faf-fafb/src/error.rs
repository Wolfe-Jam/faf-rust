//! FAFB Error Types
//!
//! Error handling for binary format operations.

use thiserror::Error;

/// Errors that can occur when working with .fafb files
#[derive(Error, Debug)]
pub enum FafbError {
    /// Invalid magic number - not a FAFB file
    #[error("Invalid magic number: expected FAFB (0x46414642), got {0:#010x}")]
    InvalidMagic(u32),

    /// Incompatible major version
    #[error(
        "Incompatible version: expected major version {expected}, got {actual}. FAFb v1 is pre-release history — re-compile from the .faf source"
    )]
    IncompatibleVersion { expected: u8, actual: u8 },

    /// Checksum mismatch - file may be corrupted
    #[error("Checksum mismatch: expected {expected:#010x}, got {actual:#010x}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    /// File too small to contain valid header
    #[error("File too small: expected at least {expected} bytes, got {actual}")]
    FileTooSmall { expected: usize, actual: usize },

    /// Section table offset points outside file bounds
    #[error("Invalid section table offset: {offset} exceeds file size {file_size}")]
    InvalidSectionTableOffset { offset: u32, file_size: u32 },

    /// Section count exceeds maximum allowed
    #[error("Section count {count} exceeds maximum {max}")]
    TooManySections { count: u16, max: u16 },

    /// IO error during read/write
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Total size in header doesn't match actual data
    #[error("Size mismatch: header says {header_size} bytes, actual {actual_size}")]
    SizeMismatch {
        header_size: u32,
        actual_size: usize,
    },

    /// String table index out of bounds
    #[error("String table index {index} out of bounds (table has {count} entries)")]
    StringTableIndexOutOfBounds { index: u8, count: u16 },

    /// String table entry exceeds maximum length
    #[error("String table entry too long: {length} bytes exceeds maximum {max}")]
    StringTableEntryTooLong { length: usize, max: usize },

    /// String table is full (256 entries max)
    #[error("String table full: maximum {max} entries")]
    StringTableFull { max: usize },

    /// Missing string table section in v2 file
    #[error("Missing string table section (required for FAFb v2)")]
    MissingStringTable,

    /// Invalid UTF-8 in string table
    #[error("Invalid UTF-8 in string table: {0}")]
    InvalidUtf8(String),
}

/// Result type for FAFB operations
pub type FafbResult<T> = Result<T, FafbError>;
