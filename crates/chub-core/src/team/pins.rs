use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::team::project::project_chub_dir;

/// A single pinned doc entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// The pins.yaml file structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinsFile {
    #[serde(default)]
    pub pins: Vec<PinEntry>,
}

fn pins_path() -> Option<std::path::PathBuf> {
    project_chub_dir().map(|d| d.join("pins.yaml"))
}

/// Load pins from `.chub/pins.yaml`.
pub fn load_pins() -> PinsFile {
    let path = match pins_path() {
        Some(p) if p.exists() => p,
        _ => return PinsFile::default(),
    };
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_yaml::from_str(&s).ok())
        .unwrap_or_default()
}

/// Save pins to `.chub/pins.yaml`.
pub fn save_pins(pins: &PinsFile) -> Result<()> {
    let path = pins_path().ok_or_else(|| {
        Error::Config("No .chub/ directory found. Run `chub init` first.".to_string())
    })?;
    let yaml = serde_yaml::to_string(pins).map_err(|e| Error::Config(e.to_string()))?;
    fs::write(&path, yaml)?;
    Ok(())
}

/// Add or update a pin.
pub fn add_pin(
    id: &str,
    lang: Option<String>,
    version: Option<String>,
    reason: Option<String>,
    source: Option<String>,
) -> Result<()> {
    let mut pins = load_pins();

    // Update existing or add new
    if let Some(existing) = pins.pins.iter_mut().find(|p| p.id == id) {
        if lang.is_some() {
            existing.lang = lang;
        }
        if version.is_some() {
            existing.version = version;
        }
        if reason.is_some() {
            existing.reason = reason;
        }
        if source.is_some() {
            existing.source = source;
        }
    } else {
        pins.pins.push(PinEntry {
            id: id.to_string(),
            lang,
            version,
            reason,
            source,
        });
    }

    save_pins(&pins)
}

/// Remove a pin by ID. Returns true if it existed.
pub fn remove_pin(id: &str) -> Result<bool> {
    let mut pins = load_pins();
    let before = pins.pins.len();
    pins.pins.retain(|p| p.id != id);
    let removed = pins.pins.len() < before;
    if removed {
        save_pins(&pins)?;
    }
    Ok(removed)
}

/// Get a specific pin by entry ID.
pub fn get_pin(id: &str) -> Option<PinEntry> {
    load_pins().pins.into_iter().find(|p| p.id == id)
}

/// List all pins.
pub fn list_pins() -> Vec<PinEntry> {
    load_pins().pins
}
