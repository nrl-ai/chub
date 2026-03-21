//! Tier 3 (org-level) annotation storage — remote HTTP API with local cache.
//!
//! Configuration:
//! - URL: `.chub/config.yaml` `annotation_server.url` or `CHUB_ANNOTATION_SERVER` env var
//! - Token: `~/.chub/config.yaml` `annotation_token` or `CHUB_ANNOTATION_TOKEN` env var
//!
//! ## API contract (server must implement):
//!
//! ```text
//! GET  /api/v1/annotations              → 200 [{TeamAnnotation}, ...]
//! GET  /api/v1/annotations/:id          → 200 TeamAnnotation | 404
//! POST /api/v1/annotations/:id          → 200 TeamAnnotation
//!      Body: {"note":"..","kind":"..","severity":"..","author":".."}
//! DELETE /api/v1/annotations/:id        → 200 | 404
//!
//! Auth: Authorization: Bearer <token>   (optional if server doesn't require it)
//! Content-Type: application/json
//! Entry ID encoding: replace "/" with "--" in URL path segment
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::annotations::AnnotationKind;
use crate::config::{chub_dir, get_annotation_token};
use crate::team::project::{load_project_config, AnnotationServerConfig};
use crate::team::team_annotations::TeamAnnotation;

const DEFAULT_CACHE_TTL_SECS: u64 = 3600;
const ORG_FETCH_TIMEOUT_SECS: u64 = 10;

/// Load the annotation server config.
/// Priority: CHUB_ANNOTATION_SERVER env var > .chub/config.yaml annotation_server.
pub fn get_annotation_server_config() -> Option<AnnotationServerConfig> {
    if let Ok(url) = std::env::var("CHUB_ANNOTATION_SERVER") {
        return Some(AnnotationServerConfig {
            url,
            auto_push: false,
            cache_ttl_secs: None,
        });
    }
    load_project_config()?.annotation_server
}

fn org_cache_dir() -> PathBuf {
    chub_dir().join("cache").join("org-annotations")
}

fn org_cache_path(entry_id: &str) -> PathBuf {
    let safe = entry_id.replace('/', "--");
    org_cache_dir().join(format!("{}.json", safe))
}

fn is_cache_fresh(path: &Path, ttl_secs: u64) -> bool {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .map(|age| age.as_secs() < ttl_secs)
        .unwrap_or(false)
}

fn read_cache(entry_id: &str) -> Option<TeamAnnotation> {
    let path = org_cache_path(entry_id);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn write_cache(ann: &TeamAnnotation) {
    let dir = org_cache_dir();
    let _ = fs::create_dir_all(&dir);
    let path = org_cache_path(&ann.id);
    let _ = fs::write(path, serde_json::to_string_pretty(ann).unwrap_or_default());
}

fn invalidate_cache(entry_id: &str) {
    let _ = fs::remove_file(org_cache_path(entry_id));
}

fn entry_id_to_path(entry_id: &str) -> String {
    entry_id.replace('/', "--")
}

fn make_client() -> Option<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(ORG_FETCH_TIMEOUT_SECS))
        .build()
        .ok()
}

fn add_auth(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if let Some(token) = get_annotation_token() {
        req.bearer_auth(token)
    } else {
        req
    }
}

/// Fetch org annotation for an entry. Returns None if no org server is configured,
/// the entry doesn't exist, or the server is unreachable (falls back to cache).
pub async fn read_org_annotation(entry_id: &str) -> Option<TeamAnnotation> {
    let config = get_annotation_server_config()?;
    let ttl = config.cache_ttl_secs.unwrap_or(DEFAULT_CACHE_TTL_SECS);
    let cache_path = org_cache_path(entry_id);

    // Serve from cache if still fresh.
    if is_cache_fresh(&cache_path, ttl) {
        if let Some(cached) = read_cache(entry_id) {
            return Some(cached);
        }
    }

    let url = format!(
        "{}/api/v1/annotations/{}",
        config.url.trim_end_matches('/'),
        entry_id_to_path(entry_id)
    );

    let client = make_client()?;
    let resp = match add_auth(client.get(&url)).send().await {
        Ok(r) => r,
        Err(_) => return read_cache(entry_id), // network error → fall back to stale cache
    };

    if resp.status().as_u16() == 404 {
        invalidate_cache(entry_id);
        return None;
    }
    if !resp.status().is_success() {
        return read_cache(entry_id); // server error → fall back to stale cache
    }

    match resp.json::<TeamAnnotation>().await {
        Ok(ann) => {
            write_cache(&ann);
            Some(ann)
        }
        Err(_) => read_cache(entry_id),
    }
}

/// Write a note to the org annotation server.
pub async fn write_org_annotation(
    entry_id: &str,
    note: &str,
    author: &str,
    kind: AnnotationKind,
    severity: Option<String>,
) -> Result<TeamAnnotation, String> {
    let config = get_annotation_server_config()
        .ok_or_else(|| "No annotation_server configured. Set annotation_server.url in .chub/config.yaml or CHUB_ANNOTATION_SERVER env var.".to_string())?;

    let url = format!(
        "{}/api/v1/annotations/{}",
        config.url.trim_end_matches('/'),
        entry_id_to_path(entry_id)
    );

    let body = serde_json::json!({
        "note": note,
        "kind": kind.as_str(),
        "severity": severity,
        "author": author,
    });

    let client = make_client().ok_or_else(|| "Failed to build HTTP client".to_string())?;
    let resp = add_auth(client.post(&url).json(&body))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("Server returned {}: {}", status, body_text));
    }

    let ann: TeamAnnotation = resp
        .json()
        .await
        .map_err(|e| format!("Invalid response: {}", e))?;

    write_cache(&ann);
    Ok(ann)
}

/// Remove all org annotations for an entry.
pub async fn clear_org_annotation(entry_id: &str) -> Result<bool, String> {
    let config = get_annotation_server_config()
        .ok_or_else(|| "No annotation_server configured.".to_string())?;

    let url = format!(
        "{}/api/v1/annotations/{}",
        config.url.trim_end_matches('/'),
        entry_id_to_path(entry_id)
    );

    let client = make_client().ok_or_else(|| "Failed to build HTTP client".to_string())?;
    let resp = add_auth(client.delete(&url))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    if status.is_success() {
        invalidate_cache(entry_id);
        Ok(true)
    } else if status.as_u16() == 404 {
        invalidate_cache(entry_id);
        Ok(false)
    } else {
        Err(format!("Server returned {}", status.as_u16()))
    }
}

/// List all org annotations.
pub async fn list_org_annotations() -> Vec<TeamAnnotation> {
    let config = match get_annotation_server_config() {
        Some(c) => c,
        None => return vec![],
    };

    let url = format!("{}/api/v1/annotations", config.url.trim_end_matches('/'));
    let client = match make_client() {
        Some(c) => c,
        None => return vec![],
    };

    match add_auth(client.get(&url)).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<TeamAnnotation>>().await.unwrap_or_default()
        }
        _ => vec![],
    }
}

/// Clear the local org annotation cache for an entry (force-refresh on next read).
pub fn invalidate_org_cache(entry_id: &str) {
    invalidate_cache(entry_id);
}

/// Clear the entire org annotation cache.
pub fn clear_org_cache() {
    let dir = org_cache_dir();
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
}
