//! Content ID and file digest — computed, never stored.
//!
//! See BINARY-FORMAT.md § Identities. Stamps and `__`-prefixed chunks are
//! excluded from the Content ID.

use sha2::{Digest, Sha256};

use super::canon::{CANONICAL_CHUNKS, is_structural_name};
use super::compile::DecompiledFafb;
use super::error::FafbResult;
use super::section::SectionEntry;

/// SHA-256 as 64 lowercase hex characters, no `0x` prefix.
pub fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// File digest: SHA-256 over every byte of the file.
pub fn file_digest(fafb_bytes: &[u8]) -> String {
    hex_sha256(fafb_bytes)
}

/// Content ID of a compiled `.fafb`.
///
/// Concatenation, in canonical table order, of each present non-structural
/// chunk encoded as:
/// `name_len (u8) ‖ name (UTF-8) ‖ class (u8) ‖ priority (u8) ‖ payload_len (u32 LE) ‖ payload`
pub fn content_id(fafb_bytes: &[u8]) -> FafbResult<String> {
    let decoded = super::compile::decompile(fafb_bytes)?;
    Ok(content_id_of(&decoded))
}

/// Content ID from an already-decompiled file.
pub fn content_id_of(decoded: &DecompiledFafb) -> String {
    hex_sha256(&content_id_preimage(decoded))
}

fn content_id_preimage(decoded: &DecompiledFafb) -> Vec<u8> {
    let mut buf = Vec::new();
    for chunk in CANONICAL_CHUNKS {
        let Some(entry) = find_named(decoded, chunk.name) else {
            continue;
        };
        let Some(payload) = decoded.section_data(entry) else {
            continue;
        };
        encode_chunk(&mut buf, chunk.name, entry, payload);
    }
    buf
}

fn find_named<'a>(decoded: &'a DecompiledFafb, name: &str) -> Option<&'a SectionEntry> {
    decoded
        .section_table
        .entries()
        .iter()
        .find(|e| decoded.section_name(e) == name && !is_structural_name(name))
}

fn encode_chunk(buf: &mut Vec<u8>, name: &str, entry: &SectionEntry, payload: &[u8]) {
    let name_bytes = name.as_bytes();
    buf.push(name_bytes.len() as u8);
    buf.extend_from_slice(name_bytes);
    buf.push(entry.classification().bits() as u8);
    buf.push(entry.priority.value());
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(payload);
}

/// Canonical text rendering: concatenation of content-chunk payloads in
/// canonical table order. Each payload already begins with `name:\n`.
pub fn canonical_rendering(decoded: &DecompiledFafb) -> String {
    let mut out = String::new();
    for chunk in CANONICAL_CHUNKS {
        if let Some(text) = decoded.get_section_string_by_name(chunk.name) {
            out.push_str(&text);
        }
    }
    out
}

/// Canonical rendering after dropping whole priority tiers from the tail.
///
/// `drop_up_to` is the last tier to drop, in spec order: 64, then 128, then
/// 150–200. Critical (255) chunks always remain.
pub fn truncated_rendering(decoded: &DecompiledFafb, drop_up_to: TruncationTier) -> String {
    let mut out = String::new();
    for chunk in CANONICAL_CHUNKS {
        if drop_up_to.drops(chunk.priority) {
            continue;
        }
        if let Some(text) = decoded.get_section_string_by_name(chunk.name) {
            out.push_str(&text);
        }
    }
    out
}

/// Spec truncation tiers, dropped from the tail in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TruncationTier {
    /// Drop nothing.
    None,
    /// Drop priority 64 (scores, context).
    P64,
    /// Also drop priority 128 (architecture).
    P128,
    /// Also drop priorities 150–200.
    P150_200,
}

impl TruncationTier {
    fn drops(self, priority: u8) -> bool {
        if priority == 255 {
            return false;
        }
        match self {
            Self::None => false,
            Self::P64 => priority == 64,
            Self::P128 => priority == 64 || priority == 128,
            Self::P150_200 => (150..=200).contains(&priority) || priority == 64 || priority == 128,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompileOptions, compile};

    fn det() -> CompileOptions {
        CompileOptions {
            use_timestamp: false,
        }
    }

    #[test]
    fn hex_is_64_lowercase() {
        let h = hex_sha256(b"abc");
        assert_eq!(h.len(), 64);
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
        assert!(!h.starts_with("0x"));
    }

    #[test]
    fn comment_does_not_change_content_id() {
        let a = "faf_version: 2.5.0\nproject:\n  name: x\n";
        let b = "# a comment the AI never sees\nfaf_version: 2.5.0\nproject:\n  name: x\n";
        let ba = compile(a, &det()).unwrap();
        let bb = compile(b, &det()).unwrap();
        assert_eq!(content_id(&ba).unwrap(), content_id(&bb).unwrap());
        assert_ne!(file_digest(&ba), file_digest(&bb));
    }
}
