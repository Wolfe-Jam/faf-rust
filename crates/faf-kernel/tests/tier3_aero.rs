//! WJTTC Tier 3 — AERO 🪽 (Edge cases / polish / honesty under load).
//!
//! Where the engine earns trust at the margins: value-type handling, trimming,
//! the canonical symbol set (no medals), and the honest scoring of unfinished
//! files (every slot is Y = populated or N = slotignored; an undecided slot
//! counts against the score until it is one or the other).

use faf_kernel::{score, tier_symbol};
use std::fs;

fn fixture(name: &str) -> String {
    fs::read_to_string(format!(
        "{}/tests/parity/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    ))
    .unwrap()
}

// ── Value-type handling ──────────────────────────────────────────────────────

#[test]
fn unicode_and_emoji_values_populate() {
    let r = score("project:\n  name: 测试-项目\n  goal: 🏎️🔥\n").unwrap();
    assert_eq!(r.populated, 2);
}

#[test]
fn numbers_bools_sequences_and_maps_populate() {
    let yaml = "project:\n  name: 42\nmonorepo:\n  packages_count: 12\nstack:\n  workspaces: true\ntech_stack:\n  - Rust\n";
    let r = score(yaml).unwrap();
    // project.name(42), monorepo.packages_count(12), stack.workspaces(true) are slots.
    assert!(
        r.populated >= 3,
        "scalars/bools count as populated: {}",
        r.populated
    );
}

#[test]
fn empty_sequences_and_maps_are_empty() {
    let r = score("project:\n  name: []\n  goal: {}\n").unwrap();
    assert_eq!(r.populated, 0, "empty containers are not populated data");
}

#[test]
fn whitespace_only_string_is_empty() {
    let r = score("project:\n  name: \"   \"\n").unwrap();
    assert_eq!(r.populated, 0);
}

#[test]
fn slotignored_is_trimmed_before_matching() {
    let r = score("project:\n  name: x\n  goal: \"  slotignored  \"\n").unwrap();
    let goal_ignored = r
        .slots
        .iter()
        .any(|(n, s)| n == "project.goal" && *s == faf_kernel::SlotState::Slotignored);
    assert!(
        goal_ignored,
        "padded `slotignored` must still be Slotignored"
    );
}

#[test]
fn nested_non_empty_mapping_populates() {
    let r = score("project:\n  name:\n    nested: value\n").unwrap();
    assert!(r.populated >= 1);
}

// ── Canonical symbols — Trophy is the ONLY emoji, no medals ───────────────────

#[test]
fn no_medal_emoji_anywhere() {
    for s in [100u32, 99, 95, 85, 70, 55, 1, 0] {
        let sym = tier_symbol(s);
        assert!(!sym.contains('🥇') && !sym.contains('🥈') && !sym.contains('🥉'));
        assert!(
            !sym.contains('🟢')
                && !sym.contains('🟡')
                && !sym.contains('🔴')
                && !sym.contains('🤍')
        );
    }
    // And the JSON payload carries no medals either.
    let json = score(&fixture("cli-trophy.faf")).unwrap().to_json();
    for medal in ['🥇', '🥈', '🥉', '🟢', '🟡', '🔴'] {
        assert!(!json.contains(medal));
    }
}

#[test]
fn trophy_is_the_only_emoji_symbol() {
    assert_eq!(tier_symbol(100), "🏆");
    assert_eq!(tier_symbol(99), "★");
    assert_eq!(tier_symbol(95), "◆");
    assert_eq!(tier_symbol(85), "◇");
    assert_eq!(tier_symbol(0), "♡");
}

#[test]
fn green_and_yellow_share_the_geometric_glyph() {
    // The kernel omits color, so GREEN (≥70) and YELLOW (≥55) share ● — but
    // the tier NAME still distinguishes them.
    assert_eq!(tier_symbol(70), "●");
    assert_eq!(tier_symbol(55), "●");
}

// ── Incomplete files ─────────────────────────────────────────────────────────

#[test]
fn barely_started_file_scores_low_against_full_33() {
    // A file with one slot filled and nothing slotignored is just an unfinished
    // file: 33 active, 1 populated → 3%. Every slot is still to be decided
    // (filled = Y, or slotignored = N). The score honestly reflects that.
    let r = score(&fixture("minimal-raw.faf")).unwrap();
    assert_eq!(r.active, 33);
    assert_eq!(r.populated, 1);
    assert_eq!(r.score, 3);
}
