use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_CDN_URL: &str = "https://cdn.aichub.org/v1";
const DEFAULT_TELEMETRY_URL: &str = "https://api.aichub.org/v1";

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

/// Load config from ~/.chub/config.yaml with defaults and env var overrides.
pub fn load_config() -> Config {
    let defaults = Config::default();
    let config_path = chub_dir().join("config.yaml");

    let file_config: FileConfig = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|raw| serde_yaml::from_str(&raw).ok())
        .unwrap_or_default();

    // Build sources list
    let sources = if let Some(sources) = file_config.sources {
        sources
    } else {
        let url = std::env::var("CHUB_BUNDLE_URL")
            .ok()
            .or(file_config.cdn_url)
            .unwrap_or_else(|| DEFAULT_CDN_URL.to_string());
        vec![SourceConfig {
            name: "default".to_string(),
            url: Some(url),
            path: None,
        }]
    };

    Config {
        sources,
        output_dir: file_config.output_dir.unwrap_or(defaults.output_dir),
        refresh_interval: file_config
            .refresh_interval
            .unwrap_or(defaults.refresh_interval),
        output_format: file_config.output_format.unwrap_or(defaults.output_format),
        source: file_config.source.unwrap_or(defaults.source),
        telemetry: file_config.telemetry.unwrap_or(defaults.telemetry),
        feedback: file_config.feedback.unwrap_or(defaults.feedback),
        telemetry_url: file_config.telemetry_url.unwrap_or(defaults.telemetry_url),
    }
}
