//! Data types compatible with entire.io checkpoint and session formats.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Agent types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "cursor")]
    Cursor,
    #[serde(rename = "gemini-cli")]
    GeminiCli,
    #[serde(rename = "copilot")]
    Copilot,
    #[serde(rename = "opencode")]
    OpenCode,
    #[serde(rename = "aider")]
    Aider,
    #[serde(rename = "codex")]
    Codex,
    #[serde(rename = "windsurf")]
    Windsurf,
    #[serde(rename = "cline")]
    Cline,
    #[serde(other, rename = "unknown")]
    Unknown,
}

impl AgentType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "claude-code" | "claude" | "claudecode" => Self::ClaudeCode,
            "cursor" => Self::Cursor,
            "gemini-cli" | "gemini" => Self::GeminiCli,
            "copilot" | "github-copilot" => Self::Copilot,
            "opencode" => Self::OpenCode,
            "aider" => Self::Aider,
            "codex" => Self::Codex,
            "windsurf" => Self::Windsurf,
            "cline" => Self::Cline,
            _ => Self::Unknown,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Cursor => "cursor",
            Self::GeminiCli => "gemini-cli",
            Self::Copilot => "copilot",
            Self::OpenCode => "opencode",
            Self::Aider => "aider",
            Self::Codex => "codex",
            Self::Windsurf => "windsurf",
            Self::Cline => "cline",
            Self::Unknown => "unknown",
        }
    }
}

// ---------------------------------------------------------------------------
// Token usage (entire.io compatible)
// ---------------------------------------------------------------------------

/// Token usage breakdown — compatible with entire.io's `agent.TokenUsage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub cache_creation_tokens: i64,
    #[serde(default)]
    pub cache_read_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    /// Extended thinking / reasoning tokens (Claude, o1/o3, Gemini thinking).
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub reasoning_tokens: i64,
    #[serde(default)]
    pub api_call_count: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_tokens: Option<Box<TokenUsage>>,
}

fn is_zero_i64(v: &i64) -> bool {
    *v == 0
}

impl TokenUsage {
    pub fn total(&self) -> i64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_read_tokens
            + self.cache_creation_tokens
            + self.reasoning_tokens
    }

    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_read_tokens += other.cache_read_tokens;
        self.cache_creation_tokens += other.cache_creation_tokens;
        self.reasoning_tokens += other.reasoning_tokens;
        self.api_call_count += other.api_call_count;
    }

    pub fn add_subagent(&mut self, other: &TokenUsage) {
        let sub = self
            .subagent_tokens
            .get_or_insert_with(|| Box::new(TokenUsage::default()));
        sub.add(other);
    }

    pub fn is_empty(&self) -> bool {
        self.input_tokens == 0
            && self.output_tokens == 0
            && self.cache_read_tokens == 0
            && self.cache_creation_tokens == 0
            && self.reasoning_tokens == 0
    }

    /// Convert from chub's internal TokenUsage format.
    pub fn from_chub(t: &super::super::sessions::TokenUsage) -> Self {
        Self {
            input_tokens: t.input as i64,
            output_tokens: t.output as i64,
            cache_read_tokens: t.cache_read as i64,
            cache_creation_tokens: t.cache_write as i64,
            reasoning_tokens: t.reasoning as i64,
            api_call_count: 0,
            subagent_tokens: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Session phase
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    #[serde(rename = "idle")]
    #[default]
    Idle,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "ended")]
    Ended,
}

// ---------------------------------------------------------------------------
// Session metrics (from agents like Cursor)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetrics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_count: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_tokens: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window_size: Option<i64>,
}

// ---------------------------------------------------------------------------
// Attribution
// ---------------------------------------------------------------------------

/// Line-level attribution — compatible with entire.io's `InitialAttribution`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialAttribution {
    pub calculated_at: String,
    #[serde(default)]
    pub agent_lines: i64,
    #[serde(default)]
    pub human_added: i64,
    #[serde(default)]
    pub human_modified: i64,
    #[serde(default)]
    pub human_removed: i64,
    #[serde(default)]
    pub total_committed: i64,
    #[serde(default)]
    pub agent_percentage: f64,
}

/// Per-prompt attribution snapshot — matches entire.io's `PromptAttribution`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptAttribution {
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub agent_lines: i64,
    #[serde(default)]
    pub human_added: i64,
    #[serde(default)]
    pub human_modified: i64,
    #[serde(default)]
    pub human_removed: i64,
}

// ---------------------------------------------------------------------------
// Checkpoint ID
// ---------------------------------------------------------------------------

/// 12 hex character checkpoint ID (6 random bytes), compatible with entire.io.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckpointID(pub String);

impl CheckpointID {
    /// Generate a new random checkpoint ID (12 hex chars).
    pub fn generate() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        std::process::id().hash(&mut hasher);
        std::thread::current().id().hash(&mut hasher);
        // Mix in a counter for uniqueness within same ms
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        COUNTER
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .hash(&mut hasher);
        let hash = hasher.finish();
        // Use 12 hex chars (matches entire.io's 6-byte random)
        Self(format!("{:012x}", hash).chars().take(12).collect())
    }

    /// Shard path: first 2 chars as directory, rest as subfolder.
    /// e.g. "a3b2c4d5e6f7" → "a3/b2c4d5e6f7"
    pub fn shard_path(&self) -> String {
        if self.0.len() < 3 {
            return self.0.clone();
        }
        format!("{}/{}", &self.0[..2], &self.0[2..])
    }
}

impl std::fmt::Display for CheckpointID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Summary (AI-generated, optional)
// ---------------------------------------------------------------------------

/// Summary — matches entire.io's `Summary` format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learnings: Option<LearningsSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub friction: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_items: Vec<String>,
}

/// Learnings breakdown — matches entire.io.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningsSummary {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repo: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code: Vec<CodeLearning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflow: Vec<String>,
}

/// Code learning entry — matches entire.io.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLearning {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    pub finding: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_id_generation() {
        let id = CheckpointID::generate();
        assert_eq!(id.0.len(), 12);
        assert!(id.0.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn checkpoint_shard_path() {
        let id = CheckpointID("a3b2c4d5e6f7".to_string());
        assert_eq!(id.shard_path(), "a3/b2c4d5e6f7");
    }

    #[test]
    fn token_usage_add() {
        let mut a = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            api_call_count: 1,
            ..Default::default()
        };
        let b = TokenUsage {
            input_tokens: 200,
            output_tokens: 100,
            api_call_count: 2,
            ..Default::default()
        };
        a.add(&b);
        assert_eq!(a.input_tokens, 300);
        assert_eq!(a.api_call_count, 3);
    }

    #[test]
    fn token_usage_subagent() {
        let mut main = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            ..Default::default()
        };
        let sub = TokenUsage {
            input_tokens: 200,
            output_tokens: 100,
            ..Default::default()
        };
        main.add_subagent(&sub);
        assert!(main.subagent_tokens.is_some());
        assert_eq!(main.subagent_tokens.as_ref().unwrap().input_tokens, 200);
    }

    #[test]
    fn phase_serde() {
        let json = serde_json::to_string(&Phase::Active).unwrap();
        assert_eq!(json, "\"active\"");
        let parsed: Phase = serde_json::from_str("\"idle\"").unwrap();
        assert_eq!(parsed, Phase::Idle);
    }

    #[test]
    fn token_usage_json_compat() {
        // Verify JSON field names match entire.io (camelCase)
        let t = TokenUsage {
            input_tokens: 100,
            cache_creation_tokens: 10,
            cache_read_tokens: 20,
            output_tokens: 50,
            reasoning_tokens: 30,
            api_call_count: 3,
            subagent_tokens: None,
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"inputTokens\""));
        assert!(json.contains("\"cacheCreationTokens\""));
        assert!(json.contains("\"cacheReadTokens\""));
        assert!(json.contains("\"outputTokens\""));
        assert!(json.contains("\"apiCallCount\""));
    }

    // --- Agent type parsing ---

    #[test]
    fn agent_type_canonical_names() {
        assert_eq!(AgentType::from_str("claude-code"), AgentType::ClaudeCode);
        assert_eq!(AgentType::from_str("cursor"), AgentType::Cursor);
        assert_eq!(AgentType::from_str("gemini-cli"), AgentType::GeminiCli);
        assert_eq!(AgentType::from_str("copilot"), AgentType::Copilot);
        assert_eq!(AgentType::from_str("codex"), AgentType::Codex);
        assert_eq!(AgentType::from_str("windsurf"), AgentType::Windsurf);
        assert_eq!(AgentType::from_str("cline"), AgentType::Cline);
        assert_eq!(AgentType::from_str("aider"), AgentType::Aider);
        assert_eq!(AgentType::from_str("opencode"), AgentType::OpenCode);
    }

    #[test]
    fn agent_type_aliases() {
        assert_eq!(AgentType::from_str("claude"), AgentType::ClaudeCode);
        assert_eq!(AgentType::from_str("claudecode"), AgentType::ClaudeCode);
        assert_eq!(AgentType::from_str("gemini"), AgentType::GeminiCli);
        assert_eq!(AgentType::from_str("github-copilot"), AgentType::Copilot);
    }

    #[test]
    fn agent_type_case_insensitive() {
        assert_eq!(AgentType::from_str("CLAUDE-CODE"), AgentType::ClaudeCode);
        assert_eq!(AgentType::from_str("Cursor"), AgentType::Cursor);
        assert_eq!(AgentType::from_str("GEMINI-CLI"), AgentType::GeminiCli);
    }

    #[test]
    fn agent_type_unknown_inputs() {
        assert_eq!(AgentType::from_str(""), AgentType::Unknown);
        assert_eq!(AgentType::from_str("my-custom-agent"), AgentType::Unknown);
        assert_eq!(AgentType::from_str("   "), AgentType::Unknown);
    }

    #[test]
    fn agent_type_name_roundtrip() {
        for agent in &[
            AgentType::ClaudeCode,
            AgentType::Cursor,
            AgentType::GeminiCli,
            AgentType::Copilot,
            AgentType::Codex,
            AgentType::Windsurf,
            AgentType::Cline,
            AgentType::Aider,
            AgentType::OpenCode,
            AgentType::Unknown,
        ] {
            let name = agent.name();
            assert!(!name.is_empty(), "name() should never be empty");
            // Canonical name should round-trip through from_str
            let parsed = AgentType::from_str(name);
            assert_eq!(
                &parsed, agent,
                "from_str(name()) should round-trip for {:?}",
                agent
            );
        }
    }

    #[test]
    fn agent_type_serde_roundtrip() {
        let agent = AgentType::ClaudeCode;
        let json = serde_json::to_string(&agent).unwrap();
        assert_eq!(json, "\"claude-code\"");
        let parsed: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, agent);
    }

    #[test]
    fn agent_type_unknown_serde() {
        // Unknown JSON values deserialize to Unknown via serde(other)
        let parsed: AgentType = serde_json::from_str("\"some-future-agent\"").unwrap();
        assert_eq!(parsed, AgentType::Unknown);
    }

    // --- Checkpoint ID ---

    #[test]
    fn checkpoint_id_uniqueness() {
        let ids: Vec<CheckpointID> = (0..100).map(|_| CheckpointID::generate()).collect();
        let unique: std::collections::HashSet<String> = ids.iter().map(|id| id.0.clone()).collect();
        assert_eq!(unique.len(), 100, "100 checkpoint IDs should all be unique");
    }

    #[test]
    fn checkpoint_id_always_hex() {
        for _ in 0..50 {
            let id = CheckpointID::generate();
            assert_eq!(id.0.len(), 12, "ID length must be 12");
            assert!(
                id.0.chars().all(|c| c.is_ascii_hexdigit()),
                "non-hex char in ID: {}",
                id.0
            );
        }
    }

    #[test]
    fn checkpoint_id_short_string_shard() {
        let short = CheckpointID("ab".to_string());
        assert_eq!(short.shard_path(), "ab", "short IDs should not panic");
        let tiny = CheckpointID("a".to_string());
        assert_eq!(tiny.shard_path(), "a");
        let empty = CheckpointID(String::new());
        assert_eq!(empty.shard_path(), "");
    }

    #[test]
    fn checkpoint_id_display() {
        let id = CheckpointID("a3b2c4d5e6f7".to_string());
        assert_eq!(format!("{}", id), "a3b2c4d5e6f7");
    }

    // --- Token usage edge cases ---

    #[test]
    fn token_usage_default_is_empty() {
        let t = TokenUsage::default();
        assert!(t.is_empty());
        assert_eq!(t.total(), 0);
    }

    #[test]
    fn token_usage_total_includes_reasoning() {
        let t = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: 10,
            cache_creation_tokens: 5,
            reasoning_tokens: 200,
            api_call_count: 1,
            subagent_tokens: None,
        };
        assert_eq!(t.total(), 365);
    }

    #[test]
    fn token_usage_is_empty_with_only_reasoning() {
        let t = TokenUsage {
            reasoning_tokens: 100,
            ..Default::default()
        };
        assert!(!t.is_empty());
    }

    #[test]
    fn token_usage_subagent_accumulates() {
        let mut main = TokenUsage::default();
        let sub1 = TokenUsage {
            input_tokens: 100,
            ..Default::default()
        };
        let sub2 = TokenUsage {
            input_tokens: 200,
            output_tokens: 50,
            ..Default::default()
        };
        main.add_subagent(&sub1);
        main.add_subagent(&sub2);
        let sub = main.subagent_tokens.unwrap();
        assert_eq!(sub.input_tokens, 300);
        assert_eq!(sub.output_tokens, 50);
    }

    #[test]
    fn token_usage_reasoning_skipped_when_zero() {
        let t = TokenUsage {
            input_tokens: 100,
            ..Default::default()
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(
            !json.contains("reasoningTokens"),
            "zero reasoning should be skipped"
        );
    }

    #[test]
    fn token_usage_reasoning_present_when_nonzero() {
        let t = TokenUsage {
            reasoning_tokens: 500,
            ..Default::default()
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"reasoningTokens\":500"));
    }

    #[test]
    fn token_usage_subagent_skipped_when_none() {
        let t = TokenUsage::default();
        let json = serde_json::to_string(&t).unwrap();
        assert!(!json.contains("subagentTokens"));
    }

    // --- Phase ---

    #[test]
    fn phase_default_is_idle() {
        assert_eq!(Phase::default(), Phase::Idle);
    }

    #[test]
    fn phase_all_variants_serde() {
        for (variant, expected_json) in &[
            (Phase::Idle, "\"idle\""),
            (Phase::Active, "\"active\""),
            (Phase::Ended, "\"ended\""),
        ] {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json);
            let parsed: Phase = serde_json::from_str(expected_json).unwrap();
            assert_eq!(&parsed, variant);
        }
    }
}
