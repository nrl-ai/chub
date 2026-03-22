//! Checkpoint storage on orphan git branch.
//!
//! Stores checkpoints on `entire/checkpoints/v1` (orphan branch) with
//! sharded directory structure compatible with entire.io.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use super::session_state::SessionState;
use super::types::{CheckpointID, InitialAttribution, Summary, TokenUsage};
use crate::util::now_iso8601;

// ---------------------------------------------------------------------------
// Committed metadata (per-session, stored at <shard>/0/metadata.json)
// ---------------------------------------------------------------------------

/// Metadata for a single session within a checkpoint.
/// Compatible with entire.io's `CommittedMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommittedMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_version: Option<String>,
    #[serde(rename = "checkpointID")]
    pub checkpoint_id: CheckpointID,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(default)]
    pub strategy: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default)]
    pub checkpoints_count: i32,
    #[serde(default)]
    pub files_touched: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "turnID")]
    pub turn_id: Option<String>,
    #[serde(default)]
    pub is_task: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "toolUseID")]
    pub tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_identifier_at_start: Option<String>,
    #[serde(default)]
    pub checkpoint_transcript_start: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_attribution: Option<InitialAttribution>,
}

// ---------------------------------------------------------------------------
// Checkpoint summary (root-level, stored at <shard>/metadata.json)
// ---------------------------------------------------------------------------

/// Root-level checkpoint summary aggregating all sessions.
/// Compatible with entire.io's `CheckpointSummary`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_version: Option<String>,
    #[serde(rename = "checkpointID")]
    pub checkpoint_id: CheckpointID,
    #[serde(default)]
    pub strategy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default)]
    pub checkpoints_count: i32,
    #[serde(default)]
    pub files_touched: Vec<String>,
    #[serde(default)]
    pub sessions: Vec<SessionFilePaths>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
}

/// Paths to session files within a checkpoint (relative).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFilePaths {
    pub metadata: String,
    pub transcript: String,
    pub content_hash: String,
    pub prompt: String,
}

// ---------------------------------------------------------------------------
// Checkpoint branch operations
// ---------------------------------------------------------------------------

const CHECKPOINT_BRANCH: &str = "entire/checkpoints/v1";

/// Create a checkpoint from the current session state.
/// Stores metadata and transcript on the orphan checkpoint branch.
pub fn create_checkpoint(
    state: &SessionState,
    transcript_path: Option<&Path>,
    attribution: Option<InitialAttribution>,
) -> Option<CheckpointID> {
    let checkpoint_id = CheckpointID::generate();

    // Prepare checkpoint data in a temp directory
    let tmp_dir = std::env::temp_dir().join(format!("chub-checkpoint-{}", checkpoint_id));
    let _ = fs::create_dir_all(&tmp_dir);

    let shard_path = checkpoint_id.shard_path();
    let session_dir = tmp_dir.join(&shard_path).join("0");
    let _ = fs::create_dir_all(&session_dir);

    // Write committed metadata
    let metadata = CommittedMetadata {
        cli_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        checkpoint_id: checkpoint_id.clone(),
        session_id: state.session_id.clone(),
        strategy: "chub-track".to_string(),
        created_at: now_iso8601(),
        branch: None,
        checkpoints_count: state.step_count,
        files_touched: state.files_touched.clone(),
        agent: state.agent_type.clone(),
        turn_id: state.turn_id.clone(),
        is_task: false,
        tool_use_id: None,
        transcript_identifier_at_start: state.transcript_identifier_at_start.clone(),
        checkpoint_transcript_start: state.checkpoint_transcript_start,
        token_usage: state.token_usage.clone(),
        summary: None,
        initial_attribution: attribution,
    };

    let meta_json = serde_json::to_string_pretty(&metadata).unwrap_or_default() + "\n";
    let _ = fs::write(session_dir.join("metadata.json"), &meta_json);

    // Copy transcript if available
    let transcript_rel = if let Some(tp) = transcript_path {
        if tp.exists() {
            let dest = session_dir.join("full.jsonl");
            let _ = fs::copy(tp, &dest);

            // Write content hash
            if let Ok(content) = fs::read(tp) {
                use sha2::{Digest, Sha256};
                let hash = format!("{:x}", Sha256::digest(&content));
                let _ = fs::write(session_dir.join("content_hash.txt"), &hash);
            }
            "0/full.jsonl".to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Write prompt
    if let Some(ref prompt) = state.first_prompt {
        let _ = fs::write(session_dir.join("prompt.txt"), prompt);
    }

    // Write root checkpoint summary
    let summary = CheckpointSummary {
        cli_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        checkpoint_id: checkpoint_id.clone(),
        strategy: "chub-track".to_string(),
        branch: None,
        checkpoints_count: state.step_count,
        files_touched: state.files_touched.clone(),
        sessions: vec![SessionFilePaths {
            metadata: "0/metadata.json".to_string(),
            transcript: transcript_rel,
            content_hash: "0/content_hash.txt".to_string(),
            prompt: "0/prompt.txt".to_string(),
        }],
        token_usage: state.token_usage.clone(),
    };

    let summary_json = serde_json::to_string_pretty(&summary).unwrap_or_default() + "\n";
    let root_dir = tmp_dir.join(&shard_path);
    let _ = fs::write(root_dir.join("metadata.json"), &summary_json);

    // Commit to orphan branch using git
    let committed =
        commit_to_checkpoint_branch(&tmp_dir, &shard_path, &state.session_id, &checkpoint_id);

    // Cleanup temp
    let _ = fs::remove_dir_all(&tmp_dir);

    if committed {
        Some(checkpoint_id)
    } else {
        None
    }
}

/// Commit checkpoint data to the orphan branch.
fn commit_to_checkpoint_branch(
    tmp_dir: &Path,
    shard_path: &str,
    session_id: &str,
    checkpoint_id: &CheckpointID,
) -> bool {
    // Ensure orphan branch exists
    ensure_checkpoint_branch();

    // Use git worktree or direct tree manipulation
    // For simplicity, use a temporary checkout approach
    let worktree_dir = std::env::temp_dir().join(format!("chub-wt-{}", checkpoint_id));

    // Create a temporary worktree for the checkpoint branch
    let wt_result = Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(worktree_dir.to_str().unwrap_or(""))
        .arg(CHECKPOINT_BRANCH)
        .output();

    if wt_result.is_err() || !wt_result.as_ref().unwrap().status.success() {
        // Fallback: try without worktree (direct git operations)
        return commit_direct(tmp_dir, shard_path, session_id, checkpoint_id);
    }

    // Copy checkpoint files
    let dest_dir = worktree_dir.join(shard_path);
    let _ = fs::create_dir_all(&dest_dir);
    copy_dir_recursive(&tmp_dir.join(shard_path), &dest_dir);

    // Stage and commit
    let success = Command::new("git")
        .args(["-C", worktree_dir.to_str().unwrap_or(""), "add", "."])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        && Command::new("git")
            .args([
                "-C",
                worktree_dir.to_str().unwrap_or(""),
                "commit",
                "-m",
                &format!(
                    "Checkpoint: {}",
                    &checkpoint_id.0[..12.min(checkpoint_id.0.len())]
                ),
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    // Cleanup worktree
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(worktree_dir.to_str().unwrap_or(""))
        .output();

    success
}

/// Direct commit approach without worktree (fallback).
fn commit_direct(
    tmp_dir: &Path,
    shard_path: &str,
    session_id: &str,
    checkpoint_id: &CheckpointID,
) -> bool {
    // Use git hash-object + update-index + write-tree + commit-tree
    // This is more complex but doesn't require a worktree

    let src_dir = tmp_dir.join(shard_path);
    if !src_dir.is_dir() {
        return false;
    }

    // Get the current tree of the checkpoint branch
    let parent = Command::new("git")
        .args(["rev-parse", CHECKPOINT_BRANCH])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() || !o.status.success() {
                None
            } else {
                Some(s)
            }
        });

    // For each file in the checkpoint, hash it and build a tree
    let mut blobs: Vec<(String, String)> = Vec::new();
    collect_files(&src_dir, &src_dir, &mut blobs);

    if blobs.is_empty() {
        return false;
    }

    // Hash all blobs
    let mut index_entries = Vec::new();
    for (rel_path, abs_path) in &blobs {
        let hash = Command::new("git")
            .args(["hash-object", "-w", abs_path])
            .output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            });
        if let Some(hash) = hash {
            index_entries.push((format!("{}/{}", shard_path, rel_path), hash));
        }
    }

    // Build index and tree using a temporary index
    let tmp_index = std::env::temp_dir().join(format!("chub-index-{}", checkpoint_id));

    // If we have a parent, read its tree first
    if let Some(ref parent_hash) = parent {
        let _ = Command::new("git")
            .env("GIT_INDEX_FILE", tmp_index.to_str().unwrap_or(""))
            .args(["read-tree", parent_hash])
            .output();
    }

    // Add our entries
    for (path, hash) in &index_entries {
        let _ = Command::new("git")
            .env("GIT_INDEX_FILE", tmp_index.to_str().unwrap_or(""))
            .args(["update-index", "--add", "--cacheinfo", "100644", hash, path])
            .output();
    }

    // Write tree
    let tree = Command::new("git")
        .env("GIT_INDEX_FILE", tmp_index.to_str().unwrap_or(""))
        .args(["write-tree"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        });

    let _ = fs::remove_file(&tmp_index);

    let tree = match tree {
        Some(t) => t,
        None => return false,
    };

    // Create commit
    let msg = format!(
        "Checkpoint: {}\n\nEntire-Session: {}\nEntire-Strategy: chub-track",
        &checkpoint_id.0[..12.min(checkpoint_id.0.len())],
        session_id
    );

    let mut commit_args = vec!["commit-tree".to_string(), tree];
    if let Some(ref parent_hash) = parent {
        commit_args.push("-p".to_string());
        commit_args.push(parent_hash.clone());
    }
    commit_args.push("-m".to_string());
    commit_args.push(msg);

    let commit = Command::new("git")
        .args(&commit_args)
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        });

    if let Some(commit_hash) = commit {
        // Update branch ref
        Command::new("git")
            .args([
                "update-ref",
                &format!("refs/heads/{}", CHECKPOINT_BRANCH),
                &commit_hash,
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    }
}

/// Ensure the checkpoint orphan branch exists.
fn ensure_checkpoint_branch() {
    let exists = Command::new("git")
        .args(["rev-parse", "--verify", CHECKPOINT_BRANCH])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !exists {
        // Create orphan branch with empty tree
        let empty_tree = Command::new("git")
            .args(["hash-object", "-t", "tree", "/dev/null"])
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
            .unwrap_or_else(|| {
                // Fallback: create empty tree manually
                "4b825dc642cb6eb9a060e54bf899d69f7264209e".to_string()
            });

        let commit = Command::new("git")
            .args([
                "commit-tree",
                &empty_tree,
                "-m",
                "Initialize checkpoint branch",
            ])
            .output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            });

        if let Some(hash) = commit {
            let _ = Command::new("git")
                .args([
                    "update-ref",
                    &format!("refs/heads/{}", CHECKPOINT_BRANCH),
                    &hash,
                ])
                .output();
        }
    }
}

/// List checkpoints from the orphan branch.
pub fn list_checkpoints() -> Vec<CheckpointSummary> {
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", CHECKPOINT_BRANCH])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut summaries = Vec::new();
    let mut seen_checkpoints: std::collections::HashSet<String> = std::collections::HashSet::new();

    for line in text.lines() {
        // Look for root metadata.json files: <xx>/<rest>/metadata.json
        // But NOT <xx>/<rest>/0/metadata.json (those are per-session)
        let parts: Vec<&str> = line.split('/').collect();
        if parts.len() == 3 && parts[2] == "metadata.json" {
            let checkpoint_id = format!("{}{}", parts[0], parts[1]);
            if seen_checkpoints.insert(checkpoint_id.clone()) {
                // Read the metadata
                if let Some(summary) = read_checkpoint_summary(&checkpoint_id) {
                    summaries.push(summary);
                }
            }
        }
    }

    summaries
}

/// Read a checkpoint summary from the orphan branch.
fn read_checkpoint_summary(checkpoint_id: &str) -> Option<CheckpointSummary> {
    let id = CheckpointID(checkpoint_id.to_string());
    let path = format!("{}/metadata.json", id.shard_path());

    let output = Command::new("git")
        .args(["show", &format!("{}:{}", CHECKPOINT_BRANCH, path)])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&content).ok()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn copy_dir_recursive(src: &Path, dst: &Path) {
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                let _ = fs::create_dir_all(&dst_path);
                copy_dir_recursive(&src_path, &dst_path);
            } else {
                let _ = fs::copy(&src_path, &dst_path);
            }
        }
    }
}

fn collect_files(base: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(base, &path, out);
            } else if let Ok(rel) = path.strip_prefix(base) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                out.push((rel_str, path.to_string_lossy().to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_metadata_json_compat() {
        let meta = CommittedMetadata {
            cli_version: Some("0.1.15".to_string()),
            checkpoint_id: CheckpointID("a3b2c4d5e6f7".to_string()),
            session_id: "2026-03-22-abc12345".to_string(),
            strategy: "chub-track".to_string(),
            created_at: "2026-03-22T10:00:00.000Z".to_string(),
            branch: None,
            checkpoints_count: 1,
            files_touched: vec!["src/main.rs".to_string()],
            agent: Some("Claude Code".to_string()),
            turn_id: None,
            is_task: false,
            tool_use_id: None,
            transcript_identifier_at_start: None,
            checkpoint_transcript_start: 0,
            token_usage: Some(TokenUsage {
                input_tokens: 1000,
                output_tokens: 500,
                ..Default::default()
            }),
            summary: None,
            initial_attribution: None,
        };

        let json = serde_json::to_string_pretty(&meta).unwrap();
        // Verify camelCase field names (entire.io compatible)
        assert!(json.contains("\"checkpointID\""));
        assert!(json.contains("\"sessionID\""));
        assert!(json.contains("\"filesTouched\""));
        assert!(json.contains("\"Claude Code\""));
        assert!(json.contains("\"inputTokens\""));

        // Roundtrip
        let parsed: CommittedMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.checkpoint_id.0, "a3b2c4d5e6f7");
    }
}
