use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};

use crate::config::chub_dir;

static FIRST_RUN: AtomicBool = AtomicBool::new(false);

/// Get or create a stable, anonymous client ID.
/// Reads from ~/.chub/client_id, or generates from machine UUID via SHA-256.
/// Sets the first-run flag if a new ID is generated.
pub fn get_or_create_client_id() -> Option<String> {
    let chub = chub_dir();
    let id_path = chub.join("client_id");

    // Try to read existing
    if let Ok(existing) = fs::read_to_string(&id_path) {
        let trimmed = existing.trim();
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(trimmed.to_string());
        }
    }

    // Generate from machine UUID — this is a first-time user
    let uuid = get_machine_uuid()?;
    let mut hasher = Sha256::new();
    hasher.update(uuid.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let _ = fs::create_dir_all(&chub);
    let _ = crate::util::atomic_write(&id_path, hash.as_bytes());

    FIRST_RUN.store(true, Ordering::Relaxed);

    Some(hash)
}

/// Returns true if this is the first time the CLI has run on this machine.
/// Only valid after `get_or_create_client_id()` has been called.
pub fn is_first_run() -> bool {
    FIRST_RUN.load(Ordering::Relaxed)
}

#[cfg(target_os = "windows")]
fn get_machine_uuid() -> Option<String> {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography",
            "/v",
            "MachineGuid",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(pos) = line.find("REG_SZ") {
            return Some(line[pos + "REG_SZ".len()..].trim().to_string());
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_machine_uuid() -> Option<String> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("IOPlatformUUID") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 4 {
                return Some(parts[3].to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn get_machine_uuid() -> Option<String> {
    fs::read_to_string("/etc/machine-id")
        .or_else(|_| fs::read_to_string("/var/lib/dbus/machine-id"))
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn get_machine_uuid() -> Option<String> {
    None
}

/// Auto-detect the AI coding tool from environment variables.
pub fn detect_agent() -> &'static str {
    if std::env::var("CLAUDECODE").is_ok()
        || std::env::var("CLAUDE_CODE").is_ok()
        || std::env::var("CLAUDE_CODE_SSE_PORT").is_ok()
        || std::env::var("CLAUDE_SESSION_ID").is_ok()
    {
        return "claude-code";
    }
    if std::env::var("CURSOR_SESSION_ID").is_ok() || std::env::var("CURSOR_TRACE_ID").is_ok() {
        return "cursor";
    }
    if std::env::var("CODEX_HOME").is_ok() || std::env::var("CODEX_SESSION").is_ok() {
        return "codex";
    }
    if std::env::var("WINDSURF_SESSION").is_ok() {
        return "windsurf";
    }
    if std::env::var("AIDER_MODEL").is_ok() || std::env::var("AIDER").is_ok() {
        return "aider";
    }
    if std::env::var("CLINE_SESSION").is_ok() {
        return "cline";
    }
    if std::env::var("GITHUB_COPILOT").is_ok() {
        return "copilot";
    }
    "unknown"
}

/// Detect the version of the AI coding tool, if available.
pub fn detect_agent_version() -> Option<String> {
    std::env::var("CLAUDE_CODE_VERSION")
        .or_else(|_| std::env::var("CURSOR_VERSION"))
        .ok()
}

/// Detect the model name from agent environment variables.
pub fn detect_model() -> Option<String> {
    std::env::var("CLAUDE_MODEL")
        .or_else(|_| std::env::var("ANTHROPIC_MODEL"))
        .or_else(|_| std::env::var("CURSOR_MODEL"))
        .or_else(|_| std::env::var("AIDER_MODEL"))
        .ok()
}
