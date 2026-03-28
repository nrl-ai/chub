//! Session state management — compatible with entire.io's session state format.
//!
//! State files stored at `.git/entire-sessions/<session-id>.json` for
//! cross-tool compatibility.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::types::{CheckpointID, Phase, PromptAttribution, TokenUsage};
use crate::team::project::find_project_root;
use crate::util::now_iso8601;

// ---------------------------------------------------------------------------
// Session state — entire.io compatible
// ---------------------------------------------------------------------------

/// Session state persisted at `.git/entire-sessions/<session-id>.json`.
/// Field names and JSON format match entire.io's `session.State`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    /// entire.io uses `sessionID` (camelCase with capital ID).
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_version: Option<String>,
    #[serde(default)]
    pub base_commit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution_base_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "worktreeID")]
    pub worktree_id: Option<String>,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(default)]
    pub phase: Phase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "turnID")]
    pub turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "turnCheckpointIDs")]
    pub turn_checkpoint_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_interaction_time: Option<String>,
    #[serde(default)]
    pub step_count: i32,
    #[serde(default)]
    pub checkpoint_transcript_start: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub untracked_files_at_start: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_touched: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastCheckpointID")]
    pub last_checkpoint_id: Option<CheckpointID>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_identifier_at_start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prompt_attributions: Vec<PromptAttribution>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_prompt_attribution: Option<PromptAttribution>,

    // --- chub extensions (not in entire.io, skipped if empty) ---
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub tools_used: HashSet<String>,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub tool_calls: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commits: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub est_cost_usd: Option<f64>,
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

impl SessionState {
    /// Create a new session state.
    pub fn new(agent: &str, _model: Option<&str>) -> Self {
        let base_commit = get_head_commit().unwrap_or_default();
        Self {
            session_id: generate_session_id(),
            cli_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            base_commit: base_commit.clone(),
            attribution_base_commit: Some(base_commit),
            worktree_path: None,
            worktree_id: None,
            started_at: now_iso8601(),
            ended_at: None,
            phase: Phase::Active,
            turn_id: None,
            turn_checkpoint_ids: Vec::new(),
            last_interaction_time: Some(now_iso8601()),
            step_count: 0,
            checkpoint_transcript_start: 0,
            untracked_files_at_start: Vec::new(),
            files_touched: Vec::new(),
            last_checkpoint_id: None,
            agent_type: Some(agent_type_display(agent)),
            token_usage: Some(TokenUsage::default()),
            transcript_identifier_at_start: None,
            transcript_path: None,
            first_prompt: None,
            prompt_attributions: Vec::new(),
            pending_prompt_attribution: None,
            tools_used: HashSet::new(),
            tool_calls: 0,
            commits: Vec::new(),
            est_cost_usd: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase state machine
// ---------------------------------------------------------------------------

/// Events that drive session phase transitions.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    SessionStart,
    TurnStart,
    TurnEnd,
    GitCommit,
    SessionStop,
    Compaction,
}

impl SessionState {
    /// Apply a state machine event. Returns true if the transition was valid.
    pub fn apply_event(&mut self, event: SessionEvent) -> bool {
        let now = now_iso8601();
        match (&self.phase, event) {
            // IDLE → ACTIVE on turn start
            (Phase::Idle, SessionEvent::TurnStart) => {
                self.phase = Phase::Active;
                self.last_interaction_time = Some(now);
                self.step_count += 1;
                true
            }
            // IDLE → ENDED on session stop
            (Phase::Idle, SessionEvent::SessionStop) => {
                self.phase = Phase::Ended;
                self.ended_at = Some(now.clone());
                self.last_interaction_time = Some(now);
                true
            }
            // IDLE + git commit → stay IDLE (condense)
            (Phase::Idle, SessionEvent::GitCommit) => {
                self.last_interaction_time = Some(now);
                true
            }
            // ACTIVE → IDLE on turn end
            (Phase::Active, SessionEvent::TurnEnd) => {
                self.phase = Phase::Idle;
                self.last_interaction_time = Some(now);
                true
            }
            // ACTIVE + git commit → stay ACTIVE (condense)
            (Phase::Active, SessionEvent::GitCommit) => {
                self.last_interaction_time = Some(now);
                true
            }
            // ACTIVE + compaction → stay ACTIVE (save checkpoint)
            (Phase::Active, SessionEvent::Compaction) => {
                self.last_interaction_time = Some(now);
                true
            }
            // ACTIVE → stop directly
            (Phase::Active, SessionEvent::SessionStop) => {
                self.phase = Phase::Ended;
                self.ended_at = Some(now.clone());
                self.last_interaction_time = Some(now);
                true
            }
            // ENDED + turn start → re-activate
            (Phase::Ended, SessionEvent::TurnStart) => {
                self.phase = Phase::Active;
                self.ended_at = None;
                self.last_interaction_time = Some(now);
                self.step_count += 1;
                true
            }
            // ENDED + git commit with files → condense
            (Phase::Ended, SessionEvent::GitCommit) => {
                if !self.files_touched.is_empty() {
                    self.last_interaction_time = Some(now);
                }
                true
            }
            _ => false,
        }
    }

    /// Add a file to the touched set. Absolute paths are relativized to the project root.
    pub fn touch_file(&mut self, path: &str) {
        let relative = crate::team::hooks::relativize_path(path);
        let normalized = relative.replace('\\', "/");
        if !self.files_touched.contains(&normalized) {
            self.files_touched.push(normalized);
        }
    }

    /// Add token usage from a tool call or response.
    pub fn add_tokens(&mut self, tokens: &TokenUsage) {
        let usage = self.token_usage.get_or_insert_with(TokenUsage::default);
        usage.add(tokens);
    }
}

// ---------------------------------------------------------------------------
// Session state persistence (.git/entire-sessions/)
// ---------------------------------------------------------------------------

/// Get the sessions directory — uses `.git/entire-sessions/` for
/// compatibility with entire.io. Falls back to `.git/chub-sessions/`.
fn sessions_dir() -> Option<PathBuf> {
    let project_root = find_project_root(None)?;
    let git_dir = project_root.join(".git");
    if git_dir.is_dir() {
        // Prefer entire-sessions for compatibility
        Some(git_dir.join("entire-sessions"))
    } else {
        None
    }
}

/// Save session state to disk.
pub fn save_state(state: &SessionState) -> bool {
    let dir = match sessions_dir() {
        Some(d) => d,
        None => return false,
    };
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.json", state.session_id));
    let json = match serde_json::to_string_pretty(state) {
        Ok(j) => j + "\n", // entire.io terminates with newline
        Err(_) => return false,
    };
    crate::util::atomic_write(&path, json.as_bytes()).is_ok()
}

/// Load a session state by ID.
pub fn load_state(session_id: &str) -> Option<SessionState> {
    let dir = sessions_dir()?;
    let path = dir.join(format!("{}.json", session_id));
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Delete a session state file.
pub fn delete_state(session_id: &str) -> bool {
    let dir = match sessions_dir() {
        Some(d) => d,
        None => return false,
    };
    let path = dir.join(format!("{}.json", session_id));
    fs::remove_file(&path).is_ok()
}

/// List all session states, filtering out stale ones (>7 days old).
pub fn list_states() -> Vec<SessionState> {
    let dir = match sessions_dir() {
        Some(d) => d,
        None => return vec![],
    };
    if !dir.is_dir() {
        return vec![];
    }

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let stale_threshold = 7 * 86400; // 7 days

    let mut states = Vec::new();
    let mut stale_ids = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(state) = serde_json::from_str::<SessionState>(&content) {
                        // Check staleness
                        let last_time = state
                            .last_interaction_time
                            .as_deref()
                            .unwrap_or(&state.started_at);
                        if is_stale(last_time, now_secs, stale_threshold) {
                            stale_ids.push(state.session_id.clone());
                            continue;
                        }
                        states.push(state);
                    }
                }
            }
        }
    }

    // Auto-clean stale sessions
    for id in stale_ids {
        delete_state(&id);
    }

    states.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    states
}

/// Get the currently active session (phase != Ended).
pub fn get_active_state() -> Option<SessionState> {
    list_states().into_iter().find(|s| s.phase != Phase::Ended)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate a session ID in YYYY-MM-DD-<uuid> format (entire.io compatible).
fn generate_session_id() -> String {
    let now = now_iso8601();
    let date = now.get(..10).unwrap_or("0000-00-00");
    let hex = random_hex(8);
    format!("{}-{}", date, hex)
}

fn random_hex(len: usize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    let hash = hasher.finish();
    let hex = format!("{:016x}", hash);
    hex[..len.min(16)].to_string()
}

/// Convert agent CLI name to entire.io display name.
/// entire.io uses: "Claude Code", "Cursor IDE", "Gemini CLI", "OpenCode", "Agent"
fn agent_type_display(agent: &str) -> String {
    match agent.to_lowercase().as_str() {
        "claude-code" | "claude" | "claudecode" => "Claude Code".to_string(),
        "cursor" => "Cursor IDE".to_string(),
        "gemini-cli" | "gemini" => "Gemini CLI".to_string(),
        "copilot" | "github-copilot" => "GitHub Copilot".to_string(),
        "opencode" => "OpenCode".to_string(),
        "aider" => "Aider".to_string(),
        "codex" => "Codex".to_string(),
        "windsurf" => "Windsurf".to_string(),
        "cline" => "Cline".to_string(),
        _ => "Agent".to_string(),
    }
}

fn get_head_commit() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
}

fn is_stale(iso_time: &str, now_secs: u64, threshold_secs: u64) -> bool {
    // Simple heuristic: parse year/month/day and approximate
    let parts: Vec<&str> = iso_time.split('T').collect();
    if parts.is_empty() {
        return false;
    }
    let date_parts: Vec<u64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    if date_parts.len() != 3 {
        return false;
    }
    let (y, m, d) = (date_parts[0], date_parts[1], date_parts[2]);
    let approx_secs = y * 365 * 86400 + m * 30 * 86400 + d * 86400;

    let now_y = now_secs / (365 * 86400);
    let now_approx = now_y * 365 * 86400
        + ((now_secs % (365 * 86400)) / (30 * 86400)) * 30 * 86400
        + ((now_secs % (30 * 86400)) / 86400) * 86400;

    now_approx.saturating_sub(approx_secs) > threshold_secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_state_json_roundtrip() {
        let state = SessionState::new("claude-code", Some("claude-opus-4-6"));
        let json = serde_json::to_string_pretty(&state).unwrap();
        let parsed: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, state.session_id);
        assert_eq!(parsed.phase, Phase::Active);
        assert!(json.contains("\"sessionID\"")); // camelCase field names
        assert!(json.contains("\"stepCount\"")); // entire.io field name
        assert!(json.contains("\"startedAt\""));
        assert!(json.contains("\"agentType\""));
    }

    #[test]
    fn phase_transitions() {
        let mut state = SessionState::new("claude-code", None);
        // Active → TurnEnd → Idle
        assert!(state.apply_event(SessionEvent::TurnEnd));
        assert_eq!(state.phase, Phase::Idle);
        // Idle → TurnStart → Active
        assert!(state.apply_event(SessionEvent::TurnStart));
        assert_eq!(state.phase, Phase::Active);
        assert_eq!(state.step_count, 1); // new() starts at 0, TurnStart increments to 1
                                         // Active → Stop → Ended
        assert!(state.apply_event(SessionEvent::SessionStop));
        assert_eq!(state.phase, Phase::Ended);
        // Ended → TurnStart → Active (re-activate)
        assert!(state.apply_event(SessionEvent::TurnStart));
        assert_eq!(state.phase, Phase::Active);
    }

    #[test]
    fn touch_file_deduplicates() {
        let mut state = SessionState::new("test", None);
        state.touch_file("src/main.rs");
        state.touch_file("src/main.rs");
        state.touch_file("src\\lib.rs"); // backslash normalized
        assert_eq!(state.files_touched.len(), 2);
        assert!(state.files_touched.contains(&"src/lib.rs".to_string()));
    }

    // --- Phase transition coverage ---

    #[test]
    fn invalid_transitions_rejected() {
        // IDLE → TurnEnd should be invalid
        let mut state = SessionState::new("test", None);
        state.phase = Phase::Idle;
        assert!(!state.apply_event(SessionEvent::TurnEnd));
        assert_eq!(state.phase, Phase::Idle, "phase should not change");

        // IDLE → Compaction should be invalid
        let mut state2 = SessionState::new("test", None);
        state2.phase = Phase::Idle;
        assert!(!state2.apply_event(SessionEvent::Compaction));

        // ENDED → TurnEnd should be invalid
        let mut state3 = SessionState::new("test", None);
        state3.phase = Phase::Ended;
        assert!(!state3.apply_event(SessionEvent::TurnEnd));

        // ENDED → SessionStop should be invalid
        let mut state4 = SessionState::new("test", None);
        state4.phase = Phase::Ended;
        assert!(!state4.apply_event(SessionEvent::SessionStop));

        // ENDED → Compaction should be invalid
        let mut state5 = SessionState::new("test", None);
        state5.phase = Phase::Ended;
        assert!(!state5.apply_event(SessionEvent::Compaction));
    }

    #[test]
    fn idle_git_commit_stays_idle() {
        let mut state = SessionState::new("test", None);
        state.phase = Phase::Idle;
        assert!(state.apply_event(SessionEvent::GitCommit));
        assert_eq!(state.phase, Phase::Idle);
    }

    #[test]
    fn active_git_commit_stays_active() {
        let mut state = SessionState::new("test", None);
        assert!(state.apply_event(SessionEvent::GitCommit));
        assert_eq!(state.phase, Phase::Active);
    }

    #[test]
    fn active_compaction_stays_active() {
        let mut state = SessionState::new("test", None);
        assert!(state.apply_event(SessionEvent::Compaction));
        assert_eq!(state.phase, Phase::Active);
    }

    #[test]
    fn ended_turn_start_reactivates() {
        let mut state = SessionState::new("test", None);
        state.apply_event(SessionEvent::SessionStop);
        assert_eq!(state.phase, Phase::Ended);
        assert!(state.ended_at.is_some());

        // Re-activate
        assert!(state.apply_event(SessionEvent::TurnStart));
        assert_eq!(state.phase, Phase::Active);
        assert!(state.ended_at.is_none(), "ended_at should be cleared");
    }

    #[test]
    fn ended_git_commit_with_files_stays_ended() {
        let mut state = SessionState::new("test", None);
        state.files_touched.push("src/main.rs".to_string());
        state.apply_event(SessionEvent::SessionStop);

        // GitCommit on Ended with files should succeed and stay Ended
        assert!(state.apply_event(SessionEvent::GitCommit));
        assert_eq!(state.phase, Phase::Ended, "should stay Ended");
        // last_interaction_time should be set (may or may not differ within same second)
        assert!(state.last_interaction_time.is_some());
    }

    #[test]
    fn step_count_increments_on_turn_start() {
        let mut state = SessionState::new("test", None);
        assert_eq!(state.step_count, 0);

        // Active → TurnEnd → Idle → TurnStart → Active
        state.apply_event(SessionEvent::TurnEnd);
        state.apply_event(SessionEvent::TurnStart);
        assert_eq!(state.step_count, 1);

        state.apply_event(SessionEvent::TurnEnd);
        state.apply_event(SessionEvent::TurnStart);
        assert_eq!(state.step_count, 2);
    }

    // --- Touch file edge cases ---

    #[test]
    fn touch_file_normalizes_backslashes() {
        let mut state = SessionState::new("test", None);
        state.touch_file("src\\nested\\deep\\file.rs");
        assert!(state
            .files_touched
            .contains(&"src/nested/deep/file.rs".to_string()));
    }

    #[test]
    fn touch_file_empty_string() {
        let mut state = SessionState::new("test", None);
        state.touch_file("");
        // Empty should still be added (it's a relative path)
        assert_eq!(state.files_touched.len(), 1);
    }

    // --- Token addition ---

    #[test]
    fn add_tokens_creates_usage_if_none() {
        let mut state = SessionState::new("test", None);
        state.token_usage = None;
        let tokens = super::TokenUsage {
            input_tokens: 500,
            output_tokens: 200,
            ..Default::default()
        };
        state.add_tokens(&tokens);
        assert!(state.token_usage.is_some());
        assert_eq!(state.token_usage.as_ref().unwrap().input_tokens, 500);
    }

    #[test]
    fn add_tokens_accumulates() {
        let mut state = SessionState::new("test", None);
        let t1 = super::TokenUsage {
            input_tokens: 100,
            ..Default::default()
        };
        let t2 = super::TokenUsage {
            input_tokens: 200,
            ..Default::default()
        };
        state.add_tokens(&t1);
        state.add_tokens(&t2);
        assert_eq!(state.token_usage.as_ref().unwrap().input_tokens, 300);
    }

    // --- Staleness ---

    #[test]
    fn is_stale_recent_date_not_stale() {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // A date from today should not be stale
        assert!(!is_stale("2026-03-28T12:00:00.000Z", now_secs, 7 * 86400));
    }

    #[test]
    fn is_stale_malformed_date() {
        assert!(!is_stale("not-a-date", 1000000, 86400));
        assert!(!is_stale("", 1000000, 86400));
        assert!(!is_stale("2026", 1000000, 86400));
    }

    // --- Session state JSON field names ---

    #[test]
    fn session_state_entire_io_field_names() {
        let mut state = SessionState::new("claude-code", None);
        // Add a file so filesTouched is not empty (empty vecs are skipped)
        state.touch_file("src/main.rs");

        let json = serde_json::to_string(&state).unwrap();

        // Must use entire.io's capital-ID convention
        assert!(
            json.contains("\"sessionID\""),
            "must be sessionID not sessionId"
        );
        assert!(json.contains("\"baseCommit\""));
        assert!(json.contains("\"startedAt\""));
        assert!(json.contains("\"filesTouched\""));
        assert!(json.contains("\"stepCount\""));
        assert!(json.contains("\"agentType\""));

        // These should only appear when set
        assert!(
            !json.contains("\"endedAt\""),
            "endedAt should be skipped when None"
        );
        assert!(
            !json.contains("\"worktreeID\""),
            "worktreeID should be skipped"
        );
        assert!(!json.contains("\"turnID\""), "turnID should be skipped");
    }

    #[test]
    fn session_state_roundtrip_with_all_fields() {
        let mut state = SessionState::new("cursor", Some("gpt-4o"));
        state.touch_file("src/app.tsx");
        state.tool_calls = 5;
        state.tools_used.insert("Read".to_string());
        state.tools_used.insert("Edit".to_string());
        state.commits.push("abc1234".to_string());
        state.est_cost_usd = Some(1.23);

        let json = serde_json::to_string_pretty(&state).unwrap();
        let parsed: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.session_id, state.session_id);
        assert_eq!(parsed.tool_calls, 5);
        assert!(parsed.tools_used.contains("Read"));
        assert!(parsed.tools_used.contains("Edit"));
        assert_eq!(parsed.commits, vec!["abc1234"]);
        assert_eq!(parsed.est_cost_usd, Some(1.23));
        assert_eq!(parsed.files_touched, vec!["src/app.tsx"]);
    }
}
