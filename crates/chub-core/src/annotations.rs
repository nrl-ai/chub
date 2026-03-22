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
    let safe = crate::util::sanitize_entry_id(entry_id);
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
    let updated_at = crate::util::now_iso8601();

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

    let json = serde_json::to_string_pretty(&data).unwrap_or_default();
    let _ = crate::util::atomic_write(&annotation_path(entry_id), json.as_bytes());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_note_trims_whitespace() {
        assert_eq!(sanitize_note("  hello  "), "hello");
    }

    #[test]
    fn sanitize_note_short_ascii_unchanged() {
        let note = "This is a short note.";
        assert_eq!(sanitize_note(note), note);
    }

    #[test]
    fn sanitize_note_truncates_long_ascii() {
        let long = "a".repeat(5000);
        let result = sanitize_note(&long);
        // 4000 chars + " [truncated]"
        assert_eq!(result.chars().count(), 4000 + " [truncated]".len());
        assert!(result.ends_with(" [truncated]"));
    }

    #[test]
    fn sanitize_note_multi_byte_no_panic() {
        // 2000 emoji characters (each 4 bytes) — exceeds 4000 byte limit
        // but should not panic since we count chars, not bytes
        let emoji_note: String = "🦀".repeat(5000);
        let result = sanitize_note(&emoji_note);
        assert!(result.ends_with(" [truncated]"));
        // The first 4000 chars should all be 🦀
        let crab_part: String = result.chars().take(4000).collect();
        assert_eq!(crab_part, "🦀".repeat(4000));
    }

    #[test]
    fn sanitize_note_cjk_no_panic() {
        // CJK characters are 3 bytes each in UTF-8
        let cjk_note: String = "漢".repeat(5000);
        let result = sanitize_note(&cjk_note);
        assert!(result.ends_with(" [truncated]"));
        let cjk_part: String = result.chars().take(4000).collect();
        assert_eq!(cjk_part, "漢".repeat(4000));
    }

    #[test]
    fn sanitize_note_exact_limit_not_truncated() {
        let exact = "a".repeat(MAX_ANNOTATION_LENGTH);
        let result = sanitize_note(&exact);
        assert_eq!(result.len(), MAX_ANNOTATION_LENGTH);
        assert!(!result.contains("[truncated]"));
    }

    #[test]
    fn sanitize_note_one_over_limit_truncated() {
        let over = "a".repeat(MAX_ANNOTATION_LENGTH + 1);
        let result = sanitize_note(&over);
        assert!(result.ends_with(" [truncated]"));
        assert_eq!(
            result.chars().count(),
            MAX_ANNOTATION_LENGTH + " [truncated]".len()
        );
    }

    #[test]
    fn sanitize_note_mixed_multibyte_and_ascii() {
        // Mix of 1-byte ASCII and 4-byte emoji
        let mut note = String::new();
        for _ in 0..2500 {
            note.push('a');
            note.push('🎉');
        }
        // 5000 chars total > 4000 limit, should truncate safely
        let result = sanitize_note(&note);
        assert!(result.ends_with(" [truncated]"));
        // Verify no panic and correct char count
        let content_chars = result.chars().count() - " [truncated]".len();
        assert_eq!(content_chars, MAX_ANNOTATION_LENGTH);
    }

    #[test]
    fn annotation_kind_parse_roundtrip() {
        for kind in [
            AnnotationKind::Note,
            AnnotationKind::Issue,
            AnnotationKind::Fix,
            AnnotationKind::Practice,
        ] {
            let s = kind.as_str();
            let parsed = AnnotationKind::parse(s).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn annotation_kind_parse_case_insensitive() {
        assert_eq!(AnnotationKind::parse("NOTE"), Some(AnnotationKind::Note));
        assert_eq!(AnnotationKind::parse("Issue"), Some(AnnotationKind::Issue));
        assert_eq!(AnnotationKind::parse("FIX"), Some(AnnotationKind::Fix));
    }

    #[test]
    fn annotation_kind_parse_invalid() {
        assert_eq!(AnnotationKind::parse("unknown"), None);
        assert_eq!(AnnotationKind::parse(""), None);
    }
}
