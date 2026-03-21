use std::fs;

use crate::error::{Error, Result};
use crate::team::context::list_context_docs;
use crate::team::pins::list_pins;
use crate::team::project::AgentRules;

/// Supported agent config targets.
#[derive(Debug, Clone)]
pub enum Target {
    ClaudeMd,
    CursorRules,
    WindsurfRules,
    AgentsMd,
    Copilot,
}

impl Target {
    pub fn parse_target(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude.md" | "claudemd" => Some(Target::ClaudeMd),
            "cursorrules" | ".cursorrules" => Some(Target::CursorRules),
            "windsurfrules" | ".windsurfrules" => Some(Target::WindsurfRules),
            "agents.md" | "agentsmd" => Some(Target::AgentsMd),
            "copilot" | "copilot-instructions" => Some(Target::Copilot),
            _ => None,
        }
    }

    pub fn filename(&self) -> &'static str {
        match self {
            Target::ClaudeMd => "CLAUDE.md",
            Target::CursorRules => ".cursorrules",
            Target::WindsurfRules => ".windsurfrules",
            Target::AgentsMd => "AGENTS.md",
            Target::Copilot => ".github/copilot-instructions.md",
        }
    }
}

/// Load agent rules from the project config.
pub fn load_agent_rules() -> Option<AgentRules> {
    let config = crate::team::project::load_project_config()?;
    config.agent_rules
}

/// Generate agent config content for a specific target.
pub fn generate_config(rules: &AgentRules) -> String {
    let mut output = String::new();

    // Header
    output.push_str("# Project Rules\n\n");

    // Global rules
    if !rules.global.is_empty() {
        for rule in &rules.global {
            output.push_str(&format!("- {}\n", rule));
        }
        output.push('\n');
    }

    // Pinned docs
    if rules.include_pins {
        let pins = list_pins();
        if !pins.is_empty() {
            output.push_str("## Pinned Documentation\n");
            output.push_str(
                "Use `chub get <id>` to fetch these docs when working with these libraries:\n",
            );
            for pin in &pins {
                let mut desc = format!("- {}", pin.id);
                if let Some(ref lang) = pin.lang {
                    desc.push_str(&format!(" ({})", lang));
                }
                if let Some(ref version) = pin.version {
                    desc.push_str(&format!(" v{}", version));
                }
                if let Some(ref reason) = pin.reason {
                    desc.push_str(&format!(" — {}", reason));
                }
                output.push_str(&desc);
                output.push('\n');
            }
            output.push('\n');
        }
    }

    // Project context
    if rules.include_context {
        let context_docs = list_context_docs();
        if !context_docs.is_empty() {
            output.push_str("## Project Context\n");
            output.push_str("Use `chub get project/<name>` or ask Chub for these:\n");
            for doc in &context_docs {
                let stem = doc.file.strip_suffix(".md").unwrap_or(&doc.file);
                let mut desc = format!("- project/{}", stem);
                if !doc.description.is_empty() {
                    desc.push_str(&format!(" — {}", doc.description));
                }
                output.push_str(&desc);
                output.push('\n');
            }
            output.push('\n');
        }
    }

    // Module rules
    for (module_name, module_rules) in &rules.modules {
        output.push_str(&format!(
            "## Module: {} ({})\n",
            module_name, module_rules.path
        ));
        for rule in &module_rules.rules {
            output.push_str(&format!("- {}\n", rule));
        }
        output.push('\n');
    }

    output
}

/// Result of a sync operation for one target.
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub target: String,
    pub filename: String,
    pub action: SyncAction,
}

#[derive(Debug, Clone)]
pub enum SyncAction {
    Created,
    Updated,
    Unchanged,
}

/// Generate and write all configured target files.
pub fn sync_configs() -> Result<Vec<SyncResult>> {
    let rules = load_agent_rules().ok_or_else(|| {
        Error::Config(
            "No agent_rules found in .chub/config.yaml. Add agent_rules section first.".to_string(),
        )
    })?;

    let project_root = crate::team::project::find_project_root(None).ok_or_else(|| {
        Error::Config("No .chub/ directory found. Run `chub init` first.".to_string())
    })?;

    let content = generate_config(&rules);
    let mut results = Vec::new();

    for target_name in &rules.targets {
        let target = match Target::parse_target(target_name) {
            Some(t) => t,
            None => continue,
        };

        let path = project_root.join(target.filename());

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let action = if path.exists() {
            let existing = fs::read_to_string(&path).unwrap_or_default();
            if existing == content {
                SyncAction::Unchanged
            } else {
                fs::write(&path, &content)?;
                SyncAction::Updated
            }
        } else {
            fs::write(&path, &content)?;
            SyncAction::Created
        };

        results.push(SyncResult {
            target: target_name.clone(),
            filename: target.filename().to_string(),
            action,
        });
    }

    Ok(results)
}

/// Show what would change without writing.
pub fn diff_configs() -> Result<Vec<(String, String, Option<String>)>> {
    let rules = load_agent_rules()
        .ok_or_else(|| Error::Config("No agent_rules found in .chub/config.yaml.".to_string()))?;

    let project_root = crate::team::project::find_project_root(None)
        .ok_or_else(|| Error::Config("No .chub/ directory found.".to_string()))?;

    let content = generate_config(&rules);
    let mut diffs = Vec::new();

    for target_name in &rules.targets {
        let target = match Target::parse_target(target_name) {
            Some(t) => t,
            None => continue,
        };

        let path = project_root.join(target.filename());
        let existing = if path.exists() {
            Some(fs::read_to_string(&path).unwrap_or_default())
        } else {
            None
        };

        diffs.push((target.filename().to_string(), content.clone(), existing));
    }

    Ok(diffs)
}
