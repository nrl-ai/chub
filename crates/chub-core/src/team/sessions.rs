//! Session model and summary persistence for AI usage tracking.
//!
//! Team-visible session summaries are stored as YAML in `.chub/sessions/`.
//! Full transcripts go to `.git/chub-sessions/` (local-only, via session_journal).

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::team::project::{find_project_root, project_chub_dir};
use crate::util::now_iso8601;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// Token usage breakdown for a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub input: u64,
    #[serde(default)]
    pub output: u64,
    #[serde(default)]
    pub cache_read: u64,
    #[serde(default)]
    pub cache_write: u64,
    /// Extended thinking / reasoning tokens.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub reasoning: u64,
}

fn is_zero_u64(v: &u64) -> bool {
    *v == 0
}

impl TokenUsage {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write + self.reasoning
    }

    pub fn add(&mut self, other: &TokenUsage) {
        self.input += other.input;
        self.output += other.output;
        self.cache_read += other.cache_read;
        self.cache_write += other.cache_write;
        self.reasoning += other.reasoning;
    }
}

/// Environment snapshot captured at session start.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Environment {
    /// Operating system (e.g. "windows", "macos", "linux").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// CPU architecture (e.g. "x86_64", "aarch64").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// Git branch at session start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Repository name (from remote URL or directory name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Git user.name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_user: Option<String>,
    /// Git user.email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_email: Option<String>,
    /// Chub CLI version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chub_version: Option<String>,
    /// Whether extended thinking / reasoning was used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extended_thinking: Option<bool>,
}

impl Environment {
    /// Capture the current environment.
    pub fn capture() -> Self {
        Self {
            os: Some(std::env::consts::OS.to_string()),
            arch: Some(std::env::consts::ARCH.to_string()),
            branch: git_config_value(&["rev-parse", "--abbrev-ref", "HEAD"]),
            repo: detect_repo_name(),
            git_user: git_config_value(&["config", "user.name"]),
            git_email: git_config_value(&["config", "user.email"]),
            chub_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            extended_thinking: None, // set later from transcript analysis
        }
    }
}

fn git_config_value(args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() || s == "HEAD" {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            }
        })
}

fn detect_repo_name() -> Option<String> {
    // Try remote URL first
    if let Some(url) = git_config_value(&["config", "--get", "remote.origin.url"]) {
        // Parse repo name from git URL: git@github.com:user/repo.git or https://github.com/user/repo.git
        let name = url
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .or_else(|| url.trim_end_matches(".git").rsplit(':').next())
            .map(|s| s.to_string());
        if name.as_deref() != Some("") {
            return name;
        }
    }
    // Fall back to directory name
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
}

/// A session summary (team-visible, stored in `.chub/sessions/`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<u64>,
    #[serde(default)]
    pub turns: u32,
    #[serde(default)]
    pub tokens: TokenUsage,
    #[serde(default)]
    pub tool_calls: u32,
    #[serde(default)]
    pub tools_used: Vec<String>,
    #[serde(default)]
    pub files_changed: Vec<String>,
    #[serde(default)]
    pub commits: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub est_cost_usd: Option<f64>,
    /// Environment snapshot from session start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<Environment>,
}

/// Active session state (kept in `.git/chub-sessions/active.json`).
/// This tracks the in-progress session and gets finalized to a Session summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub session_id: String,
    pub agent: String,
    #[serde(default)]
    pub model: Option<String>,
    pub started_at: String,
    #[serde(default)]
    pub turns: u32,
    #[serde(default)]
    pub tokens: TokenUsage,
    #[serde(default)]
    pub tool_calls: u32,
    #[serde(default)]
    pub tools_used: HashSet<String>,
    #[serde(default)]
    pub files_changed: HashSet<String>,
    #[serde(default)]
    pub commits: Vec<String>,
    /// Environment snapshot from session start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<Environment>,
}

impl ActiveSession {
    /// Convert to a finalized Session summary.
    pub fn finalize(self) -> Session {
        let ended_at = now_iso8601();
        let duration_s = calc_duration_s(&self.started_at, &ended_at);
        let mut tools_used: Vec<String> = self.tools_used.into_iter().collect();
        tools_used.sort();
        let mut files_changed: Vec<String> = self.files_changed.into_iter().collect();
        files_changed.sort();

        Session {
            session_id: self.session_id,
            agent: self.agent,
            model: self.model,
            started_at: self.started_at,
            ended_at: Some(ended_at),
            duration_s,
            turns: self.turns,
            tokens: self.tokens,
            tool_calls: self.tool_calls,
            tools_used,
            files_changed,
            commits: self.commits,
            est_cost_usd: None, // Calculated by cost module
            env: self.env,
        }
    }
}

// ---------------------------------------------------------------------------
// Session ID generation
// ---------------------------------------------------------------------------

/// Generate a session ID in the format `YYYY-MM-DDTHH-MM-<6hex>`.
pub fn generate_session_id() -> String {
    let now = now_iso8601();
    // Take "2026-03-22T10:05:00" → "2026-03-22T10-05"
    let prefix = now.get(..16).unwrap_or(&now).replace(':', "-");
    let hex = random_hex(6);
    format!("{}-{}", prefix, hex)
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

// ---------------------------------------------------------------------------
// Active session persistence (.git/chub-sessions/)
// ---------------------------------------------------------------------------

/// Get the `.git/chub-sessions/` directory (local-only, not pushed).
pub fn git_sessions_dir() -> Option<PathBuf> {
    let project_root = find_project_root(None)?;
    let git_dir = project_root.join(".git");
    if git_dir.is_dir() {
        Some(git_dir.join("chub-sessions"))
    } else {
        None
    }
}

/// Get the active session, if any.
pub fn get_active_session() -> Option<ActiveSession> {
    let dir = git_sessions_dir()?;
    let path = dir.join("active.json");
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save the active session state.
pub fn save_active_session(session: &ActiveSession) -> bool {
    let dir = match git_sessions_dir() {
        Some(d) => d,
        None => return false,
    };
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("active.json");
    let json = match serde_json::to_string_pretty(session) {
        Ok(j) => j,
        Err(_) => return false,
    };
    crate::util::atomic_write(&path, json.as_bytes()).is_ok()
}

/// Clear the active session.
pub fn clear_active_session() -> bool {
    let dir = match git_sessions_dir() {
        Some(d) => d,
        None => return false,
    };
    let path = dir.join("active.json");
    if path.exists() {
        fs::remove_file(&path).is_ok()
    } else {
        true
    }
}

/// Start a new session. Returns the session ID.
pub fn start_session(agent: &str, model: Option<&str>) -> Option<String> {
    let session_id = generate_session_id();
    let session = ActiveSession {
        session_id: session_id.clone(),
        agent: agent.to_string(),
        model: model.map(|s| s.to_string()),
        started_at: now_iso8601(),
        turns: 0,
        tokens: TokenUsage::default(),
        tool_calls: 0,
        tools_used: HashSet::new(),
        files_changed: HashSet::new(),
        commits: Vec::new(),
        env: Some(Environment::capture()),
    };
    if save_active_session(&session) {
        Some(session_id)
    } else {
        None
    }
}

/// End the active session, finalize it, and write the summary.
pub fn end_session() -> Option<Session> {
    let active = get_active_session()?;
    let session = active.finalize();

    // Write summary to .chub/sessions/
    write_session_summary(&session);

    // Clear active state
    clear_active_session();

    Some(session)
}

// ---------------------------------------------------------------------------
// Session summaries
// Primary store: `.git/chub/sessions/` (local-only, fast reads)
// Team store: `chub/sessions/v1` orphan branch (pushed via pre-push hook)
// Legacy: `.chub/sessions/` (git-tracked, kept for migration reads)
// ---------------------------------------------------------------------------

const SESSIONS_BRANCH: &str = "chub/sessions/v1";

/// Primary session dir inside `.git` (local-only).
fn git_session_summaries_dir() -> Option<PathBuf> {
    let project_root = find_project_root(None)?;
    let git_dir = project_root.join(".git");
    if git_dir.is_dir() {
        Some(git_dir.join("chub").join("sessions"))
    } else {
        None
    }
}

/// Legacy session dir in `.chub/sessions/` (git-tracked, read-only for migration).
fn chub_sessions_dir() -> Option<PathBuf> {
    project_chub_dir().map(|d| d.join("sessions"))
}

/// Shard prefix for a session ID (last 2 hex chars of the random suffix).
/// Session IDs look like `2026-03-22T10-05-abc123` — uses `ab` as shard.
fn session_shard(session_id: &str) -> String {
    let len = session_id.len();
    if len >= 6 {
        session_id[len - 6..len - 4].to_string()
    } else {
        "00".to_string()
    }
}

/// Write a finalized session summary as YAML.
/// Writes to `.git/chub/sessions/` (local) and `chub/sessions/v1` (orphan branch).
pub fn write_session_summary(session: &Session) -> bool {
    let yaml = match serde_yaml::to_string(session) {
        Ok(y) => y,
        Err(_) => return false,
    };
    let filename = format!("{}.yaml", session.session_id);
    let mut wrote = false;

    // 1. Local fast store: .git/chub/sessions/
    if let Some(dir) = git_session_summaries_dir() {
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(&filename);
        if crate::util::atomic_write(&path, yaml.as_bytes()).is_ok() {
            wrote = true;
        }
    }

    // 2. Orphan branch: chub/sessions/v1 (team-visible, pushed via pre-push)
    let shard = session_shard(&session.session_id);
    let branch_path = format!("{}/{}", shard, filename);
    let files: Vec<(&str, &[u8])> = vec![(&branch_path, yaml.as_bytes())];
    let commit_msg = format!("Session: {}", session.session_id);
    if crate::team::tracking::branch_store::write_files(SESSIONS_BRANCH, &files, &commit_msg) {
        wrote = true;
    }

    wrote
}

/// List all session summaries, most recent first.
/// Reads from: 1) `.git/chub/sessions/` (local), 2) `chub/sessions/v1` branch,
/// 3) `.chub/sessions/` (legacy fallback). Deduplicates by session_id.
pub fn list_sessions(days: u64) -> Vec<Session> {
    let cutoff = now_secs().saturating_sub(days * 86400);
    let mut seen_ids = std::collections::HashSet::new();
    let mut sessions = Vec::new();

    // 1. Local filesystem dirs
    let dirs: Vec<Option<PathBuf>> = vec![git_session_summaries_dir(), chub_sessions_dir()];
    for dir in dirs.into_iter().flatten() {
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).ok().into_iter().flatten().flatten() {
            if entry
                .path()
                .extension()
                .map(|ext| ext == "yaml")
                .unwrap_or(false)
            {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(s) = serde_yaml::from_str::<Session>(&content) {
                        if parse_iso_to_secs(&s.started_at).unwrap_or(0) >= cutoff
                            && seen_ids.insert(s.session_id.clone())
                        {
                            sessions.push(s);
                        }
                    }
                }
            }
        }
    }

    // 2. Orphan branch (picks up team members' sessions after fetch)
    let branch_files = crate::team::tracking::branch_store::list_files(SESSIONS_BRANCH);
    for file in &branch_files {
        if file.ends_with(".yaml") {
            if let Some(content) =
                crate::team::tracking::branch_store::read_file(SESSIONS_BRANCH, file)
            {
                if let Ok(s) = serde_yaml::from_slice::<Session>(&content) {
                    if parse_iso_to_secs(&s.started_at).unwrap_or(0) >= cutoff
                        && seen_ids.insert(s.session_id.clone())
                    {
                        sessions.push(s);
                    }
                }
            }
        }
    }

    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    sessions
}

/// Get a session by ID.
/// Checks: 1) `.git/chub/sessions/`, 2) orphan branch, 3) `.chub/sessions/` (legacy).
pub fn get_session(session_id: &str) -> Option<Session> {
    let filename = format!("{}.yaml", session_id);

    // 1. Local fast store
    if let Some(dir) = git_session_summaries_dir() {
        let path = dir.join(&filename);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(s) = serde_yaml::from_str(&content) {
                return Some(s);
            }
        }
    }

    // 2. Orphan branch
    let shard = session_shard(session_id);
    let branch_path = format!("{}/{}", shard, filename);
    if let Some(content) =
        crate::team::tracking::branch_store::read_file(SESSIONS_BRANCH, &branch_path)
    {
        if let Ok(s) = serde_yaml::from_slice(&content) {
            return Some(s);
        }
    }

    // 3. Legacy fallback
    let dir = chub_sessions_dir()?;
    let path = dir.join(&filename);
    let content = fs::read_to_string(&path).ok()?;
    serde_yaml::from_str(&content).ok()
}

/// Push the sessions branch to a remote.
pub fn push_sessions(remote: &str) -> bool {
    crate::team::tracking::branch_store::push_branch(SESSIONS_BRANCH, remote)
}

// ---------------------------------------------------------------------------
// Aggregate report
// ---------------------------------------------------------------------------

/// Aggregate stats across sessions.
#[derive(Debug, Clone, Serialize)]
pub struct SessionReport {
    pub period_days: u64,
    pub session_count: usize,
    pub total_duration_s: u64,
    pub total_tokens: TokenUsage,
    pub total_tool_calls: u32,
    pub total_est_cost_usd: f64,
    pub by_agent: Vec<(String, usize, f64)>, // (agent, sessions, cost)
    pub by_model: Vec<(String, usize, u64)>, // (model, sessions, tokens)
    pub top_tools: Vec<(String, u32)>,
}

pub fn generate_report(days: u64) -> SessionReport {
    let sessions = list_sessions(days);
    let mut total_tokens = TokenUsage::default();
    let mut total_duration_s = 0u64;
    let mut total_tool_calls = 0u32;
    let mut total_cost = 0.0f64;
    let mut agent_map: std::collections::HashMap<String, (usize, f64)> =
        std::collections::HashMap::new();
    let mut model_map: std::collections::HashMap<String, (usize, u64)> =
        std::collections::HashMap::new();
    let mut tool_map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for s in &sessions {
        total_tokens.add(&s.tokens);
        total_duration_s += s.duration_s.unwrap_or(0);
        total_tool_calls += s.tool_calls;
        let cost = s.est_cost_usd.unwrap_or(0.0);
        total_cost += cost;

        let ae = agent_map.entry(s.agent.clone()).or_insert((0, 0.0));
        ae.0 += 1;
        ae.1 += cost;

        if let Some(ref model) = s.model {
            let me = model_map.entry(model.clone()).or_insert((0, 0));
            me.0 += 1;
            me.1 += s.tokens.total();
        }

        for tool in &s.tools_used {
            *tool_map.entry(tool.clone()).or_insert(0) +=
                s.tool_calls / s.tools_used.len().max(1) as u32;
        }
    }

    let mut by_agent: Vec<_> = agent_map.into_iter().map(|(k, v)| (k, v.0, v.1)).collect();
    by_agent.sort_by(|a, b| b.1.cmp(&a.1));

    let mut by_model: Vec<_> = model_map.into_iter().map(|(k, v)| (k, v.0, v.1)).collect();
    by_model.sort_by(|a, b| b.2.cmp(&a.2));

    let mut top_tools: Vec<_> = tool_map.into_iter().collect();
    top_tools.sort_by(|a, b| b.1.cmp(&a.1));

    SessionReport {
        period_days: days,
        session_count: sessions.len(),
        total_duration_s,
        total_tokens,
        total_tool_calls,
        total_est_cost_usd: total_cost,
        by_agent,
        by_model,
        top_tools,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Parse a simplified ISO 8601 timestamp to seconds since epoch.
fn parse_iso_to_secs(iso: &str) -> Option<u64> {
    // Parse "2026-03-22T10:05:00.000Z" or similar
    let clean = iso.trim().trim_end_matches('Z');
    let parts: Vec<&str> = clean.split('T').collect();
    if parts.len() != 2 {
        return None;
    }
    let date_parts: Vec<u64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    if date_parts.len() != 3 {
        return None;
    }
    let time_clean = parts[1].split('.').next()?;
    let time_parts: Vec<u64> = time_clean
        .split(':')
        .filter_map(|p| p.parse().ok())
        .collect();
    if time_parts.len() != 3 {
        return None;
    }

    let (y, m, d) = (date_parts[0], date_parts[1], date_parts[2]);
    let (h, min, s) = (time_parts[0], time_parts[1], time_parts[2]);

    // Simplified days calculation (good enough for relative comparisons)
    let days = y * 365 + y / 4 - y / 100 + y / 400 + (m * 30) + d;
    Some(days * 86400 + h * 3600 + min * 60 + s)
}

fn calc_duration_s(start: &str, end: &str) -> Option<u64> {
    let s = parse_iso_to_secs(start)?;
    let e = parse_iso_to_secs(end)?;
    Some(e.saturating_sub(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_format() {
        let id = generate_session_id();
        assert!(id.len() > 20, "session ID should be substantial: {}", id);
        assert!(
            id.contains('T'),
            "session ID should contain T separator: {}",
            id
        );
    }

    #[test]
    fn token_usage_add() {
        let mut a = TokenUsage {
            input: 100,
            output: 50,
            cache_read: 10,
            cache_write: 5,
            ..Default::default()
        };
        let b = TokenUsage {
            input: 200,
            output: 100,
            cache_read: 20,
            cache_write: 10,
            ..Default::default()
        };
        a.add(&b);
        assert_eq!(a.input, 300);
        assert_eq!(a.output, 150);
        assert_eq!(a.total(), 300 + 150 + 30 + 15);
    }

    #[test]
    fn parse_iso_round_trip() {
        let ts = "2026-03-22T10:05:00.000Z";
        let secs = parse_iso_to_secs(ts);
        assert!(secs.is_some());
    }

    #[test]
    fn session_yaml_roundtrip() {
        let session = Session {
            session_id: "2026-03-22T10-05-abc123".to_string(),
            agent: "claude-code".to_string(),
            model: Some("claude-opus-4-6".to_string()),
            started_at: "2026-03-22T10:05:00.000Z".to_string(),
            ended_at: Some("2026-03-22T10:42:00.000Z".to_string()),
            duration_s: Some(2220),
            turns: 14,
            tokens: TokenUsage {
                input: 45000,
                output: 12000,
                cache_read: 8000,
                cache_write: 3000,
                ..Default::default()
            },
            tool_calls: 23,
            tools_used: vec!["Read".to_string(), "Edit".to_string()],
            files_changed: vec!["src/main.rs".to_string()],
            commits: vec!["abc1234".to_string()],
            est_cost_usd: Some(0.85),
            env: None,
        };
        let yaml = serde_yaml::to_string(&session).unwrap();
        let parsed: Session = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.session_id, session.session_id);
        assert_eq!(parsed.tokens.input, 45000);
        assert_eq!(parsed.est_cost_usd, Some(0.85));
    }

    #[test]
    fn environment_capture_returns_os_and_arch() {
        let env = Environment::capture();
        assert!(env.os.is_some(), "os should be captured");
        assert!(env.arch.is_some(), "arch should be captured");
        assert!(env.chub_version.is_some(), "chub_version should be set");
        // extended_thinking starts as None
        assert!(env.extended_thinking.is_none());
    }

    #[test]
    fn session_yaml_roundtrip_with_env() {
        let session = Session {
            session_id: "2026-03-22T10-05-env123".to_string(),
            agent: "claude-code".to_string(),
            model: Some("claude-opus-4-6".to_string()),
            started_at: "2026-03-22T10:05:00.000Z".to_string(),
            ended_at: Some("2026-03-22T10:42:00.000Z".to_string()),
            duration_s: Some(2220),
            turns: 14,
            tokens: TokenUsage::default(),
            tool_calls: 0,
            tools_used: vec![],
            files_changed: vec![],
            commits: vec![],
            est_cost_usd: None,
            env: Some(Environment {
                os: Some("windows".to_string()),
                arch: Some("x86_64".to_string()),
                branch: Some("main".to_string()),
                repo: Some("my-project".to_string()),
                git_user: Some("Jane".to_string()),
                git_email: Some("jane@chub.nrl.ai".to_string()),
                chub_version: Some("0.1.15".to_string()),
                extended_thinking: Some(true),
            }),
        };
        let yaml = serde_yaml::to_string(&session).unwrap();
        assert!(yaml.contains("os: windows"));
        assert!(yaml.contains("extended_thinking: true"));

        let parsed: Session = serde_yaml::from_str(&yaml).unwrap();
        let env = parsed.env.unwrap();
        assert_eq!(env.os.as_deref(), Some("windows"));
        assert_eq!(env.arch.as_deref(), Some("x86_64"));
        assert_eq!(env.branch.as_deref(), Some("main"));
        assert_eq!(env.repo.as_deref(), Some("my-project"));
        assert_eq!(env.extended_thinking, Some(true));
    }

    // --- Session shard ---

    #[test]
    fn session_shard_normal_id() {
        // "2026-03-22T10-05-abc123" → shard = "ab" (chars at [-6..-4])
        assert_eq!(session_shard("2026-03-22T10-05-abc123"), "ab");
    }

    #[test]
    fn session_shard_various_ids() {
        assert_eq!(session_shard("2026-03-28T04-54-9e3efd"), "9e");
        assert_eq!(session_shard("2026-03-28T04-54-64bd27"), "64");
        assert_eq!(session_shard("2026-03-28T11-22-ff0011"), "ff");
    }

    #[test]
    fn session_shard_short_id() {
        assert_eq!(session_shard("abc"), "00", "short IDs should return 00");
        assert_eq!(session_shard(""), "00");
    }

    // --- parse_iso_to_secs edge cases ---

    #[test]
    fn parse_iso_missing_time() {
        assert!(parse_iso_to_secs("2026-03-22").is_none());
    }

    #[test]
    fn parse_iso_garbage() {
        assert!(parse_iso_to_secs("").is_none());
        assert!(parse_iso_to_secs("not-a-date").is_none());
        assert!(parse_iso_to_secs("T10:00:00").is_none());
    }

    #[test]
    fn parse_iso_with_z_suffix() {
        let with_z = parse_iso_to_secs("2026-03-22T10:05:30.000Z");
        let without_z = parse_iso_to_secs("2026-03-22T10:05:30.000");
        assert_eq!(with_z, without_z, "Z suffix should not affect parsing");
    }

    #[test]
    fn parse_iso_with_milliseconds() {
        let result = parse_iso_to_secs("2026-03-22T10:05:30.123Z");
        assert!(result.is_some());
    }

    // --- calc_duration_s ---

    #[test]
    fn duration_same_time_is_zero() {
        let d = calc_duration_s("2026-03-22T10:00:00.000Z", "2026-03-22T10:00:00.000Z");
        assert_eq!(d, Some(0));
    }

    #[test]
    fn duration_one_hour() {
        let d = calc_duration_s("2026-03-22T10:00:00.000Z", "2026-03-22T11:00:00.000Z");
        assert_eq!(d, Some(3600));
    }

    #[test]
    fn duration_end_before_start_is_zero() {
        let d = calc_duration_s("2026-03-22T11:00:00.000Z", "2026-03-22T10:00:00.000Z");
        assert_eq!(d, Some(0), "saturating_sub should prevent underflow");
    }

    // --- ActiveSession finalize ---

    #[test]
    fn active_session_finalize_sorts_fields() {
        let active = ActiveSession {
            session_id: "test-123".to_string(),
            agent: "claude-code".to_string(),
            model: Some("opus".to_string()),
            started_at: "2026-03-22T10:00:00.000Z".to_string(),
            turns: 5,
            tokens: TokenUsage {
                input: 1000,
                output: 500,
                ..Default::default()
            },
            tool_calls: 3,
            tools_used: ["Edit", "Read", "Bash"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            files_changed: ["z.rs", "a.rs", "m.rs"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            commits: vec!["abc".to_string(), "def".to_string()],
            env: None,
        };

        let session = active.finalize();
        assert_eq!(session.agent, "claude-code");
        assert!(session.ended_at.is_some());
        assert!(session.duration_s.is_some());
        // tools_used and files_changed should be sorted
        assert_eq!(session.tools_used, vec!["Bash", "Edit", "Read"]);
        assert_eq!(session.files_changed, vec!["a.rs", "m.rs", "z.rs"]);
        assert_eq!(session.commits, vec!["abc", "def"]);
    }

    // --- Token usage ---

    #[test]
    fn token_usage_total_with_reasoning() {
        let t = TokenUsage {
            input: 100,
            output: 50,
            cache_read: 10,
            cache_write: 5,
            reasoning: 200,
        };
        assert_eq!(t.total(), 365);
    }

    // --- Environment ---

    #[test]
    fn environment_default_all_none() {
        let env = Environment::default();
        assert!(env.os.is_none());
        assert!(env.arch.is_none());
        assert!(env.branch.is_none());
        assert!(env.repo.is_none());
        assert!(env.git_user.is_none());
        assert!(env.git_email.is_none());
        assert!(env.chub_version.is_none());
        assert!(env.extended_thinking.is_none());
    }

    #[test]
    fn environment_none_fields_skipped_in_yaml() {
        let session = Session {
            session_id: "test".to_string(),
            agent: "test".to_string(),
            model: None,
            started_at: "2026-01-01T00:00:00.000Z".to_string(),
            ended_at: None,
            duration_s: None,
            turns: 0,
            tokens: TokenUsage::default(),
            tool_calls: 0,
            tools_used: vec![],
            files_changed: vec![],
            commits: vec![],
            est_cost_usd: None,
            env: None,
        };
        let yaml = serde_yaml::to_string(&session).unwrap();
        assert!(!yaml.contains("env:"), "env should be omitted when None");
        assert!(
            !yaml.contains("model:"),
            "model should be omitted when None"
        );
        assert!(
            !yaml.contains("est_cost_usd:"),
            "cost should be omitted when None"
        );
    }
}
