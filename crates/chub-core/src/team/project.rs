use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::SourceConfig;
use crate::error::{Error, Result};

/// Search upward from CWD (or a given path) for a `.chub/` directory.
pub fn find_project_root(start: Option<&Path>) -> Option<PathBuf> {
    let start = start
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let mut current = start.as_path();
    loop {
        let candidate = current.join(".chub");
        if candidate.is_dir() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

/// Get the `.chub/` directory for the current project (if any).
pub fn project_chub_dir() -> Option<PathBuf> {
    find_project_root(None).map(|root| root.join(".chub"))
}

/// Project-level config that extends the global config.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectFileConfig {
    #[serde(default)]
    pub sources: Option<Vec<SourceConfig>>,
    #[serde(default)]
    pub cdn_url: Option<String>,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub refresh_interval: Option<u64>,
    #[serde(default)]
    pub output_format: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub telemetry: Option<bool>,
    #[serde(default)]
    pub feedback: Option<bool>,
    #[serde(default)]
    pub telemetry_url: Option<String>,
    #[serde(default)]
    pub agent_rules: Option<AgentRules>,
    #[serde(default)]
    pub auto_profile: Option<Vec<AutoProfileEntry>>,
}

/// Agent rules configuration for generating CLAUDE.md, .cursorrules, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentRules {
    #[serde(default)]
    pub global: Vec<String>,
    #[serde(default)]
    pub modules: std::collections::HashMap<String, ModuleRules>,
    #[serde(default)]
    pub include_pins: bool,
    #[serde(default)]
    pub include_context: bool,
    #[serde(default)]
    pub targets: Vec<String>,
}

/// Rules scoped to a module/path pattern.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleRules {
    pub path: String,
    #[serde(default)]
    pub rules: Vec<String>,
}

/// Auto-profile entry: maps a path glob to a profile name.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutoProfileEntry {
    pub path: String,
    pub profile: String,
}

/// Load the project-level config from `.chub/config.yaml`.
pub fn load_project_config() -> Option<ProjectFileConfig> {
    let chub_dir = project_chub_dir()?;
    let config_path = chub_dir.join("config.yaml");
    let raw = fs::read_to_string(&config_path).ok()?;
    serde_yaml::from_str(&raw).ok()
}

/// Initialize a `.chub/` directory in the current working directory.
pub fn init_project(from_deps: bool, monorepo: bool) -> Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(|e| Error::Config(e.to_string()))?;
    let chub_dir = cwd.join(".chub");

    if chub_dir.exists() {
        return Err(Error::Config(format!(
            ".chub/ directory already exists at {}",
            cwd.display()
        )));
    }

    fs::create_dir_all(&chub_dir)?;
    fs::create_dir_all(chub_dir.join("annotations"))?;
    fs::create_dir_all(chub_dir.join("context"))?;
    fs::create_dir_all(chub_dir.join("profiles"))?;

    // Write default config.yaml
    let config_content = r#"# Chub project configuration
# This file is shared with the team via git.
# It overrides personal settings in ~/.chub/config.yaml.

# sources:
#   - name: official
#     url: https://cdn.aichub.org/v1
#   - name: company
#     url: https://docs.internal.company.com/chub

# Agent rules for generating CLAUDE.md, .cursorrules, etc.
# agent_rules:
#   global:
#     - "Follow the project coding conventions"
#   modules: {}
#   include_pins: true
#   include_context: true
#   targets:
#     - claude.md
"#;
    fs::write(chub_dir.join("config.yaml"), config_content)?;

    // Write empty pins.yaml
    fs::write(chub_dir.join("pins.yaml"), "pins: []\n")?;

    // Write base profile
    let base_profile = r#"name: Base
description: "Shared rules for all roles"
rules: []
context: []
"#;
    fs::write(chub_dir.join("profiles").join("base.yaml"), base_profile)?;

    // Write example context doc
    let example_context = r#"---
name: Project Architecture
description: "High-level architecture overview"
tags: architecture
---

# Architecture Overview

Describe your project architecture here.
"#;
    fs::write(
        chub_dir.join("context").join("architecture.md"),
        example_context,
    )?;

    // Write .gitignore for .chub/
    // Nothing to ignore by default — everything is git-tracked
    fs::write(chub_dir.join(".gitignore"), "# .chub/ is git-tracked\n")?;

    if monorepo {
        // For monorepo, also create auto_profile example in config
        let monorepo_config = r#"# Chub project configuration (monorepo)

# auto_profile:
#   - path: "packages/api/**"
#     profile: backend
#   - path: "packages/web/**"
#     profile: frontend

agent_rules:
  global:
    - "Follow the project coding conventions"
  modules: {}
  include_pins: true
  include_context: true
  targets:
    - claude.md
"#;
        fs::write(chub_dir.join("config.yaml"), monorepo_config)?;
    }

    if from_deps {
        // Will be handled by detect module after init
        // Just create a marker so the caller knows to run detect
        fs::write(chub_dir.join(".init_from_deps"), "")?;
    }

    Ok(chub_dir)
}
