//! Compile/decompile API for .faf ↔ .fafb conversion — FAFb v2, closed canonical.
//!
//! The writer emits exactly the canonical chunk set (see `canon`), in canonical
//! order. Non-canonical top-level YAML keys are folded into the `context`
//! chunk — nothing is lost, and the string table stays fixed. Identical
//! content therefore produces identical bytes regardless of input key order:
//! the brick is content-addressable.

use std::io::Write;

use super::canon::{CANONICAL_CHUNKS, CanonicalChunk, ChunkClassification, canonical_chunk};
use super::error::{FafbError, FafbResult};
use super::header::{FafbHeader, HEADER_SIZE, MAX_FILE_SIZE, MAX_SECTIONS};
use super::priority::Priority;
use super::section::{SECTION_ENTRY_SIZE, SectionEntry, SectionTable};
use super::string_table::StringTable;

/// Options for compilation
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Whether to include a timestamp (set to false for deterministic output in tests)
    pub use_timestamp: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            use_timestamp: true,
        }
    }
}

/// A decompiled .fafb file with header, section table, string table, and raw data
#[derive(Debug, Clone)]
pub struct DecompiledFafb {
    /// The 32-byte header
    pub header: FafbHeader,
    /// Section table with all entries
    pub section_table: SectionTable,
    /// Raw file data (for extracting section content)
    pub data: Vec<u8>,
    /// String table — maps section_name_index to name strings
    string_table: StringTable,
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

    /// Get the string table
    pub fn string_table(&self) -> &StringTable {
        &self.string_table
    }

    /// Get section name by entry — looks up in string table.
    /// Unknown indices read as "UNKNOWN" (the IFF rule: skip, don't fail).
    pub fn section_name(&self, entry: &SectionEntry) -> String {
        self.string_table
            .get(entry.name_index)
            .unwrap_or("UNKNOWN")
            .to_string()
    }

    /// Get section data by name
    pub fn get_section_by_name(&self, name: &str) -> Option<&[u8]> {
        let idx = self.string_table.index_of(name)?;
        self.section_table
            .entries()
            .iter()
            .find(|e| e.name_index == idx)
            .and_then(|entry| self.section_data(entry))
    }

    /// Get section data by name as string
    pub fn get_section_string_by_name(&self, name: &str) -> Option<String> {
        self.get_section_by_name(name)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .map(|s| s.to_string())
    }

    /// Get all DNA sections
    pub fn dna_sections(&self) -> Vec<&SectionEntry> {
        self.section_table
            .entries()
            .iter()
            .filter(|e| e.classification() == ChunkClassification::Dna)
            .collect()
    }

    /// Get all Context sections
    pub fn context_sections(&self) -> Vec<&SectionEntry> {
        self.section_table
            .entries()
            .iter()
            .filter(|e| e.classification() == ChunkClassification::Context)
            .collect()
    }

    /// Get the Pointer section (docs)
    pub fn pointer_section(&self) -> Option<&SectionEntry> {
        self.section_table
            .entries()
            .iter()
            .find(|e| e.classification() == ChunkClassification::Pointer)
    }
}

/// Compile a .faf YAML source string into .fafb v2 binary bytes.
///
/// FAFb v2 is closed canonical: only the canonical chunk set produces
/// sections, serialized in canonical order. Non-canonical top-level keys are
/// folded into the `context` chunk (sorted alphabetically, after any authored
/// `context` content) — preserved, never named.
///
/// # Example
///
/// ```rust
/// use faf_fafb::{compile, CompileOptions};
///
/// let yaml = r#"
/// faf_version: 2.5.0
/// project:
///   name: my-project
///   goal: Build something great
/// custom_data:
///   key: value
/// "#;
///
/// let opts = CompileOptions { use_timestamp: false };
/// let fafb_bytes = compile(yaml, &opts).unwrap();
/// assert_eq!(&fafb_bytes[0..4], b"FAFB");
/// assert_eq!(fafb_bytes[4], 2); // FAFb v2
/// ```
pub fn compile(yaml_source: &str, options: &CompileOptions) -> Result<Vec<u8>, String> {
    let source_bytes = yaml_source.as_bytes();
    if source_bytes.is_empty() {
        return Err("Source content is empty".to_string());
    }

    let yaml: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(yaml_source).map_err(|e| format!("Invalid YAML: {}", e))?;

    let mapping = yaml
        .as_mapping()
        .ok_or_else(|| "YAML root must be a mapping".to_string())?;

    // Partition top-level keys: canonical chunks vs keys to fold into context.
    let mut canonical_values: Vec<(&'static CanonicalChunk, serde_yaml_ng::Value)> = Vec::new();
    let mut folded: Vec<(String, serde_yaml_ng::Value)> = Vec::new();

    for (key, value) in mapping {
        let key_str = key
            .as_str()
            .ok_or_else(|| "YAML key must be a string".to_string())?;
        match canonical_chunk(key_str) {
            Some(chunk) => canonical_values.push((chunk, value.clone())),
            None => folded.push((key_str.to_string(), value.clone())),
        }
    }

    // Fold non-canonical keys into the context chunk: authored context entries
    // first, folded keys after, sorted alphabetically.
    if !folded.is_empty() {
        folded.sort_by(|a, b| a.0.cmp(&b.0));

        let context_chunk = canonical_chunk("context").expect("context is canonical");
        let existing = canonical_values
            .iter_mut()
            .find(|(chunk, _)| chunk.name == "context");

        let mut context_map = match existing.as_ref().map(|(_, v)| v) {
            None => serde_yaml_ng::Mapping::new(),
            Some(serde_yaml_ng::Value::Mapping(m)) => m.clone(),
            Some(_) => {
                return Err("context must be a mapping to fold non-canonical keys into".to_string());
            }
        };

        for (key, value) in folded {
            let yaml_key = serde_yaml_ng::Value::String(key.clone());
            if context_map.contains_key(&yaml_key) {
                return Err(format!(
                    "Cannot fold non-canonical key '{}' into context: key already exists there",
                    key
                ));
            }
            context_map.insert(yaml_key, value);
        }

        let merged = serde_yaml_ng::Value::Mapping(context_map);
        match existing {
            Some((_, v)) => *v = merged,
            None => canonical_values.push((context_chunk, merged)),
        }
    }

    if canonical_values.is_empty() {
        return Err("No sections found in YAML".to_string());
    }

    // Canonical serialization order — same content, same bytes, any key order.
    canonical_values.sort_by_key(|(chunk, _)| {
        CANONICAL_CHUNKS
            .iter()
            .position(|c| c.name == chunk.name)
            .expect("chunk is canonical")
    });

    // Build string table and sections in canonical order.
    let mut string_table = StringTable::new();
    let mut sections: Vec<(u8, ChunkClassification, Priority, Vec<u8>)> = Vec::new();

    for (chunk, value) in &canonical_values {
        let name_idx = string_table
            .add(chunk.name)
            .map_err(|e| format!("String table error: {}", e))?;

        let content = serde_yaml_ng::to_string(value)
            .map_err(|e| format!("Failed to serialize '{}': {}", chunk.name, e))?;
        let data = format!("{}:\n{}", chunk.name, content).into_bytes();

        sections.push((
            name_idx,
            chunk.classification,
            Priority::new(chunk.priority),
            data,
        ));
    }

    if sections.len() > MAX_SECTIONS as usize {
        return Err(format!(
            "Too many sections: {} exceeds maximum {}",
            sections.len(),
            MAX_SECTIONS
        ));
    }

    // Add __string_table__ name to string table before serializing
    let st_name_idx = string_table
        .add("__string_table__")
        .map_err(|e| format!("String table error: {}", e))?;

    let string_table_bytes = string_table
        .to_bytes()
        .map_err(|e| format!("String table serialization error: {}", e))?;

    // Layout: [HEADER 32B] [section data...] [string table data] [section table entries...]
    let mut data_offset: u32 = HEADER_SIZE as u32;
    let mut section_data: Vec<u8> = Vec::new();
    let mut section_table = SectionTable::new();

    for (name_idx, classification, priority, data) in &sections {
        let entry = SectionEntry::new(*name_idx, data_offset, data.len() as u32)
            .with_priority(*priority)
            .with_classification(*classification);

        section_table.push(entry);
        section_data.extend_from_slice(data);
        data_offset = data_offset
            .checked_add(data.len() as u32)
            .ok_or_else(|| "Section data exceeds u32::MAX bytes".to_string())?;
    }

    // String table section (last content section)
    let st_section_index = section_table.len() as u16;
    let st_entry = SectionEntry::new(st_name_idx, data_offset, string_table_bytes.len() as u32)
        .with_priority(Priority::critical());

    section_table.push(st_entry);
    section_data.extend_from_slice(&string_table_bytes);
    data_offset = data_offset
        .checked_add(string_table_bytes.len() as u32)
        .ok_or_else(|| "Section data exceeds u32::MAX bytes".to_string())?;

    let section_count = section_table.len();
    let section_table_size = section_count * SECTION_ENTRY_SIZE;
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
    let mut header = if options.use_timestamp {
        FafbHeader::with_timestamp()
    } else {
        FafbHeader::new()
    };
    header.set_source_checksum(source_bytes);
    header.section_count = section_count as u16;
    header.section_table_offset = section_table_offset;
    header.total_size = total_size;
    header.string_table_index = st_section_index;

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

/// Decompile .fafb v2 binary bytes into a structured representation.
///
/// Parses header, section table, and string table. FAFb v1 binaries are
/// rejected (`IncompatibleVersion`) — re-compile from the `.faf` source.
///
/// # Example
///
/// ```rust
/// use faf_fafb::{compile, decompile, CompileOptions};
///
/// let yaml = "faf_version: 2.5.0\nproject:\n  name: test\n";
/// let opts = CompileOptions { use_timestamp: false };
/// let fafb_bytes = compile(yaml, &opts).unwrap();
///
/// let result = decompile(&fafb_bytes).unwrap();
/// assert_eq!(result.header.version_major, 2);
///
/// let project = result.get_section_string_by_name("project").unwrap();
/// assert!(project.contains("test"));
/// ```
pub fn decompile(fafb_bytes: &[u8]) -> FafbResult<DecompiledFafb> {
    let header = FafbHeader::from_bytes(fafb_bytes)?;
    header.validate(fafb_bytes)?;

    // Read section table
    let table_start = header.section_table_offset as usize;
    let table_data = &fafb_bytes[table_start..];
    let section_table = SectionTable::from_bytes(table_data, header.section_count as usize)?;
    section_table.validate_bounds(header.total_size)?;

    // Extract string table (required)
    let st_index = header.string_table_index as usize;
    if st_index >= section_table.len() {
        return Err(FafbError::MissingStringTable);
    }
    let st_entry = section_table.get(st_index).unwrap();
    let st_start = st_entry.offset as usize;
    let st_end = st_start + st_entry.length as usize;
    if st_end > fafb_bytes.len() {
        return Err(FafbError::MissingStringTable);
    }
    let string_table = StringTable::from_bytes(&fafb_bytes[st_start..st_end])?;

    Ok(DecompiledFafb {
        header,
        section_table,
        data: fafb_bytes.to_vec(),
        string_table,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> CompileOptions {
        CompileOptions {
            use_timestamp: false,
        }
    }

    fn minimal_yaml() -> &'static str {
        "faf_version: 2.5.0\nproject:\n  name: test-project\n"
    }

    fn full_yaml() -> &'static str {
        r#"faf_version: 2.5.0
project:
  name: full-project
  goal: Test the compiler
tech_stack:
  languages:
    - Rust
    - TypeScript
commands:
  build: cargo build
  test: cargo test
architecture:
  style: microservices
context:
  notes: some context
docs:
  readme: README.md
custom_field:
  key: value
another_custom:
  deep:
    nested: data
"#
    }

    // ─── Core compile/decompile ───

    #[test]
    fn test_compile_produces_valid_v2_header() {
        let bytes = compile(minimal_yaml(), &opts()).unwrap();
        assert_eq!(&bytes[0..4], b"FAFB");
        assert_eq!(bytes[4], 2); // FAFb v2
        assert!(bytes.len() >= HEADER_SIZE);
    }

    #[test]
    fn test_compile_empty_fails() {
        assert!(compile("", &opts()).is_err());
    }

    #[test]
    fn test_compile_options_default() {
        let o = CompileOptions::default();
        assert!(o.use_timestamp);
    }

    #[test]
    fn test_roundtrip_minimal() {
        let bytes = compile(minimal_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        assert_eq!(result.header.version_major, 2);
        assert!(result.header.flags.has_string_table());

        // faf_version + project + __string_table__
        assert!(result.section_table.len() >= 3);

        let project = result.get_section_string_by_name("project").unwrap();
        assert!(project.contains("test-project"));
    }

    #[test]
    fn test_v1_binaries_rejected() {
        let mut bytes = compile(minimal_yaml(), &opts()).unwrap();
        bytes[4] = 1; // forge a v1 header
        let err = decompile(&bytes).unwrap_err();
        assert!(matches!(
            err,
            FafbError::IncompatibleVersion { actual: 1, .. }
        ));
    }

    #[test]
    fn test_roundtrip_full() {
        let bytes = compile(full_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let st = result.string_table();
        assert!(st.index_of("faf_version").is_some());
        assert!(st.index_of("project").is_some());
        assert!(st.index_of("tech_stack").is_some());
        assert!(st.index_of("commands").is_some());
        assert!(st.index_of("docs").is_some());

        // Closed canonical: non-canonical keys never become section names
        assert!(st.index_of("custom_field").is_none());
        assert!(st.index_of("another_custom").is_none());
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
    fn test_source_checksum() {
        let yaml = full_yaml();
        let bytes = compile(yaml, &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let expected = FafbHeader::compute_checksum(yaml.as_bytes());
        assert_eq!(result.header.source_checksum, expected);
    }

    #[test]
    fn test_deterministic_without_timestamp() {
        let yaml = minimal_yaml();
        let bytes1 = compile(yaml, &opts()).unwrap();
        let bytes2 = compile(yaml, &opts()).unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_canonical_order_key_order_independent() {
        // Same content, shuffled top-level key order → byte-identical output.
        // (The checksum seals the SOURCE, which differs — compare structure by
        // zeroing the source_checksum field, bytes 8..12.)
        let a = "faf_version: 2.5.0\nproject:\n  name: x\ncommands:\n  build: make\n";
        let b = "commands:\n  build: make\nfaf_version: 2.5.0\nproject:\n  name: x\n";

        let mut bytes_a = compile(a, &opts()).unwrap();
        let mut bytes_b = compile(b, &opts()).unwrap();
        for buf in [&mut bytes_a, &mut bytes_b] {
            for byte in &mut buf[8..12] {
                *byte = 0;
            }
        }
        assert_eq!(bytes_a, bytes_b);
    }

    // ─── Folding (closed canonical) ───

    #[test]
    fn test_non_canonical_keys_folded_into_context() {
        let yaml =
            "faf_version: 2.5.0\nproject:\n  name: test\nmy_exotic_field:\n  data: preserved\n";
        let bytes = compile(yaml, &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        // Not a section of its own…
        assert!(result.string_table().index_of("my_exotic_field").is_none());
        // …but fully preserved inside context.
        let context = result.get_section_string_by_name("context").unwrap();
        assert!(context.contains("my_exotic_field"));
        assert!(context.contains("preserved"));
    }

    #[test]
    fn test_folding_preserves_authored_context() {
        let bytes = compile(full_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let context = result.get_section_string_by_name("context").unwrap();
        assert!(context.contains("notes")); // authored content kept
        assert!(context.contains("custom_field")); // folded
        assert!(context.contains("another_custom")); // folded
    }

    #[test]
    fn test_folding_collision_is_an_error() {
        let yaml = "project:\n  name: x\ncontext:\n  dupe: authored\ndupe: folded\n";
        // 'dupe' is non-canonical and collides with context.dupe
        assert!(compile(yaml, &opts()).is_err());
    }

    #[test]
    fn test_folding_into_scalar_context_is_an_error() {
        let yaml = "project:\n  name: x\ncontext: just a string\nweird_key: value\n";
        assert!(compile(yaml, &opts()).is_err());
    }

    // ─── Section names ───

    #[test]
    fn test_section_names() {
        let bytes = compile(full_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        for entry in result.section_table.entries() {
            let name = result.section_name(entry);
            assert!(!name.is_empty());
            assert_ne!(name, "UNKNOWN");
        }
    }

    #[test]
    fn test_get_section_by_name() {
        let bytes = compile(full_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let project = result.get_section_string_by_name("project");
        assert!(project.is_some());
        assert!(project.unwrap().contains("full-project"));

        let docs = result.get_section_string_by_name("docs");
        assert!(docs.is_some());
        assert!(docs.unwrap().contains("README.md"));
    }

    // ─── Classification ───

    #[test]
    fn test_classification_dna() {
        let bytes = compile(full_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let dna = result.dna_sections();
        let dna_names: Vec<String> = dna.iter().map(|e| result.section_name(e)).collect();

        assert!(dna_names.contains(&"faf_version".to_string()));
        assert!(dna_names.contains(&"project".to_string()));
        assert!(dna_names.contains(&"tech_stack".to_string()));
        assert!(dna_names.contains(&"commands".to_string()));
        assert!(dna_names.contains(&"architecture".to_string()));
        assert!(dna_names.contains(&"context".to_string()));
    }

    #[test]
    fn test_classification_pointer() {
        let bytes = compile(full_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let ptr = result.pointer_section();
        assert!(ptr.is_some());
        let ptr_name = result.section_name(ptr.unwrap());
        assert_eq!(ptr_name, "docs");
    }

    // ─── String table ───

    #[test]
    fn test_string_table_flag_set() {
        let bytes = compile(minimal_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();
        assert!(result.header.flags.has_string_table());
    }

    #[test]
    fn test_string_table_index_valid() {
        let bytes = compile(minimal_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let st_idx = result.header.string_table_index as usize;
        assert!(st_idx < result.section_table.len());
    }

    // ─── Priority ───

    #[test]
    fn test_priority_from_canon_table() {
        let bytes = compile(full_yaml(), &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        for entry in result.section_table.entries() {
            let name = result.section_name(entry);
            match name.as_str() {
                "faf_version" | "project" => assert!(entry.priority.is_critical()),
                "commands" => assert_eq!(entry.priority.value(), 180),
                "architecture" | "docs" => assert_eq!(entry.priority.value(), 128),
                "context" => assert_eq!(entry.priority.value(), 64),
                _ => {}
            }
        }
    }

    // ─── Canonical chunk coverage ───

    #[test]
    fn test_all_canonical_chunks_compile() {
        let yaml = r#"faf_version: 2.5.0
project:
  name: all-types
instant_context:
  summary: test
human_context:
  who: devs
stack:
  build: cargo
tech_stack:
  - Rust
key_files:
  - main.rs
commands:
  build: make
monorepo:
  packages_count: 3
architecture:
  style: monolith
context:
  note: x
bi_sync:
  enabled: true
meta:
  extra: data
generated: 2026-06-12
docs:
  readme: README.md
"#;
        let bytes = compile(yaml, &opts()).unwrap();
        let result = decompile(&bytes).unwrap();

        let st = result.string_table();
        for key in &[
            "faf_version",
            "project",
            "instant_context",
            "human_context",
            "stack",
            "tech_stack",
            "key_files",
            "commands",
            "monorepo",
            "architecture",
            "context",
            "bi_sync",
            "meta",
            "generated",
            "docs",
        ] {
            assert!(
                st.index_of(key).is_some(),
                "Expected '{}' in string table",
                key
            );
        }
    }
}
