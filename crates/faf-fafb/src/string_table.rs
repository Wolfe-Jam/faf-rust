//! FAFb v2 String Table
//!
//! A dedicated string table section (classic ELF/IFF pattern).
//! Section entries reference names by `u8` index into the table.
//! Supports up to 256 unique names (matches `MAX_SECTIONS`).
//!
//! ## Wire format
//!
//! ```text
//! [u16 count] [u16 len₀][utf8₀] [u16 len₁][utf8₁] ...
//! ```
//!
//! Max 256 entries, max 255 bytes per name.

use std::collections::HashMap;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::error::{FafbError, FafbResult};

/// Maximum number of entries in the string table
pub const MAX_STRING_TABLE_ENTRIES: usize = 256;

/// Maximum byte length of a single string table entry
pub const MAX_STRING_LENGTH: usize = 255;

/// A string table mapping u8 indices to UTF-8 names
#[derive(Debug, Clone, Default)]
pub struct StringTable {
    /// Ordered list of names (index = position)
    entries: Vec<String>,
    /// Reverse lookup: name → index (for dedup)
    index_map: HashMap<String, u8>,
}

impl StringTable {
    /// Create an empty string table
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index_map: HashMap::new(),
        }
    }

    /// Add a name to the table. Returns the index.
    /// If the name already exists, returns the existing index (dedup).
    pub fn add(&mut self, name: &str) -> FafbResult<u8> {
        // Dedup: return existing index if name is already in the table
        if let Some(&idx) = self.index_map.get(name) {
            return Ok(idx);
        }

        // Validate
        if self.entries.len() >= MAX_STRING_TABLE_ENTRIES {
            return Err(FafbError::StringTableFull {
                max: MAX_STRING_TABLE_ENTRIES,
            });
        }
        if name.len() > MAX_STRING_LENGTH {
            return Err(FafbError::StringTableEntryTooLong {
                length: name.len(),
                max: MAX_STRING_LENGTH,
            });
        }

        let idx = self.entries.len() as u8;
        self.entries.push(name.to_string());
        self.index_map.insert(name.to_string(), idx);
        Ok(idx)
    }

    /// Get a name by index
    pub fn get(&self, index: u8) -> FafbResult<&str> {
        self.entries.get(index as usize).map(|s| s.as_str()).ok_or(
            FafbError::StringTableIndexOutOfBounds {
                index,
                count: self.entries.len() as u16,
            },
        )
    }

    /// Get the index for a name (if it exists)
    pub fn index_of(&self, name: &str) -> Option<u8> {
        self.index_map.get(name).copied()
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries as a slice
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// Serialize to bytes
    ///
    /// Format: `[u16 count] [u16 len₀][utf8₀] [u16 len₁][utf8₁] ...`
    pub fn to_bytes(&self) -> FafbResult<Vec<u8>> {
        let mut buf = Vec::new();
        buf.write_u16::<LittleEndian>(self.entries.len() as u16)?;
        for entry in &self.entries {
            buf.write_u16::<LittleEndian>(entry.len() as u16)?;
            buf.extend_from_slice(entry.as_bytes());
        }
        Ok(buf)
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> FafbResult<Self> {
        if data.len() < 2 {
            return Err(FafbError::FileTooSmall {
                expected: 2,
                actual: data.len(),
            });
        }

        let mut cursor = std::io::Cursor::new(data);
        let count = cursor.read_u16::<LittleEndian>()? as usize;

        if count > MAX_STRING_TABLE_ENTRIES {
            return Err(FafbError::StringTableFull {
                max: MAX_STRING_TABLE_ENTRIES,
            });
        }

        let mut table = Self::new();
        for _ in 0..count {
            let len = cursor.read_u16::<LittleEndian>()? as usize;
            if len > MAX_STRING_LENGTH {
                return Err(FafbError::StringTableEntryTooLong {
                    length: len,
                    max: MAX_STRING_LENGTH,
                });
            }

            let pos = cursor.position() as usize;
            if pos + len > data.len() {
                return Err(FafbError::FileTooSmall {
                    expected: pos + len,
                    actual: data.len(),
                });
            }

            let name = std::str::from_utf8(&data[pos..pos + len])
                .map_err(|e| FafbError::InvalidUtf8(e.to_string()))?;
            cursor.set_position((pos + len) as u64);

            // Use internal push to avoid re-validation
            let idx = table.entries.len() as u8;
            table.entries.push(name.to_string());
            table.index_map.insert(name.to_string(), idx);
        }

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut table = StringTable::new();
        let idx = table.add("project").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(table.get(0).unwrap(), "project");
    }

    #[test]
    fn test_dedup() {
        let mut table = StringTable::new();
        let idx1 = table.add("project").unwrap();
        let idx2 = table.add("project").unwrap();
        assert_eq!(idx1, idx2);
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_multiple_entries() {
        let mut table = StringTable::new();
        assert_eq!(table.add("project").unwrap(), 0);
        assert_eq!(table.add("tech_stack").unwrap(), 1);
        assert_eq!(table.add("commands").unwrap(), 2);
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn test_index_of() {
        let mut table = StringTable::new();
        table.add("project").unwrap();
        table.add("commands").unwrap();
        assert_eq!(table.index_of("project"), Some(0));
        assert_eq!(table.index_of("commands"), Some(1));
        assert_eq!(table.index_of("missing"), None);
    }

    #[test]
    fn test_roundtrip() {
        let mut table = StringTable::new();
        table.add("project").unwrap();
        table.add("tech_stack").unwrap();
        table.add("docs").unwrap();

        let bytes = table.to_bytes().unwrap();
        let recovered = StringTable::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.len(), 3);
        assert_eq!(recovered.get(0).unwrap(), "project");
        assert_eq!(recovered.get(1).unwrap(), "tech_stack");
        assert_eq!(recovered.get(2).unwrap(), "docs");
    }

    #[test]
    fn test_max_entries() {
        let mut table = StringTable::new();
        for i in 0..256 {
            table.add(&format!("key_{}", i)).unwrap();
        }
        assert_eq!(table.len(), 256);

        // 257th should fail
        let result = table.add("overflow");
        assert!(matches!(result, Err(FafbError::StringTableFull { .. })));
    }

    #[test]
    fn test_max_name_length() {
        let mut table = StringTable::new();
        // 255 bytes is OK
        let name_255 = "a".repeat(255);
        assert!(table.add(&name_255).is_ok());

        // 256 bytes should fail
        let name_256 = "b".repeat(256);
        let result = table.add(&name_256);
        assert!(matches!(
            result,
            Err(FafbError::StringTableEntryTooLong { .. })
        ));
    }

    #[test]
    fn test_unicode_names() {
        let mut table = StringTable::new();
        table.add("日本語").unwrap();
        table.add("émojis").unwrap();

        let bytes = table.to_bytes().unwrap();
        let recovered = StringTable::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.get(0).unwrap(), "日本語");
        assert_eq!(recovered.get(1).unwrap(), "émojis");
    }

    #[test]
    fn test_empty_table_roundtrip() {
        let table = StringTable::new();
        let bytes = table.to_bytes().unwrap();
        let recovered = StringTable::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.len(), 0);
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_index_out_of_bounds() {
        let table = StringTable::new();
        let result = table.get(0);
        assert!(matches!(
            result,
            Err(FafbError::StringTableIndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_truncated_data() {
        let mut table = StringTable::new();
        table.add("hello").unwrap();
        let bytes = table.to_bytes().unwrap();

        // Truncate the data mid-string
        let truncated = &bytes[..5]; // count(2) + len(2) + 1 byte of "hello"
        let result = StringTable::from_bytes(truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_string_name() {
        let mut table = StringTable::new();
        let idx = table.add("").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(table.get(0).unwrap(), "");
    }

    #[test]
    fn test_dedup_preserves_first_index() {
        let mut table = StringTable::new();
        table.add("a").unwrap();
        table.add("b").unwrap();
        table.add("c").unwrap();
        // Adding "a" again should return 0, not 3
        assert_eq!(table.add("a").unwrap(), 0);
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn test_serialized_size() {
        let mut table = StringTable::new();
        table.add("abc").unwrap(); // 3 bytes
        table.add("de").unwrap(); // 2 bytes

        let bytes = table.to_bytes().unwrap();
        // 2 (count) + 2+3 (entry 0) + 2+2 (entry 1) = 11
        assert_eq!(bytes.len(), 11);
    }

    #[test]
    fn test_too_small_data() {
        // Less than 2 bytes for count
        let result = StringTable::from_bytes(&[0x01]);
        assert!(matches!(result, Err(FafbError::FileTooSmall { .. })));
    }
}
