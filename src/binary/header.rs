//! FAFB Header Implementation
//!
//! The 32-byte header that identifies and describes a .fafb file.
//!
//! Layout:
//! ```text
//! Offset  Size  Field
//! ------  ----  -----
//! 0       4     magic (b"FAFB")
//! 4       1     version_major
//! 5       1     version_minor
//! 6       2     flags
//! 8       4     source_checksum (CRC32)
//! 12      8     created_timestamp (Unix)
//! 20      2     section_count
//! 22      4     section_table_offset
//! 26      2     reserved
//! 28      4     total_size
//! ------  ----
//! Total: 32 bytes
//! ```

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

use super::error::{FafbError, FafbResult};
use super::flags::Flags;

/// Magic number identifying FAFB files: "FAFB" in ASCII
pub const MAGIC: [u8; 4] = *b"FAFB";

/// Magic number as u32 (little-endian)
pub const MAGIC_U32: u32 = 0x4246_4146; // "FAFB" little-endian

/// Current format major version (breaking changes)
pub const VERSION_MAJOR: u8 = 1;

/// Current format minor version (additive changes)
pub const VERSION_MINOR: u8 = 0;

/// Header size in bytes
pub const HEADER_SIZE: usize = 32;

/// Maximum allowed section count (DoS protection)
pub const MAX_SECTIONS: u16 = 256;

/// Maximum allowed file size (DoS protection): 10MB
pub const MAX_FILE_SIZE: u32 = 10 * 1024 * 1024;

/// The 32-byte FAFB file header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FafbHeader {
    // Identification (8 bytes)
    /// Format version - major (breaking changes)
    pub version_major: u8,
    /// Format version - minor (additive changes)
    pub version_minor: u8,
    /// Feature flags
    pub flags: Flags,

    // Integrity (12 bytes)
    /// CRC32 checksum of original .faf YAML source
    pub source_checksum: u32,
    /// Unix timestamp when .fafb was created
    pub created_timestamp: u64,

    // Index (8 bytes)
    /// Number of sections in the file
    pub section_count: u16,
    /// Byte offset to section table (from start of file)
    pub section_table_offset: u32,
    /// Reserved for future use
    pub reserved: u16,

    // Size (4 bytes)
    /// Total file size in bytes
    pub total_size: u32,
}

impl FafbHeader {
    /// Create a new header with default version and empty flags
    pub fn new() -> Self {
        Self {
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags: Flags::new(),
            source_checksum: 0,
            created_timestamp: 0,
            section_count: 0,
            section_table_offset: HEADER_SIZE as u32,
            reserved: 0,
            total_size: HEADER_SIZE as u32,
        }
    }

    /// Create header with current timestamp
    pub fn with_timestamp() -> Self {
        let mut header = Self::new();
        header.created_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        header
    }

    /// Compute CRC32 checksum of source YAML
    pub fn compute_checksum(yaml_source: &[u8]) -> u32 {
        // WHY: CRC32 of source YAML (not binary) - enables integrity verification
        // that the .fafb was created from a specific .faf source file
        crc32fast::hash(yaml_source)
    }

    /// Set the source checksum from YAML content
    pub fn set_source_checksum(&mut self, yaml_source: &[u8]) {
        self.source_checksum = Self::compute_checksum(yaml_source);
    }

    /// Write header to a byte buffer
    pub fn write<W: Write>(&self, writer: &mut W) -> FafbResult<()> {
        // WHY: Field order matches FAFB spec exactly - parsers depend on this layout
        // WHY: Little-endian used throughout for cross-platform compatibility (x86/ARM native)

        // Magic (4 bytes) - identifies file type before any other parsing
        writer.write_all(&MAGIC)?;

        // Version (2 bytes) - enables format evolution without breaking old readers
        writer.write_u8(self.version_major)?;
        writer.write_u8(self.version_minor)?;

        // Flags (2 bytes) - feature detection without version bump
        writer.write_u16::<LittleEndian>(self.flags.raw())?;

        // Integrity (12 bytes) - checksum enables source→binary verification
        writer.write_u32::<LittleEndian>(self.source_checksum)?;
        writer.write_u64::<LittleEndian>(self.created_timestamp)?;

        // Index (8 bytes) - section table location for random access
        writer.write_u16::<LittleEndian>(self.section_count)?;
        writer.write_u32::<LittleEndian>(self.section_table_offset)?;
        // WHY: Reserved bytes allow future header extensions without version bump
        writer.write_u16::<LittleEndian>(self.reserved)?;

        // Size (4 bytes) - enables pre-allocation and bounds validation
        writer.write_u32::<LittleEndian>(self.total_size)?;

        Ok(())
    }

    /// Write header to a new `Vec<u8>`
    pub fn to_bytes(&self) -> FafbResult<Vec<u8>> {
        let mut buf = Vec::with_capacity(HEADER_SIZE);
        self.write(&mut buf)?;
        Ok(buf)
    }

    /// Read header from a byte buffer
    pub fn read<R: Read>(reader: &mut R) -> FafbResult<Self> {
        // WHY: Fail fast on magic check - 4 bytes tells us if this is even FAFB
        // Checking magic first avoids parsing garbage data as valid fields
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        let magic_u32 = u32::from_le_bytes(magic);
        if magic_u32 != MAGIC_U32 {
            return Err(FafbError::InvalidMagic(magic_u32));
        }

        // WHY: Version check before full parse - incompatible versions may have
        // different field layouts, so we must reject early to avoid misreading
        let version_major = reader.read_u8()?;
        let version_minor = reader.read_u8()?;

        if version_major != VERSION_MAJOR {
            return Err(FafbError::IncompatibleVersion {
                expected: VERSION_MAJOR,
                actual: version_major,
            });
        }

        // WHY: Unknown flags ignored per spec - enables forward compatibility
        // New readers can parse old files; old readers can parse new files with new flags
        let flags = Flags::from_raw(reader.read_u16::<LittleEndian>()?);

        // Integrity fields - read unconditionally, validate later with source
        let source_checksum = reader.read_u32::<LittleEndian>()?;
        let created_timestamp = reader.read_u64::<LittleEndian>()?;

        // WHY: DoS protection - reject before allocating large section tables
        let section_count = reader.read_u16::<LittleEndian>()?;
        if section_count > MAX_SECTIONS {
            return Err(FafbError::TooManySections {
                count: section_count,
                max: MAX_SECTIONS,
            });
        }

        let section_table_offset = reader.read_u32::<LittleEndian>()?;
        let reserved = reader.read_u16::<LittleEndian>()?;

        // WHY: DoS protection - reject unreasonably large files before processing
        let total_size = reader.read_u32::<LittleEndian>()?;
        if total_size > MAX_FILE_SIZE {
            return Err(FafbError::SizeMismatch {
                header_size: total_size,
                actual_size: MAX_FILE_SIZE as usize,
            });
        }

        Ok(Self {
            version_major,
            version_minor,
            flags,
            source_checksum,
            created_timestamp,
            section_count,
            section_table_offset,
            reserved,
            total_size,
        })
    }

    /// Read header from a byte slice
    pub fn from_bytes(data: &[u8]) -> FafbResult<Self> {
        if data.len() < HEADER_SIZE {
            return Err(FafbError::FileTooSmall {
                expected: HEADER_SIZE,
                actual: data.len(),
            });
        }

        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor)
    }

    /// Validate header against actual file data
    pub fn validate(&self, file_data: &[u8]) -> FafbResult<()> {
        // WHY: Validation order is cheapest checks first - size comparison is O(1)
        // and catches truncated files before more expensive parsing

        // Size check detects truncated downloads or corrupted files
        if self.total_size as usize != file_data.len() {
            return Err(FafbError::SizeMismatch {
                header_size: self.total_size,
                actual_size: file_data.len(),
            });
        }

        // WHY: Offset check prevents out-of-bounds reads when seeking to section table
        if self.section_table_offset > self.total_size {
            return Err(FafbError::InvalidSectionTableOffset {
                offset: self.section_table_offset,
                file_size: self.total_size,
            });
        }

        Ok(())
    }

    /// Check if this header is compatible with the current version
    pub fn is_compatible(&self) -> bool {
        self.version_major == VERSION_MAJOR
    }

    /// Get version as string (e.g., "1.0")
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.version_major, self.version_minor)
    }
}

impl Default for FafbHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        let header = FafbHeader::new();
        let bytes = header.to_bytes().unwrap();
        assert_eq!(bytes.len(), HEADER_SIZE);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_magic_bytes() {
        let header = FafbHeader::new();
        let bytes = header.to_bytes().unwrap();
        assert_eq!(&bytes[0..4], b"FAFB");
    }

    #[test]
    fn test_roundtrip() {
        let mut original = FafbHeader::with_timestamp();
        original.source_checksum = 0xDEADBEEF;
        original.section_count = 5;
        original.section_table_offset = 1024;
        original.total_size = 2048;
        original.flags.set_compressed(true);
        original.flags.set_embeddings(true);

        let bytes = original.to_bytes().unwrap();
        let recovered = FafbHeader::from_bytes(&bytes).unwrap();

        assert_eq!(original.version_major, recovered.version_major);
        assert_eq!(original.version_minor, recovered.version_minor);
        assert_eq!(original.flags, recovered.flags);
        assert_eq!(original.source_checksum, recovered.source_checksum);
        assert_eq!(original.created_timestamp, recovered.created_timestamp);
        assert_eq!(original.section_count, recovered.section_count);
        assert_eq!(
            original.section_table_offset,
            recovered.section_table_offset
        );
        assert_eq!(original.total_size, recovered.total_size);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = FafbHeader::new().to_bytes().unwrap();
        bytes[0] = 0x00; // Corrupt magic

        let result = FafbHeader::from_bytes(&bytes);
        assert!(matches!(result, Err(FafbError::InvalidMagic(_))));
    }

    #[test]
    fn test_incompatible_version() {
        let mut bytes = FafbHeader::new().to_bytes().unwrap();
        bytes[4] = 99; // Set major version to 99

        let result = FafbHeader::from_bytes(&bytes);
        assert!(matches!(
            result,
            Err(FafbError::IncompatibleVersion {
                expected: 1,
                actual: 99
            })
        ));
    }

    #[test]
    fn test_file_too_small() {
        let bytes = vec![0u8; 16]; // Only 16 bytes, need 32

        let result = FafbHeader::from_bytes(&bytes);
        assert!(matches!(
            result,
            Err(FafbError::FileTooSmall {
                expected: 32,
                actual: 16
            })
        ));
    }

    #[test]
    fn test_too_many_sections() {
        let mut header = FafbHeader::new();
        header.section_count = 300; // Over MAX_SECTIONS (256)

        let bytes = header.to_bytes().unwrap();
        let result = FafbHeader::from_bytes(&bytes);

        assert!(matches!(
            result,
            Err(FafbError::TooManySections {
                count: 300,
                max: 256
            })
        ));
    }

    #[test]
    fn test_checksum_computation() {
        let yaml = b"faf_version: 2.5.0\nproject:\n  name: test";
        let checksum = FafbHeader::compute_checksum(yaml);

        // CRC32 is deterministic
        assert_eq!(checksum, FafbHeader::compute_checksum(yaml));

        // Different content = different checksum
        let yaml2 = b"faf_version: 2.5.0\nproject:\n  name: different";
        assert_ne!(checksum, FafbHeader::compute_checksum(yaml2));
    }

    #[test]
    fn test_validate_size_mismatch() {
        let mut header = FafbHeader::new();
        header.total_size = 100;

        let data = vec![0u8; 50]; // Actual size doesn't match header

        let result = header.validate(&data);
        assert!(matches!(
            result,
            Err(FafbError::SizeMismatch {
                header_size: 100,
                actual_size: 50
            })
        ));
    }

    #[test]
    fn test_validate_invalid_section_offset() {
        let mut header = FafbHeader::new();
        header.total_size = 100;
        header.section_table_offset = 200; // Beyond file size

        let data = vec![0u8; 100];

        let result = header.validate(&data);
        assert!(matches!(
            result,
            Err(FafbError::InvalidSectionTableOffset {
                offset: 200,
                file_size: 100
            })
        ));
    }

    #[test]
    fn test_version_string() {
        let header = FafbHeader::new();
        assert_eq!(header.version_string(), "1.0");
    }

    #[test]
    fn test_flags_preserved() {
        let mut header = FafbHeader::new();
        header.flags.set_compressed(true);
        header.flags.set_signed(true);

        let bytes = header.to_bytes().unwrap();
        let recovered = FafbHeader::from_bytes(&bytes).unwrap();

        assert!(recovered.flags.is_compressed());
        assert!(recovered.flags.is_signed());
        assert!(!recovered.flags.has_embeddings());
    }

    #[test]
    fn test_unknown_flags_ignored() {
        let mut header = FafbHeader::new();
        // Set some "future" flags in reserved bits
        header.flags = Flags::from_raw(0xFF00);

        let bytes = header.to_bytes().unwrap();
        // Should read successfully despite unknown flags
        let recovered = FafbHeader::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.flags.raw(), 0xFF00);
    }
}
