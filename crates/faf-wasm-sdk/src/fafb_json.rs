//! WASM JSON wrappers over `faf-fafb` (v2) — compile / decompile / info / score.
//!
//! This is shell presentation only: the binary format lives in `faf-fafb`,
//! scoring in `faf-kernel`. These functions just JSON-stringify the results
//! for clean JS interop (string/bytes in, JSON/bytes out).

use faf_fafb::{CompileOptions, DecompiledFafb, compile, decompile};

/// Compile YAML source to FAFb v2 binary bytes (WASM-safe: no SystemTime).
pub fn compile_fafb(yaml: &str) -> Result<Vec<u8>, String> {
    let opts = CompileOptions {
        use_timestamp: false,
    };
    compile(yaml, &opts)
}

/// Decompile FAFb binary bytes to JSON (full content).
pub fn decompile_fafb(bytes: &[u8]) -> Result<String, String> {
    let result = decompile(bytes).map_err(|e| e.to_string())?;
    Ok(decompiled_to_json(&result))
}

/// Get FAFb file info as JSON (header + section table, no content).
pub fn fafb_info(bytes: &[u8]) -> Result<String, String> {
    let result = decompile(bytes).map_err(|e| e.to_string())?;
    Ok(info_to_json(&result))
}

/// Score a FAFb binary: reconstruct the `.faf` from its chunks and run the
/// kernel (always-33). Returns the same JSON shape as `score_faf`.
///
/// Each chunk stores `key:\n<serialized value>` (a display form whose value is
/// not re-indented under the key), so reconstruction parses each chunk's value
/// back and rebuilds a proper top-level mapping — concatenating raw chunk text
/// would not be valid YAML.
pub fn score_fafb(bytes: &[u8]) -> Result<String, String> {
    use serde_yaml_ng::{Mapping, Value};

    let result = decompile(bytes).map_err(|e| e.to_string())?;
    let st_idx = result.header.string_table_index as usize;

    let mut map = Mapping::new();
    for (i, entry) in result.section_table.entries().iter().enumerate() {
        if i == st_idx {
            continue; // skip the structural string-table section
        }
        let name = result.section_name(entry);
        let raw = result.section_string(entry).unwrap_or_default();
        // raw = "<name>:\n<serialized value>" — drop the key line, parse the rest.
        let value_text = raw.split_once('\n').map(|x| x.1).unwrap_or("");
        let value: Value = serde_yaml_ng::from_str(value_text).unwrap_or(Value::Null);
        map.insert(Value::String(name), value);
    }

    let yaml = serde_yaml_ng::to_string(&Value::Mapping(map)).map_err(|e| e.to_string())?;
    let scored = faf_kernel::score(&yaml)?;
    Ok(scored.to_json())
}

/// Convert DecompiledFafb to JSON with all section data.
fn decompiled_to_json(result: &DecompiledFafb) -> String {
    let header = &result.header;
    let sections = &result.section_table;

    let mut json = String::from("{");
    json.push_str(&format!(
        "\"version\":\"{}.{}\",\"flags\":{},\"section_count\":{},\"total_size\":{},\"source_checksum\":\"{:#010x}\",",
        header.version_major,
        header.version_minor,
        header.flags.raw(),
        header.section_count,
        header.total_size,
        header.source_checksum
    ));

    json.push_str("\"sections\":[");
    for (i, entry) in sections.entries().iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let content = result.section_string(entry).unwrap_or_default();
        let name = result.section_name(entry);
        json.push_str(&format!(
            "{{\"name\":\"{}\",\"name_index\":{},\"priority\":{},\"offset\":{},\"length\":{},\"token_count\":{},\"classification\":\"{}\",\"content\":\"{}\"}}",
            escape_json_string(&name),
            entry.name_index,
            entry.priority.value(),
            entry.offset,
            entry.length,
            entry.token_count,
            entry.classification().name(),
            escape_json_string(&content)
        ));
    }
    json.push_str("]}");
    json
}

/// Convert DecompiledFafb to info-only JSON (no section content).
fn info_to_json(result: &DecompiledFafb) -> String {
    let header = &result.header;
    let sections = &result.section_table;

    let mut json = String::from("{");
    json.push_str(&format!(
        "\"version\":\"{}.{}\",\"flags\":{},\"section_count\":{},\"total_size\":{},\"source_checksum\":\"{:#010x}\",\"created\":{},",
        header.version_major,
        header.version_minor,
        header.flags.raw(),
        header.section_count,
        header.total_size,
        header.source_checksum,
        header.created_timestamp
    ));

    json.push_str("\"sections\":[");
    for (i, entry) in sections.entries().iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let name = result.section_name(entry);
        json.push_str(&format!(
            "{{\"name\":\"{}\",\"name_index\":{},\"priority\":{},\"length\":{},\"token_count\":{},\"classification\":\"{}\"}}",
            escape_json_string(&name),
            entry.name_index,
            entry.priority.value(),
            entry.length,
            entry.token_count,
            entry.classification().name()
        ));
    }
    json.push_str("]}");
    json
}

/// Escape a string for JSON embedding.
fn escape_json_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}
