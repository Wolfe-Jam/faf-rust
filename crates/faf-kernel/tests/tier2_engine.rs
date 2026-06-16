//! WJTTC Tier 2 — ENGINE ⚙️ (Core scoring truth + parity).
//!
//! The engine of the trust engine: the always-33 model, the slot states, the
//! legacy aliases, the placeholder rejection, the canonical tier ladder, and
//! the GOLDEN-MASTER parity fixtures that pin the kernel's deterministic
//! output so it can never silently drift.
//!
//! The golden master IS the "FAF don't lie" receipt: the score is mechanical,
//! and these fixtures make any change to it loud and falsifiable.

use faf_kernel::{score, tier_name};
use std::fs;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/parity/{}", env!("CARGO_MANIFEST_DIR"), name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path, e))
}

// ── Always-33 model ────────────────────────────────────────────────────────

#[test]
fn total_is_always_33_regardless_of_app_type() {
    for yaml in [
        "project:\n  name: x\n",
        "app_type: enterprise\nproject:\n  name: y\n",
        "project:\n  name: z\n  type: documentation\n",
    ] {
        let r = score(yaml).unwrap();
        assert_eq!(r.total, 33);
        assert_eq!(r.slots.len(), 33);
    }
}

#[test]
fn slotignored_sets_the_active_denominator() {
    let r = score(&fixture("cli-trophy.faf")).unwrap();
    assert_eq!(r.ignored, 21);
    assert_eq!(r.active, 12);
    assert_eq!(r.populated, 12);
    assert_eq!(r.score, 100);
}

#[test]
fn missing_slot_counts_against_score_unlike_slotignored() {
    // cli-partial drops 3 active human_context slots: they are Empty (counted),
    // not Slotignored (excluded) — so the score is honest about the gap.
    let r = score(&fixture("cli-partial.faf")).unwrap();
    assert_eq!(r.ignored, 21);
    assert_eq!(
        r.active, 12,
        "still 12 active — the gaps are Empty, not Ignored"
    );
    assert_eq!(r.populated, 9);
    assert_eq!(r.score, 75);
}

// ── Legacy aliases ───────────────────────────────────────────────────────────

#[test]
fn each_legacy_alias_scores_as_its_canonical_slot() {
    let cases = [
        ("stack.framework", "frontend"),
        ("stack.css", "css_framework"),
        ("stack.state", "state_management"),
        ("stack.api", "api_type"),
        ("stack.db", "database"),
        ("stack.pkg_manager", "package_manager"),
    ];
    for (canonical, legacy) in cases {
        let yaml = format!("project:\n  name: x\nstack:\n  {}: SomeValue\n", legacy);
        let r = score(&yaml).unwrap();
        let populated_canonical = r
            .slots
            .iter()
            .any(|(n, s)| n == canonical && *s == faf_kernel::SlotState::Populated);
        assert!(
            populated_canonical,
            "legacy `{}` should populate `{}`",
            legacy, canonical
        );
    }
}

// ── Placeholder rejection (honest truth) ─────────────────────────────────────

#[test]
fn every_placeholder_is_rejected() {
    let placeholders = [
        "describe your project goal",
        "development teams",
        "cloud platform",
        "null",
        "none",
        "unknown",
        "n/a",
        "not applicable",
        "N/A",
        "Unknown",
        "NONE", // case-insensitive
    ];
    for ph in placeholders {
        let yaml = format!("project:\n  name: \"{}\"\n", ph);
        let r = score(&yaml).unwrap();
        assert_eq!(
            r.populated, 0,
            "placeholder `{}` must not count as populated",
            ph
        );
    }
}

// ── Canonical tier ladder (tiers.ts) ─────────────────────────────────────────

#[test]
fn tier_ladder_boundaries_are_exact() {
    // Each threshold and the value just below it.
    let cases = [
        (100, "TROPHY"),
        (99, "GOLD"),
        (98, "SILVER"),
        (95, "SILVER"),
        (94, "BRONZE"),
        (85, "BRONZE"),
        (84, "GREEN"),
        (70, "GREEN"),
        (69, "YELLOW"),
        (55, "YELLOW"),
        (54, "RED"),
        (1, "RED"),
        (0, "WHITE"),
    ];
    for (score_val, tier) in cases {
        assert_eq!(
            tier_name(score_val),
            tier,
            "score {} should be {}",
            score_val,
            tier
        );
    }
}

#[test]
fn full_33_all_populated_is_trophy() {
    let r = score(&fixture("enterprise-full.faf")).unwrap();
    assert_eq!(r.populated, 33);
    assert_eq!(r.ignored, 0);
    assert_eq!(r.active, 33);
    assert_eq!(r.score, 100);
    assert_eq!(r.tier, "TROPHY");
}

#[test]
fn to_json_accounting_is_correct() {
    let r = score(&fixture("cli-partial.faf")).unwrap();
    let json = r.to_json();
    // empty = total - populated - ignored = 33 - 9 - 21 = 3
    assert!(json.contains("\"empty\":3"), "json: {}", json);
    assert!(json.contains("\"populated\":9"));
    assert!(json.contains("\"ignored\":21"));
    assert!(json.contains("\"active\":12"));
    assert!(json.contains("\"total\":33"));
}

// ── GOLDEN MASTER — deterministic-output parity ──────────────────────────────
//
// Provenance: recorded 2026-06-15. The kernel scores always-33 + slotignored.
// A free faf-cli file uses up to 21 slots, with the 12 Team/Org/Enterprise
// slots slotignored; an enterprise file fills all 33. faf-cli's active-fraction
// display ("12/12", "21/21") is the SAME Y/N math as the kernel's "X active,
// N ignored" — cross-checked equal where shown.
//
//   fixture           kernel             faf-cli           note
//   cli-trophy     100 TROPHY 12/12    100 TROPHY 12/12   free, complete
//   cli-partial     75 GREEN  9/12      75 GREEN  9/12    free, mid-interview
//   enterprise-full 100 TROPHY 33/33    —                 enterprise, all 33
//   minimal-raw      3 RED    1/33      —                 barely-started file
//
// This table pins the kernel's deterministic output so it can never silently
// drift — the mechanical, falsifiable "FAF don't lie" receipt.

struct Golden {
    file: &'static str,
    score: u32,
    tier: &'static str,
    populated: u32,
    ignored: u32,
    active: u32,
}

const GOLDEN: &[Golden] = &[
    Golden {
        file: "cli-trophy.faf",
        score: 100,
        tier: "TROPHY",
        populated: 12,
        ignored: 21,
        active: 12,
    },
    Golden {
        file: "cli-partial.faf",
        score: 75,
        tier: "GREEN",
        populated: 9,
        ignored: 21,
        active: 12,
    },
    Golden {
        file: "enterprise-full.faf",
        score: 100,
        tier: "TROPHY",
        populated: 33,
        ignored: 0,
        active: 33,
    },
    Golden {
        file: "minimal-raw.faf",
        score: 3,
        tier: "RED",
        populated: 1,
        ignored: 0,
        active: 33,
    },
];

#[test]
fn golden_master_parity_holds() {
    for g in GOLDEN {
        let r = score(&fixture(g.file)).unwrap();
        assert_eq!(r.score, g.score, "{}: score drift", g.file);
        assert_eq!(r.tier, g.tier, "{}: tier drift", g.file);
        assert_eq!(r.populated, g.populated, "{}: populated drift", g.file);
        assert_eq!(r.ignored, g.ignored, "{}: ignored drift", g.file);
        assert_eq!(r.active, g.active, "{}: active drift", g.file);
        assert_eq!(r.total, 33, "{}: total must be 33", g.file);
    }
}
