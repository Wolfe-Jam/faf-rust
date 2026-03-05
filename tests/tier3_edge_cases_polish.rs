//! WJTTC Tier 3: AERO - Edge Cases & Polish Tests
//!
//! Unicode, large inputs, YAML quirks, and resilience testing.
//! The aerodynamic details that separate good from championship.

use faf_rust_sdk::{find_and_parse, parse, validate};
use std::fs;
use tempfile::TempDir;

// =============================================================================
// UNICODE & SPECIAL CHARACTERS (Tests 1-4)
// =============================================================================

#[test]
fn test_unicode_name() {
    let content = r#"
faf_version: 2.5.0
project:
  name: "测试プロジェクト🚀"
  goal: "Build something 大きい"
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.project_name(), "测试プロジェクト🚀");
    assert_eq!(faf.goal(), Some("Build something 大きい"));
}

#[test]
fn test_emoji() {
    let content = r#"
faf_version: 2.5.0
project:
  name: "🏎️ F1 Project"
instant_context:
  what_building: "🚀 Rocket launcher"
  tech_stack: "Rust 🦀, Python 🐍"
"#;
    let faf = parse(content).unwrap();
    assert!(faf.project_name().contains("🏎️"));
    assert!(faf.tech_stack().unwrap().contains("🦀"));
}

#[test]
fn test_special_chars() {
    let content = r#"
faf_version: 2.5.0
project:
  name: "test<>&\"'project"
  goal: "Handle \t tabs \n newlines"
"#;
    let faf = parse(content).unwrap();
    assert!(faf.project_name().contains("<>&"));
}

#[test]
fn test_multiline() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: |
    This is a multiline
    goal that spans
    multiple lines
"#;
    let faf = parse(content).unwrap();
    assert!(faf.goal().unwrap().contains("multiline"));
    assert!(faf.goal().unwrap().contains("multiple lines"));
}

// =============================================================================
// YAML QUIRKS (Tests 5-10)
// =============================================================================

#[test]
fn test_yaml_anchors() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
defaults: &defaults
  testing: required
preferences:
  <<: *defaults
  documentation: inline
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.project_name(), "test");
}

#[test]
fn test_null_values() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: null
  main_language: ~
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.goal(), None);
}

#[test]
fn test_empty_string() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
  goal: ""
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.goal(), Some(""));
}

#[test]
fn test_empty_arrays() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
instant_context:
  key_files: []
tags: []
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.key_files().len(), 0);
}

#[test]
fn test_boolean_coercion() {
    let content = r#"
faf_version: 2.5.0
project:
  name: "yes"
  goal: "true"
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.project_name(), "yes");
}

#[test]
fn test_numeric_strings() {
    let content = r#"
faf_version: "2.5.0"
project:
  name: "123"
  version: "1.0.0"
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.project_name(), "123");
}

// =============================================================================
// SCORE EDGE CASES (Tests 11-13)
// =============================================================================

#[test]
fn test_score_space() {
    let content = r#"
faf_version: 2.5.0
ai_score: "85 %"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), None, "Space before % should fail parse");
}

#[test]
fn test_score_float() {
    let content = r#"
faf_version: 2.5.0
ai_score: "85.5%"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), None, "Float won't parse to u8");
}

#[test]
fn test_score_text() {
    let content = r#"
faf_version: 2.5.0
ai_score: "HIGH"
project:
  name: test
"#;
    let faf = parse(content).unwrap();
    assert_eq!(faf.score(), None, "Text should return None");
}

// =============================================================================
// LARGE INPUTS (Tests 14-16)
// =============================================================================

#[test]
fn test_1000_key_files() {
    let mut files = Vec::new();
    for i in 0..1000 {
        files.push(format!("    - file{}.rs", i));
    }
    let files_yaml = files.join("\n");

    let content = format!(
        r#"
faf_version: 2.5.0
project:
  name: test
instant_context:
  key_files:
{}
"#,
        files_yaml
    );

    let faf = parse(&content).unwrap();
    assert_eq!(faf.key_files().len(), 1000);
}

#[test]
fn test_500_tags() {
    let mut tags = Vec::new();
    for i in 0..500 {
        tags.push(format!("  - tag{}", i));
    }
    let tags_yaml = tags.join("\n");

    let content = format!(
        r#"
faf_version: 2.5.0
project:
  name: test
tags:
{}
"#,
        tags_yaml
    );

    let faf = parse(&content).unwrap();
    assert_eq!(faf.data.tags.len(), 500);
}

#[test]
fn test_10k_strings() {
    let long_name = "x".repeat(10000);
    let content = format!(
        r#"
faf_version: 2.5.0
project:
  name: "{}"
"#,
        long_name
    );

    let faf = parse(&content).unwrap();
    assert_eq!(faf.project_name().len(), 10000);
}

// =============================================================================
// ACCESSOR EDGE CASES (Test 17)
// =============================================================================

#[test]
fn test_accessors_missing_sections() {
    let content = r#"
faf_version: 2.5.0
project:
  name: test
"#;
    let faf = parse(content).unwrap();

    assert_eq!(faf.tech_stack(), None);
    assert_eq!(faf.what_building(), None);
    assert_eq!(faf.key_files().len(), 0);
    assert_eq!(faf.goal(), None);
    assert_eq!(faf.score(), None);
    assert!(!faf.is_high_quality());
}

// =============================================================================
// RESILIENCE (Tests 18-20)
// =============================================================================

#[test]
fn test_unicode_corruption() {
    let content = r#"
faf_version: 2.5.0
ai_score: 90%

project:
  name: unicode-test-🦀
  goal: Test émojis and spëcial châractérs

instant_context:
  what_building: 日本語テスト
  tech_stack: Rust 🦀, Python 🐍
"#;
    let result = parse(content);
    assert!(result.is_ok(), "Should handle Unicode");
    let faf = result.unwrap();
    assert!(faf.project_name().contains("🦀") || faf.project_name().contains("unicode"));
}

#[test]
fn test_bisync_conflict() {
    let version_a = r#"
faf_version: 2.5.0
ai_score: 80%

project:
  name: shared-project
  goal: Version A - local changes

instant_context:
  what_building: Feature A
  tech_stack: Rust
"#;

    let version_b = r#"
faf_version: 2.5.0
ai_score: 85%

project:
  name: shared-project
  goal: Version B - remote changes

instant_context:
  what_building: Feature B
  tech_stack: Rust, Python
"#;

    let faf_a = parse(version_a).unwrap();
    let faf_b = parse(version_b).unwrap();

    let score_diff = (faf_a.score().unwrap_or(0) as i32 - faf_b.score().unwrap_or(0) as i32).abs();
    let goal_a = faf_a.data.project.goal.as_deref().unwrap_or("");
    let goal_b = faf_b.data.project.goal.as_deref().unwrap_or("");

    assert_ne!(goal_a, goal_b, "Should detect goal conflict");
    assert!(score_diff > 0, "Should detect score difference");
}

#[test]
fn test_rapid_modification() {
    let temp = TempDir::new().unwrap();
    let faf_path = temp.path().join("project.faf");

    let mut success_count = 0;

    for i in 0..100 {
        let content = format!(
            r#"
faf_version: 2.5.0
ai_score: {}%

project:
  name: rapid-test
  goal: Iteration {}
"#,
            50 + (i % 50),
            i
        );

        fs::write(&faf_path, &content).unwrap();

        if let Ok(faf) = find_and_parse::<std::path::PathBuf>(Some(temp.path().to_path_buf())) {
            if validate(&faf).valid {
                success_count += 1;
            }
        }
    }

    assert!(
        success_count >= 95,
        "Should handle rapid modifications reliably"
    );
}
