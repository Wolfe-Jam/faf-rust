//! WJTTC Tier 1 — BRAKE 🛑 (Safety / must-never-fail).
//!
//! The brakes of the trust engine. These guarantee the scorer can never
//! panic, never divide by zero, never produce an out-of-range score, and
//! always returns the *same* answer for the same input. If any of these
//! fail, the engine is unsafe to ship at any score — they are the floor
//! beneath "FAF don't lie": a score is only trustworthy if it is bounded,
//! deterministic, and crash-proof.

use faf_kernel::score;

/// A battery of adversarial / pathological inputs. The contract is narrow:
/// the scorer may return `Ok` or `Err`, but it must NEVER panic.
fn adversarial_inputs() -> Vec<String> {
    vec![
        String::new(),                                        // empty
        "   ".to_string(),                                    // whitespace only
        "\n\n\n".to_string(),                                 // blank lines
        "\u{feff}project:\n  name: x".to_string(),            // UTF-8 BOM
        "project:\n  name: \"\u{0}\u{1}\u{2}\"".to_string(),  // control chars
        "42".to_string(),                                     // bare scalar
        "- a\n- b\n- c".to_string(),                          // top-level sequence
        "just a string".to_string(),                          // bare string
        "true".to_string(),                                   // bare bool
        "null".to_string(),                                   // bare null
        "项目:\n  名称: 测试".to_string(),                    // non-ASCII keys
        "project:\n  name: 🏎️🔥💯".to_string(),               // emoji value
        format!("project:\n  name: {}", "x".repeat(100_000)), // huge value
        format!("a:\n{}", "  b:\n".repeat(5_000)),            // deeply nested
        "project: {name: x, name: y}".to_string(),            // duplicate keys
        "stack:\n  framework: slotignored".to_string(),       // sparse
        std::iter::repeat_n("k: v\n", 10_000).collect(),      // many keys
    ]
}

#[test]
fn never_panics_on_adversarial_input() {
    for input in adversarial_inputs() {
        // The only requirement: this call returns (Ok or Err), it does not panic.
        let _ = score(&input);
    }
}

#[test]
fn score_is_always_within_0_to_100() {
    for input in adversarial_inputs() {
        if let Ok(r) = score(&input) {
            assert!(r.score <= 100, "score {} out of range for input", r.score);
        }
    }
}

#[test]
fn slot_accounting_always_balances() {
    // populated + empty + ignored == 33, and active == 33 - ignored, always.
    for input in adversarial_inputs() {
        if let Ok(r) = score(&input) {
            let empty = r.total - r.populated - r.ignored;
            assert_eq!(r.populated + empty + r.ignored, 33, "slots must sum to 33");
            assert_eq!(r.active, r.total - r.ignored, "active = total - ignored");
            assert_eq!(r.total, 33, "total is always 33");
            assert!(r.active <= 33 && r.populated <= 33 && r.ignored <= 33);
        }
    }
}

#[test]
fn empty_and_whitespace_score_zero_white() {
    // Spaces/newlines parse as an empty (null) doc → 0. (A lone tab is invalid
    // YAML and errors out — that's covered by the never-panic battery, not here.)
    for input in ["", "   ", "\n\n"] {
        let r = score(input).expect("blank input parses as empty doc");
        assert_eq!(r.score, 0);
        assert_eq!(r.tier, "WHITE");
        assert_eq!(r.populated, 0);
    }
}

#[test]
fn malformed_yaml_is_error_not_panic() {
    for bad in [
        "invalid: yaml: [",
        "{ unclosed",
        "key: : value",
        "  - bad: indent\n bad",
    ] {
        // Must be a clean Err, never a panic.
        let _ = score(bad);
    }
    assert!(score("invalid: yaml: [").is_err());
}

#[test]
fn all_slotignored_never_divides_by_zero() {
    // Every one of the 33 slots marked slotignored → active = 0. The score
    // must be 0 (WHITE), NOT NaN, infinity, or a panic. This is the single
    // most important brake: the denominator can legitimately reach zero.
    let mut yaml = String::from(
        "project:\n  name: slotignored\n  goal: slotignored\n  main_language: slotignored\n",
    );
    yaml.push_str("human_context:\n");
    for w in ["who", "what", "why", "where", "when", "how"] {
        yaml.push_str(&format!("  {}: slotignored\n", w));
    }
    yaml.push_str("stack:\n");
    for s in [
        "frontend",
        "css_framework",
        "ui_library",
        "state_management",
        "backend",
        "api_type",
        "runtime",
        "database",
        "connection",
        "hosting",
        "build",
        "cicd",
        "monorepo_tool",
        "package_manager",
        "workspaces",
        "admin",
        "cache",
        "search",
        "storage",
    ] {
        yaml.push_str(&format!("  {}: slotignored\n", s));
    }
    yaml.push_str("monorepo:\n");
    for m in [
        "packages_count",
        "build_orchestrator",
        "versioning_strategy",
        "shared_configs",
        "remote_cache",
    ] {
        yaml.push_str(&format!("  {}: slotignored\n", m));
    }
    let r = score(&yaml).unwrap();
    assert_eq!(r.ignored, 33);
    assert_eq!(r.active, 0);
    assert_eq!(r.score, 0, "0 active slots → 0 score, never NaN/panic");
    assert_eq!(r.tier, "WHITE");
}

#[test]
fn scoring_is_deterministic() {
    // Same input, 1000 runs → byte-identical JSON every time. The score is
    // mechanical and falsifiable; it must not vary run to run.
    let yaml = "project:\n  name: x\n  goal: y\nstack:\n  build: cargo\n";
    let first = score(yaml).unwrap().to_json();
    for _ in 0..1000 {
        assert_eq!(score(yaml).unwrap().to_json(), first);
    }
}

#[test]
fn key_order_does_not_change_the_score() {
    let a = "project:\n  name: x\n  goal: y\nstack:\n  build: cargo\n";
    let b = "stack:\n  build: cargo\nproject:\n  goal: y\n  name: x\n";
    assert_eq!(score(a).unwrap().score, score(b).unwrap().score);
    assert_eq!(score(a).unwrap().populated, score(b).unwrap().populated);
}

#[test]
fn non_mapping_inputs_score_zero_no_panic() {
    // Scalars/sequences have no addressable slots → 0, but must not crash.
    for input in ["42", "- a\n- b", "true", "3.14"] {
        let r = score(input).unwrap();
        assert_eq!(r.populated, 0);
        assert_eq!(r.score, 0);
        assert_eq!(r.total, 33);
    }
}
