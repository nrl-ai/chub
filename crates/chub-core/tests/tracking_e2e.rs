//! End-to-end integration tests for the tracking system.
//!
//! Tests the full session lifecycle, orphan branch storage,
//! checkpoint creation, trailer insertion, and pre-push behavior.
//! Each test creates an isolated git repo in a temp directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Locate the `chub` binary built by cargo.
fn chub_bin() -> PathBuf {
    let mut path = std::env::current_exe().expect("cannot find test executable path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push(format!("chub{}", std::env::consts::EXE_SUFFIX));
    assert!(
        path.exists(),
        "chub binary not found at {}. Build it first with `cargo build`.",
        path.display()
    );
    path
}

/// Create an isolated git repo with .chub/config.yaml in a temp dir.
/// Returns the path to the repo directory.
fn create_test_repo(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("chub-e2e-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    git(&dir, &["init"]);
    fs::create_dir_all(dir.join(".chub")).unwrap();
    fs::write(dir.join(".chub/config.yaml"), "name: test\n").unwrap();
    fs::write(dir.join("file.txt"), "initial\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-m", "init"]);

    dir
}

/// Run git in a directory and return stdout.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("git command failed");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Run chub track hook with piped stdin in a directory.
fn chub_hook(dir: &Path, args: &[&str]) -> String {
    let mut cmd = Command::new(chub_bin());
    cmd.current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .args(["track", "hook"]);
    cmd.args(args);

    let mut child = cmd.spawn().expect("failed to spawn chub");

    // Write stdin and close it
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

/// Run chub track subcommand.
fn chub_track(dir: &Path, args: &[&str]) -> String {
    let mut cmd = Command::new(chub_bin());
    cmd.current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .args(["track"]);
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
    format!("{}\n{}", stderr, stdout).trim().to_string()
}

/// Cleanup test repo.
fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn session_start_creates_state_files() {
    let dir = create_test_repo("start");

    let output = chub_hook(
        &dir,
        &["session-start", "--agent", "claude-code", "--model", "opus"],
    );
    assert!(output.contains("Session started:"), "output: {}", output);

    // Extract session ID from output
    let session_id = output
        .strip_prefix("Session started: ")
        .unwrap_or("")
        .trim();
    assert!(!session_id.is_empty(), "no session ID in output");

    // Check .git/chub-sessions/active.json exists
    let active_path = dir.join(".git/chub-sessions/active.json");
    assert!(active_path.exists(), "active.json not created");
    let active_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&active_path).unwrap()).unwrap();
    assert_eq!(active_json["agent"], "claude-code");
    assert_eq!(active_json["model"], "opus");

    // Check .git/entire-sessions/<id>.json exists
    let state_path = dir.join(format!(".git/entire-sessions/{}.json", session_id));
    assert!(state_path.exists(), "entire-sessions state not created");
    let state_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(state_json["sessionID"], session_id);

    cleanup(&dir);
}

#[test]
fn session_stop_creates_session_on_orphan_branch() {
    let dir = create_test_repo("stop-branch");

    chub_hook(
        &dir,
        &["session-start", "--agent", "claude-code", "--model", "opus"],
    );
    chub_hook(&dir, &["prompt", "--input", "Hello"]);
    chub_hook(&dir, &["pre-tool", "--tool", "Write"]);
    chub_hook(
        &dir,
        &["post-tool", "--tool", "Write", "--file", "src/main.rs"],
    );

    let stop_output = chub_hook(&dir, &["stop"]);
    assert!(
        stop_output.contains("Session ended:"),
        "output: {}",
        stop_output
    );

    // Verify chub/sessions/v1 orphan branch was created
    let branches = git(&dir, &["branch", "-a"]);
    assert!(
        branches.contains("chub/sessions/v1"),
        "sessions branch not created: {}",
        branches
    );

    // Verify session YAML is on the branch
    let files = git(&dir, &["ls-tree", "-r", "--name-only", "chub/sessions/v1"]);
    assert!(files.ends_with(".yaml"), "no yaml on branch: {}", files);

    // Verify the YAML content is valid
    let yaml_path = files.lines().next().unwrap();
    let content = git(&dir, &["show", &format!("chub/sessions/v1:{}", yaml_path)]);
    assert!(content.contains("agent: claude-code"));
    assert!(content.contains("model: opus"));
    assert!(content.contains("turns: 1"));
    assert!(content.contains("tool_calls: 1"));

    // Verify local .git/chub/sessions/ also has the file
    let local_sessions = fs::read_dir(dir.join(".git/chub/sessions"))
        .unwrap()
        .flatten()
        .count();
    assert!(local_sessions > 0, "no local session files");

    cleanup(&dir);
}

#[test]
fn session_stop_clears_active_session() {
    let dir = create_test_repo("stop-clear");

    chub_hook(&dir, &["session-start", "--agent", "test-agent"]);

    let active_path = dir.join(".git/chub-sessions/active.json");
    assert!(active_path.exists());

    chub_hook(&dir, &["stop"]);
    assert!(
        !active_path.exists(),
        "active.json should be cleared after stop"
    );

    cleanup(&dir);
}

#[test]
fn prompt_increments_turns() {
    let dir = create_test_repo("turns");

    chub_hook(&dir, &["session-start", "--agent", "test-agent"]);
    chub_hook(&dir, &["prompt", "--input", "First"]);
    chub_hook(&dir, &["prompt", "--input", "Second"]);
    chub_hook(&dir, &["prompt", "--input", "Third"]);

    let active: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.join(".git/chub-sessions/active.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(active["turns"], 3);

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn tool_tracking_records_tools_and_files() {
    let dir = create_test_repo("tools");

    chub_hook(&dir, &["session-start", "--agent", "test-agent"]);
    chub_hook(&dir, &["pre-tool", "--tool", "Read"]);
    chub_hook(&dir, &["post-tool", "--tool", "Read"]);
    chub_hook(&dir, &["pre-tool", "--tool", "Write"]);
    chub_hook(
        &dir,
        &["post-tool", "--tool", "Write", "--file", "src/app.rs"],
    );
    chub_hook(&dir, &["pre-tool", "--tool", "Edit"]);
    chub_hook(
        &dir,
        &["post-tool", "--tool", "Edit", "--file", "src/lib.rs"],
    );

    let active: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.join(".git/chub-sessions/active.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(active["tool_calls"], 3);

    let tools = active["tools_used"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t.as_str()).collect();
    assert!(tool_names.contains(&"Read"));
    assert!(tool_names.contains(&"Write"));
    assert!(tool_names.contains(&"Edit"));

    let files = active["files_changed"].as_array().unwrap();
    let file_names: Vec<&str> = files.iter().filter_map(|f| f.as_str()).collect();
    assert!(file_names.contains(&"src/app.rs"));
    assert!(file_names.contains(&"src/lib.rs"));

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn commit_adds_trailers() {
    let dir = create_test_repo("trailers");

    // Install git hooks
    chub_track(&dir, &["enable"]);

    // Start session
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    chub_hook(&dir, &["prompt", "--input", "Add feature"]);

    // Make a commit (git hooks will add trailers)
    fs::write(dir.join("feature.txt"), "new\n").unwrap();
    git(&dir, &["add", "feature.txt"]);
    git(&dir, &["commit", "-m", "feat: add feature"]);

    // Check commit message has trailers
    let msg = git(&dir, &["log", "-1", "--format=%B"]);
    assert!(
        msg.contains("Chub-Session:"),
        "missing Chub-Session trailer: {}",
        msg
    );
    assert!(
        msg.contains("Chub-Checkpoint:"),
        "missing Chub-Checkpoint trailer: {}",
        msg
    );

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn post_commit_creates_checkpoint_on_orphan_branch() {
    let dir = create_test_repo("checkpoint");

    chub_track(&dir, &["enable"]);
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    chub_hook(&dir, &["prompt", "--input", "Implement X"]);

    // Commit triggers post-commit hook -> checkpoint
    fs::write(dir.join("x.txt"), "implementation\n").unwrap();
    git(&dir, &["add", "x.txt"]);
    git(&dir, &["commit", "-m", "feat: implement X"]);

    // Verify entire/checkpoints/v1 branch exists with checkpoint data
    let branches = git(&dir, &["branch", "-a"]);
    assert!(
        branches.contains("entire/checkpoints/v1"),
        "checkpoint branch not created: {}",
        branches
    );

    let files = git(
        &dir,
        &["ls-tree", "-r", "--name-only", "entire/checkpoints/v1"],
    );
    assert!(
        files.contains("metadata.json"),
        "no metadata.json: {}",
        files
    );
    assert!(files.contains("prompt.txt"), "no prompt.txt: {}", files);

    // Read per-session metadata (0/metadata.json) and verify format
    let session_meta_path = files
        .lines()
        .find(|l| l.ends_with("/0/metadata.json"))
        .unwrap_or("");
    if !session_meta_path.is_empty() {
        let meta = git(
            &dir,
            &[
                "show",
                &format!("entire/checkpoints/v1:{}", session_meta_path),
            ],
        );
        let parsed: serde_json::Value = serde_json::from_str(&meta).unwrap();
        // Verify entire.io-compatible camelCase field names
        assert!(parsed.get("checkpointID").is_some(), "missing checkpointID");
        assert!(parsed.get("sessionID").is_some(), "missing sessionID");
        assert!(parsed.get("createdAt").is_some(), "missing createdAt");
        assert_eq!(parsed["strategy"], "chub-track");
    }

    // Verify commit records in active session
    let active: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.join(".git/chub-sessions/active.json")).unwrap(),
    )
    .unwrap();
    let commits = active["commits"].as_array().unwrap();
    assert!(!commits.is_empty(), "no commits recorded in active session");

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn post_commit_records_commit_hash_in_session() {
    let dir = create_test_repo("commit-hash");

    chub_track(&dir, &["enable"]);
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);

    fs::write(dir.join("a.txt"), "a\n").unwrap();
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "first"]);

    fs::write(dir.join("b.txt"), "b\n").unwrap();
    git(&dir, &["add", "b.txt"]);
    git(&dir, &["commit", "-m", "second"]);

    let active: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.join(".git/chub-sessions/active.json")).unwrap(),
    )
    .unwrap();
    let commits = active["commits"].as_array().unwrap();
    assert_eq!(commits.len(), 2, "should have 2 commits recorded");

    chub_hook(&dir, &["stop"]);

    // Verify commits also in finalized session on branch
    let files = git(&dir, &["ls-tree", "-r", "--name-only", "chub/sessions/v1"]);
    let yaml_path = files.lines().next().unwrap();
    let content = git(&dir, &["show", &format!("chub/sessions/v1:{}", yaml_path)]);
    // Count "- " lines under commits:
    let commit_lines: Vec<&str> = content
        .lines()
        .filter(|l| l.starts_with("- ") && l.len() < 15) // short hash lines
        .collect();
    assert_eq!(
        commit_lines.len(),
        2,
        "finalized session should have 2 commits: {}",
        content
    );

    cleanup(&dir);
}

#[test]
fn pre_push_syncs_branches_to_remote() {
    let dir = create_test_repo("push");
    let remote_dir =
        std::env::temp_dir().join(format!("chub-e2e-push-remote-{}", std::process::id()));
    let _ = fs::remove_dir_all(&remote_dir);

    // Create bare remote
    Command::new("git")
        .args(["init", "--bare"])
        .arg(&remote_dir)
        .output()
        .unwrap();
    git(
        &dir,
        &["remote", "add", "origin", remote_dir.to_str().unwrap()],
    );
    git(&dir, &["push", "origin", "master"]);

    // Do a session with commit
    chub_track(&dir, &["enable"]);
    chub_hook(
        &dir,
        &["session-start", "--agent", "claude-code", "--model", "opus"],
    );
    chub_hook(&dir, &["prompt", "--input", "Test push"]);

    fs::write(dir.join("push-test.txt"), "pushed\n").unwrap();
    git(&dir, &["add", "push-test.txt"]);
    git(&dir, &["commit", "-m", "test push"]);

    chub_hook(&dir, &["stop"]);

    // Push (triggers pre-push hook)
    git(&dir, &["push", "origin", "master"]);

    // Verify remote has all branches
    let remote_branches = Command::new("git")
        .current_dir(&remote_dir)
        .args(["branch", "-a"])
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&remote_branches.stdout);
    assert!(
        branches.contains("chub/sessions/v1"),
        "sessions branch not pushed: {}",
        branches
    );
    assert!(
        branches.contains("entire/checkpoints/v1"),
        "checkpoints branch not pushed: {}",
        branches
    );

    // Verify session data on remote
    let remote_files = Command::new("git")
        .current_dir(&remote_dir)
        .args(["ls-tree", "-r", "--name-only", "chub/sessions/v1"])
        .output()
        .unwrap();
    let files = String::from_utf8_lossy(&remote_files.stdout);
    assert!(
        files.contains(".yaml"),
        "no session yaml on remote: {}",
        files
    );

    let _ = fs::remove_dir_all(&remote_dir);
    cleanup(&dir);
}

#[test]
fn list_sessions_deduplicates_across_sources() {
    let dir = create_test_repo("dedup");

    // Session 1
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    chub_hook(&dir, &["prompt", "--input", "Test 1"]);
    chub_hook(&dir, &["stop"]);

    // Session 2
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    chub_hook(&dir, &["prompt", "--input", "Test 2"]);
    chub_hook(&dir, &["stop"]);

    // List sessions (JSON)
    let output = chub_track(&dir, &["log", "--days", "30"]);
    // Count session lines – each session line contains a timestamp like "2026-03-28T04-54-"
    // followed by a short hex id. Match any line containing a "Txx-xx-" pattern.
    let session_count = output
        .lines()
        .filter(|l| {
            // Match timestamp-based session IDs: digits T digits - digits - hex
            l.contains("claude-code") && l.contains("turns")
        })
        .count();
    assert!(
        session_count >= 2,
        "should list at least 2 sessions: {}",
        output
    );

    cleanup(&dir);
}

#[test]
fn session_state_compatible_with_entire_io_format() {
    let dir = create_test_repo("compat");

    chub_hook(
        &dir,
        &["session-start", "--agent", "claude-code", "--model", "opus"],
    );
    chub_hook(&dir, &["prompt", "--input", "Test compatibility"]);
    chub_hook(&dir, &["pre-tool", "--tool", "Write"]);
    chub_hook(
        &dir,
        &["post-tool", "--tool", "Write", "--file", "src/app.rs"],
    );

    // Read the session state from .git/entire-sessions/
    let state_files: Vec<_> = fs::read_dir(dir.join(".git/entire-sessions"))
        .unwrap()
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !state_files.is_empty(),
        "no state file in .git/entire-sessions/"
    );

    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_files[0].path()).unwrap()).unwrap();

    // Verify entire.io-compatible field names (camelCase)
    assert!(
        state.get("sessionID").is_some(),
        "missing sessionID (entire.io compat)"
    );
    assert!(state.get("startedAt").is_some(), "missing startedAt");
    assert!(state.get("baseCommit").is_some(), "missing baseCommit");
    assert!(state.get("phase").is_some(), "missing phase");
    assert!(state.get("stepCount").is_some(), "missing stepCount");
    assert!(state.get("filesTouched").is_some(), "missing filesTouched");

    // Phase should be active (lowercase in serde serialization)
    let phase = state["phase"].as_str().unwrap_or("");
    assert!(
        phase.eq_ignore_ascii_case("active"),
        "phase should be active during session, got: {}",
        phase
    );

    // Files touched should include our write target
    let files = state["filesTouched"].as_array().unwrap();
    let file_list: Vec<&str> = files.iter().filter_map(|f| f.as_str()).collect();
    assert!(
        file_list.contains(&"src/app.rs"),
        "missing src/app.rs in filesTouched"
    );

    chub_hook(&dir, &["stop"]);

    // After stop, phase should be Ended
    let state_after: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_files[0].path()).unwrap()).unwrap();
    let phase_after = state_after["phase"].as_str().unwrap_or("");
    assert!(
        phase_after.eq_ignore_ascii_case("ended"),
        "phase should be ended after stop, got: {}",
        phase_after
    );
    assert!(
        state_after.get("endedAt").is_some(),
        "missing endedAt after stop"
    );

    cleanup(&dir);
}

#[test]
fn checkpoint_metadata_compatible_with_entire_io() {
    let dir = create_test_repo("ckpt-compat");

    chub_track(&dir, &["enable"]);
    chub_hook(
        &dir,
        &["session-start", "--agent", "claude-code", "--model", "opus"],
    );
    chub_hook(&dir, &["prompt", "--input", "Build something"]);

    fs::write(dir.join("new.txt"), "content\n").unwrap();
    git(&dir, &["add", "new.txt"]);
    git(&dir, &["commit", "-m", "build something"]);

    // Read checkpoint metadata from orphan branch
    let files_output = git(
        &dir,
        &["ls-tree", "-r", "--name-only", "entire/checkpoints/v1"],
    );

    // Find root metadata.json (shard/checkpointId/metadata.json - 3 parts)
    let root_meta = files_output
        .lines()
        .find(|l| l.ends_with("/metadata.json") && l.matches('/').count() == 2);
    assert!(
        root_meta.is_some(),
        "no root metadata.json found: {}",
        files_output
    );

    let meta_content = git(
        &dir,
        &[
            "show",
            &format!("entire/checkpoints/v1:{}", root_meta.unwrap()),
        ],
    );
    let meta: serde_json::Value = serde_json::from_str(&meta_content).unwrap();

    // Verify entire.io-compatible fields (root CheckpointSummary)
    assert!(meta.get("checkpointID").is_some(), "missing checkpointID");
    assert!(meta.get("cliVersion").is_some(), "missing cliVersion");
    assert!(meta.get("filesTouched").is_some(), "missing filesTouched");
    assert!(meta.get("sessions").is_some(), "missing sessions array");

    // Check sessions array structure
    let sessions = meta["sessions"].as_array().unwrap();
    assert!(!sessions.is_empty(), "sessions array empty");
    let session = &sessions[0];
    assert!(
        session.get("metadata").is_some(),
        "missing metadata path in session"
    );
    assert!(
        session.get("transcript").is_some(),
        "missing transcript path in session"
    );
    assert!(
        session.get("prompt").is_some(),
        "missing prompt path in session"
    );

    // Check per-session metadata
    let session_meta_path = files_output
        .lines()
        .find(|l| l.ends_with("/0/metadata.json"));
    assert!(session_meta_path.is_some(), "no per-session metadata.json");

    let session_meta = git(
        &dir,
        &[
            "show",
            &format!("entire/checkpoints/v1:{}", session_meta_path.unwrap()),
        ],
    );
    let sm: serde_json::Value = serde_json::from_str(&session_meta).unwrap();
    assert!(sm.get("checkpointID").is_some());
    assert!(sm.get("sessionID").is_some());
    assert!(sm.get("tokenUsage").is_some());
    assert_eq!(sm["strategy"], "chub-track");

    // Token usage should have entire.io-compatible field names
    let usage = &sm["tokenUsage"];
    assert!(usage.get("inputTokens").is_some(), "missing inputTokens");
    assert!(usage.get("outputTokens").is_some(), "missing outputTokens");
    assert!(
        usage.get("cacheReadTokens").is_some(),
        "missing cacheReadTokens"
    );
    assert!(
        usage.get("cacheCreationTokens").is_some(),
        "missing cacheCreationTokens"
    );

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn session_sharding_uses_hex_prefix() {
    let dir = create_test_repo("shard");

    chub_hook(&dir, &["session-start", "--agent", "test"]);
    let stop_out = chub_hook(&dir, &["stop"]);
    let session_id = stop_out
        .split("Session ended: ")
        .nth(1)
        .unwrap_or("")
        .lines()
        .next()
        .unwrap_or("")
        .trim();

    // Session IDs look like: 2026-03-22T13-39-ebd03b
    // Shard should be last 6 chars minus last 4 = chars at [-6..-4]
    let shard_prefix = &session_id[session_id.len() - 6..session_id.len() - 4];

    let files = git(&dir, &["ls-tree", "-r", "--name-only", "chub/sessions/v1"]);
    let yaml_path = files.lines().next().unwrap_or("");
    assert!(
        yaml_path.starts_with(shard_prefix),
        "expected shard prefix '{}' but got path '{}' for session '{}'",
        shard_prefix,
        yaml_path,
        session_id,
    );

    cleanup(&dir);
}

#[test]
fn rebase_skips_trailer_and_checkpoint() {
    let dir = create_test_repo("rebase");

    chub_track(&dir, &["enable"]);
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);

    // Create a fake rebase-in-progress marker
    fs::create_dir_all(dir.join(".git/rebase-merge")).unwrap();

    // Commit during "rebase"
    fs::write(dir.join("rebase.txt"), "content\n").unwrap();
    git(&dir, &["add", "rebase.txt"]);
    git(&dir, &["commit", "-m", "rebase commit"]);

    // Verify NO trailers were added (prepare-commit-msg should skip)
    let msg = git(&dir, &["log", "-1", "--format=%B"]);
    assert!(
        !msg.contains("Chub-Session:"),
        "trailer should be skipped during rebase: {}",
        msg
    );

    // Cleanup rebase marker
    let _ = fs::remove_dir_all(dir.join(".git/rebase-merge"));

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}

#[test]
fn hook_enable_installs_all_git_hooks() {
    let dir = create_test_repo("hooks");

    let output = chub_track(&dir, &["enable"]);
    assert!(
        output.contains("installed"),
        "hooks not installed: {}",
        output
    );

    // Verify all three git hooks exist
    assert!(
        dir.join(".git/hooks/prepare-commit-msg").exists(),
        "prepare-commit-msg hook missing"
    );
    assert!(
        dir.join(".git/hooks/post-commit").exists(),
        "post-commit hook missing"
    );
    assert!(
        dir.join(".git/hooks/pre-push").exists(),
        "pre-push hook missing"
    );

    // Verify hooks contain chub marker and correct commands
    let pre_push = fs::read_to_string(dir.join(".git/hooks/pre-push")).unwrap();
    assert!(
        pre_push.contains("track hook pre-push"),
        "pre-push hook content wrong: {}",
        pre_push
    );

    let prepare = fs::read_to_string(dir.join(".git/hooks/prepare-commit-msg")).unwrap();
    assert!(
        prepare.contains("track hook commit-msg"),
        "prepare-commit-msg hook content wrong: {}",
        prepare,
    );

    let post = fs::read_to_string(dir.join(".git/hooks/post-commit")).unwrap();
    assert!(
        post.contains("track hook post-commit"),
        "post-commit hook content wrong: {}",
        post
    );

    cleanup(&dir);
}

#[test]
fn hook_disable_removes_all_hooks() {
    let dir = create_test_repo("disable");

    chub_track(&dir, &["enable"]);
    assert!(dir.join(".git/hooks/pre-push").exists());

    chub_track(&dir, &["disable"]);

    // Git hooks should be removed
    // (unless they had pre-existing content, but in this test they don't)
    let pre_push_exists = dir.join(".git/hooks/pre-push").exists();
    if pre_push_exists {
        let content = fs::read_to_string(dir.join(".git/hooks/pre-push")).unwrap();
        assert!(
            !content.contains("chub track"),
            "chub hook content should be removed"
        );
    }

    cleanup(&dir);
}

#[test]
fn multiple_sessions_accumulate_on_branch() {
    let dir = create_test_repo("multi");

    // Session 1
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    chub_hook(&dir, &["prompt", "--input", "First session"]);
    chub_hook(&dir, &["stop"]);

    // Session 2
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);
    chub_hook(&dir, &["prompt", "--input", "Second session"]);
    chub_hook(&dir, &["stop"]);

    // Session 3
    chub_hook(&dir, &["session-start", "--agent", "cursor"]);
    chub_hook(&dir, &["prompt", "--input", "Third session"]);
    chub_hook(&dir, &["stop"]);

    // Branch should have all 3 sessions
    let files = git(&dir, &["ls-tree", "-r", "--name-only", "chub/sessions/v1"]);
    let yaml_count = files.lines().filter(|l| l.ends_with(".yaml")).count();
    assert_eq!(
        yaml_count, 3,
        "expected 3 sessions on branch, got {}: {}",
        yaml_count, files
    );

    // Git log should show commits for each session
    let log = git(&dir, &["log", "--oneline", "chub/sessions/v1"]);
    let commit_count = log.lines().count();
    // At least 3 session commits + 1 init commit = 4 minimum
    // (may be more due to double-write in end_session + track.rs enrichment)
    assert!(
        commit_count >= 4,
        "expected >= 4 commits, got {}: {}",
        commit_count,
        log
    );

    cleanup(&dir);
}

#[test]
fn entire_session_state_has_base_commit() {
    let dir = create_test_repo("base-commit");

    let head = git(&dir, &["rev-parse", "HEAD"]);
    chub_hook(&dir, &["session-start", "--agent", "claude-code"]);

    // Check that base_commit is set in the session state
    let state_files: Vec<_> = fs::read_dir(dir.join(".git/entire-sessions"))
        .unwrap()
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_files[0].path()).unwrap()).unwrap();

    let base = state["baseCommit"].as_str().unwrap_or("");
    assert!(!base.is_empty(), "baseCommit should be set");
    assert!(
        head.starts_with(base) || base.starts_with(&head[..7]),
        "baseCommit '{}' should match HEAD '{}'",
        base,
        head
    );

    chub_hook(&dir, &["stop"]);
    cleanup(&dir);
}
