use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::team::project::project_chub_dir;

/// A context profile definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub pins: Vec<String>,
    #[serde(default)]
    pub context: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
}

/// A resolved profile with inheritance applied.
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub name: String,
    pub description: Option<String>,
    pub pins: Vec<String>,
    pub context: Vec<String>,
    pub rules: Vec<String>,
}

fn profiles_dir() -> Option<PathBuf> {
    project_chub_dir().map(|d| d.join("profiles"))
}

/// Load a raw profile by name.
pub fn load_profile(name: &str) -> Result<Profile> {
    let dir = profiles_dir().ok_or_else(|| {
        Error::Config("No .chub/ directory found. Run `chub init` first.".to_string())
    })?;

    let path = dir.join(format!("{}.yaml", name));
    if !path.exists() {
        let alt = dir.join(format!("{}.yml", name));
        if alt.exists() {
            let raw = fs::read_to_string(&alt)?;
            return serde_yaml::from_str(&raw).map_err(|e| Error::Config(e.to_string()));
        }
        return Err(Error::Config(format!("Profile \"{}\" not found.", name)));
    }

    let raw = fs::read_to_string(&path)?;
    serde_yaml::from_str(&raw).map_err(|e| Error::Config(e.to_string()))
}

/// Resolve a profile with inheritance (max depth 10 to prevent cycles).
pub fn resolve_profile(name: &str) -> Result<ResolvedProfile> {
    let mut chain = Vec::new();
    let mut current = name.to_string();
    let max_depth = 10;

    for _ in 0..max_depth {
        if chain.contains(&current) {
            return Err(Error::Config(format!(
                "Circular profile inheritance detected: {}",
                chain.join(" → ")
            )));
        }
        let profile = load_profile(&current)?;
        chain.push(current.clone());
        if let Some(ref parent) = profile.extends {
            current = parent.clone();
        } else {
            break;
        }
    }

    // Resolve from root to leaf
    let mut resolved = ResolvedProfile {
        name: name.to_string(),
        description: None,
        pins: Vec::new(),
        context: Vec::new(),
        rules: Vec::new(),
    };

    // Load in reverse order (root first, leaf last)
    for profile_name in chain.iter().rev() {
        let profile = load_profile(profile_name)?;
        if profile.description.is_some() {
            resolved.description = profile.description;
        }
        // Extend (not replace) — child adds to parent
        for pin in &profile.pins {
            if !resolved.pins.contains(pin) {
                resolved.pins.push(pin.clone());
            }
        }
        for ctx in &profile.context {
            if !resolved.context.contains(ctx) {
                resolved.context.push(ctx.clone());
            }
        }
        for rule in &profile.rules {
            if !resolved.rules.contains(rule) {
                resolved.rules.push(rule.clone());
            }
        }
    }

    Ok(resolved)
}

/// List available profile names.
pub fn list_profiles() -> Vec<(String, Option<String>)> {
    let dir = match profiles_dir() {
        Some(d) if d.exists() => d,
        _ => return vec![],
    };

    let mut profiles = Vec::new();
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let desc = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_yaml::from_str::<Profile>(&s).ok())
            .and_then(|p| p.description);
        profiles.push((stem, desc));
    }

    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    profiles
}

/// Get the active profile name (from env var or session file).
pub fn get_active_profile() -> Option<String> {
    // Check env var first
    if let Ok(profile) = std::env::var("CHUB_PROFILE") {
        if !profile.is_empty() {
            return Some(profile);
        }
    }

    // Check session file
    let session_path = project_chub_dir()?.join(".active_profile");
    fs::read_to_string(&session_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Set the active profile for this session.
pub fn set_active_profile(name: Option<&str>) -> Result<()> {
    let chub_dir = project_chub_dir().ok_or_else(|| {
        Error::Config("No .chub/ directory found. Run `chub init` first.".to_string())
    })?;

    let session_path = chub_dir.join(".active_profile");

    match name {
        Some(n) => {
            // Validate profile exists
            let _ = load_profile(n)?;
            fs::write(&session_path, n)?;
        }
        None => {
            let _ = fs::remove_file(&session_path);
        }
    }

    Ok(())
}

/// Auto-detect profile based on file path and auto_profile config.
pub fn auto_detect_profile(file_path: &str) -> Option<String> {
    let project_config = crate::team::project::load_project_config()?;
    let auto_profiles = project_config.auto_profile?;

    for entry in &auto_profiles {
        let pattern = format!("**/{}", entry.path);
        if let Ok(glob) = globset::Glob::new(&pattern) {
            let matcher = glob.compile_matcher();
            if matcher.is_match(file_path) {
                return Some(entry.profile.clone());
            }
        }
        // Simple prefix match fallback
        let prefix = entry.path.trim_end_matches("**").trim_end_matches('/');
        if file_path.starts_with(prefix) {
            return Some(entry.profile.clone());
        }
    }

    None
}
