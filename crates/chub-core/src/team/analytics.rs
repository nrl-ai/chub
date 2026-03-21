use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::chub_dir;

/// A single fetch event in the analytics log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchEvent {
    pub entry_id: String,
    pub timestamp: u64,
    #[serde(default)]
    pub agent: Option<String>,
}

/// Usage statistics summary.
#[derive(Debug, Clone, Serialize)]
pub struct UsageStats {
    pub most_fetched: Vec<(String, usize)>,
    pub never_fetched_pins: Vec<String>,
    pub total_fetches: usize,
    pub period_days: u64,
}

fn analytics_path() -> PathBuf {
    chub_dir().join("analytics.jsonl")
}

/// Record a doc fetch event.
pub fn record_fetch(entry_id: &str, agent: Option<&str>) {
    let event = FetchEvent {
        entry_id: entry_id.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        agent: agent.map(|s| s.to_string()),
    };

    let path = analytics_path();
    let _ = fs::create_dir_all(path.parent().unwrap_or(&PathBuf::from(".")));

    // Append as JSONL (one JSON object per line)
    let line = serde_json::to_string(&event).unwrap_or_default();
    let mut file = fs::OpenOptions::new().create(true).append(true).open(&path);

    if let Ok(ref mut f) = file {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

/// Load all fetch events.
fn load_events() -> Vec<FetchEvent> {
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
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// Compute usage statistics for the last N days.
pub fn get_stats(days: u64) -> UsageStats {
    let events = load_events();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(days * 86400);

    let recent: Vec<&FetchEvent> = events.iter().filter(|e| e.timestamp >= cutoff).collect();

    // Count fetches per entry
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for event in &recent {
        *counts.entry(event.entry_id.clone()).or_insert(0) += 1;
    }

    let mut most_fetched: Vec<(String, usize)> = counts.into_iter().collect();
    most_fetched.sort_by(|a, b| b.1.cmp(&a.1));

    // Find pinned but never-fetched
    let pins = crate::team::pins::list_pins();
    let fetched_ids: std::collections::HashSet<&str> =
        recent.iter().map(|e| e.entry_id.as_str()).collect();

    let never_fetched_pins: Vec<String> = pins
        .iter()
        .filter(|p| !fetched_ids.contains(p.id.as_str()))
        .map(|p| p.id.clone())
        .collect();

    UsageStats {
        total_fetches: recent.len(),
        most_fetched,
        never_fetched_pins,
        period_days: days,
    }
}
