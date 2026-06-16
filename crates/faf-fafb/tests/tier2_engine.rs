//! WJTTC Tier 2 — ENGINE ⚙️ (v2 format contract + golden-master byte parity).
//!
//! The format contract that callers and other engines depend on: the FAFB v2
//! magic + version, the determinism guarantee (zeroed timestamp), the
//! tamper-evident source checksum, and — the centerpiece — a BYTE-EXACT golden
//! master. The inline tests check that two compiles match each other; this
//! pins the exact bytes against the vendored fixture, so any change to the v2
//! wire format is loud and falsifiable.

use faf_fafb::{CompileOptions, compile, decompile};
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
fn det() -> CompileOptions {
    CompileOptions {
        use_timestamp: false,
    }
}

/// GOLDEN MASTER — byte-exact parity.
///
/// Provenance: `tests/parity/golden.fafb` was generated 2026-06-15 from
/// `golden-input.faf` via `cargo run -p faf-fafb --example compile`
/// (use_timestamp:false). If this fails, the FAFb v2 wire format changed —
/// regenerate the fixture INTENTIONALLY with the same command, never blindly.
#[test]
fn golden_master_byte_parity() {
    let produced = compile(&golden_input(), &det()).unwrap();
    let golden = golden_bytes();
    assert_eq!(
        produced.len(),
        golden.len(),
        "byte length drift: produced {} vs golden {}",
        produced.len(),
        golden.len()
    );
    // First differing offset, for a readable failure.
    if let Some((i, (a, b))) = produced
        .iter()
        .zip(golden.iter())
        .enumerate()
        .find(|(_, (a, b))| a != b)
    {
        panic!("FAFb v2 byte-parity violated at offset {i}: produced {a:#04x} vs golden {b:#04x}");
    }
    assert_eq!(produced, golden);
}

#[test]
fn header_format_contract() {
    let b = compile(&golden_input(), &det()).unwrap();
    assert_eq!(&b[0..4], b"FAFB", "magic must be FAFB");
    assert_eq!(b[4], 2, "version major = 2");
    assert_eq!(b[5], 0, "version minor = 0");
    // Timestamp (bytes 12..20) is zeroed when use_timestamp:false — the
    // determinism guarantee. (Mirrors faf-cli's parity-brake invariant.)
    assert!(
        b[12..20].iter().all(|&x| x == 0),
        "timestamp must be zero for determinism"
    );
}

#[test]
fn source_checksum_is_tamper_evident() {
    let x =
        decompile(&compile("faf_version: 2.5.0\nproject:\n  name: x\n", &det()).unwrap()).unwrap();
    let y =
        decompile(&compile("faf_version: 2.5.0\nproject:\n  name: y\n", &det()).unwrap()).unwrap();
    assert_ne!(
        x.header.source_checksum, y.header.source_checksum,
        "different source must yield a different checksum"
    );
    // Stable: identical source → identical checksum.
    let x2 =
        decompile(&compile("faf_version: 2.5.0\nproject:\n  name: x\n", &det()).unwrap()).unwrap();
    assert_eq!(x.header.source_checksum, x2.header.source_checksum);
}

#[test]
fn decompiled_header_reports_v2() {
    let d = decompile(&golden_bytes()).unwrap();
    assert_eq!(d.header.version_major, 2);
    assert_eq!(d.header.version_minor, 0);
}

#[test]
fn decompile_rejects_v1_bytes() {
    // FAFb v1 is pre-release history. A v2 reader must HARD-REJECT it, never
    // misparse. The guard lives in the header read (header.rs); this pins it at
    // the public decompile boundary.
    let mut bytes = golden_bytes();
    bytes[4] = 1; // version_major = 1
    let err = decompile(&bytes).unwrap_err();
    assert!(
        matches!(
            err,
            faf_fafb::FafbError::IncompatibleVersion { actual: 1, .. }
        ),
        "v1 must be rejected as IncompatibleVersion, got: {err}"
    );
}
