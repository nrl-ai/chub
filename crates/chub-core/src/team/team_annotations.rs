use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::team::project::project_chub_dir;

/// A team annotation note entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnnotationNote {
    pub author: String,
    pub date: String,
    pub note: String,
}

/// A team annotation file (`.chub/annotations/<id>.yaml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnnotation {
    pub id: String,
    #[serde(default)]
    pub notes: Vec<TeamAnnotationNote>,
}

fn team_annotations_dir() -> Option<PathBuf> {
    project_chub_dir().map(|d| d.join("annotations"))
}

fn team_annotation_path(entry_id: &str) -> Option<PathBuf> {
    let safe = entry_id.replace('/', "--");
    team_annotations_dir().map(|d| d.join(format!("{}.yaml", safe)))
}

/// Read team annotations for an entry.
pub fn read_team_annotation(entry_id: &str) -> Option<TeamAnnotation> {
    let path = team_annotation_path(entry_id)?;
    if !path.exists() {
        return None;
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_yaml::from_str(&s).ok())
}

/// Write a team annotation (append a note).
pub fn write_team_annotation(entry_id: &str, note: &str, author: &str) -> Option<TeamAnnotation> {
    let dir = team_annotations_dir()?;
    let _ = fs::create_dir_all(&dir);

    let mut ann = read_team_annotation(entry_id).unwrap_or(TeamAnnotation {
        id: entry_id.to_string(),
        notes: vec![],
    });

    let now = crate::build::builder::days_to_date(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400,
    );
    let date = format!("{:04}-{:02}-{:02}", now.0, now.1, now.2);

    ann.notes.push(TeamAnnotationNote {
        author: author.to_string(),
        date,
        note: note.to_string(),
    });

    let path = team_annotation_path(entry_id)?;
    let yaml = serde_yaml::to_string(&ann).ok()?;
    fs::write(&path, yaml).ok()?;
    Some(ann)
}

/// List all team annotations.
pub fn list_team_annotations() -> Vec<TeamAnnotation> {
    let dir = match team_annotations_dir() {
        Some(d) if d.exists() => d,
        _ => return vec![],
    };

    let files = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    files
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "yaml" || ext == "yml")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            fs::read_to_string(e.path())
                .ok()
                .and_then(|s| serde_yaml::from_str::<TeamAnnotation>(&s).ok())
        })
        .collect()
}

/// Merge annotations: team annotations + personal annotations.
/// Resolution order: public doc → team annotations → personal annotations.
/// Returns a combined annotation string for display.
pub fn get_merged_annotation(entry_id: &str) -> Option<String> {
    let team = read_team_annotation(entry_id);
    let personal = crate::annotations::read_annotation(entry_id);

    let mut parts = Vec::new();

    if let Some(ref team_ann) = team {
        for note in &team_ann.notes {
            parts.push(format!(
                "[Team — {} ({})] {}",
                note.author, note.date, note.note
            ));
        }
    }

    if let Some(ref personal_ann) = personal {
        parts.push(format!(
            "[Personal — {}] {}",
            personal_ann.updated_at, personal_ann.note
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Get the annotation to append when serving a pinned doc.
pub fn get_pin_notice(
    _entry_id: &str,
    pinned_version: Option<&str>,
    pinned_lang: Option<&str>,
    reason: Option<&str>,
) -> String {
    let mut notice = String::from("\n---\n[Team pin]");
    if let Some(ver) = pinned_version {
        notice.push_str(&format!(" Locked to v{}", ver));
    }
    if let Some(lang) = pinned_lang {
        notice.push_str(&format!(" ({})", lang));
    }
    notice.push('.');
    if let Some(reason) = reason {
        notice.push_str(&format!(" Reason: {}", reason));
    }
    notice.push('\n');
    notice
}
