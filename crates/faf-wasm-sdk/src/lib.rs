//! FAF WASM SDK — the kernel for the edge.
//!
//! A thin `wasm-bindgen` shell over the workspace kernel: scoring is
//! [`faf-kernel`](https://docs.rs/faf-kernel), the binary form is
//! [`faf-fafb`](https://docs.rs/faf-fafb). No scoring or format logic lives
//! here — the same engine that runs in the CLI and the MCP server runs in the
//! browser and at the edge, so there is nothing to drift.
//!
//! 7 pure-function exports. No classes. JSON / bytes in, JSON / bytes out.
//!
//! # Usage (JavaScript)
//! ```js
//! import init, { sdk_version, score_faf, validate_faf,
//!     compile_fafb, decompile_fafb, score_fafb, fafb_info } from 'faf-wasm-sdk';
//!
//! await init();
//! const result = score_faf(yamlContent);   // JSON string (always-33 Mk4)
//! const bytes  = compile_fafb(yamlContent); // Uint8Array (FAFb v2)
//! const json   = decompile_fafb(bytes);     // JSON string
//! ```

mod fafb_json;

use wasm_bindgen::prelude::*;

/// Get SDK version.
#[wasm_bindgen]
pub fn sdk_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Score FAF YAML content with the Mk4 kernel (always-33) — returns JSON.
///
/// Always 33 slots. A 21-slot file carries the 12 enterprise slots as
/// `slotignored`, so it scores the same here as in every other FAF app.
#[wasm_bindgen]
pub fn score_faf(yaml: String) -> Result<String, JsValue> {
    faf_kernel::score(&yaml)
        .map(|r| r.to_json())
        .map_err(|e| JsValue::from_str(&e))
}

/// Same as [`score_faf`] (always-33). Kept so existing callers keep working.
#[wasm_bindgen]
pub fn score_faf_enterprise(yaml: String) -> Result<String, JsValue> {
    score_faf(yaml)
}

/// Validate FAF YAML content — true if it parses as a YAML mapping.
#[wasm_bindgen]
pub fn validate_faf(yaml: String) -> bool {
    use serde_yaml_ng::Value;
    matches!(
        serde_yaml_ng::from_str::<Value>(&yaml),
        Ok(Value::Mapping(_))
    )
}

/// Compile YAML to a FAFb v2 binary — returns Uint8Array.
#[wasm_bindgen]
pub fn compile_fafb(yaml: String) -> Result<Vec<u8>, JsValue> {
    fafb_json::compile_fafb(&yaml).map_err(|e| JsValue::from_str(&e))
}

/// Decompile a FAFb binary to JSON (full content) — returns JSON string.
#[wasm_bindgen]
pub fn decompile_fafb(bytes: &[u8]) -> Result<String, JsValue> {
    fafb_json::decompile_fafb(bytes).map_err(|e| JsValue::from_str(&e))
}

/// Score a FAFb binary — returns JSON string (same shape as `score_faf`).
#[wasm_bindgen]
pub fn score_fafb(bytes: &[u8]) -> Result<String, JsValue> {
    fafb_json::score_fafb(bytes).map_err(|e| JsValue::from_str(&e))
}

/// Get FAFb file info (header + section metadata, no content) — returns JSON.
#[wasm_bindgen]
pub fn fafb_info(bytes: &[u8]) -> Result<String, JsValue> {
    fafb_json::fafb_info(bytes).map_err(|e| JsValue::from_str(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_version_is_3x() {
        assert!(sdk_version().starts_with("3."));
    }

    #[test]
    fn test_validate_faf_accepts_mapping() {
        assert!(validate_faf("project:\n  name: test".to_string()));
    }

    #[test]
    fn test_validate_faf_rejects_non_mapping() {
        assert!(!validate_faf("just a string".to_string()));
        assert!(!validate_faf("- list\n- items".to_string()));
        assert!(!validate_faf("42".to_string()));
    }

    #[test]
    fn test_validate_faf_rejects_broken_yaml() {
        assert!(!validate_faf("[invalid: yaml: {{{".to_string()));
    }

    #[test]
    fn test_score_faf_is_always_33() {
        let result = score_faf("project:\n  name: test".to_string()).unwrap();
        assert!(result.contains("\"score\":"));
        assert!(result.contains("\"tier\":"));
        assert!(result.contains("\"total\":33"));
    }

    // One engine: every score export returns the kernel's always-33 result.
    #[test]
    fn test_every_score_export_is_the_kernel() {
        let files = [
            "project:\n  name: test",
            "faf_version: 2.5.0\nproject:\n  name: a\n  goal: b\n  main_language: Rust\n",
            // 21 filled-or-marked base slots, no enterprise markers
            "project:\n  name: a\n  goal: b\n  main_language: Rust\nstack:\n  frontend: None\n",
        ];
        for yaml in files {
            let kernel = faf_kernel::score(yaml).unwrap().to_json();
            assert_eq!(score_faf(yaml.to_string()).unwrap(), kernel);
            assert_eq!(score_faf_enterprise(yaml.to_string()).unwrap(), kernel);
            if let Ok(fafb) = compile_fafb(yaml.to_string()) {
                assert_eq!(
                    score_of(&score_fafb(&fafb).unwrap()),
                    score_of(&kernel),
                    "score_fafb != score_faf for {yaml:?}"
                );
            }
        }
    }

    fn score_of(json: &str) -> u32 {
        let rest = &json[json.find("\"score\":").expect("score field") + 8..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        rest[..end].parse().expect("numeric score")
    }

    #[test]
    fn test_canonical_tiers_not_medals() {
        // Trophy is the only emoji; sub-Trophy tiers are canonical names.
        let result = score_faf("project:\n  name: test".to_string()).unwrap();
        assert!(!result.contains("🥇") && !result.contains("🥈") && !result.contains("🥉"));
    }

    #[test]
    fn test_compile_decompile_fafb_roundtrip_v2() {
        let yaml = "faf_version: 2.5.0\nproject:\n  name: test\n".to_string();
        let bytes = compile_fafb(yaml).unwrap();
        assert_eq!(&bytes[0..4], b"FAFB");
        assert_eq!(bytes[4], 2); // FAFb v2
        let json = decompile_fafb(&bytes).unwrap();
        assert!(json.contains("\"sections\":"));
        assert!(json.contains("\"version\":\"2.0\""));
    }

    #[test]
    fn test_score_fafb_from_compiled() {
        let yaml = "faf_version: 2.5.0\nproject:\n  name: test\n".to_string();
        let bytes = compile_fafb(yaml).unwrap();
        let score = score_fafb(&bytes).unwrap();
        assert!(score.contains("\"score\":"));
        assert!(score.contains("\"total\":33"));
    }

    #[test]
    fn test_fafb_info_from_compiled() {
        let yaml = "faf_version: 2.5.0\nproject:\n  name: test\n".to_string();
        let bytes = compile_fafb(yaml).unwrap();
        let info = fafb_info(&bytes).unwrap();
        assert!(info.contains("\"section_count\":"));
        assert!(!info.contains("\"content\":"));
    }
}
