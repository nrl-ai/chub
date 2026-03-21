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
///
/// **Append semantics**: each `write_team_annotation()` call adds a new entry to the
/// appropriate section. Entries are never replaced — use `clear_team_annotation()` to
/// remove the entire file. Unlike personal annotations (which overwrite), team annotations
/// maintain a full history with author and date for each entry.
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
/// Severity is only stored when kind=Issue; it is ignored for other kinds.
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

/// Delete the entire team annotation file for an entry.
/// Returns true if a file was removed, false if it didn't exist.
pub fn clear_team_annotation(entry_id: &str) -> bool {
    match team_annotation_path(entry_id) {
        Some(path) => fs::remove_file(path).is_ok(),
        None => false,
    }
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

/// Format a TeamAnnotation's entries as display strings with a given tier label prefix.
fn format_tier_parts(ann: &TeamAnnotation, tier_label: &str) -> Vec<String> {
    let mut parts = Vec::new();
    for note in &ann.issues {
        let sev = note
            .severity
            .as_deref()
            .map(|s| format!(" ({})", s))
            .unwrap_or_default();
        parts.push(format!(
            "[{} issue{} — {} ({})] {}",
            tier_label, sev, note.author, note.date, note.note
        ));
    }
    for note in &ann.fixes {
        parts.push(format!(
            "[{} fix — {} ({})] {}",
            tier_label, note.author, note.date, note.note
        ));
    }
    for note in &ann.practices {
        parts.push(format!(
            "[{} practice — {} ({})] {}",
            tier_label, note.author, note.date, note.note
        ));
    }
    for note in &ann.notes {
        parts.push(format!(
            "[{} — {} ({})] {}",
            tier_label, note.author, note.date, note.note
        ));
    }
    parts
}

/// Merge team + personal annotations into a display string, grouped by kind.
/// Team annotations are shown first (issues → fixes → practices → notes),
/// followed by any personal annotation.
pub fn get_merged_annotation(entry_id: &str) -> Option<String> {
    let team = read_team_annotation(entry_id);
    let personal = crate::annotations::read_annotation(entry_id);

    let mut parts = Vec::new();

    if let Some(ref ann) = team {
        parts.extend(format_tier_parts(ann, "Team"));
    }

    if let Some(ref p) = personal {
        let kind_tag = p.kind.as_str();
        let sev = p
            .severity
            .as_deref()
            .map(|s| format!(" ({})", s))
            .unwrap_or_default();
        parts.push(format!(
            "[Personal {}{} — {}] {}",
            kind_tag, sev, p.updated_at, p.note
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Merge all three tiers: Org baseline → Team overlay → Personal wins.
/// Falls back gracefully if Tier 3 is not configured or unreachable.
pub async fn get_merged_annotation_async(entry_id: &str) -> Option<String> {
    let org = crate::team::org_annotations::read_org_annotation(entry_id).await;
    let team = read_team_annotation(entry_id);
    let personal = crate::annotations::read_annotation(entry_id);

    let mut parts = Vec::new();

    if let Some(ref ann) = org {
        parts.extend(format_tier_parts(ann, "Org"));
    }
    if let Some(ref ann) = team {
        parts.extend(format_tier_parts(ann, "Team"));
    }
    if let Some(ref p) = personal {
        let kind_tag = p.kind.as_str();
        let sev = p
            .severity
            .as_deref()
            .map(|s| format!(" ({})", s))
            .unwrap_or_default();
        parts.push(format!(
            "[Personal {}{} — {}] {}",
            kind_tag, sev, p.updated_at, p.note
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Generate the notice appended to a pinned doc when it is served.
pub fn get_pin_notice(
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
