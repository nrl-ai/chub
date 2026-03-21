use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::annotations::AnnotationKind;
use crate::team::project::project_chub_dir;

/// A single annotation note (used across notes/issues/fixes/practices sections).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnnotationNote {
    pub author: String,
    pub date: String,
    pub note: String,
    /// Severity level — only meaningful for issues (high | medium | low).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// A team annotation file (`.chub/annotations/<id>.yaml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnnotation {
    pub id: String,
    /// General notes (kind=note, backward-compatible).
    #[serde(default)]
    pub notes: Vec<TeamAnnotationNote>,
    /// Known bugs, broken params, misleading examples (kind=issue).
    #[serde(default)]
    pub issues: Vec<TeamAnnotationNote>,
    /// Workarounds that resolve issues (kind=fix).
    #[serde(default)]
    pub fixes: Vec<TeamAnnotationNote>,
    /// Team conventions and validated patterns (kind=practice).
    #[serde(default)]
    pub practices: Vec<TeamAnnotationNote>,
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

/// Write a team annotation (append a note to the appropriate section).
pub fn write_team_annotation(
    entry_id: &str,
    note: &str,
    author: &str,
    kind: AnnotationKind,
    severity: Option<String>,
) -> Option<TeamAnnotation> {
    let dir = team_annotations_dir()?;
    let _ = fs::create_dir_all(&dir);

    let mut ann = read_team_annotation(entry_id).unwrap_or(TeamAnnotation {
        id: entry_id.to_string(),
        notes: vec![],
        issues: vec![],
        fixes: vec![],
        practices: vec![],
    });

    let now = crate::build::builder::days_to_date(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400,
    );
    let date = format!("{:04}-{:02}-{:02}", now.0, now.1, now.2);

    let entry = TeamAnnotationNote {
        author: author.to_string(),
        date,
        note: crate::annotations::sanitize_note(note),
        severity: if kind == AnnotationKind::Issue {
            severity
        } else {
            None
        },
    };

    match kind {
        AnnotationKind::Issue => ann.issues.push(entry),
        AnnotationKind::Fix => ann.fixes.push(entry),
        AnnotationKind::Practice => ann.practices.push(entry),
        AnnotationKind::Note => ann.notes.push(entry),
    }

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
/// Returns a combined annotation string for display, grouped by kind.
pub fn get_merged_annotation(entry_id: &str) -> Option<String> {
    let team = read_team_annotation(entry_id);
    let personal = crate::annotations::read_annotation(entry_id);

    let mut parts = Vec::new();

    if let Some(ref team_ann) = team {
        for note in &team_ann.issues {
            let severity_tag = note
                .severity
                .as_deref()
                .map(|s| format!(" ({})", s))
                .unwrap_or_default();
            parts.push(format!(
                "[Team issue{} — {} ({})] {}",
                severity_tag, note.author, note.date, note.note
            ));
        }
        for note in &team_ann.fixes {
            parts.push(format!(
                "[Team fix — {} ({})] {}",
                note.author, note.date, note.note
            ));
        }
        for note in &team_ann.practices {
            parts.push(format!(
                "[Team practice — {} ({})] {}",
                note.author, note.date, note.note
            ));
        }
        for note in &team_ann.notes {
            parts.push(format!(
                "[Team — {} ({})] {}",
                note.author, note.date, note.note
            ));
        }
    }

    if let Some(ref personal_ann) = personal {
        let kind_tag = personal_ann.kind.as_str();
        parts.push(format!(
            "[Personal {} — {}] {}",
            kind_tag, personal_ann.updated_at, personal_ann.note
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
