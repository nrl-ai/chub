use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_CDN_URL: &str = "https://cdn.aichub.org/v1";
const DEFAULT_TELEMETRY_URL: &str = "https://api.aichub.org/v1";

/// Maximum size for YAML config files (1 MB) to prevent denial-of-service via
/// anchor bombs or deeply nested structures.
const MAX_CONFIG_FILE_SIZE: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub sources: Vec<SourceConfig>,
    pub output_dir: String,
    pub refresh_interval: u64,
    pub output_format: String,
    pub source: String,
    pub telemetry: bool,
    pub feedback: bool,
    pub telemetry_url: String,
    pub annotation_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sources: vec![SourceConfig {
                name: "default".to_string(),
                url: Some(DEFAULT_CDN_URL.to_string()),
                path: None,
            }],
            output_dir: ".context".to_string(),
            refresh_interval: 21600,
            output_format: "human".to_string(),
            source: "official,maintainer,community".to_string(),
            telemetry: true,
            feedback: true,
            telemetry_url: DEFAULT_TELEMETRY_URL.to_string(),
            annotation_token: None,
        }
    }
}

/// Raw YAML config file structure.
#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    sources: Option<Vec<SourceConfig>>,
    #[serde(default)]
    cdn_url: Option<String>,
    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    refresh_interval: Option<u64>,
    #[serde(default)]
    output_format: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    telemetry: Option<bool>,
    #[serde(default)]
    feedback: Option<bool>,
    #[serde(default)]
    telemetry_url: Option<String>,
    #[serde(default)]
    annotation_token: Option<String>,
}

/// Get the chub data directory (~/.chub or CHUB_DIR env var).
pub fn chub_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CHUB_DIR") {
        PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".chub")
    }
}

/// Load config with three-tier merge:
/// Tier 1: ~/.chub/config.yaml (personal defaults)
/// Tier 2: .chub/config.yaml (project, overrides personal)
/// Tier 3: Active profile (additive — doesn't override config fields)
pub fn load_config() -> Config {
    let defaults = Config::default();
    let config_path = chub_dir().join("config.yaml");

    // Tier 1: personal config (with size limit)
    let file_config: FileConfig = read_config_file(&config_path).unwrap_or_default();

    // Tier 2: project config (override personal, with size limit)
    let project_config: FileConfig = find_project_chub_dir()
        .map(|d| d.join("config.yaml"))
        .and_then(|p| read_config_file(&p))
        .unwrap_or_default();

    // Merge: project overrides personal, personal overrides defaults
    let sources = project_config
        .sources
        .or(file_config.sources)
        .unwrap_or_else(|| {
            let raw_url = std::env::var("CHUB_BUNDLE_URL")
                .ok()
                .or(project_config.cdn_url)
                .or(file_config.cdn_url)
                .unwrap_or_else(|| DEFAULT_CDN_URL.to_string());
            // Validate the URL scheme (HTTPS required, HTTP only for localhost)
            let url = match crate::util::validate_url(&raw_url, "CHUB_BUNDLE_URL") {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("Warning: {}", e);
                    DEFAULT_CDN_URL.to_string()
                }
            };
            vec![SourceConfig {
                name: "default".to_string(),
                url: Some(url),
                path: None,
            }]
        });

    Config {
        sources,
        output_dir: project_config
            .output_dir
            .or(file_config.output_dir)
            .unwrap_or(defaults.output_dir),
        refresh_interval: project_config
            .refresh_interval
            .or(file_config.refresh_interval)
            .unwrap_or(defaults.refresh_interval),
        output_format: project_config
            .output_format
            .or(file_config.output_format)
            .unwrap_or(defaults.output_format),
        source: project_config
            .source
            .or(file_config.source)
            .unwrap_or(defaults.source),
        telemetry: project_config
            .telemetry
            .or(file_config.telemetry)
            .unwrap_or(defaults.telemetry),
        feedback: project_config
            .feedback
            .or(file_config.feedback)
            .unwrap_or(defaults.feedback),
        telemetry_url: project_config
            .telemetry_url
            .or(file_config.telemetry_url)
            .unwrap_or(defaults.telemetry_url),
        // annotation_token intentionally comes from personal config only —
        // never from project config to avoid accidental token commits.
        annotation_token: file_config.annotation_token,
    }
}

/// Get the annotation server auth token.
/// Priority: CHUB_ANNOTATION_TOKEN env var > ~/.chub/config.yaml annotation_token.
/// Token is intentionally NOT read from .chub/config.yaml to prevent accidental commits.
pub fn get_annotation_token() -> Option<String> {
    std::env::var("CHUB_ANNOTATION_TOKEN")
        .ok()
        .or_else(|| load_config().annotation_token)
}

/// Read and parse a YAML config file with a size limit.
fn read_config_file(path: &std::path::Path) -> Option<FileConfig> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > MAX_CONFIG_FILE_SIZE {
        eprintln!(
            "Warning: config file {} exceeds size limit ({} bytes, max {}). Skipping.",
            path.display(),
            meta.len(),
            MAX_CONFIG_FILE_SIZE
        );
        return None;
    }
    let raw = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&raw).ok()
}

/// Search upward from CWD for a `.chub/` directory (project-level).
fn find_project_chub_dir() -> Option<PathBuf> {
    let start = std::env::current_dir().ok()?;
    let mut current = start.as_path();
    loop {
        let candidate = current.join(".chub");
        if candidate.is_dir() {
            // Don't use ~/.chub as a project dir
            let home_chub = chub_dir();
            if candidate != home_chub {
                return Some(candidate);
            }
        }
        current = current.parent()?;
    }
}
