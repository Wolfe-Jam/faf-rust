//! Type definitions for the FAF format.
//!
//! These structs are the in-memory shape of a parsed `.faf` file. Field names
//! map directly to the YAML keys; most are `Option` because `.faf` is
//! progressive — a project fills in as much context as it has, and the score
//! reflects how complete that picture is.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete parsed `.faf` file — the root of the context document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FafData {
    /// The `.faf` format version this file was written against (e.g. `"2.5.0"`).
    pub faf_version: String,
    /// Core project identity (name, goal, language). Always present.
    pub project: Project,

    /// Cached AI-readiness score, if one was written by a generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_score: Option<String>,

    /// Cached confidence band for the score, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_confidence: Option<String>,

    /// Short, AI-facing summary lines keyed by topic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_tldr: Option<HashMap<String, String>>,

    /// The fast-path context an assistant reads first (what/stack/files).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instant_context: Option<InstantContext>,

    /// Self-reported completeness metrics for the context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_quality: Option<ContextQuality>,

    /// The technical stack (frontend, backend, database, build, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<Stack>,

    /// The human side — the 6 W's (who/what/why/how/where/when).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_context: Option<HumanContext>,

    /// Working preferences (quality bar, testing, docs, code style).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Preferences>,

    /// Where the project is right now (phase, version, focus, milestones).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,

    /// Free-form classification tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Core project identity — the one block every `.faf` file has.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// The project's name.
    pub name: String,

    /// One sentence on what the project is for — the seed for the 6 W's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,

    /// Primary implementation language (e.g. `"Rust"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_language: Option<String>,

    /// How the project is built or approached (architecture, methodology).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approach: Option<String>,

    /// The project's own version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// SPDX license identifier (e.g. `"MIT"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// The fast-path context an AI assistant reads first.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantContext {
    /// A one-line description of what is being built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_building: Option<String>,

    /// The stack in brief, as a single human-readable line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_stack: Option<String>,

    /// How and where the project is deployed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment: Option<String>,

    /// The files most worth reading first to understand the project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_files: Vec<String>,

    /// Named commands (e.g. `build`, `test`) → their shell invocations.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub commands: HashMap<String, String>,
}

/// The technical stack — one field per layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    /// Frontend framework or library.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend: Option<String>,

    /// Backend framework or runtime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    /// Database or data store.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,

    /// Hosting / infrastructure platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infrastructure: Option<String>,

    /// Build tool or bundler.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_tool: Option<String>,

    /// Test framework or runner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing: Option<String>,

    /// CI/CD platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cicd: Option<String>,
}

/// Self-reported metrics on how complete the context is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuality {
    /// How many of the 33 slots are filled, as reported by the generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slots_filled: Option<String>,

    /// Confidence band for the reported completeness.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,

    /// Whether the context is considered ready to hand off to an AI.
    #[serde(default)]
    pub handoff_ready: bool,

    /// Slots the author knows are still missing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_context: Vec<String>,
}

/// The human side of the project — the 6 W's.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanContext {
    /// Who the project is for / who works on it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,

    /// What the project is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,

    /// Why it exists. Serialized as `why` (`why_field` avoids the keyword).
    #[serde(rename = "why", skip_serializing_if = "Option::is_none")]
    pub why_field: Option<String>,

    /// How it works or is built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how: Option<String>,

    /// Where it runs / lives. Serialized as `where`.
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_field: Option<String>,

    /// When — timeline, stage, or cadence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

/// Working preferences that shape how the project should be developed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// The quality bar to hold (e.g. zero-errors, championship-grade).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_bar: Option<String>,

    /// Testing expectations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing: Option<String>,

    /// Documentation expectations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,

    /// Code-style conventions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_style: Option<String>,
}

/// Where the project is right now.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Current phase (e.g. `"production"`, `"prototype"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,

    /// Version associated with the current state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// What the work is focused on right now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,

    /// Recent or upcoming milestones.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub milestones: Vec<String>,
}
