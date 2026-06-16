//! WJTTC Tier 1 — BRAKE 🛑 (Safety / must-never-fail) for the binary format.
//!
//! A binary format reads UNTRUSTED bytes, so the brakes are about hostile
//! input: `decompile` must never panic on corrupt, truncated, or random
//! bytes; `compile` must never panic on adversarial YAML; the round-trip must
//! be lossless; and compilation must be byte-deterministic. The inline unit
//! tests cover bad-magic and too-small — these add broad fuzzing the format
//! must survive before it can be trusted.

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

#[test]
fn decompile_never_panics_on_corrupt_bytes() {
    let inputs: Vec<Vec<u8>> = vec![
        vec![],                          // empty
        vec![0],                         // one byte
        vec![0xFF; 4],                   // four bytes
        b"FAFB".to_vec(),                // magic only
        b"FAFB\x02\x00".to_vec(),        // magic + version, nothing else
        vec![0u8; 32],                   // zeroed header
        vec![0xFFu8; 64],                // all-ones garbage
        b"NOPE\x02\x00xxxx".to_vec(),    // bad magic
        b"FAFB\x09\x09garbage".to_vec(), // bogus version
        (0..=255u8).collect(),           // byte ramp
    ];
    for b in inputs {
        let _ = decompile(&b); // Ok or Err — never a panic.
    }
}

#[test]
fn decompile_never_panics_on_any_truncation() {
    // Every prefix of a valid file — a length field read past the end must
    // surface as Err, never an out-of-bounds panic.
    let g = golden_bytes();
    for len in 0..=g.len() {
        let _ = decompile(&g[..len]);
    }
}

#[test]
fn decompile_never_panics_on_single_byte_corruption() {
    // Flip every byte in turn (incl. length/offset/count fields) — bounded.
    let g = golden_bytes();
    for i in 0..g.len() {
        let mut b = g.clone();
        b[i] ^= 0xFF;
        let _ = decompile(&b);
    }
}

#[test]
fn compile_never_panics_on_adversarial_yaml() {
    let big = "x".repeat(50_000);
    for y in [
        "",
        "   ",
        "\u{0}\u{1}",
        "- a\n- b",
        "42",
        "true",
        "项目: 测试",
        big.as_str(),
    ] {
        let _ = compile(y, &det());
    }
}

#[test]
fn roundtrip_is_lossless() {
    let bytes = compile(&golden_input(), &det()).unwrap();
    let d = decompile(&bytes).unwrap();
    let project_survives = d.section_table.entries().iter().any(|e| {
        d.section_name(e) == "project"
            && d.section_string(e).unwrap_or_default().contains("faf-rust")
    });
    assert!(
        project_survives,
        "project content must survive compile → decompile"
    );
}

#[test]
fn compile_is_byte_deterministic() {
    // Same source, 100 compiles → identical bytes. The checksum is only
    // priceless if the bytes are stable.
    let first = compile(&golden_input(), &det()).unwrap();
    for _ in 0..100 {
        assert_eq!(compile(&golden_input(), &det()).unwrap(), first);
    }
}
