use crate::config::load_config;
use crate::identity::{detect_agent, detect_agent_version, get_or_create_client_id};
use crate::team::analytics;

pub fn is_telemetry_enabled() -> bool {
    if let Ok(val) = std::env::var("CHUB_TELEMETRY") {
        if val == "0" || val == "false" {
            return false;
        }
    }
    load_config().telemetry
}

pub fn is_feedback_enabled() -> bool {
    if let Ok(val) = std::env::var("CHUB_FEEDBACK") {
        if val == "0" || val == "false" {
            return false;
        }
    }
    load_config().feedback
}

/// Get the remote telemetry URL, if configured.
/// Returns None when no URL is set — online forwarding is opt-in.
pub fn get_telemetry_url() -> Option<String> {
    if let Ok(url) = std::env::var("CHUB_TELEMETRY_URL") {
        if !url.is_empty() {
            return Some(url);
        }
    }
    let config = load_config();
    if config.telemetry_url.is_empty() {
        None
    } else {
        Some(config.telemetry_url)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FeedbackResult {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
}

#[derive(Default)]
pub struct FeedbackOpts {
    pub comment: Option<String>,
    pub doc_lang: Option<String>,
    pub doc_version: Option<String>,
    pub target_file: Option<String>,
    pub labels: Option<Vec<String>>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub cli_version: Option<String>,
    pub source: Option<String>,
}

/// Send feedback. Always records locally. Forwards to remote only if telemetry_url is configured.
pub async fn send_feedback(
    entry_id: &str,
    entry_type: &str,
    rating: &str,
    opts: FeedbackOpts,
) -> FeedbackResult {
    if !is_feedback_enabled() {
        return FeedbackResult {
            status: "skipped".to_string(),
            reason: Some("feedback_disabled".to_string()),
            feedback_id: None,
            code: None,
        };
    }

    // Always record locally
    analytics::record_feedback(entry_id, rating);

    let telemetry_url = match get_telemetry_url() {
        Some(url) => url,
        None => {
            // No remote endpoint — local-only mode
            return FeedbackResult {
                status: "recorded_locally".to_string(),
                reason: None,
                feedback_id: None,
                code: None,
            };
        }
    };

    let client_id = get_or_create_client_id().unwrap_or_default();
    let agent_name = opts.agent.unwrap_or_else(|| detect_agent().to_string());
    let agent_version = detect_agent_version();

    let body = serde_json::json!({
        "entry_id": entry_id,
        "entry_type": entry_type,
        "rating": rating,
        "doc_lang": opts.doc_lang,
        "doc_version": opts.doc_version,
        "target_file": opts.target_file,
        "labels": opts.labels,
        "comment": opts.comment,
        "agent": {
            "name": agent_name,
            "version": agent_version,
            "model": opts.model,
        },
        "cli_version": opts.cli_version,
        "source": opts.source,
    });

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            return FeedbackResult {
                status: "recorded_locally".to_string(),
                reason: Some("network_unavailable".to_string()),
                feedback_id: None,
                code: None,
            }
        }
    };

    let send_result = client
        .post(format!("{}/feedback", telemetry_url))
        .header("Content-Type", "application/json")
        .header("X-Client-ID", &client_id)
        .json(&body)
        .send()
        .await;

    match send_result {
        Ok(res) => {
            let status_code = res.status();
            if status_code.is_success() {
                let data: serde_json::Value = res.json().await.unwrap_or_default();
                let feedback_id = data
                    .get("feedback_id")
                    .or_else(|| data.get("id"))
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .map(|s: &str| s.to_string());
                FeedbackResult {
                    status: "sent".to_string(),
                    reason: None,
                    feedback_id,
                    code: None,
                }
            } else {
                FeedbackResult {
                    status: "recorded_locally".to_string(),
                    reason: Some(format!("remote_http_{}", status_code.as_u16())),
                    feedback_id: None,
                    code: Some(status_code.as_u16()),
                }
            }
        }
        Err(_) => FeedbackResult {
            status: "recorded_locally".to_string(),
            reason: Some("network_error".to_string()),
            feedback_id: None,
            code: None,
        },
    }
}

/// Valid feedback labels.
pub const VALID_LABELS: &[&str] = &[
    "accurate",
    "well-structured",
    "helpful",
    "good-examples",
    "outdated",
    "inaccurate",
    "incomplete",
    "wrong-examples",
    "wrong-version",
    "poorly-structured",
];
