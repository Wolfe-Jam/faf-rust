//! WJTTC Tier 1: BRAKES - Security & Validation Tests
//!
//! Corruption detection, type safety, and validation failures.
//! If brakes fail, everything fails.

use faf_rust_sdk::{parse, validate, find_and_parse};
use std::fs;
use tempfile::TempDir;

// =============================================================================
// MISSING/CORRUPT REQUIRED FIELDS (Tests 1-4)
// =============================================================================

#[test]
fn test_missing_version() {
    let content = r#"
project:
  name: broken-project
  goal: Missing version field
"#;
    let result = parse(content);
    assert!(result.is_err(), "Parser should reject missing faf_version");
}

#[test]
fn test_malformed_yaml() {
    let content = r#"
faf_version: 2.5.0
project:
name: bad-indent
  goal: Malformed YAML
"#;
    let result = parse(content);
    assert!(result.is_err(), "Should reject malformed YAML");
}

#[test]
fn test_truncated_file() {
    let content = r#"
faf_version: 2.5.0
ai_score: 75%

project:
  name: truncated
  goal: File was cut o"#;

    let result = parse(content);
    // Should either parse partially or detect as corrupt
    if let Ok(faf) = result {
        assert_eq!(faf.project_name(), "truncated");
    }
    // Both outcomes acceptable - graceful handling is key
}

#[test]
fn test_recovery_workflow() {
    let temp = TempDir::new().unwrap();
    let faf_path = temp.path().join("project.faf");

    let valid = r#"
faf_version: 2.5.0
ai_score: 85%
ai_confidence: HIGH

project:
  name: grok-integration
  goal: Demonstrate corruption recovery

instant_context:
  what_building: Resilient AI context system
  tech_stack: Rust, YAML, FAF
  key_files:
    - src/lib.rs
    - src/parser.rs

stack:
  backend: Rust
  infrastructure: xAI

human_context:
  who: xAI team
  what: Test bi-sync resilience
  why: Production readiness
"#;

    // Step 1: Create valid file
    fs::write(&faf_path, valid).unwrap();

    // Step 2: Verify it's valid
    let faf = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())).unwrap();
    let validation = validate(&faf);
    assert!(validation.valid, "Initial file should be valid");
    assert!(faf.score().unwrap_or(0) > 80);

    // Step 3: Corrupt it
    let corrupted = r#"
faf_version: 2.5.0
ai_score: CORRUPTED

project:
  name: corrupted
  goal: Corrupted file
"#;
    fs::write(&faf_path, corrupted).unwrap();

    // Step 4: Detect corruption
    let corrupt_faf = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())).unwrap();
    assert!(corrupt_faf.score().is_none() || validate(&corrupt_faf).warnings.len() > 0);

    // Step 5: Self-heal by restoring
    fs::write(&faf_path, valid).unwrap();
    let healed_faf = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())).unwrap();
    let healed_validation = validate(&healed_faf);
    assert!(healed_validation.valid);
}

// =============================================================================
// EMPTY/WHITESPACE CONTENT (Tests 5-6)
// =============================================================================

#[test]
fn test_empty_whitespace() {
    let result = parse("   \n\t\n   ");
    assert!(result.is_err());
}

#[test]
fn test_comments_only() {
    let content = r#"
# Just a comment
# Another comment
"#;
    let result = parse(content);
    assert!(result.is_err());
}

// =============================================================================
// MISSING REQUIRED SECTIONS (Tests 7-8)
// =============================================================================

#[test]
fn test_missing_project_section() {
    let content = r#"
faf_version: 2.5.0
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_missing_project_name() {
    let content = r#"
faf_version: 2.5.0
project:
  goal: Testing
"#;
    let result = parse(content);
    assert!(result.is_err());
}

// =============================================================================
// WRONG TYPES (Tests 9-10)
// =============================================================================

#[test]
fn test_wrong_type_key_files() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
instant_context:
  key_files: "main.rs"
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_wrong_type_tags() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
tags: "rust"
"#;
    let result = parse(content);
    assert!(result.is_err());
}

// =============================================================================
// INVALID YAML SYNTAX (Tests 11-12)
// =============================================================================

#[test]
fn test_unclosed_bracket() {
    let content = r#"
faf_version: 2.5.0
project:
  name: [unclosed
"#;
    let result = parse(content);
    assert!(result.is_err());
}

#[test]
fn test_bad_indentation() {
    let content = r#"
faf_version: 2.5.0
project:
name: test
"#;
    let result = parse(content);
    assert!(result.is_err());
}

// =============================================================================
// VALIDATION FAILURES (Tests 13-14)
// =============================================================================

#[test]
fn test_empty_version() {
    let content = r#"
faf_version: ""
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    let result = validate(&faf);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.contains("faf_version")));
}

#[test]
fn test_empty_name() {
    let content = r#"
faf_version: 2.5.0
project:
  name: ""
"#;
    let faf = parse(content).unwrap();
    let result = validate(&faf);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.contains("project.name")));
}

// =============================================================================
// SCORE BOUNDARY SAFETY (Tests 15-16)
// =============================================================================

#[test]
fn test_score_overflow() {
    let content = r#"
faf_version: 2.5.0
ai_score: "256%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), None, "256 overflows u8");
}

#[test]
fn test_negative_score() {
    let content = r#"
faf_version: 2.5.0
ai_score: "-5%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), None, "Negative should fail u8 parse");
}
