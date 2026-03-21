use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::team::pins::{load_pins, save_pins, PinEntry, PinsFile};
use crate::team::project::project_chub_dir;

fn validate_name(name: &str) -> Result<()> {
    crate::util::validate_filename(name, "snapshot")
}

/// A point-in-time snapshot of all pins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub name: String,
    pub created_at: String,
    pub pins: Vec<PinEntry>,
}

/// A diff between two snapshots.
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotDiff {
    pub id: String,
    pub change: DiffChange,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffChange {
    Added {
        version: Option<String>,
    },
    Removed {
        version: Option<String>,
    },
    Changed {
        from_version: Option<String>,
        to_version: Option<String>,
    },
}

fn snapshots_dir() -> Option<PathBuf> {
    project_chub_dir().map(|d| d.join("snapshots"))
}

fn now_iso() -> String {
    crate::util::now_iso8601()
}

/// Create a snapshot of the current pins.
pub fn create_snapshot(name: &str) -> Result<Snapshot> {
    validate_name(name)?;
    let dir =
        snapshots_dir().ok_or_else(|| Error::Config("No .chub/ directory found.".to_string()))?;
    fs::create_dir_all(&dir)?;

    let pins = load_pins();
    let snapshot = Snapshot {
        name: name.to_string(),
        created_at: now_iso(),
        pins: pins.pins,
    };

    let path = dir.join(format!("{}.yaml", name));
    let yaml = serde_yaml::to_string(&snapshot).map_err(|e| Error::Config(e.to_string()))?;
    fs::write(&path, yaml)?;

    Ok(snapshot)
}

/// Restore pins from a snapshot.
pub fn restore_snapshot(name: &str) -> Result<Snapshot> {
    validate_name(name)?;
    let dir =
        snapshots_dir().ok_or_else(|| Error::Config("No .chub/ directory found.".to_string()))?;

    let path = dir.join(format!("{}.yaml", name));
    if !path.exists() {
        return Err(Error::Config(format!("Snapshot \"{}\" not found.", name)));
    }

    let raw = fs::read_to_string(&path)?;
    let snapshot: Snapshot =
        serde_yaml::from_str(&raw).map_err(|e| Error::Config(e.to_string()))?;

    let pins_file = PinsFile {
        pins: snapshot.pins.clone(),
    };
    save_pins(&pins_file)?;

    Ok(snapshot)
}

/// Diff two snapshots.
pub fn diff_snapshots(name_a: &str, name_b: &str) -> Result<Vec<SnapshotDiff>> {
    validate_name(name_a)?;
    validate_name(name_b)?;
    let dir =
        snapshots_dir().ok_or_else(|| Error::Config("No .chub/ directory found.".to_string()))?;

    let load = |name: &str| -> Result<Snapshot> {
        let path = dir.join(format!("{}.yaml", name));
        if !path.exists() {
            return Err(Error::Config(format!("Snapshot \"{}\" not found.", name)));
        }
        let raw = fs::read_to_string(&path)?;
        serde_yaml::from_str(&raw).map_err(|e| Error::Config(e.to_string()))
    };

    let snap_a = load(name_a)?;
    let snap_b = load(name_b)?;

    let mut diffs = Vec::new();

    // Find added/changed in B
    for pin_b in &snap_b.pins {
        if let Some(pin_a) = snap_a.pins.iter().find(|p| p.id == pin_b.id) {
            if pin_a.version != pin_b.version {
                diffs.push(SnapshotDiff {
                    id: pin_b.id.clone(),
                    change: DiffChange::Changed {
                        from_version: pin_a.version.clone(),
                        to_version: pin_b.version.clone(),
                    },
                });
            }
        } else {
            diffs.push(SnapshotDiff {
                id: pin_b.id.clone(),
                change: DiffChange::Added {
                    version: pin_b.version.clone(),
                },
            });
        }
    }

    // Find removed from A
    for pin_a in &snap_a.pins {
        if !snap_b.pins.iter().any(|p| p.id == pin_a.id) {
            diffs.push(SnapshotDiff {
                id: pin_a.id.clone(),
                change: DiffChange::Removed {
                    version: pin_a.version.clone(),
                },
            });
        }
    }

    Ok(diffs)
}

/// List all snapshots.
pub fn list_snapshots() -> Vec<(String, String)> {
    let dir = match snapshots_dir() {
        Some(d) if d.exists() => d,
        _ => return vec![],
    };

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut snapshots = Vec::new();
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

        let created_at = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_yaml::from_str::<Snapshot>(&s).ok())
            .map(|s| s.created_at)
            .unwrap_or_default();

        snapshots.push((stem, created_at));
    }

    snapshots.sort_by(|a, b| a.0.cmp(&b.0));
    snapshots
}
