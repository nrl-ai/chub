use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::team::pins;
use crate::team::project::project_chub_dir;

/// Validate that a name is safe for use as a filename (no path traversal).
fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.starts_with('.')
    {
        return Err(Error::Config(format!(
            "Invalid bundle name \"{}\": must not contain path separators or \"..\"",
            name
        )));
    }
    Ok(())
}

/// A doc bundle — a curated, shareable collection of docs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub entries: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn bundles_dir() -> Option<std::path::PathBuf> {
    project_chub_dir().map(|d| d.join("bundles"))
}

/// Load a bundle from a YAML file.
pub fn load_bundle(path: &Path) -> Result<Bundle> {
    let raw = fs::read_to_string(path)?;
    serde_yaml::from_str(&raw).map_err(|e| Error::Config(format!("Invalid bundle YAML: {}", e)))
}

/// Load a bundle by name from `.chub/bundles/`.
pub fn load_bundle_by_name(name: &str) -> Result<Bundle> {
    validate_name(name)?;
    let dir =
        bundles_dir().ok_or_else(|| Error::Config("No .chub/ directory found.".to_string()))?;
    let path = dir.join(format!("{}.yaml", name));
    if !path.exists() {
        let alt = dir.join(format!("{}.yml", name));
        if alt.exists() {
            return load_bundle(&alt);
        }
        return Err(Error::Config(format!("Bundle \"{}\" not found.", name)));
    }
    load_bundle(&path)
}

/// Install a bundle: pin all its entries.
pub fn install_bundle(bundle: &Bundle) -> Result<Vec<String>> {
    let mut pinned = Vec::new();
    for entry_id in &bundle.entries {
        pins::add_pin(
            entry_id,
            None,
            None,
            Some(format!("From bundle: {}", bundle.name)),
            None,
        )?;
        pinned.push(entry_id.clone());
    }
    Ok(pinned)
}

/// Create a new bundle file.
pub fn create_bundle(
    name: &str,
    description: Option<&str>,
    author: Option<&str>,
    entries: Vec<String>,
    notes: Option<&str>,
) -> Result<std::path::PathBuf> {
    validate_name(name)?;
    let dir =
        bundles_dir().ok_or_else(|| Error::Config("No .chub/ directory found.".to_string()))?;
    fs::create_dir_all(&dir)?;

    let bundle = Bundle {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        author: author.map(|s| s.to_string()),
        entries,
        notes: notes.map(|s| s.to_string()),
    };

    let path = dir.join(format!("{}.yaml", name));
    let yaml = serde_yaml::to_string(&bundle).map_err(|e| Error::Config(e.to_string()))?;
    fs::write(&path, yaml)?;

    Ok(path)
}

/// List all available bundles.
pub fn list_bundles() -> Vec<Bundle> {
    let dir = match bundles_dir() {
        Some(d) if d.exists() => d,
        _ => return vec![],
    };

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "yaml" || ext == "yml")
                .unwrap_or(false)
        })
        .filter_map(|e| load_bundle(&e.path()).ok())
        .collect()
}
