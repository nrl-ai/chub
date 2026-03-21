use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::chub_dir;

/// Maximum annotation length in characters. Notes exceeding this are truncated.
const MAX_ANNOTATION_LENGTH: usize = 4000;

/// The kind of annotation — classifies what the agent learned.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationKind {
    /// General observation (default).
    #[default]
    Note,
    /// Undocumented bug, broken param, or misleading example.
    Issue,
    /// Workaround that resolved an issue.
    Fix,
    /// Team convention or validated pattern.
    Practice,
}

impl AnnotationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationKind::Note => "note",
            AnnotationKind::Issue => "issue",
            AnnotationKind::Fix => "fix",
            AnnotationKind::Practice => "practice",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "note" => Some(AnnotationKind::Note),
            "issue" => Some(AnnotationKind::Issue),
            "fix" => Some(AnnotationKind::Fix),
            "practice" => Some(AnnotationKind::Practice),
            _ => None,
        }
    }
}

/// A personal annotation stored at `~/.chub/annotations/<id>.json`.
///
/// **Overwrite semantics**: writing a new annotation for the same entry ID replaces the previous
/// one entirely. There is no history. Use team annotations (`.chub/annotations/<id>.yaml`) if
/// you need an append-based history with multiple authors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub note: String,
    #[serde(default)]
    pub kind: AnnotationKind,
    /// Severity level for issue annotations: "high", "medium", or "low". Only used when
    /// kind=issue; ignored for other kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

fn annotations_dir() -> PathBuf {
    chub_dir().join("annotations")
}

fn annotation_path(entry_id: &str) -> PathBuf {
    let safe = entry_id.replace('/', "--");
    annotations_dir().join(format!("{}.json", safe))
}

pub fn read_annotation(entry_id: &str) -> Option<Annotation> {
    let path = annotation_path(entry_id);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Sanitize annotation text: truncate to max length.
pub fn sanitize_note(note: &str) -> String {
    let trimmed = note.trim();
    if trimmed.chars().count() <= MAX_ANNOTATION_LENGTH {
        trimmed.to_string()
    } else {
        let mut s: String = trimmed.chars().take(MAX_ANNOTATION_LENGTH).collect();
        s.push_str(" [truncated]");
        s
    }
}

/// Write a personal annotation. **Overwrites** any existing annotation for this entry.
/// Severity is only stored when kind=Issue; it is ignored for other kinds.
pub fn write_annotation(
    entry_id: &str,
    note: &str,
    kind: AnnotationKind,
    severity: Option<String>,
) -> Annotation {
    let dir = annotations_dir();
    let _ = fs::create_dir_all(&dir);

    let note = sanitize_note(note);

    let now = crate::build::builder::days_to_date(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400,
    );
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let tod = secs % 86400;

    let updated_at = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z",
        now.0,
        now.1,
        now.2,
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    );

    let data = Annotation {
        id: entry_id.to_string(),
        note,
        kind: kind.clone(),
        severity: if kind == AnnotationKind::Issue {
            severity
        } else {
            None
        },
        updated_at,
    };

    let _ = fs::write(
        annotation_path(entry_id),
        serde_json::to_string_pretty(&data).unwrap_or_default(),
    );

    data
}

pub fn clear_annotation(entry_id: &str) -> bool {
    fs::remove_file(annotation_path(entry_id)).is_ok()
}

pub fn list_annotations() -> Vec<Annotation> {
    let dir = annotations_dir();
    let files = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    files
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            fs::read_to_string(e.path())
                .ok()
                .and_then(|s| serde_json::from_str::<Annotation>(&s).ok())
        })
        .collect()
}
