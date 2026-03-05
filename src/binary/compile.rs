//! Compile/decompile API for .faf ↔ .fafb conversion
//!
//! Provides high-level functions to convert between YAML (.faf) and binary (.fafb) formats.

use std::io::Write;

use super::error::FafbResult;
use super::header::{FafbHeader, HEADER_SIZE, MAX_FILE_SIZE, MAX_SECTIONS};
use super::priority::Priority;
use super::section::{SectionEntry, SectionTable, SECTION_ENTRY_SIZE};
use super::section_type::SectionType;

/// A decompiled .fafb file with header, section table, and raw data
#[derive(Debug, Clone)]
pub struct DecompiledFafb {
    /// The 32-byte header
    pub header: FafbHeader,
    /// Section table with all entries
    pub section_table: SectionTable,
    /// Raw file data (for extracting section content)
    pub data: Vec<u8>,
}

impl DecompiledFafb {
    /// Extract the raw bytes for a section entry
    pub fn section_data(&self, entry: &SectionEntry) -> Option<&[u8]> {
        let start = entry.offset as usize;
        let end = start + entry.length as usize;
        if end <= self.data.len() {
            Some(&self.data[start..end])
        } else {
            None
        }
    }

    /// Extract section data as a UTF-8 string
    pub fn section_string(&self, entry: &SectionEntry) -> Option<String> {
        self.section_data(entry)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .map(|s| s.to_string())
    }

    /// Get section data by type
    pub fn get_section(&self, section_type: SectionType) -> Option<&[u8]> {
        self.section_table
            .get_by_type(section_type)
            .and_then(|entry| self.section_data(entry))
    }

    /// Get section data by type as string
    pub fn get_section_string(&self, section_type: SectionType) -> Option<String> {
        self.section_table
            .get_by_type(section_type)
            .and_then(|entry| self.section_string(entry))
    }
}

/// Compile a .faf YAML source string into .fafb binary bytes.
///
/// Extracts sections from the YAML and assembles them into the binary format
/// with header, section data, and section table.
///
/// # Example
///
/// ```rust
/// use faf_rust_sdk::binary::compile::compile;
///
/// let yaml = r#"
/// faf_version: 2.5.0
/// project:
///   name: my-project
///   goal: Build something great
/// instant_context:
///   tech_stack: Rust, TypeScript
///   key_files:
///     - src/main.rs
/// "#;
///
/// let fafb_bytes = compile(yaml).unwrap();
/// assert_eq!(&fafb_bytes[0..4], b"FAFB");
/// ```
pub fn compile(yaml_source: &str) -> Result<Vec<u8>, String> {
    let source_bytes = yaml_source.as_bytes();
    if source_bytes.is_empty() {
        return Err("Source content is empty".to_string());
    }

    // Parse as raw YAML value for section extraction
    let yaml: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(yaml_source).map_err(|e| format!("Invalid YAML: {}", e))?;

    // Build sections from YAML
    let mut sections: Vec<(SectionType, Priority, Vec<u8>)> = Vec::new();

    // META section (critical) — project identity
    let version = yaml
        .get("faf_version")
        .and_then(|v| v.as_str())
        .unwrap_or("2.5.0");
    let name = yaml
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");
    let meta_content = format!("faf_version: {}\nname: {}\n", version, name);
    sections.push((
        SectionType::Meta,
        Priority::critical(),
        meta_content.into_bytes(),
    ));

    // TECH_STACK section (high)
    if let Some(content) = extract_section(&yaml, "tech_stack") {
        sections.push((
            SectionType::TechStack,
            Priority::high(),
            format!("tech_stack:\n{}", content).into_bytes(),
        ));
    }
    // Also check instant_context.tech_stack
    if sections
        .iter()
        .all(|(t, _, _)| *t != SectionType::TechStack)
    {
        if let Some(tech) = yaml
            .get("instant_context")
            .and_then(|ic| ic.get("tech_stack"))
        {
            if let Ok(content) = serde_yaml_ng::to_string(tech) {
                if !content.trim().is_empty() {
                    sections.push((
                        SectionType::TechStack,
                        Priority::high(),
                        format!("tech_stack: {}", content).into_bytes(),
                    ));
                }
            }
        }
    }

    // KEY_FILES section (high)
    if let Some(content) = extract_section(&yaml, "key_files") {
        sections.push((
            SectionType::KeyFiles,
            Priority::high(),
            format!("key_files:\n{}", content).into_bytes(),
        ));
    } else if let Some(kf) = yaml
        .get("instant_context")
        .and_then(|ic| ic.get("key_files"))
    {
        if let Ok(content) = serde_yaml_ng::to_string(kf) {
            if !content.trim().is_empty() {
                sections.push((
                    SectionType::KeyFiles,
                    Priority::high(),
                    format!("key_files:\n{}", content).into_bytes(),
                ));
            }
        }
    }

    // COMMANDS section (high)
    if let Some(content) = extract_section(&yaml, "commands") {
        sections.push((
            SectionType::Commands,
            Priority::new(180),
            format!("commands:\n{}", content).into_bytes(),
        ));
    } else if let Some(cmds) = yaml
        .get("instant_context")
        .and_then(|ic| ic.get("commands"))
    {
        if let Ok(content) = serde_yaml_ng::to_string(cmds) {
            if !content.trim().is_empty() {
                sections.push((
                    SectionType::Commands,
                    Priority::new(180),
                    format!("commands:\n{}", content).into_bytes(),
                ));
            }
        }
    }

    // ARCHITECTURE section (medium)
    if let Some(content) = extract_section(&yaml, "architecture") {
        sections.push((
            SectionType::Architecture,
            Priority::medium(),
            format!("architecture:\n{}", content).into_bytes(),
        ));
    }

    // CONTEXT section (low)
    if let Some(content) = extract_section(&yaml, "context") {
        sections.push((
            SectionType::Context,
            Priority::low(),
            format!("context:\n{}", content).into_bytes(),
        ));
    }

    if sections.len() > MAX_SECTIONS as usize {
        return Err(format!(
            "Too many sections: {} exceeds maximum {}",
            sections.len(),
            MAX_SECTIONS
        ));
    }

    // Calculate layout
    let section_count = sections.len();
    let section_table_size = section_count * SECTION_ENTRY_SIZE;

    let mut data_offset: u32 = HEADER_SIZE as u32;
    let mut section_data: Vec<u8> = Vec::new();
    let mut section_table = SectionTable::new();

    for (section_type, priority, data) in &sections {
        let entry = SectionEntry::new(*section_type, data_offset, data.len() as u32)
            .with_priority(*priority);
        section_table.push(entry);
        section_data.extend_from_slice(data);
        data_offset = data_offset
            .checked_add(data.len() as u32)
            .ok_or_else(|| "Section data exceeds u32::MAX bytes".to_string())?;
    }

    let section_table_offset = data_offset;
    let total_size = section_table_offset
        .checked_add(section_table_size as u32)
        .ok_or_else(|| "Total file size exceeds u32::MAX bytes".to_string())?;

    if total_size > MAX_FILE_SIZE {
        return Err(format!(
            "Output size {} bytes exceeds maximum {} bytes (10MB)",
            total_size, MAX_FILE_SIZE
        ));
    }

    // Build header
    let mut header = FafbHeader::with_timestamp();
    header.set_source_checksum(source_bytes);
    header.section_count = section_count as u16;
    header.section_table_offset = section_table_offset;
    header.total_size = total_size;

    // Assemble binary
    let mut output: Vec<u8> = Vec::with_capacity(total_size as usize);
    header.write(&mut output).map_err(|e| e.to_string())?;
    output.write_all(&section_data).map_err(|e| e.to_string())?;
    section_table
        .write(&mut output)
        .map_err(|e| e.to_string())?;

    if output.len() != total_size as usize {
        return Err(format!(
            "Internal error: size mismatch (expected {} bytes, got {} bytes)",
            total_size,
            output.len()
        ));
    }

    Ok(output)
}

/// Decompile .fafb binary bytes into a structured representation.
///
/// Parses the header and section table, returning a `DecompiledFafb`
/// that allows access to individual sections.
///
/// # Example
///
/// ```rust
/// use faf_rust_sdk::binary::compile::{compile, decompile};
/// use faf_rust_sdk::binary::SectionType;
///
/// let yaml = "faf_version: 2.5.0\nproject:\n  name: test\n";
/// let fafb_bytes = compile(yaml).unwrap();
///
/// let result = decompile(&fafb_bytes).unwrap();
/// assert_eq!(result.header.version_major, 1);
///
/// let meta = result.get_section_string(SectionType::Meta).unwrap();
/// assert!(meta.contains("test"));
/// ```
pub fn decompile(fafb_bytes: &[u8]) -> FafbResult<DecompiledFafb> {
    let header = FafbHeader::from_bytes(fafb_bytes)?;
    header.validate(fafb_bytes)?;

    // Read section table from the offset
    let table_start = header.section_table_offset as usize;
    let table_data = &fafb_bytes[table_start..];
    let section_table = SectionTable::from_bytes(table_data, header.section_count as usize)?;
    section_table.validate_bounds(header.total_size)?;

    Ok(DecompiledFafb {
        header,
        section_table,
        data: fafb_bytes.to_vec(),
    })
}

/// Extract a YAML section as string
fn extract_section(yaml: &serde_yaml_ng::Value, key: &str) -> Option<String> {
    yaml.get(key)
        .and_then(|v| serde_yaml_ng::to_string(v).ok())
        .filter(|s| !s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_yaml() -> &'static str {
        r#"faf_version: 2.5.0
project:
  name: test-project
  goal: Test the compiler
instant_context:
  tech_stack: Rust
  key_files:
    - src/main.rs
    - src/lib.rs
  commands:
    build: cargo build
    test: cargo test
"#
    }

    #[test]
    fn test_compile_produces_valid_fafb() {
        let bytes = compile(sample_yaml()).unwrap();
        assert_eq!(&bytes[0..4], b"FAFB");
        assert!(bytes.len() >= HEADER_SIZE);
    }

    #[test]
    fn test_compile_empty_fails() {
        assert!(compile("").is_err());
    }

    #[test]
    fn test_roundtrip_compile_decompile() {
        let yaml = sample_yaml();
        let bytes = compile(yaml).unwrap();
        let result = decompile(&bytes).unwrap();

        assert_eq!(result.header.version_major, 1);
        assert_eq!(result.header.version_minor, 0);
        assert!(result.section_table.len() >= 1);

        // META section must exist
        let meta = result.get_section_string(SectionType::Meta).unwrap();
        assert!(meta.contains("test-project"));
        assert!(meta.contains("2.5.0"));
    }

    #[test]
    fn test_roundtrip_preserves_sections() {
        let yaml = sample_yaml();
        let bytes = compile(yaml).unwrap();
        let result = decompile(&bytes).unwrap();

        // Check tech stack section
        let tech = result.get_section_string(SectionType::TechStack);
        assert!(tech.is_some());

        // Check key files section
        let kf = result.get_section_string(SectionType::KeyFiles);
        assert!(kf.is_some());

        // Check commands section
        let cmds = result.get_section_string(SectionType::Commands);
        assert!(cmds.is_some());
    }

    #[test]
    fn test_decompile_invalid_magic() {
        let bytes = vec![0u8; 32];
        assert!(decompile(&bytes).is_err());
    }

    #[test]
    fn test_decompile_too_small() {
        let bytes = vec![0u8; 16];
        assert!(decompile(&bytes).is_err());
    }

    #[test]
    fn test_compile_minimal_yaml() {
        let yaml = "faf_version: 2.5.0\nproject:\n  name: minimal\n";
        let bytes = compile(yaml).unwrap();
        let result = decompile(&bytes).unwrap();

        assert_eq!(result.section_table.len(), 1); // Just META
        let meta = result.get_section_string(SectionType::Meta).unwrap();
        assert!(meta.contains("minimal"));
    }

    #[test]
    fn test_source_checksum_matches() {
        let yaml = sample_yaml();
        let bytes = compile(yaml).unwrap();
        let result = decompile(&bytes).unwrap();

        let expected = FafbHeader::compute_checksum(yaml.as_bytes());
        assert_eq!(result.header.source_checksum, expected);
    }
}
