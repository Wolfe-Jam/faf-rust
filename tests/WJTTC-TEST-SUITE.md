# WJTTC Test Suite - faf-rust-sdk

**Championship-Grade Testing for FAF Rust SDK**

## Test Plan

| Tier | Name | Tests | File |
|------|------|-------|------|
| T1 | BRAKES - Security & Validation | 16 | `tier1_security_validation.rs` |
| T2 | ENGINE - Core Functionality | 22 | `tier2_core_functionality.rs` |
| T3 | AERO - Edge Cases & Polish | 20 | `tier3_edge_cases_polish.rs` |
| - | Unit Tests (inline) | 17 | `src/*.rs` |
| - | Doc Tests | 7 | `src/*.rs` |
| **Total** | | **82** | |

## Tier 1: BRAKES - 16 Tests

Corruption detection, type safety, validation failures.

| # | Test | What |
|---|------|------|
| 1 | `test_missing_version` | Missing faf_version rejected |
| 2 | `test_malformed_yaml` | Bad indentation rejected |
| 3 | `test_truncated_file` | Truncated content handled |
| 4 | `test_recovery_workflow` | Create/corrupt/detect/heal cycle |
| 5 | `test_empty_whitespace` | Whitespace-only rejected |
| 6 | `test_comments_only` | Comments-only rejected |
| 7 | `test_missing_project_section` | No project section rejected |
| 8 | `test_missing_project_name` | No project.name rejected |
| 9 | `test_wrong_type_key_files` | String instead of array rejected |
| 10 | `test_wrong_type_tags` | String instead of array rejected |
| 11 | `test_unclosed_bracket` | Invalid YAML syntax rejected |
| 12 | `test_bad_indentation` | Indentation error rejected |
| 13 | `test_empty_version` | Empty string faf_version invalid |
| 14 | `test_empty_name` | Empty string project.name invalid |
| 15 | `test_score_overflow` | 256% returns None (u8 overflow) |
| 16 | `test_negative_score` | -5% returns None |

## Tier 2: ENGINE - 22 Tests

Core parsing, validation, compression, discovery.

| # | Test | What |
|---|------|------|
| 1 | `test_parse_minimal` | Version + name parsing |
| 2 | `test_parse_full` | All major sections |
| 3 | `test_parse_score` | Score "85%" -> 85 |
| 4 | `test_score_0` | Boundary: 0% |
| 5 | `test_score_85` | Normal: 85% |
| 6 | `test_score_100` | Boundary: 100% |
| 7 | `test_score_no_percent` | "85" without % |
| 8 | `test_score_double_percent` | "85%%" lenient |
| 9 | `test_validation_scoring_minimal` | Score = 20 |
| 10 | `test_validation_scoring_full` | Score = 100 |
| 11 | `test_compression_minimal` | Drops stack/human_context |
| 12 | `test_compression_standard` | Key files limited to 5 |
| 13 | `test_compression_full` | Identity transform |
| 14 | `test_discovery_current_dir` | Found in start dir |
| 15 | `test_discovery_parent` | Found in parent dir |
| 16 | `test_discovery_roundtrip` | find_and_parse integration |
| 17 | `test_discovery_not_found` | Returns None |
| 18 | `test_quality_threshold_at_70` | 70% = high quality |
| 19 | `test_quality_threshold_below_70` | 69% = not high quality |
| 20 | `test_invalid_score_graceful` | Non-numeric score = None |
| 21 | `test_stringify_round_trip` | stringify -> parse round-trip |
| 22 | `test_estimate_tokens_values` | 150/400/800 token estimates |

## Tier 3: AERO - 20 Tests

Unicode, large inputs, YAML quirks, resilience.

| # | Test | What |
|---|------|------|
| 1 | `test_unicode_name` | CJK + emoji name |
| 2 | `test_emoji` | Emoji in all fields |
| 3 | `test_special_chars` | `<>&"'` preserved |
| 4 | `test_multiline` | YAML `\|` blocks |
| 5 | `test_yaml_anchors` | `&defaults` + `*defaults` |
| 6 | `test_null_values` | `null` and `~` -> None |
| 7 | `test_empty_string` | `""` -> Some("") |
| 8 | `test_empty_arrays` | `[]` -> len 0 |
| 9 | `test_boolean_coercion` | "yes"/"true" as strings |
| 10 | `test_numeric_strings` | "123" stays string |
| 11 | `test_score_space` | "85 %" fails |
| 12 | `test_score_float` | "85.5%" fails |
| 13 | `test_score_text` | "HIGH" fails |
| 14 | `test_1000_key_files` | 1000 files parsed |
| 15 | `test_500_tags` | 500 tags parsed |
| 16 | `test_10k_strings` | 10,000 char name |
| 17 | `test_accessors_missing_sections` | Graceful None/empty |
| 18 | `test_unicode_corruption` | International chars preserved |
| 19 | `test_bisync_conflict` | Two-version diff detection |
| 20 | `test_rapid_modification` | 100 rapid parse cycles |

## Run

```bash
cargo test
```
