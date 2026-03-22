//! Transcript linking and token extraction.
//!
//! Reads Claude Code transcript files (JSONL), extracts token usage
//! (deduplicated by message.id), modified files, and spawned agent IDs.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::types::TokenUsage;

// ---------------------------------------------------------------------------
// Transcript path resolution
// ---------------------------------------------------------------------------

/// Sanitize a repo path for Claude Code's project directory naming.
/// Replaces non-alphanumeric chars with dashes (matches entire.io's SanitizePathForClaude).
fn sanitize_path_for_claude(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Find the Claude Code projects directory for a given repo path.
/// Returns `~/.claude/projects/<sanitized-repo>/`.
pub fn claude_projects_dir(repo_path: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let sanitized = sanitize_path_for_claude(repo_path);
    Some(home.join(".claude").join("projects").join(sanitized))
}

/// Find a Claude Code transcript file for a session.
pub fn find_transcript(repo_path: &str, session_id: &str) -> Option<PathBuf> {
    let dir = claude_projects_dir(repo_path)?;
    let path = dir.join(format!("{}.jsonl", session_id));
    if path.exists() {
        Some(path)
    } else {
        // Try listing files to find a matching transcript
        find_transcript_by_scan(&dir, session_id)
    }
}

/// Scan a directory for a transcript matching the session ID.
fn find_transcript_by_scan(dir: &Path, _session_id: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    // Claude Code transcripts are named by their conversation UUID
    // We can find the most recent one if no exact match
    let mut transcripts: Vec<_> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            Some((e.path(), meta.modified().ok()?))
        })
        .collect();

    transcripts.sort_by(|a, b| b.1.cmp(&a.1));
    transcripts.into_iter().next().map(|(p, _)| p)
}

// ---------------------------------------------------------------------------
// Transcript line parsing (Claude Code JSONL format)
// ---------------------------------------------------------------------------

/// A line from a Claude Code transcript JSONL file.
#[derive(Debug, Deserialize)]
struct TranscriptLine {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    uuid: Option<String>,
    #[serde(default)]
    message: Option<serde_json::Value>,
}

/// Message-level token usage from Claude API response.
#[derive(Debug, Deserialize)]
struct MessageUsage {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default, alias = "cache_creation_input_tokens")]
    cache_creation_tokens: i64,
    #[serde(default, alias = "cache_read_input_tokens")]
    cache_read_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
    /// Extended thinking / reasoning tokens (Claude, o1/o3).
    /// Claude API field: absent or 0 when thinking is off.
    #[serde(default)]
    reasoning_tokens: i64,
}

/// A captured thinking/reasoning block from an assistant message.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThinkingBlock {
    /// The thinking text content.
    pub content: String,
    /// Which message (by index) this thinking block came from.
    pub message_index: i32,
}

/// Results from parsing a transcript.
#[derive(Debug, Default)]
pub struct TranscriptAnalysis {
    pub token_usage: TokenUsage,
    pub modified_files: Vec<String>,
    pub spawned_agent_ids: Vec<String>,
    pub turn_count: i32,
    pub message_count: i32,
    pub model: Option<String>,
    /// Captured thinking/reasoning blocks from the conversation.
    pub thinking_blocks: Vec<ThinkingBlock>,
    /// Total number of thinking blocks encountered.
    pub thinking_block_count: i32,
    /// Whether extended thinking was used in this session.
    pub has_extended_thinking: bool,
}

/// Parse a Claude Code transcript file and extract token usage and metadata.
/// Deduplicates by message.id (Claude streams may create multiple entries).
pub fn parse_transcript(path: &Path) -> TranscriptAnalysis {
    parse_transcript_range(path, 0, None)
}

/// Parse a range of a transcript (from byte offset, optionally limited lines).
pub fn parse_transcript_range(
    path: &Path,
    start_byte: i64,
    _max_lines: Option<usize>,
) -> TranscriptAnalysis {
    let mut analysis = TranscriptAnalysis::default();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return analysis,
    };

    // Skip to start_byte offset
    let content = if start_byte > 0 && (start_byte as usize) < content.len() {
        &content[start_byte as usize..]
    } else {
        &content
    };

    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut token_map: HashMap<String, MessageUsage> = HashMap::new();
    let mut files: HashSet<String> = HashSet::new();
    let mut agent_ids: HashSet<String> = HashSet::new();
    let mut user_turns = 0i32;
    let mut model: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let tl: TranscriptLine = match serde_json::from_str(line) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Count user turns — only real user prompts, not system/hook messages
        if tl.r#type == "user" {
            if let Some(ref msg) = tl.message {
                let is_real_user = is_real_user_turn(msg);
                if is_real_user {
                    user_turns += 1;
                }
            }
        }

        // Track message count
        analysis.message_count += 1;

        // Deduplicate by UUID
        if let Some(ref uuid) = tl.uuid {
            if !seen_ids.insert(uuid.clone()) {
                // Already seen — update token usage (take latest streaming value)
            }
        }

        // Extract from assistant messages
        if tl.r#type == "assistant" {
            if let Some(ref msg) = tl.message {
                // Extract model name (take first non-empty model seen)
                if model.is_none() {
                    if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
                        if !m.is_empty() {
                            model = Some(m.to_string());
                        }
                    }
                }

                // Extract token usage from message.usage
                if let Some(usage) = msg.get("usage") {
                    if let Ok(mu) = serde_json::from_value::<MessageUsage>(usage.clone()) {
                        let msg_id = msg
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        // Dedup by message.id — keep latest (streaming updates)
                        token_map.insert(msg_id, mu);
                    }
                }

                // Extract thinking/reasoning content blocks
                extract_thinking_from_message(msg, analysis.message_count, &mut analysis);

                // Extract modified files from tool_use content blocks
                extract_files_from_message(msg, &mut files);

                // Extract spawned agent IDs
                extract_agent_ids_from_message(msg, &mut agent_ids);
            }
        }
    }

    // Sum token usage across all deduplicated messages
    for mu in token_map.values() {
        analysis.token_usage.input_tokens += mu.input_tokens;
        analysis.token_usage.output_tokens += mu.output_tokens;
        analysis.token_usage.cache_read_tokens += mu.cache_read_tokens;
        analysis.token_usage.cache_creation_tokens += mu.cache_creation_tokens;
        analysis.token_usage.reasoning_tokens += mu.reasoning_tokens;
    }
    analysis.token_usage.api_call_count = token_map.len() as i64;
    // Mark extended thinking if any reasoning tokens were found
    if analysis.token_usage.reasoning_tokens > 0 || !analysis.thinking_blocks.is_empty() {
        analysis.has_extended_thinking = true;
    }
    analysis.turn_count = user_turns;
    analysis.model = model;
    analysis.modified_files = files.into_iter().collect();
    analysis.modified_files.sort();
    analysis.spawned_agent_ids = agent_ids.into_iter().collect();

    analysis
}

/// Check if a user message is a real user turn (not a system message, hook output,
/// or subagent notification). Claude Code logs system-reminders and hook results as
/// "user" type messages in the transcript.
fn is_real_user_turn(msg: &serde_json::Value) -> bool {
    let content = match msg.get("content") {
        Some(c) => c,
        None => return false,
    };

    // String content
    if let Some(text) = content.as_str() {
        return !is_system_content(text);
    }

    // Array content — check text blocks
    if let Some(blocks) = content.as_array() {
        for block in blocks {
            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                if !is_system_content(text) {
                    return true;
                }
            }
            // tool_result blocks are internal
            if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                return false;
            }
        }
        return false;
    }

    false
}

/// Check if text content is a system/internal message rather than user input.
fn is_system_content(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("<system-reminder>")
        || trimmed.starts_with("<task-notification>")
        || trimmed.starts_with("<user-prompt-submit-hook>")
        || trimmed.starts_with("<available-deferred-tools>")
        || trimmed.contains("<system-reminder>")
}

/// Extract thinking/reasoning blocks from an assistant message.
/// Claude uses `type: "thinking"` content blocks with a `thinking` field.
/// OpenAI o1/o3 may use `type: "reasoning"` or include in `reasoning_content`.
fn extract_thinking_from_message(
    msg: &serde_json::Value,
    message_index: i32,
    analysis: &mut TranscriptAnalysis,
) {
    // Check content[] array for thinking blocks (Claude extended thinking)
    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
        for block in content {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match block_type {
                "thinking" => {
                    // Claude extended thinking: { "type": "thinking", "thinking": "..." }
                    if let Some(text) = block.get("thinking").and_then(|t| t.as_str()) {
                        if !text.trim().is_empty() {
                            analysis.thinking_blocks.push(ThinkingBlock {
                                content: text.to_string(),
                                message_index,
                            });
                            analysis.thinking_block_count += 1;
                        }
                    }
                }
                "reasoning" => {
                    // o1/o3 style reasoning blocks
                    if let Some(text) = block
                        .get("content")
                        .or_else(|| block.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        if !text.trim().is_empty() {
                            analysis.thinking_blocks.push(ThinkingBlock {
                                content: text.to_string(),
                                message_index,
                            });
                            analysis.thinking_block_count += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Check top-level reasoning_content (some providers put reasoning here)
    if let Some(reasoning) = msg.get("reasoning_content").and_then(|r| r.as_str()) {
        if !reasoning.trim().is_empty() {
            analysis.thinking_blocks.push(ThinkingBlock {
                content: reasoning.to_string(),
                message_index,
            });
            analysis.thinking_block_count += 1;
        }
    }
}

/// Extract file paths from tool_use blocks in a message.
fn extract_files_from_message(msg: &serde_json::Value, files: &mut HashSet<String>) {
    // Claude Code messages have content[] array with tool_use blocks
    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
        for block in content {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if block_type == "tool_use" {
                let tool_name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if let Some(input) = block.get("input") {
                    match tool_name {
                        "Write" | "Edit" | "file_write_tool" | "edit_tool" => {
                            if let Some(fp) = input
                                .get("file_path")
                                .or_else(|| input.get("path"))
                                .and_then(|v| v.as_str())
                            {
                                files.insert(crate::team::hooks::relativize_path(fp));
                            }
                        }
                        "NotebookEdit" => {
                            if let Some(fp) = input.get("notebook_path").and_then(|v| v.as_str()) {
                                files.insert(crate::team::hooks::relativize_path(fp));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Extract spawned agent IDs from tool results (Agent tool).
fn extract_agent_ids_from_message(msg: &serde_json::Value, ids: &mut HashSet<String>) {
    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
        for block in content {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if block_type == "tool_result" || block_type == "text" {
                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                    // Pattern: "agentId: <hex>" in tool result content
                    for line in text.lines() {
                        if let Some(rest) = line.strip_prefix("agentId: ") {
                            let id = rest.trim();
                            if !id.is_empty() {
                                ids.insert(id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A conversation message suitable for display.
#[derive(Debug, serde::Serialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    /// For tool_use: the tool name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// For tool_use: the file path if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Thinking/reasoning content from extended thinking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

/// Parse a transcript into displayable conversation messages.
/// Filters out system messages and deduplicates by UUID.
pub fn parse_conversation(path: &Path) -> Vec<ConversationMessage> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut messages = Vec::new();
    let mut seen_uuids: HashSet<String> = HashSet::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let tl: TranscriptLine = match serde_json::from_str(line) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Deduplicate by UUID — keep only the first occurrence
        if let Some(ref uuid) = tl.uuid {
            if !seen_uuids.insert(uuid.clone()) {
                continue;
            }
        }

        let msg = match tl.message {
            Some(ref m) => m,
            None => continue,
        };

        if tl.r#type == "user" {
            if !is_real_user_turn(msg) {
                continue;
            }
            let text = extract_text_content(msg);
            if !text.is_empty() {
                messages.push(ConversationMessage {
                    role: "user".to_string(),
                    content: text,
                    tool: None,
                    file: None,
                    thinking: None,
                });
            }
        } else if tl.r#type == "assistant" {
            // Extract thinking, text, and tool_use blocks
            if let Some(content_arr) = msg.get("content").and_then(|c| c.as_array()) {
                let mut thinking_parts = Vec::new();
                let mut text_parts = Vec::new();
                let mut tool_uses = Vec::new();

                for block in content_arr {
                    let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match block_type {
                        "thinking" => {
                            if let Some(t) = block.get("thinking").and_then(|t| t.as_str()) {
                                let trimmed = t.trim();
                                if !trimmed.is_empty() {
                                    thinking_parts.push(trimmed.to_string());
                                }
                            }
                        }
                        "reasoning" => {
                            if let Some(t) = block
                                .get("content")
                                .or_else(|| block.get("text"))
                                .and_then(|t| t.as_str())
                            {
                                let trimmed = t.trim();
                                if !trimmed.is_empty() {
                                    thinking_parts.push(trimmed.to_string());
                                }
                            }
                        }
                        "text" => {
                            if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                                let trimmed = t.trim();
                                if !trimmed.is_empty() {
                                    text_parts.push(trimmed.to_string());
                                }
                            }
                        }
                        "tool_use" => {
                            let name = block
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("unknown");
                            let file = block
                                .get("input")
                                .and_then(|i| {
                                    i.get("file_path")
                                        .or_else(|| i.get("path"))
                                        .or_else(|| i.get("command"))
                                })
                                .and_then(|v| v.as_str())
                                .map(|s| truncate_str(s, 200));
                            tool_uses.push((name.to_string(), file));
                        }
                        _ => {}
                    }
                }

                // Also check top-level reasoning_content
                if let Some(reasoning) = msg.get("reasoning_content").and_then(|r| r.as_str()) {
                    let trimmed = reasoning.trim();
                    if !trimmed.is_empty() {
                        thinking_parts.push(trimmed.to_string());
                    }
                }

                // Combine thinking into a single field
                let thinking = if thinking_parts.is_empty() {
                    None
                } else {
                    Some(thinking_parts.join("\n\n"))
                };

                // Add text message if any (with thinking attached)
                if !text_parts.is_empty() || thinking.is_some() {
                    messages.push(ConversationMessage {
                        role: "assistant".to_string(),
                        content: text_parts.join("\n\n"),
                        tool: None,
                        file: None,
                        thinking,
                    });
                }

                // Add tool uses
                for (tool_name, file) in tool_uses {
                    messages.push(ConversationMessage {
                        role: "tool".to_string(),
                        content: String::new(),
                        tool: Some(tool_name),
                        file,
                        thinking: None,
                    });
                }
            }
        }
    }

    messages
}

/// Extract text content from a message value.
fn extract_text_content(msg: &serde_json::Value) -> String {
    if let Some(text) = msg.get("content").and_then(|c| c.as_str()) {
        return text.to_string();
    }
    if let Some(blocks) = msg.get("content").and_then(|c| c.as_array()) {
        let parts: Vec<&str> = blocks
            .iter()
            .filter_map(|b| {
                if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                    b.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect();
        return parts.join("\n");
    }
    String::new()
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// Copy a transcript file into `.git/chub/transcripts/` for local review.
/// The copy is stored as `<session_id>.jsonl` inside the repo's `.git` directory
/// so it won't be committed but is available for LLM review.
pub fn archive_transcript_to_git(transcript_path: &Path, session_id: &str) -> Option<PathBuf> {
    // Find the .git directory
    let git_dir = find_git_dir()?;
    let dest_dir = git_dir.join("chub").join("transcripts");
    fs::create_dir_all(&dest_dir).ok()?;

    let dest = dest_dir.join(format!("{}.jsonl", session_id));
    fs::copy(transcript_path, &dest).ok()?;
    Some(dest)
}

/// Find the `.git` directory for the current repo.
fn find_git_dir() -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let p = PathBuf::from(&path_str);
    if p.is_dir() {
        Some(if p.is_absolute() {
            p
        } else {
            std::env::current_dir().ok()?.join(p)
        })
    } else {
        None
    }
}

/// Get the byte size of a transcript file.
pub fn transcript_size(path: &Path) -> i64 {
    fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0)
}

/// Count lines in a transcript file.
pub fn transcript_line_count(path: &Path) -> i64 {
    fs::read_to_string(path)
        .map(|c| c.lines().count() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Diff tracking
// ---------------------------------------------------------------------------

/// Get files modified since a given commit using git diff-tree.
pub fn get_diff_files(base_commit: &str) -> DiffResult {
    let mut result = DiffResult::default();

    // Get diff against base commit
    let output = std::process::Command::new("git")
        .args(["diff", "--name-status", base_commit, "HEAD"])
        .output();

    if let Ok(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let status = parts[0];
                let file = parts[1].to_string();
                match status {
                    "A" => result.new_files.push(file),
                    "D" => result.deleted_files.push(file),
                    _ => result.modified_files.push(file), // M, R, C, etc.
                }
            }
        }
    }

    result
}

/// Calculate line attribution between two commits.
pub fn calculate_attribution(base_commit: &str) -> Option<super::types::InitialAttribution> {
    // Get total diff stats
    let output = std::process::Command::new("git")
        .args(["diff", "--numstat", base_commit, "HEAD"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut agent_added: i64 = 0;
    let mut agent_removed: i64 = 0;

    for line in text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            if let (Ok(added), Ok(removed)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                agent_added += added;
                agent_removed += removed;
            }
        }
    }

    let total = agent_added;
    let percentage = if total > 0 {
        (agent_added as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Some(super::types::InitialAttribution {
        calculated_at: crate::util::now_iso8601(),
        agent_lines: agent_added,
        human_added: 0,
        human_modified: 0,
        human_removed: agent_removed,
        total_committed: total,
        agent_percentage: percentage,
    })
}

#[derive(Debug, Default)]
pub struct DiffResult {
    pub modified_files: Vec<String>,
    pub new_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

impl DiffResult {
    pub fn all_files(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(self.modified_files.iter().cloned());
        all.extend(self.new_files.iter().cloned());
        all.extend(self.deleted_files.iter().cloned());
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_path() {
        assert_eq!(
            sanitize_path_for_claude("/home/user/my-project"),
            "-home-user-my-project"
        );
        assert_eq!(
            sanitize_path_for_claude("D:\\PWorkspaces\\Context\\chub"),
            "D--PWorkspaces-Context-chub"
        );
    }

    #[test]
    fn parse_empty_transcript() {
        let dir = std::env::temp_dir().join("chub-test-transcript");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("empty.jsonl");
        let _ = fs::write(&path, "");
        let analysis = parse_transcript(&path);
        assert!(analysis.token_usage.is_empty());
        assert_eq!(analysis.turn_count, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_transcript_with_usage() {
        let dir = std::env::temp_dir().join("chub-test-transcript2");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.jsonl");

        let content = r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hello"}}
{"type":"assistant","uuid":"a1","message":{"id":"msg_1","role":"assistant","content":[{"type":"text","text":"hi"}],"usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":10,"cache_creation_input_tokens":5}}}
{"type":"user","uuid":"u2","message":{"role":"user","content":"edit file"}}
{"type":"assistant","uuid":"a2","message":{"id":"msg_2","role":"assistant","content":[{"type":"tool_use","name":"Write","input":{"file_path":"/src/main.rs","content":"fn main()"}}],"usage":{"input_tokens":200,"output_tokens":100}}}
"#;
        let _ = fs::write(&path, content);
        let analysis = parse_transcript(&path);
        assert_eq!(analysis.token_usage.input_tokens, 300);
        assert_eq!(analysis.token_usage.output_tokens, 150);
        assert_eq!(analysis.token_usage.cache_read_tokens, 10);
        assert_eq!(analysis.token_usage.api_call_count, 2);
        assert_eq!(analysis.turn_count, 2);
        assert!(analysis
            .modified_files
            .contains(&"/src/main.rs".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_thinking_blocks() {
        let dir = std::env::temp_dir().join("chub-test-thinking");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("thinking.jsonl");

        let content = r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"analyze this"}}
{"type":"assistant","uuid":"a1","message":{"id":"msg_1","role":"assistant","model":"claude-opus-4-6","content":[{"type":"thinking","thinking":"Let me analyze the code structure first."},{"type":"text","text":"I'll analyze this for you."},{"type":"tool_use","name":"Read","input":{"file_path":"src/main.rs"}}],"usage":{"input_tokens":5000,"output_tokens":2000,"reasoning_tokens":3500}}}
{"type":"user","uuid":"u2","message":{"role":"user","content":"refactor it"}}
{"type":"assistant","uuid":"a2","message":{"id":"msg_2","role":"assistant","content":[{"type":"thinking","thinking":"The user wants a refactor."},{"type":"text","text":"Refactoring now."}],"usage":{"input_tokens":8000,"output_tokens":3000,"reasoning_tokens":2000}}}
"#;
        let _ = fs::write(&path, content);
        let analysis = parse_transcript(&path);

        // Verify reasoning tokens extracted
        assert_eq!(analysis.token_usage.reasoning_tokens, 5500);
        assert!(analysis.has_extended_thinking);

        // Verify thinking blocks captured
        assert_eq!(analysis.thinking_block_count, 2);
        assert_eq!(analysis.thinking_blocks.len(), 2);
        assert!(analysis.thinking_blocks[0]
            .content
            .contains("analyze the code"));
        assert!(analysis.thinking_blocks[1]
            .content
            .contains("wants a refactor"));

        // Verify other tokens still correct
        assert_eq!(analysis.token_usage.input_tokens, 13000);
        assert_eq!(analysis.token_usage.output_tokens, 5000);
        assert_eq!(analysis.model, Some("claude-opus-4-6".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_conversation_with_thinking() {
        let dir = std::env::temp_dir().join("chub-test-conv-thinking");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("conv.jsonl");

        let content = r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hello"}}
{"type":"assistant","uuid":"a1","message":{"id":"msg_1","role":"assistant","content":[{"type":"thinking","thinking":"User said hello, I should greet back."},{"type":"text","text":"Hi there!"}],"usage":{"input_tokens":100,"output_tokens":50}}}
"#;
        let _ = fs::write(&path, content);
        let messages = parse_conversation(&path);

        assert_eq!(messages.len(), 2);
        // User message has no thinking
        assert!(messages[0].thinking.is_none());
        // Assistant message has thinking
        assert!(messages[1].thinking.is_some());
        assert!(messages[1]
            .thinking
            .as_ref()
            .unwrap()
            .contains("greet back"));
        assert_eq!(messages[1].content, "Hi there!");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dedup_by_message_id() {
        let dir = std::env::temp_dir().join("chub-test-dedup");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("dedup.jsonl");

        // Same message.id appears twice (streaming update)
        let content = r#"{"type":"assistant","uuid":"a1","message":{"id":"msg_1","role":"assistant","content":[],"usage":{"input_tokens":100,"output_tokens":50}}}
{"type":"assistant","uuid":"a1-update","message":{"id":"msg_1","role":"assistant","content":[],"usage":{"input_tokens":100,"output_tokens":80}}}
"#;
        let _ = fs::write(&path, content);
        let analysis = parse_transcript(&path);
        // Should use the LAST value for msg_1 (80, not 50)
        assert_eq!(analysis.token_usage.output_tokens, 80);
        assert_eq!(analysis.token_usage.api_call_count, 1); // deduplicated
        let _ = fs::remove_dir_all(&dir);
    }
}
