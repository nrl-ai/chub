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
    GeminiMd,
    ClineRules,
    RooRules,
    AugmentRules,
    KiroSteering,
}

impl Target {
    pub fn parse_target(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude.md" | "claudemd" => Some(Target::ClaudeMd),
            "cursorrules" | ".cursorrules" => Some(Target::CursorRules),
            "windsurfrules" | ".windsurfrules" => Some(Target::WindsurfRules),
            "agents.md" | "agentsmd" => Some(Target::AgentsMd),
            "copilot" | "copilot-instructions" => Some(Target::Copilot),
            "gemini.md" | "geminimd" => Some(Target::GeminiMd),
            "clinerules" | ".clinerules" => Some(Target::ClineRules),
            "roorules" | "roo-rules" => Some(Target::RooRules),
            "augmentrules" | "augment-rules" => Some(Target::AugmentRules),
            "kiro" | "kiro-steering" => Some(Target::KiroSteering),
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
            Target::GeminiMd => "GEMINI.md",
            Target::ClineRules => ".clinerules",
            Target::RooRules => ".roo/rules/chub-rules.md",
            Target::AugmentRules => ".augment/rules/chub-rules.md",
            Target::KiroSteering => ".kiro/steering/chub-rules.md",
        }
    }

    /// Returns all known target names for documentation and help text.
    pub fn all_target_names() -> &'static [&'static str] {
        &[
            "claude.md",
            "cursorrules",
            "windsurfrules",
            "agents.md",
            "copilot",
            "gemini.md",
            "clinerules",
            "roorules",
            "augmentrules",
            "kiro",
        ]
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

    // Annotation policy
    if rules.include_annotation_policy {
        output.push_str("## Annotation Policy\n\n");
        output.push_str(
            "**IMPORTANT**: Before implementing anything with a pinned library, ALWAYS check \
             for existing annotations first via `chub_annotate` (id only) or `chub get <id>`. \
             Annotations contain team-verified issues, workarounds, and conventions that save \
             you from repeating known mistakes.\n\n",
        );
        output.push_str(
            "When you encounter something non-obvious while using a library, record it \
             via the `chub_annotate` MCP tool (or `chub annotate` CLI):\n\n",
        );
        output.push_str(
            "- **Issue** (confirmed bug): `chub_annotate` with `id`, `kind=\"issue\"`, `severity=\"high|medium|low\"`, `note`\n",
        );
        output.push_str(
            "- **Fix** (workaround): `chub_annotate` with `id`, `kind=\"fix\"`, `note`\n",
        );
        output.push_str(
            "- **Practice** (team convention): `chub_annotate` with `id`, `kind=\"practice\"`, `note`\n\n",
        );
        output.push_str("Rules:\n");
        output.push_str("- **Read before implementing** — always check existing annotations before using a library to avoid known pitfalls\n");
        output.push_str("- **Write after discovering** — if you hit a non-obvious issue, found a workaround, or established a working pattern, annotate it for the team\n");
        output.push_str("- Annotate after confirming, not speculatively — only write what you have verified works or fails\n");
        output
            .push_str("- One fact per annotation — do not bundle multiple issues into one note\n");
        output.push_str(
            "- Be reproducible — include the exact call, param, or value, not vague descriptions\n",
        );
        output.push_str("- Check first — call `chub_annotate` with only `id` to read existing annotations before writing to avoid duplicates\n");
        output.push_str("- Do not annotate what is already in the official docs — only capture what the docs missed or got wrong\n\n");
    }

    // AI usage tracking
    if rules.include_annotation_policy {
        output.push_str("## AI Usage Tracking\n\n");
        output.push_str(
            "This project tracks AI coding sessions via `chub track`. Use `chub_track` MCP tool to:\n",
        );
        output.push_str("- `{\"action\": \"status\"}` — check current session state\n");
        output
            .push_str("- `{\"action\": \"report\"}` — view usage summary (tokens, costs, tools)\n");
        output.push_str("- `{\"action\": \"log\"}` — list recent sessions\n\n");
    }

    // Module rules — sort by name for deterministic output (HashMap iteration is unordered).
    let mut sorted_modules: Vec<_> = rules.modules.iter().collect();
    sorted_modules.sort_by_key(|(name, _)| name.as_str());
    for (module_name, module_rules) in sorted_modules {
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
    Unknown,
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
            None => {
                results.push(SyncResult {
                    target: target_name.clone(),
                    filename: target_name.clone(),
                    action: SyncAction::Unknown,
                });
                continue;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_known_targets() {
        for name in Target::all_target_names() {
            assert!(
                Target::parse_target(name).is_some(),
                "all_target_names entry '{}' should parse",
                name
            );
        }
    }

    #[test]
    fn parse_target_aliases() {
        let cases = [
            ("claude.md", "CLAUDE.md"),
            ("claudemd", "CLAUDE.md"),
            (".cursorrules", ".cursorrules"),
            ("cursorrules", ".cursorrules"),
            (".windsurfrules", ".windsurfrules"),
            ("agents.md", "AGENTS.md"),
            ("agentsmd", "AGENTS.md"),
            ("copilot", ".github/copilot-instructions.md"),
            ("copilot-instructions", ".github/copilot-instructions.md"),
            ("gemini.md", "GEMINI.md"),
            ("geminimd", "GEMINI.md"),
            (".clinerules", ".clinerules"),
            ("clinerules", ".clinerules"),
            ("roorules", ".roo/rules/chub-rules.md"),
            ("roo-rules", ".roo/rules/chub-rules.md"),
            ("augmentrules", ".augment/rules/chub-rules.md"),
            ("augment-rules", ".augment/rules/chub-rules.md"),
            ("kiro", ".kiro/steering/chub-rules.md"),
            ("kiro-steering", ".kiro/steering/chub-rules.md"),
        ];
        for (input, expected_file) in cases {
            let target =
                Target::parse_target(input).unwrap_or_else(|| panic!("'{}' should parse", input));
            assert_eq!(target.filename(), expected_file, "input: '{}'", input);
        }
    }

    #[test]
    fn parse_unknown_target_returns_none() {
        assert!(Target::parse_target("vim").is_none());
        assert!(Target::parse_target("").is_none());
        assert!(Target::parse_target("zed").is_none());
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert!(Target::parse_target("CLAUDE.MD").is_some());
        assert!(Target::parse_target("CursorRules").is_some());
        assert!(Target::parse_target("GEMINI.MD").is_some());
        assert!(Target::parse_target("KIRO").is_some());
    }

    #[test]
    fn generate_config_includes_global_rules() {
        let rules = AgentRules {
            global: vec!["Run tests".to_string(), "Format code".to_string()],
            modules: Default::default(),
            targets: vec![],
            include_pins: false,
            include_context: false,
            include_annotation_policy: false,
        };
        let output = generate_config(&rules);
        assert!(output.contains("- Run tests"));
        assert!(output.contains("- Format code"));
        assert!(output.starts_with("# Project Rules"));
    }
}
