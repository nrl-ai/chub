use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::chub_dir;

/// Maximum annotation length in characters. Notes exceeding this are truncated.
const MAX_ANNOTATION_LENGTH: usize = 4000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub note: String,
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
    if trimmed.len() <= MAX_ANNOTATION_LENGTH {
        trimmed.to_string()
    } else {
        let mut s = trimmed[..MAX_ANNOTATION_LENGTH].to_string();
        s.push_str(" [truncated]");
        s
    }
}

pub fn write_annotation(entry_id: &str, note: &str) -> Annotation {
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
