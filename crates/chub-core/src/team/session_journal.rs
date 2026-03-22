//! JSONL event journal for full session transcripts.
//!
//! Stored in `.git/chub-sessions/<session-id>.jsonl` (local-only, never pushed).

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::sessions::{git_sessions_dir, TokenUsage};
use crate::util::now_iso8601;

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionEvent {
    #[serde(rename = "session_start")]
    SessionStart {
        ts: String,
        session_id: String,
        agent: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
    #[serde(rename = "prompt")]
    Prompt {
        ts: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    #[serde(rename = "tool_call")]
    ToolCall {
        ts: String,
        tool: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        input_summary: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        ts: String,
        tool: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        output_size: Option<u64>,
    },
    #[serde(rename = "response")]
    Response {
        ts: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tokens: Option<TokenUsage>,
    },
    #[serde(rename = "thinking")]
    Thinking {
        ts: String,
        /// Truncated thinking/reasoning content (first 500 chars).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// Size of the full thinking block in bytes.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_size: Option<u64>,
        /// Reasoning tokens consumed (from API usage).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning_tokens: Option<u64>,
    },
    #[serde(rename = "model_update")]
    ModelUpdate { ts: String, model: String },
    #[serde(rename = "file_change")]
    FileChange {
        ts: String,
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<String>, // "create", "edit", "delete"
    },
    #[serde(rename = "session_end")]
    SessionEnd {
        ts: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_s: Option<u64>,
        #[serde(default)]
        turns: u32,
    },
}

// ---------------------------------------------------------------------------
// Journal operations
// ---------------------------------------------------------------------------

fn journal_path(session_id: &str) -> Option<PathBuf> {
    git_sessions_dir().map(|d| d.join(format!("{}.jsonl", session_id)))
}

/// Append an event to the session journal.
pub fn append_event(session_id: &str, event: &SessionEvent) {
    let path = match journal_path(session_id) {
        Some(p) => p,
        None => return,
    };
    let _ = fs::create_dir_all(path.parent().unwrap_or(&PathBuf::from(".")));

    let line = serde_json::to_string(event).unwrap_or_default();
    let mut file = fs::OpenOptions::new().create(true).append(true).open(&path);

    if let Ok(ref mut f) = file {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

/// Load all events for a session.
pub fn load_events(session_id: &str) -> Vec<SessionEvent> {
    let path = match journal_path(session_id) {
        Some(p) => p,
        None => return vec![],
    };
    if !path.exists() {
        return vec![];
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// Get journal file size for a session.
pub fn journal_size(session_id: &str) -> u64 {
    journal_path(session_id)
        .and_then(|p| fs::metadata(&p).ok())
        .map(|m| m.len())
        .unwrap_or(0)
}

/// List all journal files in `.git/chub-sessions/`.
pub fn list_journal_files() -> Vec<String> {
    let dir = match git_sessions_dir() {
        Some(d) => d,
        None => return vec![],
    };
    if !dir.is_dir() {
        return vec![];
    }
    fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect()
}

/// Clear all local journal files.
pub fn clear_journals() -> usize {
    let dir = match git_sessions_dir() {
        Some(d) => d,
        None => return 0,
    };
    if !dir.is_dir() {
        return 0;
    }
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false)
                && fs::remove_file(&path).is_ok()
            {
                count += 1;
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Convenience: record events with auto-timestamp
// ---------------------------------------------------------------------------

pub fn record_session_start(session_id: &str, agent: &str, model: Option<&str>) {
    append_event(
        session_id,
        &SessionEvent::SessionStart {
            ts: now_iso8601(),
            session_id: session_id.to_string(),
            agent: agent.to_string(),
            model: model.map(|s| s.to_string()),
        },
    );
}

pub fn record_prompt(session_id: &str, text: Option<&str>) {
    append_event(
        session_id,
        &SessionEvent::Prompt {
            ts: now_iso8601(),
            text: text.map(|s| s.to_string()),
        },
    );
}

pub fn record_tool_call(session_id: &str, tool: &str, input_summary: Option<&str>) {
    append_event(
        session_id,
        &SessionEvent::ToolCall {
            ts: now_iso8601(),
            tool: tool.to_string(),
            input_summary: input_summary.map(|s| s.to_string()),
            duration_ms: None,
        },
    );
}

pub fn record_tool_result(session_id: &str, tool: &str, output_size: Option<u64>) {
    append_event(
        session_id,
        &SessionEvent::ToolResult {
            ts: now_iso8601(),
            tool: tool.to_string(),
            output_size,
        },
    );
}

pub fn record_response(session_id: &str, tokens: Option<TokenUsage>) {
    append_event(
        session_id,
        &SessionEvent::Response {
            ts: now_iso8601(),
            tokens,
        },
    );
}

pub fn record_file_change(session_id: &str, path: &str, action: Option<&str>) {
    append_event(
        session_id,
        &SessionEvent::FileChange {
            ts: now_iso8601(),
            path: path.to_string(),
            action: action.map(|s| s.to_string()),
        },
    );
}

pub fn record_thinking(
    session_id: &str,
    content: Option<&str>,
    content_size: Option<u64>,
    reasoning_tokens: Option<u64>,
) {
    // Truncate content to 500 chars for journal storage
    let truncated = content.map(|c| {
        if c.len() <= 500 {
            c.to_string()
        } else {
            format!("{}...", &c[..497])
        }
    });
    append_event(
        session_id,
        &SessionEvent::Thinking {
            ts: now_iso8601(),
            content: truncated,
            content_size,
            reasoning_tokens,
        },
    );
}

pub fn record_session_end(session_id: &str, duration_s: Option<u64>, turns: u32) {
    append_event(
        session_id,
        &SessionEvent::SessionEnd {
            ts: now_iso8601(),
            duration_s,
            turns,
        },
    );
}
