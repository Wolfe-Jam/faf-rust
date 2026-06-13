//! Mk4 Championship Engine — 33-slot scoring.
//!
//! Philosophy: every slot is Populated, Empty, or Slotignored.
//! Score = populated ÷ active, where active = universe − slotignored.
//!
//! The slot universe is derived from the document's `app_type` (falling back
//! to `project.type`) — enterprise-shaped types (`enterprise`, `saas`,
//! `mcpaas`, `monorepo-root`) score against all 33 slots; everything else
//! scores against the base 21. `slotignored` markers written at generation
//! time reduce the active denominator. There is no license logic in the
//! kernel: the type of the project alone decides the universe.
//!
//! Canonical with `~/FAF/cli/src/core/slots.ts` (.faf-33 / Mk4) and
//! `tiers.ts` (Trophy 🏆 · ★ ◆ ◇ ● ● ○ ♡ — no medal emoji).

use serde_yaml_ng::Value;

/// The three technical states of a FAF slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    /// Missing or placeholder.
    Empty,
    /// Valid, project-specific data.
    Populated,
    /// Explicitly marked not-applicable for this app type.
    Slotignored,
}

/// The slot universe a document scores against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Universe {
    /// Base 21 slots.
    Base21,
    /// Full 33 slots (enterprise-shaped app types).
    Full33,
}

/// App types whose category set includes enterprise slots (per
/// APP_TYPE_CATEGORIES in slots.ts): these score against the full 33.
const ENTERPRISE_APP_TYPES: &[&str] = &["enterprise", "saas", "mcpaas", "monorepo-root"];

impl Universe {
    /// Derive the universe from an app type string.
    pub fn from_app_type(app_type: Option<&str>) -> Self {
        match app_type {
            Some(t) if ENTERPRISE_APP_TYPES.contains(&t.trim()) => Universe::Full33,
            _ => Universe::Base21,
        }
    }

    /// Total slot count for this universe.
    pub const fn total(&self) -> u32 {
        match self {
            Universe::Base21 => 21,
            Universe::Full33 => 33,
        }
    }
}

/// The result of an Mk4 scoring run.
#[derive(Debug, Clone)]
pub struct Mk4Result {
    /// 0–100.
    pub score: u32,
    /// Canonical tier name: TROPHY, GOLD, SILVER, BRONZE, GREEN, YELLOW, RED, WHITE.
    pub tier: String,
    pub populated: u32,
    pub ignored: u32,
    pub active: u32,
    pub total: u32,
    pub universe: Universe,
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

/// Score a `.faf` YAML document. The universe is derived from the document
/// itself (`app_type`, falling back to `project.type`).
pub fn score(yaml: &str) -> Result<Mk4Result, String> {
    Mk4Scorer::new().calculate(yaml)
}

/// The Mk4 scoring engine.
#[derive(Debug, Default)]
pub struct Mk4Scorer;

impl Mk4Scorer {
    pub fn new() -> Self {
        Self
    }

    /// Calculate the official FAF score from YAML content.
    pub fn calculate(&self, yaml: &str) -> Result<Mk4Result, String> {
        let doc: Value =
            serde_yaml_ng::from_str(yaml).map_err(|e| format!("YAML parse error: {}", e))?;

        let universe = Universe::from_app_type(document_app_type(&doc).as_deref());
        self.calculate_in_universe(&doc, universe)
    }

    /// Calculate against an explicit universe (for callers that already know).
    pub fn calculate_with_universe(
        &self,
        yaml: &str,
        universe: Universe,
    ) -> Result<Mk4Result, String> {
        let doc: Value =
            serde_yaml_ng::from_str(yaml).map_err(|e| format!("YAML parse error: {}", e))?;
        self.calculate_in_universe(&doc, universe)
    }

    fn calculate_in_universe(&self, doc: &Value, universe: Universe) -> Result<Mk4Result, String> {
        let mut populated: u32 = 0;
        let mut ignored: u32 = 0;

        let slot_paths = universal_slots(universe);
        let mut slots: Vec<(String, SlotState)> = Vec::with_capacity(slot_paths.len());

        for slot_path in &slot_paths {
            let state = slot_state(doc, slot_path);
            match state {
                SlotState::Populated => populated += 1,
                SlotState::Slotignored => ignored += 1,
                SlotState::Empty => (),
            }
            slots.push((slot_path.to_string(), state));
        }

        let total_slots = universe.total();
        let active_slots = total_slots - ignored;

        let score = if active_slots == 0 {
            0.0
        } else {
            (populated as f64 / active_slots as f64) * 100.0
        };
        let score_rounded = score.round() as u32;

        Ok(Mk4Result {
            score: score_rounded,
            tier: tier_name(score_rounded).to_string(),
            populated,
            ignored,
            active: active_slots,
            total: total_slots,
            universe,
            slots,
        })
    }
}

/// Read the document's app type: top-level `app_type`, else `project.type`.
fn document_app_type(doc: &Value) -> Option<String> {
    let direct = doc
        .get(Value::String("app_type".to_string()))
        .and_then(|v| v.as_str());
    if let Some(t) = direct {
        return Some(t.to_string());
    }
    doc.get(Value::String("project".to_string()))
        .and_then(|p| p.get(Value::String("type".to_string())))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// The Universal DNA Map — Mk4 canonical slot paths.
/// The 6 renamed slots (framework/css/state/api/db/pkg_manager) accept
/// their legacy aliases on read; see `legacy_alias_for`.
fn universal_slots(universe: Universe) -> Vec<&'static str> {
    let mut slots = vec![
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
    ];

    if universe == Universe::Full33 {
        slots.extend([
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
        ]);
    }

    slots
}

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

    #[test]
    fn empty_yaml_scores_zero_base21() {
        let result = score("empty: true").unwrap();
        assert_eq!(result.score, 0);
        assert_eq!(result.populated, 0);
        assert_eq!(result.total, 21);
        assert_eq!(result.tier, "WHITE");
    }

    #[test]
    fn invalid_yaml_returns_error() {
        assert!(score("invalid: yaml: [").is_err());
    }

    #[test]
    fn universe_from_app_type() {
        assert_eq!(Universe::from_app_type(Some("cli")), Universe::Base21);
        assert_eq!(Universe::from_app_type(Some("fullstack")), Universe::Base21);
        assert_eq!(Universe::from_app_type(None), Universe::Base21);
        assert_eq!(
            Universe::from_app_type(Some("enterprise")),
            Universe::Full33
        );
        assert_eq!(Universe::from_app_type(Some("saas")), Universe::Full33);
        assert_eq!(Universe::from_app_type(Some("mcpaas")), Universe::Full33);
        assert_eq!(
            Universe::from_app_type(Some("monorepo-root")),
            Universe::Full33
        );
    }

    #[test]
    fn app_type_read_from_project_type() {
        let yaml = "project:\n  name: x\n  type: enterprise\n";
        let result = score(yaml).unwrap();
        assert_eq!(result.total, 33);
    }

    #[test]
    fn top_level_app_type_wins() {
        let yaml = "app_type: saas\nproject:\n  name: x\n  type: cli\n";
        let result = score(yaml).unwrap();
        assert_eq!(result.total, 33);
    }

    #[test]
    fn slotignored_reduces_active() {
        // A cli-shaped file: 12 active slots (21 - 9 ignored), all populated.
        let yaml = r#"
project:
  name: my-cli
  goal: Ship fast
  main_language: Rust
  type: cli
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
"#;
        let result = score(yaml).unwrap();
        assert_eq!(result.ignored, 9);
        assert_eq!(result.active, 12);
        assert_eq!(result.populated, 12);
        assert_eq!(result.score, 100);
        assert_eq!(result.tier, "TROPHY");
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
    fn enterprise_universe_full33() {
        let yaml = "app_type: enterprise\nproject:\n  name: big\n";
        let result = score(yaml).unwrap();
        assert_eq!(result.total, 33);
        assert_eq!(result.slots.len(), 33);
    }

    #[test]
    fn to_json_shape() {
        let json = score("project:\n  name: x\n").unwrap().to_json();
        assert!(json.contains("\"score\":"));
        assert!(json.contains("\"tier\":\"RED\""));
        assert!(json.contains("\"project.name\":\"populated\""));
    }
}
