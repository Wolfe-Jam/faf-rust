//! Mk4 Championship Engine — 33-slot scoring.
//!
//! The kernel knows **33 slots** and three states (Populated, Empty,
//! Slotignored). Score = populated ÷ active, where active = 33 − slotignored.
//!
//! It knows nothing about owner, intent, users, or app_type. A complex
//! enterprise repo and a minimal profile are the *same object* to it: a fill
//! pattern over 33 slots. `app_type` is a **generation-time** concern — it
//! decides which slots get written `slotignored` — and the kernel only ever
//! reads the markers. That agnosticism is the universality: same as a JPEG
//! not knowing it's a cat or a CT scan.
//!
//! Grok Souls and other `.fafm` memory artifacts are a *different format*
//! (Memory, not Context) and are not scored here at all — by spec, memory is
//! not graded.
//!
//! Canonical with `~/FAF/cli/src/core/slots.ts` (.faf-33 / Mk4) and
//! `tiers.ts` (Trophy 🏆 · ★ ◆ ◇ ● ● ○ ♡ — no medal emoji).

use serde_yaml_ng::Value;

/// The three technical states of a FAF slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    /// Missing or placeholder — counts against the score.
    Empty,
    /// Valid, project-specific data.
    Populated,
    /// Explicitly marked not-applicable — excluded from the active denominator.
    Slotignored,
}

/// The total number of FAF slots. The kernel always scores against all 33;
/// "21-base" files simply carry the 12 enterprise slots as `slotignored`.
pub const TOTAL_SLOTS: u32 = 33;

/// The result of an Mk4 scoring run.
#[derive(Debug, Clone)]
pub struct Mk4Result {
    /// 0–100.
    pub score: u32,
    /// Canonical tier name: TROPHY, GOLD, SILVER, BRONZE, GREEN, YELLOW, RED, WHITE.
    pub tier: String,
    /// Slots filled with valid, project-specific data (the numerator).
    pub populated: u32,
    /// Slots explicitly marked not-applicable (excluded from the denominator).
    pub ignored: u32,
    /// Active denominator: `total − ignored`.
    pub active: u32,
    /// Always 33.
    pub total: u32,
    /// Per-slot breakdown: slot name → its [`SlotState`].
    pub slots: Vec<(String, SlotState)>,
}

impl Mk4Result {
    /// Export as a JSON string (stable shape across all FAF engines).
    pub fn to_json(&self) -> String {
        let mut slots_json = String::from("{");
        for (i, (name, state)) in self.slots.iter().enumerate() {
            if i > 0 {
                slots_json.push(',');
            }
            let state_str = match state {
                SlotState::Populated => "populated",
                SlotState::Empty => "empty",
                SlotState::Slotignored => "slotignored",
            };
            slots_json.push_str(&format!("\"{}\":\"{}\"", name, state_str));
        }
        slots_json.push('}');

        format!(
            r#"{{"score":{},"tier":"{}","populated":{},"empty":{},"ignored":{},"active":{},"total":{},"slots":{}}}"#,
            self.score,
            self.tier,
            self.populated,
            self.total - self.populated - self.ignored,
            self.ignored,
            self.active,
            self.total,
            slots_json
        )
    }
}

/// Score a `.faf` YAML document against the 33-slot model.
pub fn score(yaml: &str) -> Result<Mk4Result, String> {
    Mk4Scorer::new().calculate(yaml)
}

/// The Mk4 scoring engine.
#[derive(Debug, Default)]
pub struct Mk4Scorer;

impl Mk4Scorer {
    /// Create a new scorer.
    pub fn new() -> Self {
        Self
    }

    /// Calculate the official FAF score from YAML content.
    pub fn calculate(&self, yaml: &str) -> Result<Mk4Result, String> {
        let doc: Value =
            serde_yaml_ng::from_str(yaml).map_err(|e| format!("YAML parse error: {}", e))?;

        let mut populated: u32 = 0;
        let mut ignored: u32 = 0;

        let mut slots: Vec<(String, SlotState)> = Vec::with_capacity(SLOTS.len());
        for slot_path in SLOTS {
            let state = slot_state(&doc, slot_path);
            match state {
                SlotState::Populated => populated += 1,
                SlotState::Slotignored => ignored += 1,
                SlotState::Empty => (),
            }
            slots.push((slot_path.to_string(), state));
        }

        let active = TOTAL_SLOTS - ignored;
        let score = if active == 0 {
            0.0
        } else {
            (populated as f64 / active as f64) * 100.0
        };
        let score_rounded = score.round() as u32;

        Ok(Mk4Result {
            score: score_rounded,
            tier: tier_name(score_rounded).to_string(),
            populated,
            ignored,
            active,
            total: TOTAL_SLOTS,
            slots,
        })
    }
}

/// The Universal DNA Map — the 33 canonical Mk4 slot paths, in order.
/// The 6 renamed slots (framework/css/state/api/db/pkg_manager) accept their
/// legacy aliases on read; see `legacy_alias_for`.
const SLOTS: &[&str] = &[
    // Project Meta (3)
    "project.name",
    "project.goal",
    "project.main_language",
    // Human Context (6)
    "human_context.who",
    "human_context.what",
    "human_context.why",
    "human_context.where",
    "human_context.when",
    "human_context.how",
    // Frontend Stack (4)
    "stack.framework",
    "stack.css",
    "stack.ui_library",
    "stack.state",
    // Backend Stack (5)
    "stack.backend",
    "stack.api",
    "stack.runtime",
    "stack.db",
    "stack.connection",
    // Universal Stack (3)
    "stack.hosting",
    "stack.build",
    "stack.cicd",
    // Enterprise Infra (5)
    "stack.monorepo_tool",
    "stack.pkg_manager",
    "stack.workspaces",
    "monorepo.packages_count",
    "monorepo.build_orchestrator",
    // Enterprise App (4)
    "stack.admin",
    "stack.cache",
    "stack.search",
    "stack.storage",
    // Enterprise Ops (3)
    "monorepo.versioning_strategy",
    "monorepo.shared_configs",
    "monorepo.remote_cache",
];

/// Legacy alias for a Mk4 canonical slot path — backward compat so existing
/// .faf files (with legacy keys) keep scoring correctly.
fn legacy_alias_for(canonical: &str) -> Option<&'static str> {
    match canonical {
        "stack.framework" => Some("stack.frontend"),
        "stack.css" => Some("stack.css_framework"),
        "stack.state" => Some("stack.state_management"),
        "stack.api" => Some("stack.api_type"),
        "stack.db" => Some("stack.database"),
        "stack.pkg_manager" => Some("stack.package_manager"),
        _ => None,
    }
}

/// Determine the state of a specific slot — canonical path first, legacy
/// alias fallback.
fn slot_state(doc: &Value, path: &str) -> SlotState {
    let state = walk_path_state(doc, path);
    if matches!(state, SlotState::Empty) {
        if let Some(legacy) = legacy_alias_for(path) {
            return walk_path_state(doc, legacy);
        }
    }
    state
}

/// Walk a dotted path in the YAML doc and classify the value's state.
fn walk_path_state(doc: &Value, path: &str) -> SlotState {
    let mut current = doc;
    for part in path.split('.') {
        if let Some(next) = current.get(Value::String(part.to_string())) {
            current = next;
        } else {
            return SlotState::Empty;
        }
    }

    match current {
        Value::String(s) => {
            let s = s.trim();
            if s == "slotignored" {
                SlotState::Slotignored
            } else if is_valid_populated_string(s) {
                SlotState::Populated
            } else {
                SlotState::Empty
            }
        }
        Value::Number(_) | Value::Bool(_) => SlotState::Populated,
        Value::Sequence(seq) => {
            if !seq.is_empty() {
                SlotState::Populated
            } else {
                SlotState::Empty
            }
        }
        Value::Mapping(map) => {
            if !map.is_empty() {
                SlotState::Populated
            } else {
                SlotState::Empty
            }
        }
        _ => SlotState::Empty,
    }
}

/// Rule 1: Placeholder Rejection (Honest Truth).
fn is_valid_populated_string(s: &str) -> bool {
    let placeholders = [
        "describe your project goal",
        "development teams",
        "cloud platform",
        "null",
        "none",
        "unknown",
        "n/a",
        "not applicable",
    ];

    !s.is_empty() && !placeholders.contains(&s.to_lowercase().as_str())
}

/// Canonical tier name for a score — source of truth: tiers.ts.
pub fn tier_name(score: u32) -> &'static str {
    if score >= 100 {
        "TROPHY"
    } else if score >= 99 {
        "GOLD"
    } else if score >= 95 {
        "SILVER"
    } else if score >= 85 {
        "BRONZE"
    } else if score >= 70 {
        "GREEN"
    } else if score >= 55 {
        "YELLOW"
    } else if score >= 1 {
        "RED"
    } else {
        "WHITE"
    }
}

/// Canonical tier symbol — Trophy 🏆 is the ONLY emoji; sub-Trophy tiers use
/// clean Unicode geometric symbols. The medal-emoji ladder is history.
pub fn tier_symbol(score: u32) -> &'static str {
    if score >= 100 {
        "🏆"
    } else if score >= 99 {
        "★"
    } else if score >= 95 {
        "◆"
    } else if score >= 85 {
        "◇"
    } else if score >= 55 {
        // GREEN (≥70) and YELLOW (≥55) share the ● glyph; the CLI
        // differentiates them by color (bold vs dim), which the kernel omits.
        "●"
    } else if score >= 1 {
        "○"
    } else {
        "♡"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cli-shaped file: of the 33 slots, the 21 non-cli ones are slotignored,
    /// leaving 12 active — all populated → Trophy. This is the always-33 model:
    /// the universe is fixed; the file's markers set what's active.
    const CLI_TROPHY: &str = r#"
project:
  name: my-cli
  goal: Ship fast
  main_language: Rust
human_context:
  who: Devs
  what: CLI tool
  why: Speed
  where: crates.io
  when: Now
  how: Cargo
stack:
  framework: slotignored
  css: slotignored
  ui_library: slotignored
  state: slotignored
  backend: slotignored
  api: slotignored
  runtime: slotignored
  db: slotignored
  connection: slotignored
  hosting: GitHub
  build: cargo
  cicd: GitHub Actions
  monorepo_tool: slotignored
  pkg_manager: slotignored
  workspaces: slotignored
  admin: slotignored
  cache: slotignored
  search: slotignored
  storage: slotignored
monorepo:
  packages_count: slotignored
  build_orchestrator: slotignored
  versioning_strategy: slotignored
  shared_configs: slotignored
  remote_cache: slotignored
"#;

    #[test]
    fn empty_yaml_scores_zero_against_33() {
        let result = score("empty: true").unwrap();
        assert_eq!(result.score, 0);
        assert_eq!(result.populated, 0);
        assert_eq!(result.total, 33);
        assert_eq!(result.active, 33); // nothing slotignored
        assert_eq!(result.tier, "WHITE");
    }

    #[test]
    fn invalid_yaml_returns_error() {
        assert!(score("invalid: yaml: [").is_err());
    }

    #[test]
    fn always_scores_against_33() {
        // No app_type anywhere; the kernel never branches on one.
        for yaml in [
            "project:\n  name: x\n",
            "app_type: enterprise\nproject:\n  name: y\n",
        ] {
            let result = score(yaml).unwrap();
            assert_eq!(result.total, 33);
            assert_eq!(result.slots.len(), 33);
        }
    }

    #[test]
    fn slotignored_sets_the_active_denominator() {
        // 12 active (21 slotignored), all populated → Trophy. The kernel needs
        // no idea this is a "cli" — the markers do all the work.
        let result = score(CLI_TROPHY).unwrap();
        assert_eq!(result.ignored, 21);
        assert_eq!(result.active, 12);
        assert_eq!(result.populated, 12);
        assert_eq!(result.score, 100);
        assert_eq!(result.tier, "TROPHY");
    }

    #[test]
    fn missing_slot_counts_against_score_unlike_slotignored() {
        // Same as CLI_TROPHY but with one active slot (commands/build) dropped:
        // it's now Empty (missing), not Slotignored — so it counts in the
        // denominator and the score is honest about the gap.
        let yaml = CLI_TROPHY.replace("  build: cargo\n", "");
        let result = score(&yaml).unwrap();
        assert_eq!(result.ignored, 21);
        assert_eq!(result.active, 12); // still 12 active (build is empty, not ignored)
        assert_eq!(result.populated, 11);
        assert!(result.score < 100); // honest: 11/12
    }

    #[test]
    fn legacy_aliases_score() {
        let yaml = r#"
project:
  name: x
stack:
  frontend: Svelte
  css_framework: Tailwind
  state_management: Stores
  api_type: REST
  database: Postgres
  package_manager: pnpm
"#;
        let result = score(yaml).unwrap();
        let populated: Vec<&str> = result
            .slots
            .iter()
            .filter(|(_, s)| *s == SlotState::Populated)
            .map(|(n, _)| n.as_str())
            .collect();
        assert!(populated.contains(&"stack.framework"));
        assert!(populated.contains(&"stack.css"));
        assert!(populated.contains(&"stack.state"));
        assert!(populated.contains(&"stack.api"));
        assert!(populated.contains(&"stack.db"));
        assert!(populated.contains(&"stack.pkg_manager"));
    }

    #[test]
    fn placeholders_rejected() {
        let yaml = "project:\n  name: unknown\n  goal: n/a\n";
        let result = score(yaml).unwrap();
        assert_eq!(result.populated, 0);
    }

    #[test]
    fn tier_ladder_canonical() {
        assert_eq!(tier_name(100), "TROPHY");
        assert_eq!(tier_name(99), "GOLD");
        assert_eq!(tier_name(95), "SILVER");
        assert_eq!(tier_name(85), "BRONZE");
        assert_eq!(tier_name(70), "GREEN");
        assert_eq!(tier_name(55), "YELLOW");
        assert_eq!(tier_name(1), "RED");
        assert_eq!(tier_name(0), "WHITE");
        // Trophy is the ONLY emoji; the medal ladder is history.
        assert_eq!(tier_symbol(100), "🏆");
        assert_eq!(tier_symbol(99), "★");
        assert_eq!(tier_symbol(95), "◆");
        assert_eq!(tier_symbol(85), "◇");
        assert_eq!(tier_symbol(0), "♡");
    }

    #[test]
    fn full_33_all_populated_is_trophy() {
        // Every slot populated, nothing ignored → 33/33 = Trophy.
        let mut yaml = String::from("project:\n  name: x\n  goal: y\n  main_language: Rust\n");
        yaml.push_str("human_context:\n");
        for w in ["who", "what", "why", "where", "when", "how"] {
            yaml.push_str(&format!("  {}: v\n", w));
        }
        yaml.push_str("stack:\n");
        for s in [
            "framework",
            "css",
            "ui_library",
            "state",
            "backend",
            "api",
            "runtime",
            "db",
            "connection",
            "hosting",
            "build",
            "cicd",
            "monorepo_tool",
            "pkg_manager",
            "workspaces",
            "admin",
            "cache",
            "search",
            "storage",
        ] {
            yaml.push_str(&format!("  {}: v\n", s));
        }
        yaml.push_str("monorepo:\n");
        for m in [
            "packages_count",
            "build_orchestrator",
            "versioning_strategy",
            "shared_configs",
            "remote_cache",
        ] {
            yaml.push_str(&format!("  {}: v\n", m));
        }
        let result = score(&yaml).unwrap();
        assert_eq!(result.populated, 33);
        assert_eq!(result.ignored, 0);
        assert_eq!(result.active, 33);
        assert_eq!(result.score, 100);
        assert_eq!(result.tier, "TROPHY");
    }

    #[test]
    fn to_json_shape() {
        let json = score("project:\n  name: x\n").unwrap().to_json();
        assert!(json.contains("\"score\":"));
        assert!(json.contains("\"total\":33"));
        assert!(json.contains("\"tier\":\"RED\""));
        assert!(json.contains("\"project.name\":\"populated\""));
    }
}
