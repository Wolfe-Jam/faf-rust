//! WJTTC Tier 3 — AERO 🪽 (Edge cases / polish) for the binary format.
//!
//! Kept deliberately small — the inline unit tests already cover folding,
//! canon, and section mechanics. These add the end-to-end edges: unicode
//! survives the round-trip, large content compiles, and a non-canonical key
//! folds into the `context` chunk (never silently dropped).

use faf_fafb::{CompileOptions, compile, decompile};

fn det() -> CompileOptions {
    CompileOptions {
        use_timestamp: false,
    }
}

#[test]
fn unicode_content_roundtrips() {
    let yaml = "faf_version: 2.5.0\nproject:\n  name: 测试-🏎️\n  goal: ünïçødé café\n";
    let d = decompile(&compile(yaml, &det()).unwrap()).unwrap();
    let ok = d
        .section_table
        .entries()
        .iter()
        .any(|e| d.section_string(e).unwrap_or_default().contains("测试"));
    assert!(ok, "unicode must survive the round-trip intact");
}

#[test]
fn large_content_compiles_and_roundtrips() {
    let big = "data ".repeat(5_000);
    let yaml = format!(
        "faf_version: 2.5.0\nproject:\n  name: big\n  goal: {}\n",
        big
    );
    let bytes = compile(&yaml, &det()).unwrap();
    let d = decompile(&bytes).unwrap();
    assert!(d.section_table.entries().len() >= 2);
}

#[test]
fn non_canonical_key_folds_into_context_end_to_end() {
    // An exotic top-level key is never dropped — it folds into `context`.
    let yaml = "faf_version: 2.5.0\nproject:\n  name: x\nexotic_field:\n  data: kept\n";
    let d = decompile(&compile(yaml, &det()).unwrap()).unwrap();
    let folded = d.section_table.entries().iter().any(|e| {
        d.section_name(e) == "context" && d.section_string(e).unwrap_or_default().contains("kept")
    });
    assert!(folded, "non-canonical key must fold into the context chunk");
}
