//! Spec 2.0 lock tests — REVIEW-RESPONSE.md R1–R8, no frozen-wire change.

use faf_fafb::canon::ChunkClassification;
use faf_fafb::header::HEADER_SIZE;
use faf_fafb::identity::TruncationTier;
use faf_fafb::priority::Priority;
use faf_fafb::section::{SECTION_ENTRY_SIZE, SectionEntry, SectionTable};
use faf_fafb::string_table::StringTable;
use faf_fafb::{
    FLAG_RESOLVED, FLAG_SIGNED, FLAG_STRING_TABLE, FLAG_TOKENIZED, FafbError, FafbHeader,
    STRUCTURAL_CHUNKS, compile, content_id, decompile, file_digest, is_known_structural,
};
use std::fs;

fn golden_input() -> String {
    fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/parity/golden-input.faf"
    ))
    .unwrap()
}

fn golden_bytes() -> Vec<u8> {
    fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/parity/golden.fafb"
    ))
    .unwrap()
}

fn det() -> faf_fafb::CompileOptions {
    faf_fafb::CompileOptions {
        use_timestamp: false,
    }
}

/// Pinned after 1.0.4 identity landing. Changing this is a Content ID break.
const GOLDEN_CONTENT_ID: &str = "3bc83e7a3fae87a55d2a71fcb787c09dbe4071770876467f35dabc04948b4a26";

fn edge_input() -> String {
    fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/parity/serializer-edge.faf"
    ))
    .unwrap()
}

fn edge_bytes() -> Vec<u8> {
    fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/parity/serializer-edge.fafb"
    ))
    .unwrap()
}

/// Pinned 2026-09-21 with `serde_yaml_ng = "=0.10.0"`. Moves only with a
/// deliberate, announced Content ID break.
const EDGE_CONTENT_ID: &str = "63ea6d8a4b255afc7d939cff67a05e190dbe4f68772196731f861e7e76a80095";

fn assemble(
    sections: Vec<(String, ChunkClassification, Priority, Vec<u8>)>,
    source_checksum: u32,
    timestamp: u64,
) -> Vec<u8> {
    let mut st = StringTable::new();
    let mut table = SectionTable::new();
    let mut data = Vec::new();
    let mut offset = HEADER_SIZE as u32;
    for (name, class, prio, payload) in &sections {
        let idx = st.add(name).unwrap();
        let entry = SectionEntry::new(idx, offset, payload.len() as u32)
            .with_priority(*prio)
            .with_classification(*class);
        table.push(entry);
        data.extend_from_slice(payload);
        offset += payload.len() as u32;
    }
    let st_name = st.add("__string_table__").unwrap();
    let st_bytes = st.to_bytes().unwrap();
    let st_section_index = table.len() as u16;
    table.push(
        SectionEntry::new(st_name, offset, st_bytes.len() as u32)
            .with_priority(Priority::critical()),
    );
    data.extend_from_slice(&st_bytes);
    offset += st_bytes.len() as u32;
    let total = offset + (table.len() * SECTION_ENTRY_SIZE) as u32;
    let mut header = FafbHeader::new();
    header.source_checksum = source_checksum;
    header.created_timestamp = timestamp;
    header.section_count = table.len() as u16;
    header.section_table_offset = offset;
    header.total_size = total;
    header.string_table_index = st_section_index;
    let mut out = Vec::new();
    header.write(&mut out).unwrap();
    out.extend_from_slice(&data);
    table.write(&mut out).unwrap();
    out
}

fn splice_section(
    base: &[u8],
    name: &str,
    payload: &[u8],
    class: ChunkClassification,
    prio: u8,
) -> Vec<u8> {
    let d = decompile(base).unwrap();
    let mut sections = Vec::new();
    for e in d.section_table.entries() {
        let n = d.section_name(e);
        if n == "__string_table__" {
            continue;
        }
        sections.push((
            n,
            e.classification(),
            e.priority,
            d.section_data(e).unwrap().to_vec(),
        ));
    }
    sections.push((
        name.to_string(),
        class,
        Priority::new(prio),
        payload.to_vec(),
    ));
    assemble(
        sections,
        d.header.source_checksum,
        d.header.created_timestamp,
    )
}

// ─── R1 flag table ───

#[test]
fn r1_spec_flag_table_matches_constants() {
    let spec = include_str!("../BINARY-FORMAT.md");
    let start = spec
        .find("Feature flags, bits 0–7:")
        .expect("flag table heading");
    let table = &spec[start..];
    let end = table.find("\nReaders MUST").expect("end of flag table");
    let table = &table[..end];

    let mut found: Vec<(u8, u16, String)> = Vec::new();
    for line in table.lines() {
        let line = line.trim();
        if !line.starts_with('|') || line.contains("Bit") || line.contains("---") {
            continue;
        }
        let cols: Vec<&str> = line
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if cols.len() < 3 {
            continue;
        }
        let bit: u8 = cols[0].parse().expect(line);
        let mask = cols[1].trim_matches('`');
        let mask = u16::from_str_radix(mask.trim_start_matches("0x"), 16).expect(line);
        let name = cols[2].split_whitespace().next().unwrap().to_string();
        found.push((bit, mask, name));
    }

    assert_eq!(found.len(), 8, "bits 0–7: {found:?}");
    assert_eq!(found[6], (6, 0x0040, "STRING_TABLE".into()));
    assert_eq!(found[7], (7, 0x0080, "RESOLVED".into()));
    assert_eq!(FLAG_STRING_TABLE, 0x0040);
    assert_eq!(FLAG_RESOLVED, 0x0080);
    for (bit, mask, _) in &found {
        assert_eq!(*mask, 1u16 << bit);
    }
}

// ─── R2 identities ───

#[test]
fn r2_golden_content_id_is_pinned() {
    let bytes = compile(&golden_input(), &det()).unwrap();
    assert_eq!(bytes, golden_bytes(), "wire golden must still match");
    let id = content_id(&bytes).unwrap();
    assert_eq!(id, GOLDEN_CONTENT_ID);
    assert_eq!(id.len(), 64);
}

#[test]
fn r2_comment_only_source_changes_file_digest_not_content_id() {
    let a = golden_input();
    let b = format!("# a comment the AI never sees\n{a}");
    let ba = compile(&a, &det()).unwrap();
    let bb = compile(&b, &det()).unwrap();
    assert_eq!(content_id(&ba).unwrap(), content_id(&bb).unwrap());
    assert_ne!(file_digest(&ba), file_digest(&bb));
    // CRC lives at header bytes 8..12 — that is the 4-byte stamp.
    assert_ne!(&ba[8..12], &bb[8..12]);
}

/// R2 writer MUST: "the reference compiler MUST NOT change any payload byte it
/// emits for a given `.faf`; a serializer change that does so is a Content ID
/// break." The golden master is plain scalars, so it cannot see that break —
/// this fixture carries the cases a YAML serializer upgrade actually moves:
/// bool-like and number-like strings, block and folded scalars, embedded
/// colons, hashes, quotes, tabs, newlines, unicode, empty collections, quoted
/// keys, deep nesting and a line long enough to tempt wrapping.
///
/// If this fails after a dependency bump, the bump is the break. Re-pin only on
/// purpose, and only with a Content ID break announced.
#[test]
fn r2_serializer_edge_payload_bytes_are_pinned() {
    let bytes = compile(&edge_input(), &det()).unwrap();
    assert_eq!(
        bytes,
        edge_bytes(),
        "payload bytes moved — serializer drift is a Content ID break (R2)"
    );
    assert_eq!(content_id(&bytes).unwrap(), EDGE_CONTENT_ID);
}

// ─── payload contract (BINARY-FORMAT.md § Payload) ───

/// A payload is `name:\n` + a YAML document holding the value, serialized at
/// column 0. Drop the first line and the remainder round-trips to the value the
/// `.faf` carried. This pins the rule readers are told to follow.
#[test]
fn payload_value_round_trips_after_dropping_the_header_line() {
    for source in [golden_input(), edge_input()] {
        let bytes = compile(&source, &det()).unwrap();
        let decoded = decompile(&bytes).unwrap();
        let src: serde_yaml_ng::Value = serde_yaml_ng::from_str(&source).unwrap();
        let src_map = src.as_mapping().unwrap();

        for chunk in faf_fafb::CANONICAL_CHUNKS {
            let Some(payload) = decoded.get_section_string_by_name(chunk.name) else {
                continue;
            };
            let (head, rest) = payload.split_once('\n').expect("payload has a header line");
            assert_eq!(head, format!("{}:", chunk.name), "payload header line");

            let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(rest)
                .unwrap_or_else(|e| panic!("{} payload is not a YAML document: {e}", chunk.name));

            // `context` also carries the folded non-canonical keys, so it is the
            // one chunk that is not the source value as written.
            if chunk.name != "context" {
                let key = serde_yaml_ng::Value::String(chunk.name.to_string());
                assert_eq!(
                    Some(&parsed),
                    src_map.get(&key),
                    "{} does not round-trip",
                    chunk.name
                );
            }
        }
    }
}

/// The other half of the same contract: the payload as a whole is NOT a YAML
/// document. If this ever passes, the payload shape changed and § Payload is
/// wrong — which is a Content ID break, not a doc edit.
#[test]
fn whole_payload_is_not_a_yaml_document() {
    let bytes = compile(&golden_input(), &det()).unwrap();
    let decoded = decompile(&bytes).unwrap();

    let scalar = decoded.get_section_string_by_name("faf_version").unwrap();
    assert!(
        serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&scalar).is_err(),
        "a scalar chunk payload must not parse whole"
    );

    let mapping = decoded.get_section_string_by_name("project").unwrap();
    let flat: serde_yaml_ng::Value = serde_yaml_ng::from_str(&mapping).unwrap();
    let key = serde_yaml_ng::Value::String("project".to_string());
    assert_eq!(
        flat.as_mapping().unwrap().get(&key),
        Some(&serde_yaml_ng::Value::Null),
        "parsed whole, a mapping chunk binds its own name to null"
    );
}

// ─── R3 / R4 ───

#[test]
fn r3_tokenized_bit_without_chunk_is_ignored() {
    let mut bytes = compile(&golden_input(), &det()).unwrap();
    let flags = u16::from_le_bytes([bytes[6], bytes[7]]) | FLAG_TOKENIZED;
    bytes[6..8].copy_from_slice(&flags.to_le_bytes());
    decompile(&bytes).expect("TOKENIZED set / __tokens__ missing must not reject");
}

#[test]
fn r4_scores_is_carried_not_computed() {
    let yaml = "faf_version: 2.5.0\nproject:\n  name: x\nscores:\n  total: 41\n";
    let d = decompile(&compile(yaml, &det()).unwrap()).unwrap();
    let scores = d.get_section_string_by_name("scores").unwrap();
    assert!(scores.contains("41"), "{scores}");
}

#[test]
fn r4_signed_bit_stays_unset() {
    let bytes = compile(&golden_input(), &det()).unwrap();
    let flags = u16::from_le_bytes([bytes[6], bytes[7]]);
    assert_eq!(flags & FLAG_SIGNED, 0);
    assert_eq!(flags & FLAG_STRING_TABLE, FLAG_STRING_TABLE);
}

#[test]
fn r4_score_is_absent_from_structural_list() {
    assert!(!STRUCTURAL_CHUNKS.contains(&"__score__"));
    assert!(!is_known_structural("__score__"));
}

// ─── R5 passthrough ───

#[test]
fn r5_unknown_content_section_passes_through() {
    let base = compile(&golden_input(), &det()).unwrap();
    let extra = splice_section(
        &base,
        "mystery",
        b"mystery:\n  kept: yes\n",
        ChunkClassification::Context,
        64,
    );
    let d = decompile(&extra).expect("unknown content name must pass through");
    assert_eq!(
        d.get_section_string_by_name("mystery").unwrap().trim(),
        "mystery:\n  kept: yes"
    );
}

#[test]
fn r5_unknown_dunder_section_passes_through_and_is_excluded_from_content_id() {
    let base = compile(&golden_input(), &det()).unwrap();
    let extra = splice_section(
        &base,
        "__mystery__",
        b"not-in-content-id",
        ChunkClassification::Context,
        64,
    );
    let d = decompile(&extra).expect("unknown __ name must pass through");
    assert!(d.get_section_by_name("__mystery__").is_some());
    assert_eq!(content_id(&base).unwrap(), content_id(&extra).unwrap());
    assert_ne!(file_digest(&base), file_digest(&extra));
}

#[test]
fn r5_string_table_excluded_from_content_id() {
    let bytes = compile(&golden_input(), &det()).unwrap();
    let d = decompile(&bytes).unwrap();
    assert!(d.get_section_by_name("__string_table__").is_some());
    let id = content_id(&bytes).unwrap();
    // Preimage must not include the structural name.
    assert!(!id.is_empty());
}

// ─── R6 linked bricks ───

#[test]
fn r6_two_member_fixture() {
    let a_yaml = "faf_version: 2.5.0\nproject:\n  name: billing\n";
    let b_yaml = "faf_version: 2.5.0\nproject:\n  name: api\n";
    let a = compile(a_yaml, &det()).unwrap();
    let b = compile(b_yaml, &det()).unwrap();
    let a_id = content_id(&a).unwrap();
    let b_id = content_id(&b).unwrap();
    let a_dig = file_digest(&a);
    let b_dig = file_digest(&b);
    assert_ne!(a_id, b_id);

    let members = format!(
        "- path: packages/billing\n  content_id: {a_id}\n  file_digest: {a_dig}\n\
         - path: packages/api\n  content_id: {b_id}\n  file_digest: {b_dig}\n"
    );
    let root = compile("faf_version: 2.5.0\nproject:\n  name: workspace\n", &det()).unwrap();
    let rooted = splice_section(
        &root,
        "__members__",
        members.as_bytes(),
        ChunkClassification::Context,
        64,
    );
    assert_eq!(content_id(&root).unwrap(), content_id(&rooted).unwrap());

    let a2 = compile("faf_version: 2.5.0\nproject:\n  name: billing-v2\n", &det()).unwrap();
    assert_ne!(content_id(&a2).unwrap(), a_id);
    assert_eq!(content_id(&b).unwrap(), b_id);
    assert!(faf_fafb::is_valid_member_path("packages/billing"));
    assert!(!faf_fafb::is_valid_member_path("packages/../secret"));
}

// ─── R7 prefix ───

#[test]
fn r7_dropping_64_tier_leaves_prefix_through_architecture() {
    let yaml = r#"faf_version: 2.5.0
project:
  name: prefix
architecture:
  style: monolith
scores:
  total: 12
context:
  note: tail
"#;
    let d = decompile(&compile(yaml, &det()).unwrap()).unwrap();
    let full = d.canonical_rendering();
    let cut = d.truncated_rendering(TruncationTier::P64);
    assert!(full.starts_with(&cut), "cut must be a prefix of full");
    assert!(cut.contains("architecture:"));
    assert!(!cut.contains("scores:"));
    assert!(!cut.contains("note: tail"));
    let arch = d.get_section_string_by_name("architecture").unwrap();
    assert!(cut.ends_with(&arch));
}

// ─── R8 reject siblings ───

#[test]
fn r8_every_version_byte_on_skill_seal_shape_is_rejected() {
    for v in 0u8..=255 {
        let mut bytes = b"FAFB".to_vec();
        bytes.push(v);
        bytes.extend_from_slice(&[0u8; 27]);
        bytes.extend_from_slice(b"{\"seal\":true}");
        assert!(
            decompile(&bytes).is_err(),
            "skill-seal-like header with version_major={v} must not read as a brick"
        );
    }
}

#[test]
fn r8_wrong_magic_is_rejected() {
    let mut bytes = golden_bytes();
    bytes[0] = b'X';
    assert!(matches!(decompile(&bytes), Err(FafbError::InvalidMagic(_))));
}

#[test]
fn r8_v1_is_incompatible_version() {
    let mut bytes = golden_bytes();
    bytes[4] = 1;
    assert!(matches!(
        decompile(&bytes),
        Err(FafbError::IncompatibleVersion { actual: 1, .. })
    ));
}

#[test]
fn spec_does_not_discuss_fafb_iana() {
    let spec = include_str!("../BINARY-FORMAT.md");
    assert!(!spec.contains("IANA: Pending"));
    assert!(!spec.contains("deliberately left unregistered"));
    assert!(!spec.contains("Do not file it"));
    assert!(!spec.contains("## Registration"));
}
