//! Scan configuration — .gitleaks.toml / .betterleaks.toml compatible.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Top-level scan config (compatible with gitleaks/betterleaks TOML).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ScanConfig {
    pub title: Option<String>,
    #[serde(default)]
    pub extend: ExtendConfig,
    #[serde(default)]
    pub allowlist: GlobalAllowlist,
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ExtendConfig {
    /// Use default built-in rules.
    #[serde(default = "default_true")]
    pub use_default: bool,
    /// Path to a base config to extend.
    pub path: Option<String>,
    /// Rules to disable from the base config.
    #[serde(default)]
    pub disabled_rules: Vec<String>,
}

impl Default for ExtendConfig {
    fn default() -> Self {
        Self {
            use_default: true,
            path: None,
            disabled_rules: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GlobalAllowlist {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub regexes: Vec<String>,
    #[serde(default)]
    pub stopwords: Vec<String>,
    #[serde(default)]
    pub commits: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RuleConfig {
    pub id: String,
    pub description: Option<String>,
    pub regex: Option<String>,
    pub secret_group: Option<usize>,
    pub entropy: Option<f64>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub path: Option<String>,
    #[serde(default)]
    pub allowlists: Vec<RuleAllowlist>,
    /// Skip this rule in reports (used for composite-only rules like aws-secret-access-key).
    #[serde(default)]
    pub skip_report: bool,
    /// Apply BPE token-efficiency filter to findings from this rule.
    /// Mirrors betterleaks `tokenEfficiency` field.
    #[serde(rename = "tokenEfficiency", default)]
    pub token_efficiency: bool,
    /// CEL expression to validate detected secrets against a live API.
    /// Mirrors betterleaks `validate` field.
    #[serde(default)]
    pub validate: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RuleAllowlist {
    pub description: Option<String>,
    pub condition: Option<String>, // "OR" or "AND"
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub regexes: Vec<String>,
    #[serde(default)]
    pub stopwords: Vec<String>,
    pub regex_target: Option<String>, // "secret", "match", "line"
}

/// Maximum config file size (1 MB).
const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

impl ScanConfig {
    /// Load config from a file path.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(path)
            .map_err(|e| format!("Cannot read config {}: {}", path.display(), e))?;
        if meta.len() > MAX_CONFIG_SIZE {
            return Err(format!(
                "Config file {} too large ({} bytes, max {})",
                path.display(),
                meta.len(),
                MAX_CONFIG_SIZE
            ));
        }
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;

        // Support both TOML (.toml) and YAML (.yaml/.yml)
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("toml");
        match ext {
            "yaml" | "yml" => {
                serde_yaml::from_str(&raw).map_err(|e| format!("YAML parse error: {}", e))
            }
            _ => toml::from_str(&raw).map_err(|e| format!("TOML parse error: {}", e)),
        }
    }

    /// Find config file by searching standard locations.
    /// Priority: explicit path > env var > .betterleaks.toml > .gitleaks.toml in target dir.
    pub fn find_config(explicit_path: Option<&str>, target_dir: &Path) -> Option<PathBuf> {
        // 1. Explicit path
        if let Some(p) = explicit_path {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }

        // 2. Environment variable
        for var in ["CHUB_SCAN_CONFIG", "BETTERLEAKS_CONFIG", "GITLEAKS_CONFIG"] {
            if let Ok(p) = std::env::var(var) {
                let path = PathBuf::from(&p);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        // 3. Config files in target directory
        for name in [".chub-scan.toml", ".betterleaks.toml", ".gitleaks.toml"] {
            let path = target_dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_toml() {
        let raw = r#"
title = "test"

[[rules]]
id = "test-rule"
regex = '''test-[a-z]+'''
keywords = ["test"]
"#;
        let cfg: ScanConfig = toml::from_str(raw).unwrap();
        assert_eq!(cfg.title.unwrap(), "test");
        assert_eq!(cfg.rules.len(), 1);
        assert_eq!(cfg.rules[0].id, "test-rule");
    }

    #[test]
    fn parse_with_allowlist() {
        let raw = r#"
[allowlist]
description = "global"
paths = ['''test/.*''']
regexes = ['''EXAMPLE''']

[[rules]]
id = "test-rule"
regex = '''secret-[a-z]+'''
"#;
        let cfg: ScanConfig = toml::from_str(raw).unwrap();
        assert_eq!(cfg.allowlist.paths.len(), 1);
        assert_eq!(cfg.allowlist.regexes.len(), 1);
    }

    #[test]
    fn parse_rule_with_allowlists() {
        let raw = r#"
[[rules]]
id = "test-rule"
regex = '''secret-[a-z]+'''

[[rules.allowlists]]
description = "rule-specific"
condition = "OR"
regexes = ['''EXAMPLE''']
regexTarget = "secret"
"#;
        let cfg: ScanConfig = toml::from_str(raw).unwrap();
        assert_eq!(cfg.rules[0].allowlists.len(), 1);
        assert_eq!(
            cfg.rules[0].allowlists[0].regex_target.as_deref(),
            Some("secret")
        );
    }

    #[test]
    fn extend_defaults() {
        let cfg = ScanConfig::default();
        assert!(cfg.extend.use_default);
    }
}
