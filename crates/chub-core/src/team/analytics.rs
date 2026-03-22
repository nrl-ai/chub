use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::chub_dir;

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

/// Every tracked event in the local journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum Event {
    #[serde(rename = "fetch")]
    Fetch(FetchEvent),
    #[serde(rename = "search")]
    Search(SearchEvent),
    #[serde(rename = "build")]
    Build(BuildEvent),
    #[serde(rename = "mcp_call")]
    McpCall(McpCallEvent),
    #[serde(rename = "pin")]
    Pin(PinEvent),
    #[serde(rename = "annotate")]
    Annotate(AnnotateEvent),
    #[serde(rename = "feedback")]
    Feedback(FeedbackEvent),
}

impl Event {
    pub fn timestamp(&self) -> u64 {
        match self {
            Event::Fetch(e) => e.timestamp,
            Event::Search(e) => e.timestamp,
            Event::Build(e) => e.timestamp,
            Event::McpCall(e) => e.timestamp,
            Event::Pin(e) => e.timestamp,
            Event::Annotate(e) => e.timestamp,
            Event::Feedback(e) => e.timestamp,
        }
    }

    pub fn event_name(&self) -> &'static str {
        match self {
            Event::Fetch(_) => "fetch",
            Event::Search(_) => "search",
            Event::Build(_) => "build",
            Event::McpCall(_) => "mcp_call",
            Event::Pin(_) => "pin",
            Event::Annotate(_) => "annotate",
            Event::Feedback(_) => "feedback",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchEvent {
    pub entry_id: String,
    pub timestamp: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvent {
    pub query: String,
    pub timestamp: u64,
    pub result_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildEvent {
    pub timestamp: u64,
    pub doc_count: usize,
    pub duration_ms: u64,
    #[serde(default)]
    pub errors: usize,
    #[serde(default)]
    pub validate_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCallEvent {
    pub tool: String,
    pub timestamp: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinEvent {
    pub entry_id: String,
    pub action: String, // "add", "remove"
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotateEvent {
    pub entry_id: String,
    pub kind: String, // "issue", "fix", "practice"
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEvent {
    pub entry_id: String,
    pub rating: String,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Recording
// ---------------------------------------------------------------------------

fn analytics_path() -> PathBuf {
    chub_dir().join("analytics.jsonl")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn append_event(event: &Event) {
    let path = analytics_path();
    let _ = fs::create_dir_all(path.parent().unwrap_or(&PathBuf::from(".")));

    let line = serde_json::to_string(event).unwrap_or_default();
    let mut file = fs::OpenOptions::new().create(true).append(true).open(&path);

    if let Ok(ref mut f) = file {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

/// Record a doc fetch event (backwards-compatible entry point).
pub fn record_fetch(entry_id: &str, agent: Option<&str>) {
    let event = Event::Fetch(FetchEvent {
        entry_id: entry_id.to_string(),
        timestamp: now_secs(),
        agent: agent.map(|s| s.to_string()),
        lang: None,
        cache_hit: None,
        duration_ms: None,
    });
    append_event(&event);
}

/// Record a doc fetch with full metadata.
pub fn record_fetch_detailed(
    entry_id: &str,
    agent: Option<&str>,
    lang: Option<&str>,
    cache_hit: Option<bool>,
    duration_ms: Option<u64>,
) {
    let event = Event::Fetch(FetchEvent {
        entry_id: entry_id.to_string(),
        timestamp: now_secs(),
        agent: agent.map(|s| s.to_string()),
        lang: lang.map(|s| s.to_string()),
        cache_hit,
        duration_ms,
    });
    append_event(&event);
}

/// Record a search query.
pub fn record_search(
    query: &str,
    result_count: usize,
    duration_ms: Option<u64>,
    agent: Option<&str>,
) {
    let event = Event::Search(SearchEvent {
        query: query.to_string(),
        timestamp: now_secs(),
        result_count,
        duration_ms,
        agent: agent.map(|s| s.to_string()),
    });
    append_event(&event);
}

/// Record a build event.
pub fn record_build(doc_count: usize, duration_ms: u64, errors: usize, validate_only: bool) {
    let event = Event::Build(BuildEvent {
        timestamp: now_secs(),
        doc_count,
        duration_ms,
        errors,
        validate_only,
    });
    append_event(&event);
}

/// Record an MCP tool call.
pub fn record_mcp_call(tool: &str, duration_ms: Option<u64>, agent: Option<&str>) {
    let event = Event::McpCall(McpCallEvent {
        tool: tool.to_string(),
        timestamp: now_secs(),
        duration_ms,
        agent: agent.map(|s| s.to_string()),
    });
    append_event(&event);
}

/// Record a pin add/remove.
pub fn record_pin(entry_id: &str, action: &str) {
    let event = Event::Pin(PinEvent {
        entry_id: entry_id.to_string(),
        action: action.to_string(),
        timestamp: now_secs(),
    });
    append_event(&event);
}

/// Record an annotation.
pub fn record_annotate(entry_id: &str, kind: &str) {
    let event = Event::Annotate(AnnotateEvent {
        entry_id: entry_id.to_string(),
        kind: kind.to_string(),
        timestamp: now_secs(),
    });
    append_event(&event);
}

/// Record a feedback submission.
pub fn record_feedback(entry_id: &str, rating: &str) {
    let event = Event::Feedback(FeedbackEvent {
        entry_id: entry_id.to_string(),
        rating: rating.to_string(),
        timestamp: now_secs(),
    });
    append_event(&event);
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Load all events from the journal, parsing both new tagged format and legacy fetch-only lines.
pub fn load_events() -> Vec<Event> {
    let path = analytics_path();
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
        .filter_map(|l| {
            // Try new tagged format first
            if let Ok(event) = serde_json::from_str::<Event>(l) {
                return Some(event);
            }
            // Fall back to legacy format (plain FetchEvent without "event" tag)
            if let Ok(legacy) = serde_json::from_str::<LegacyFetchEvent>(l) {
                return Some(Event::Fetch(FetchEvent {
                    entry_id: legacy.entry_id,
                    timestamp: legacy.timestamp,
                    agent: legacy.agent,
                    lang: None,
                    cache_hit: None,
                    duration_ms: None,
                }));
            }
            None
        })
        .collect()
}

/// Legacy format for backwards compatibility with old analytics.jsonl files.
#[derive(Deserialize)]
struct LegacyFetchEvent {
    entry_id: String,
    timestamp: u64,
    #[serde(default)]
    agent: Option<String>,
}

/// Export the raw JSONL content.
pub fn export_raw() -> String {
    let path = analytics_path();
    fs::read_to_string(&path).unwrap_or_default()
}

/// Clear the analytics journal.
pub fn clear_journal() -> bool {
    let path = analytics_path();
    if path.exists() {
        fs::remove_file(&path).is_ok()
    } else {
        true
    }
}

/// Get the journal file size in bytes.
pub fn journal_size_bytes() -> u64 {
    let path = analytics_path();
    fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

/// Comprehensive usage statistics summary.
#[derive(Debug, Clone, Serialize)]
pub struct UsageStats {
    pub period_days: u64,
    pub total_events: usize,
    pub total_fetches: usize,
    pub total_searches: usize,
    pub total_builds: usize,
    pub total_mcp_calls: usize,
    pub total_annotations: usize,
    pub total_feedback: usize,
    pub most_fetched: Vec<(String, usize)>,
    pub top_queries: Vec<(String, usize)>,
    pub top_mcp_tools: Vec<(String, usize)>,
    pub never_fetched_pins: Vec<String>,
    pub agents: Vec<(String, usize)>,
    pub avg_search_results: f64,
}

/// Compute usage statistics for the last N days.
pub fn get_stats(days: u64) -> UsageStats {
    let events = load_events();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(days * 86400);

    let recent: Vec<&Event> = events.iter().filter(|e| e.timestamp() >= cutoff).collect();

    let mut fetch_counts: HashMap<String, usize> = HashMap::new();
    let mut query_counts: HashMap<String, usize> = HashMap::new();
    let mut mcp_tool_counts: HashMap<String, usize> = HashMap::new();
    let mut agent_counts: HashMap<String, usize> = HashMap::new();
    let mut total_fetches = 0usize;
    let mut total_searches = 0usize;
    let mut total_builds = 0usize;
    let mut total_mcp_calls = 0usize;
    let mut total_annotations = 0usize;
    let mut total_feedback = 0usize;
    let mut search_result_sum = 0usize;

    for event in &recent {
        match event {
            Event::Fetch(e) => {
                total_fetches += 1;
                *fetch_counts.entry(e.entry_id.clone()).or_insert(0) += 1;
                if let Some(ref agent) = e.agent {
                    *agent_counts.entry(agent.clone()).or_insert(0) += 1;
                }
            }
            Event::Search(e) => {
                total_searches += 1;
                *query_counts.entry(e.query.clone()).or_insert(0) += 1;
                search_result_sum += e.result_count;
                if let Some(ref agent) = e.agent {
                    *agent_counts.entry(agent.clone()).or_insert(0) += 1;
                }
            }
            Event::Build(_) => {
                total_builds += 1;
            }
            Event::McpCall(e) => {
                total_mcp_calls += 1;
                *mcp_tool_counts.entry(e.tool.clone()).or_insert(0) += 1;
                if let Some(ref agent) = e.agent {
                    *agent_counts.entry(agent.clone()).or_insert(0) += 1;
                }
            }
            Event::Annotate(_) => {
                total_annotations += 1;
            }
            Event::Feedback(_) => {
                total_feedback += 1;
            }
            Event::Pin(_) => {}
        }
    }

    let mut most_fetched: Vec<(String, usize)> = fetch_counts.into_iter().collect();
    most_fetched.sort_by(|a, b| b.1.cmp(&a.1));

    let mut top_queries: Vec<(String, usize)> = query_counts.into_iter().collect();
    top_queries.sort_by(|a, b| b.1.cmp(&a.1));

    let mut top_mcp_tools: Vec<(String, usize)> = mcp_tool_counts.into_iter().collect();
    top_mcp_tools.sort_by(|a, b| b.1.cmp(&a.1));

    let mut agents: Vec<(String, usize)> = agent_counts.into_iter().collect();
    agents.sort_by(|a, b| b.1.cmp(&a.1));

    // Find pinned but never-fetched
    let pins = crate::team::pins::list_pins();
    let fetched_ids: std::collections::HashSet<String> = recent
        .iter()
        .filter_map(|e| match e {
            Event::Fetch(f) => Some(f.entry_id.clone()),
            _ => None,
        })
        .collect();
    let never_fetched_pins: Vec<String> = pins
        .iter()
        .filter(|p| !fetched_ids.contains(&p.id))
        .map(|p| p.id.clone())
        .collect();

    let avg_search_results = if total_searches > 0 {
        search_result_sum as f64 / total_searches as f64
    } else {
        0.0
    };

    UsageStats {
        period_days: days,
        total_events: recent.len(),
        total_fetches,
        total_searches,
        total_builds,
        total_mcp_calls,
        total_annotations,
        total_feedback,
        most_fetched,
        top_queries,
        top_mcp_tools,
        never_fetched_pins,
        agents,
        avg_search_results,
    }
}
