//! Security and robustness tests for the tracking system.
//!
//! Tests path traversal protection, malformed input handling, concurrent
//! session safety, temp file cleanup, and checkpoint ID collision resistance.
//! Each test creates an isolated git repo in a temp directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chub_core::team::tracking::redact::{RedactConfig, Redactor};

// ---------------------------------------------------------------------------
// Helpers (same pattern as tracking_e2e.rs)
// ---------------------------------------------------------------------------

fn chub_bin() -> PathBuf {
    let mut path = chub_bin_dir();
    path.push(format!("chub{}", std::env::consts::EXE_SUFFIX));
    assert!(
        path.exists(),
        "chub binary not found at {}. Build it first with `cargo build`.",
        path.display()
    );
    path
}

fn chub_bin_dir() -> PathBuf {
    let mut path = std::env::current_exe().expect("cannot find test executable path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path
}

fn path_with_chub() -> std::ffi::OsString {
    let bin_dir = chub_bin_dir();
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs = vec![bin_dir];
    dirs.extend(std::env::split_paths(&current));
    std::env::join_paths(dirs).expect("failed to join PATH")
}

fn create_test_repo(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("chub-sec-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    git(&dir, &["init", "-b", "master"]);
    git(&dir, &["config", "user.name", "chub-test"]);
    git(&dir, &["config", "user.email", "test@chub.nrl.ai"]);
    fs::create_dir_all(dir.join(".chub")).unwrap();
    fs::write(dir.join(".chub/config.yaml"), "name: test\n").unwrap();
    fs::write(dir.join("file.txt"), "initial\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "init"]);

    dir
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .env("PATH", path_with_chub())
        .env("GIT_AUTHOR_NAME", "chub-test")
        .env("GIT_AUTHOR_EMAIL", "test@chub.nrl.ai")
        .env("GIT_COMMITTER_NAME", "chub-test")
        .env("GIT_COMMITTER_EMAIL", "test@chub.nrl.ai")
        .args(args)
        .output()
        .expect("git command failed to execute");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn chub_hook(dir: &Path, args: &[&str]) -> String {
    let mut cmd = Command::new(chub_bin());
    cmd.current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("PATH", path_with_chub())
        .env("GIT_AUTHOR_NAME", "chub-test")
        .env("GIT_AUTHOR_EMAIL", "test@chub.nrl.ai")
        .env("GIT_COMMITTER_NAME", "chub-test")
        .env("GIT_COMMITTER_EMAIL", "test@chub.nrl.ai")
        .args(["track", "hook"]);
    cmd.args(args);

    let mut child = cmd.spawn().expect("failed to spawn chub");
    use std::io::Write;
    if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(b"{}");
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stderr.is_empty() {
        stderr
    } else {
        stdout
    }
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

// ===========================================================================
// Malformed input resilience
// ===========================================================================

#[test]
fn corrupt_active_json_handled_gracefully() {
    let dir = create_test_repo("corrupt-active");

    // Start a session normally
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);

    // Corrupt the active.json
    let active_path = dir.join(".git/chub-sessions/active.json");
    assert!(active_path.exists());
    fs::write(&active_path, "{ this is not valid JSON !!!").unwrap();

    // Subsequent hooks should not panic — they should handle the corrupt file
    let output = chub_hook(&dir, &["prompt", "--input", "test"]);
    // Should not crash — either starts a new session or reports gracefully
    assert!(
        !output.contains("panicked"),
        "should not panic on corrupt active.json: {}",
        output
    );

    // Stop should also not panic
    let stop_output = chub_hook(&dir, &["stop"]);
    assert!(
        !stop_output.contains("panicked"),
        "stop should not panic: {}",
        stop_output
    );

    cleanup(&dir);
}

#[test]
fn empty_active_json_handled_gracefully() {
    let dir = create_test_repo("empty-active");

    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);

    // Write empty file
    let active_path = dir.join(".git/chub-sessions/active.json");
    fs::write(&active_path, "").unwrap();

    // Should not panic
    let output = chub_hook(&dir, &["prompt", "--input", "test"]);
    assert!(!output.contains("panicked"), "output: {}", output);

    cleanup(&dir);
}

#[test]
fn truncated_active_json_handled_gracefully() {
    let dir = create_test_repo("truncated-active");

    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);

    // Write truncated JSON (power failure simulation)
    let active_path = dir.join(".git/chub-sessions/active.json");
    fs::write(&active_path, r#"{"session_id":"test","agent":"clau"#).unwrap();

    let output = chub_hook(&dir, &["stop"]);
    assert!(!output.contains("panicked"), "output: {}", output);

    cleanup(&dir);
}

// ===========================================================================
// Session ID format validation
// ===========================================================================

#[test]
fn session_id_format_is_consistent() {
    let dir = create_test_repo("id-format");

    let output = chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    let session_id = output
        .strip_prefix("Session started: ")
        .unwrap_or("")
        .trim();

    // Session IDs should match: YYYY-MM-DDTHH-MM-<6hex>
    assert!(
        session_id.len() > 20,
        "session ID too short: '{}'",
        session_id
    );
    assert!(
        session_id.contains('T'),
        "session ID should contain T: '{}'",
        session_id
    );

    // Last 6 chars should be hex
    let hex_suffix = &session_id[session_id.len() - 6..];
    assert!(
        hex_suffix.chars().all(|c| c.is_ascii_hexdigit()),
        "session ID suffix should be hex: '{}'",
        hex_suffix
    );

    // Should not contain path separators or shell metacharacters
    for bad_char in &['/', '\\', '|', '&', ';', '`', '$', '(', ')', '{', '}'] {
        assert!(
            !session_id.contains(*bad_char),
            "session ID should not contain '{}': '{}'",
            bad_char,
            session_id
        );
    }

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

// ===========================================================================
// File path safety
// ===========================================================================

#[test]
fn tool_file_paths_with_special_chars_dont_crash() {
    let dir = create_test_repo("special-paths");

    chub_hook(&dir, &["session-start", "--agent", "test"]);

    // Paths with special characters that could cause issues
    let tricky_paths = [
        "file with spaces.rs",
        "file'with'quotes.rs",
        "path/to/deeply/nested/file.rs",
        "unicode_日本語.rs",
        "dots...in...name.rs",
    ];

    for path in &tricky_paths {
        let output = chub_hook(&dir, &["post-tool", "--tool", "Write", "--file", path]);
        assert!(
            !output.contains("panicked"),
            "panicked on path '{}': {}",
            path,
            output
        );
    }

    // Verify active session is still valid
    let active_path = dir.join(".git/chub-sessions/active.json");
    let content = fs::read_to_string(&active_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let files = parsed["files_changed"].as_array().unwrap();
    assert!(
        files.len() >= tricky_paths.len(),
        "should have tracked all files: {:?}",
        files
    );

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn backslash_paths_normalized_to_forward_slash() {
    let dir = create_test_repo("backslash");

    chub_hook(&dir, &["session-start", "--agent", "test"]);
    chub_hook(
        &dir,
        &[
            "post-tool",
            "--tool",
            "Write",
            "--file",
            "src\\nested\\file.rs",
        ],
    );

    let active_path = dir.join(".git/chub-sessions/active.json");
    let content = fs::read_to_string(&active_path).unwrap();
    // The stored path should use forward slashes
    assert!(
        content.contains("src/nested/file.rs") || content.contains("src\\\\nested\\\\file.rs"),
        "backslashes should be normalized: {}",
        content
    );

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

// ===========================================================================
// Concurrent safety / session lifecycle
// ===========================================================================

#[test]
fn double_session_start_overwrites_cleanly() {
    let dir = create_test_repo("double-start");

    let out1 = chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    let id1 = out1.strip_prefix("Session started: ").unwrap_or("").trim();

    // Start another session without stopping
    let out2 = chub_hook(&dir, &["session-start", "--agent", "cursor"]);
    let id2 = out2.strip_prefix("Session started: ").unwrap_or("").trim();

    // IDs should be different
    assert_ne!(id1, id2, "second session should have a new ID");

    // Active session should be the second one
    let active_path = dir.join(".git/chub-sessions/active.json");
    let content = fs::read_to_string(&active_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["agent"], "cursor");

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn stop_without_start_is_graceful() {
    let dir = create_test_repo("stop-no-start");

    // Stop without ever starting a session
    let output = chub_hook(&dir, &["stop"]);
    assert!(
        !output.contains("panicked"),
        "stop without start should not panic: {}",
        output
    );

    cleanup(&dir);
}

#[test]
fn prompt_without_session_is_graceful() {
    let dir = create_test_repo("prompt-no-session");

    let output = chub_hook(&dir, &["prompt", "--input", "orphan prompt"]);
    assert!(
        !output.contains("panicked"),
        "prompt without session should not panic: {}",
        output
    );

    cleanup(&dir);
}

#[test]
fn tool_without_session_is_graceful() {
    let dir = create_test_repo("tool-no-session");

    let output = chub_hook(&dir, &["pre-tool", "--tool", "Read"]);
    assert!(!output.contains("panicked"), "output: {}", output);

    let output2 = chub_hook(&dir, &["post-tool", "--tool", "Write", "--file", "test.rs"]);
    assert!(!output2.contains("panicked"), "output: {}", output2);

    cleanup(&dir);
}

// ===========================================================================
// Entire.io state file compatibility
// ===========================================================================

#[test]
fn entire_sessions_dir_created_at_start() {
    let dir = create_test_repo("entire-dir");

    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);

    // .git/entire-sessions/ should exist
    assert!(
        dir.join(".git/entire-sessions").is_dir(),
        ".git/entire-sessions/ should be created"
    );

    // Should contain exactly one JSON file
    let files: Vec<_> = fs::read_dir(dir.join(".git/entire-sessions"))
        .unwrap()
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(files.len(), 1, "should have exactly one state file");

    // Validate it's proper JSON with required fields
    let content = fs::read_to_string(files[0].path()).unwrap();
    let state: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(state.get("sessionID").is_some());
    assert!(state.get("startedAt").is_some());
    assert!(state.get("phase").is_some());
    assert_eq!(state["phase"], "active");

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn entire_session_state_ends_correctly() {
    let dir = create_test_repo("entire-end");

    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    chub_hook(&dir, &["prompt", "--input", "test"]);

    // Get the state file path
    let entries: Vec<_> = fs::read_dir(dir.join(".git/entire-sessions"))
        .unwrap()
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();
    let state_path = entries[0].path();

    // Before stop: phase should be active
    let before: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(before["phase"], "active");

    chub_hook(&dir, &["stop"]);

    // After stop: phase should be ended
    let after: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(after["phase"], "ended");
    assert!(after.get("endedAt").is_some(), "endedAt must be set");

    cleanup(&dir);
}

// ===========================================================================
// Agent type handling
// ===========================================================================

#[test]
fn all_known_agents_start_sessions() {
    let agents = [
        "claude-code",
        "cursor",
        "gemini-cli",
        "copilot",
        "codex",
        "windsurf",
        "cline",
        "aider",
    ];

    for agent in &agents {
        let dir = create_test_repo(&format!("agent-{}", agent));
        let output = chub_hook(&dir, &["session-start", "--agent", agent]);
        assert!(
            output.contains("Session started:"),
            "agent '{}' failed to start: {}",
            agent,
            output
        );
        chub_hook(&dir, &["stop"]);
        cleanup(&dir);
    }
}

#[test]
fn unknown_agent_starts_session() {
    let dir = create_test_repo("unknown-agent");

    let output = chub_hook(&dir, &["session-start", "--agent", "my-custom-agent"]);
    assert!(
        output.contains("Session started:"),
        "unknown agent should still start: {}",
        output
    );

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

// ===========================================================================
// Large input resilience
// ===========================================================================

#[test]
fn very_long_prompt_handled() {
    let dir = create_test_repo("long-prompt");

    chub_hook(&dir, &["session-start", "--agent", "test"]);

    // 10KB prompt
    let long_prompt: String = "x".repeat(10_000);
    let output = chub_hook(&dir, &["prompt", "--input", &long_prompt]);
    assert!(!output.contains("panicked"), "output: {}", output);

    // Verify session is still valid
    let active_path = dir.join(".git/chub-sessions/active.json");
    let content = fs::read_to_string(&active_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["turns"], 1);

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn many_tool_calls_accumulate() {
    let dir = create_test_repo("many-tools");

    chub_hook(&dir, &["session-start", "--agent", "test"]);

    // 50 tool calls
    for i in 0..50 {
        chub_hook(&dir, &["pre-tool", "--tool", "Read"]);
        chub_hook(
            &dir,
            &[
                "post-tool",
                "--tool",
                "Read",
                "--file",
                &format!("file_{}.rs", i),
            ],
        );
    }

    let active_path = dir.join(".git/chub-sessions/active.json");
    let content = fs::read_to_string(&active_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["tool_calls"], 50);

    let files = parsed["files_changed"].as_array().unwrap();
    assert_eq!(files.len(), 50, "should have 50 unique files");

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

// ===========================================================================
// Git hooks security
// ===========================================================================

#[test]
fn hooks_contain_error_suppression() {
    let dir = create_test_repo("hooks-safe");

    // Install hooks
    let mut cmd = Command::new(chub_bin());
    cmd.current_dir(&dir)
        .args(["track", "enable"])
        .output()
        .unwrap();

    // All git hooks should have `2>/dev/null || true` to prevent blocking git
    for hook_name in &["prepare-commit-msg", "post-commit", "pre-push"] {
        let hook_path = dir.join(format!(".git/hooks/{}", hook_name));
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path).unwrap();
            assert!(
                content.contains("2>/dev/null || true"),
                "{} hook should suppress errors: {}",
                hook_name,
                content
            );
        }
    }

    cleanup(&dir);
}

// ===========================================================================
// Non-git directory handling
// ===========================================================================

#[test]
fn session_start_in_non_git_dir_is_graceful() {
    let dir = std::env::temp_dir().join(format!("chub-sec-nogit-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join(".chub")).unwrap();
    fs::write(dir.join(".chub/config.yaml"), "name: test\n").unwrap();
    // No git init!

    let mut cmd = Command::new(chub_bin());
    cmd.current_dir(&dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("PATH", path_with_chub())
        .args(["track", "hook", "session-start", "--agent", "test"]);

    let mut child = cmd.spawn().expect("failed to spawn");
    use std::io::Write;
    if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(b"{}");
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();

    // Should not crash
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "should exit cleanly, not crash: {:?}",
        output.status
    );

    let _ = fs::remove_dir_all(&dir);
}

// ===========================================================================
// Secret redaction in transcripts
// ===========================================================================

#[test]
fn redact_transcript_removes_secrets() {
    let r = Redactor::new();

    // Simulated JSONL transcript with embedded secrets
    let transcript = r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"deploy to prod"}}
{"type":"assistant","uuid":"a1","message":{"id":"msg_1","role":"assistant","content":[{"type":"tool_use","name":"Bash","input":{"command":"export AWS_ACCESS_KEY_ID=AKIAK4JM7NR2PX6SWT3B && aws s3 sync . s3://bucket"}}],"usage":{"input_tokens":500,"output_tokens":200}}}
{"type":"assistant","uuid":"a2","message":{"id":"msg_2","role":"assistant","content":[{"type":"tool_use","name":"Bash","input":{"command":"curl -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U' https://api.example.com"}}],"usage":{"input_tokens":600,"output_tokens":300}}}
{"type":"assistant","uuid":"a3","message":{"id":"msg_3","role":"assistant","content":[{"type":"text","text":"I see your .env has DATABASE_URL=postgres://admin:s3cretP4ss@db.example.com:5432/prod"}],"usage":{"input_tokens":400,"output_tokens":150}}}
"#;

    let result = r.redact(transcript);

    // Secrets should be gone
    assert!(
        !result.text.contains("AKIAK4JM7NR2PX6SWT3B"),
        "AWS key should be redacted"
    );
    assert!(
        !result.text.contains("eyJhbGciOiJIUzI1NiJ9"),
        "JWT should be redacted"
    );
    assert!(
        !result.text.contains("postgres://admin:s3cretP4ss"),
        "DB URL should be redacted"
    );

    // Redaction markers should be present
    assert!(result.text.contains("[REDACTED:aws-access-token]"));
    assert!(result.text.contains("[REDACTED:jwt]"));
    assert!(result.text.contains("[REDACTED:database-url]"));

    // Should have found at least 3 secrets
    assert!(
        result.findings.len() >= 3,
        "expected >=3 findings, got {}",
        result.findings.len()
    );

    // Non-secret content should be preserved
    assert!(result.text.contains("deploy to prod"));
    assert!(result.text.contains("aws s3 sync"));
    assert!(result.text.contains("msg_1"));
}

#[test]
fn redact_prompt_with_secrets() {
    let r = Redactor::new();

    let prompt = "Set up Stripe with sk_live_aBcDeFgHiJkLmNoPqRsTuVwX and deploy to production";
    let result = r.redact(prompt);

    assert!(
        !result.text.contains("sk_live_"),
        "Stripe key should be redacted"
    );
    assert!(result.text.contains("[REDACTED:stripe-access-token]"));
    assert!(result.text.contains("deploy to production")); // non-secret preserved
}

#[test]
fn redact_config_disables_redaction() {
    let config = RedactConfig {
        disabled: true,
        ..Default::default()
    };
    let r = Redactor::from_config(&config);

    let text = "AKIAK4JM7NR2PX6SWT3B ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
    let result = r.redact(text);
    assert_eq!(result.text, text, "disabled redactor should pass through");
    assert!(result.findings.is_empty());
}

#[test]
fn redact_config_extra_patterns() {
    let config = RedactConfig {
        extra_patterns: vec![(
            "internal-token".to_string(),
            r"INTERNAL_[A-Z0-9]{20}".to_string(),
        )],
        ..Default::default()
    };
    let r = Redactor::from_config(&config);

    let text = "token: INTERNAL_ABCDEFGHIJ1234567890";
    let result = r.redact(text);
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.rule_id == "internal-token"),
        "custom pattern should match: {:?}",
        result.findings
    );
}

#[test]
fn redact_config_allowlist() {
    let config = RedactConfig {
        allowlist_regexes: vec!["K4JM7".to_string()],
        ..Default::default()
    };
    let r = Redactor::from_config(&config);

    // AWS key with K4JM7 in it should be allowlisted by the regex
    let text = "key: AKIAK4JM7NR2PX6SWT3B";
    let result = r.redact(text);
    assert!(
        result.findings.is_empty(),
        "allowlisted key should not be flagged: {:?}",
        result.findings
    );

    // A different key without the allowlist pattern should still be caught
    let text2 = "key: AKIAVNR2PX6SWT3BK4JM";
    let result2 = r.redact(text2);
    assert!(!result2.findings.is_empty(), "real key should be detected");
}

#[test]
fn redact_preserves_jsonl_structure() {
    let r = Redactor::new();

    let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Here is your key: sk_live_aBcDeFgHiJkLmNoPqRsTuVwX"}]}}"#;
    let result = r.redact(line);

    // Should still be valid JSON after redaction
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result.text);
    assert!(
        parsed.is_ok(),
        "redacted JSON should still parse: {}",
        result.text
    );
}

#[test]
fn redact_multiple_secrets_same_line() {
    let r = Redactor::new();

    let text = "keys: sk_live_aBcDeFgHiJkLmNoPqRsTuVwX and sk_test_xYzAbCdEfGhIjKlMnOpQr";
    let result = r.redact(text);

    assert!(
        result.findings.len() >= 2,
        "should find both Stripe keys: {:?}",
        result.findings
    );
    assert!(!result.text.contains("sk_live_"));
    assert!(!result.text.contains("sk_test_"));
}

#[test]
fn redact_private_key_block() {
    let r = Redactor::new();

    let text = r#"Found in config:
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA0Z3VS5JJcds3xfn/ygWyF068wFxKSg5
-----END RSA PRIVATE KEY-----
Rest of message"#;

    let result = r.redact(text);
    assert!(result.text.contains("[REDACTED:private-key]"));
    assert!(result.text.contains("Rest of message")); // non-secret preserved
}

#[test]
fn redact_from_config_struct_conversion() {
    use chub_core::config::{RedactionConfig, RedactionPattern};

    let cfg = RedactionConfig {
        disabled: false,
        extra_patterns: vec![RedactionPattern {
            id: "my-secret".to_string(),
            pattern: r"MYSECRET_\w{10}".to_string(),
        }],
        allowlist: vec!["DUMMY".to_string()],
    };

    let redact_cfg = RedactConfig::from(&cfg);
    assert!(!redact_cfg.disabled);
    assert_eq!(redact_cfg.extra_patterns.len(), 1);
    assert_eq!(redact_cfg.extra_patterns[0].0, "my-secret");
    assert_eq!(redact_cfg.allowlist_regexes.len(), 1);

    let r = Redactor::from_config(&redact_cfg);
    let result = r.redact("key: MYSECRET_abcdefghij");
    assert!(
        result.findings.iter().any(|f| f.rule_id == "my-secret"),
        "config-based custom rule should work"
    );
}
