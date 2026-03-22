use std::path::Path;

use serde::Serialize;

use crate::team::detect::{detect_dependencies, DetectedDep};
use crate::team::pins::list_pins;

/// Result of a freshness check for one pinned doc.
#[derive(Debug, Clone, Serialize)]
pub struct FreshnessResult {
    pub pin_id: String,
    pub pinned_version: Option<String>,
    pub installed_version: Option<String>,
    pub status: FreshnessStatus,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FreshnessStatus {
    Current,
    Outdated,
    Unknown,
}

/// Check freshness of all pins against installed dependency versions.
pub fn check_freshness(root: &Path) -> Vec<FreshnessResult> {
    let pins = list_pins();
    if pins.is_empty() {
        return vec![];
    }

    let deps = detect_dependencies(root);
    let mut results = Vec::new();

    for pin in &pins {
        let pinned_version = pin.version.as_deref();

        // Try to find the matching dependency
        let dep = find_matching_dep(&pin.id, &deps);

        let result = match (pinned_version, dep) {
            (Some(pinned), Some(dep)) => {
                let installed = dep.version.as_deref().unwrap_or("");
                // Strip version prefix chars for comparison
                let installed_clean = installed.trim_start_matches(|c: char| {
                    c == '^' || c == '~' || c == '=' || c == '>' || c == '<' || c == 'v'
                });
                let pinned_clean = pinned.trim_start_matches(|c: char| {
                    c == '^' || c == '~' || c == '=' || c == '>' || c == '<' || c == 'v'
                });

                if pinned_clean == installed_clean || installed_clean.is_empty() {
                    FreshnessResult {
                        pin_id: pin.id.clone(),
                        pinned_version: Some(pinned.to_string()),
                        installed_version: dep.version.clone(),
                        status: FreshnessStatus::Current,
                        suggestion: None,
                    }
                } else {
                    FreshnessResult {
                        pin_id: pin.id.clone(),
                        pinned_version: Some(pinned.to_string()),
                        installed_version: dep.version.clone(),
                        status: FreshnessStatus::Outdated,
                        suggestion: Some(format!(
                            "chub pin {} --version {}",
                            pin.id, installed_clean
                        )),
                    }
                }
            }
            (None, _) => FreshnessResult {
                pin_id: pin.id.clone(),
                pinned_version: None,
                installed_version: dep.and_then(|d| d.version.clone()),
                status: FreshnessStatus::Current,
                suggestion: None,
            },
            (Some(pinned), None) => FreshnessResult {
                pin_id: pin.id.clone(),
                pinned_version: Some(pinned.to_string()),
                installed_version: None,
                status: FreshnessStatus::Unknown,
                suggestion: None,
            },
        };

        results.push(result);
    }

    results
}

fn find_matching_dep<'a>(pin_id: &str, deps: &'a [DetectedDep]) -> Option<&'a DetectedDep> {
    let id_parts: Vec<&str> = pin_id.split('/').collect();
    let search_name = if !id_parts.is_empty() {
        id_parts[0].to_lowercase()
    } else {
        pin_id.to_lowercase()
    };

    deps.iter().find(|d| d.name.to_lowercase() == search_name)
}

/// Auto-fix outdated pins by updating versions to installed versions.
pub fn auto_fix_freshness(results: &[FreshnessResult]) -> Vec<String> {
    let mut fixed = Vec::new();

    for result in results {
        if result.status != FreshnessStatus::Outdated {
            continue;
        }
        if let Some(ref installed) = result.installed_version {
            let clean = installed.trim_start_matches(|c: char| {
                c == '^' || c == '~' || c == '=' || c == '>' || c == '<' || c == 'v'
            });
            if let Ok(()) = crate::team::pins::add_pin(
                &result.pin_id,
                None,
                Some(clean.to_string()),
                None,
                None,
            ) {
                fixed.push(result.pin_id.clone());
            }
        }
    }

    fixed
}
