//! End-to-end integration tests for context docs features.
//!
//! Tests exercise the chub CLI for pins, profiles, annotations, context docs,
//! init, snapshots, and agent-config commands. Each test creates an isolated
//! .chub/ directory in a temp dir and uses CHUB_PROJECT_DIR to avoid polluting
//! the real repo.
//!
//! Because CHUB_PROJECT_DIR is a process-global env var and Rust runs tests
//! in parallel, we use a mutex to serialize all tests that depend on it.

use std::fs;
use std::process::Command;
use std::sync::Mutex;

/// Global mutex to serialize tests that use CHUB_PROJECT_DIR env var.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Set up an isolated .chub/ directory in a temp dir.
/// Returns the temp dir path and mutex guard.
fn setup_project() -> (tempfile::TempDir, std::sync::MutexGuard<'static, ()>) {
    let guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = tempfile::tempdir().unwrap();
    let chub_dir = tmp.path().join(".chub");
    fs::create_dir_all(chub_dir.join("annotations")).unwrap();
    fs::create_dir_all(chub_dir.join("context")).unwrap();
    fs::create_dir_all(chub_dir.join("profiles")).unwrap();
    fs::create_dir_all(chub_dir.join("snapshots")).unwrap();

    // Minimal config
    fs::write(
        chub_dir.join("config.yaml"),
        "name: test-project\nagent_rules:\n  targets:\n    - claude.md\n",
    )
    .unwrap();

    // Empty pins
    fs::write(chub_dir.join("pins.yaml"), "pins: []\n").unwrap();

    // A base profile
    fs::write(
        chub_dir.join("profiles/base.yaml"),
        "name: Base\ndescription: \"Base profile\"\nrules:\n  - \"always use UTF-8\"\ncontext: []\n",
    )
    .unwrap();

    // A child profile with inheritance
    fs::write(
        chub_dir.join("profiles/backend.yaml"),
        "name: Backend\nextends: base\ndescription: \"Backend profile\"\nrules:\n  - \"use async/await\"\npins:\n  - openai/chat\ncontext:\n  - architecture.md\n",
    )
    .unwrap();

    // A context doc
    fs::write(
        chub_dir.join("context/architecture.md"),
        "---\nname: Architecture\ndescription: \"System architecture overview\"\ntags: architecture, design\n---\n\n# Architecture\n\nThis project uses a modular architecture with two crates.\n\n## Core Library\n\nAll business logic lives in `chub-core`.\n\n## CLI\n\nThe `chub-cli` crate provides the command-line interface.\n",
    )
    .unwrap();

    // A second context doc
    fs::write(
        chub_dir.join("context/conventions.md"),
        "---\nname: Conventions\ndescription: \"Coding conventions and style guide\"\ntags: conventions, style\n---\n\n# Conventions\n\n- Use `snake_case` for functions.\n- Use `CamelCase` for types.\n",
    )
    .unwrap();

    unsafe {
        std::env::set_var("CHUB_PROJECT_DIR", tmp.path());
    }

    (tmp, guard)
}

/// Run a chub CLI command with current_dir set to temp dir.
/// Returns combined stdout+stderr.
fn chub(tmp: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("chub")
        .current_dir(tmp)
        .args(args)
        .env("CHUB_TELEMETRY", "0")
        .env("CHUB_FEEDBACK", "0")
        .output()
        .expect("failed to run chub");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    format!("{}{}", stderr, stdout)
}

/// Run chub with --json and return parsed JSON.
fn chub_json(tmp: &std::path::Path, args: &[&str]) -> serde_json::Value {
    let mut full_args = vec!["--json"];
    full_args.extend_from_slice(args);
    let output = Command::new("chub")
        .current_dir(tmp)
        .args(&full_args)
        .env("CHUB_TELEMETRY", "0")
        .env("CHUB_FEEDBACK", "0")
        .output()
        .expect("failed to run chub");
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|_| {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "failed to parse JSON from chub {:?}\nstdout: {}\nstderr: {}",
            args, stdout, stderr
        );
    })
}

// ===========================================================================
// Pin commands
// ===========================================================================

#[test]
fn pin_add_list_remove_lifecycle() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Add a pin
    let out = chub(
        dir,
        &[
            "pin",
            "add",
            "openai/chat",
            "--lang",
            "python",
            "--version",
            "4.0",
            "--reason",
            "Use v4 API",
        ],
    );
    assert!(
        out.contains("Pinned") || out.contains("openai/chat"),
        "pin add output: {}",
        out
    );

    // List pins (JSON format: {"pins": [...], "total": N})
    let pins = chub_json(dir, &["pin", "list"]);
    let arr = pins["pins"]
        .as_array()
        .expect("pin list should have pins array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "openai/chat");
    assert_eq!(arr[0]["lang"], "python");
    assert_eq!(arr[0]["version"], "4.0");
    assert_eq!(arr[0]["reason"], "Use v4 API");

    // Add a second pin
    chub(dir, &["pin", "add", "stripe/api", "--lang", "javascript"]);
    let pins = chub_json(dir, &["pin", "list"]);
    assert_eq!(pins["pins"].as_array().unwrap().len(), 2);

    // Remove first pin
    let out = chub(dir, &["pin", "remove", "openai/chat"]);
    assert!(
        out.contains("Removed") || out.contains("openai/chat"),
        "pin remove output: {}",
        out
    );

    // Verify only stripe remains
    let pins = chub_json(dir, &["pin", "list"]);
    let arr = pins["pins"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "stripe/api");
}

#[test]
fn pin_update_replaces_existing() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    chub(dir, &["pin", "add", "openai/chat", "--version", "3.0"]);
    chub(
        dir,
        &[
            "pin",
            "add",
            "openai/chat",
            "--version",
            "4.0",
            "--lang",
            "python",
        ],
    );

    let pins = chub_json(dir, &["pin", "list"]);
    let arr = pins["pins"].as_array().unwrap();
    assert_eq!(arr.len(), 1, "should update, not duplicate");
    assert_eq!(arr[0]["version"], "4.0");
    assert_eq!(arr[0]["lang"], "python");
}

// ===========================================================================
// Profile commands
// ===========================================================================

#[test]
fn profile_list_shows_available_profiles() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    let out = chub(dir, &["profile", "list"]);
    assert!(out.contains("base"), "should list base profile: {}", out);
    assert!(
        out.contains("backend") || out.contains("Backend"),
        "should list backend profile: {}",
        out
    );
}

#[test]
fn profile_use_sets_active_profile() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Set profile
    let out = chub(dir, &["profile", "use", "backend"]);
    assert!(
        out.contains("backend")
            || out.contains("Backend")
            || out.contains("Active")
            || out.contains("active"),
        "profile use output: {}",
        out
    );

    // Verify it's active (list should show it)
    let out = chub(dir, &["profile", "list"]);
    assert!(out.contains("backend"), "backend should be listed: {}", out);

    // Clear profile
    let out = chub(dir, &["profile", "use", "none"]);
    assert!(
        out.contains("Cleared")
            || out.contains("none")
            || out.contains("No active")
            || out.contains("cleared"),
        "profile clear output: {}",
        out
    );
}

// ===========================================================================
// Context docs commands
// ===========================================================================

#[test]
fn context_list_shows_project_docs() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // `chub context --list` lists project context docs
    let out = chub(dir, &["context", "--list"]);
    assert!(
        out.contains("project/architecture"),
        "should list architecture doc: {}",
        out
    );
    assert!(
        out.contains("project/conventions"),
        "should list conventions doc: {}",
        out
    );
}

#[test]
fn context_get_retrieves_doc_content() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // `chub get project/<name>` retrieves a context doc
    let out = chub(dir, &["get", "project/architecture"]);
    assert!(
        out.contains("modular architecture"),
        "should contain doc content: {}",
        out
    );
    assert!(
        out.contains("chub-core"),
        "should contain crate names: {}",
        out
    );
}

#[test]
fn context_get_nonexistent_returns_error() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    let out = chub(dir, &["get", "project/nonexistent"]);
    assert!(
        out.contains("not found")
            || out.contains("No context")
            || out.contains("error")
            || out.contains("Error"),
        "should indicate not found: {}",
        out
    );
}

#[test]
fn context_list_json_has_metadata() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // JSON format: {"docs": [...], "query": "list"}
    let json = chub_json(dir, &["context", "--list"]);
    let arr = json["docs"]
        .as_array()
        .expect("context list should have docs array");
    assert!(arr.len() >= 2, "should have at least 2 docs: {:?}", arr);

    // Find Architecture doc
    let arch = arr.iter().find(|d| d["name"] == "Architecture");
    assert!(arch.is_some(), "Architecture doc missing from JSON");
    let arch = arch.unwrap();
    assert_eq!(arch["description"], "System architecture overview");
}

// ===========================================================================
// Annotation commands
// ===========================================================================

#[test]
fn annotate_write_and_read() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Write a team annotation (chub annotate <id> <note> --team)
    let out = chub(
        dir,
        &[
            "annotate",
            "openai/chat",
            "Always use streaming for chat completions",
            "--team",
        ],
    );
    assert!(
        out.contains("saved") || out.contains("annotation") || out.contains("openai/chat"),
        "annotate output: {}",
        out
    );

    // Read it back (chub annotate <id> --team — no note = read)
    let out = chub(dir, &["annotate", "openai/chat", "--team"]);
    assert!(
        out.contains("streaming"),
        "should contain annotation text: {}",
        out
    );
}

#[test]
fn annotate_with_kind_and_severity() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Write an issue annotation with severity
    chub(
        dir,
        &[
            "annotate",
            "openai/chat",
            "tool_choice none silently ignores tools",
            "--team",
            "--kind",
            "issue",
            "--severity",
            "high",
        ],
    );

    // Write a fix annotation
    chub(
        dir,
        &[
            "annotate",
            "openai/chat",
            "use tool_choice auto or remove tools",
            "--team",
            "--kind",
            "fix",
        ],
    );

    // Read back - should have both
    let out = chub(dir, &["annotate", "openai/chat", "--team"]);
    assert!(
        out.contains("silently ignores"),
        "should contain issue: {}",
        out
    );
    assert!(
        out.contains("auto") || out.contains("remove tools"),
        "should contain fix: {}",
        out
    );
}

#[test]
fn annotate_list_shows_all_annotated_docs() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    chub(dir, &["annotate", "openai/chat", "note 1", "--team"]);
    chub(dir, &["annotate", "stripe/api", "note 2", "--team"]);

    let out = chub(dir, &["annotate", "--list", "--team"]);
    assert!(
        out.contains("openai/chat"),
        "should list openai/chat: {}",
        out
    );
    assert!(
        out.contains("stripe/api"),
        "should list stripe/api: {}",
        out
    );
}

// ===========================================================================
// Snapshot commands
// ===========================================================================

#[test]
fn snapshot_create_list_restore() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Pin some docs first
    chub(dir, &["pin", "add", "openai/chat", "--version", "4.0"]);
    chub(dir, &["pin", "add", "stripe/api", "--lang", "javascript"]);

    // Create snapshot
    let out = chub(dir, &["snapshot", "create", "v1.0"]);
    assert!(
        out.contains("Created")
            || out.contains("v1.0")
            || out.contains("snapshot")
            || out.contains("Snapshot"),
        "snapshot create output: {}",
        out
    );

    // List snapshots
    let out = chub(dir, &["snapshot", "list"]);
    assert!(out.contains("v1.0"), "should list v1.0: {}", out);

    // Modify pins
    chub(dir, &["pin", "remove", "stripe/api"]);
    chub(dir, &["pin", "add", "axios/api", "--lang", "javascript"]);
    let pins = chub_json(dir, &["pin", "list"]);
    assert_eq!(pins["pins"].as_array().unwrap().len(), 2);

    // Restore snapshot
    let out = chub(dir, &["snapshot", "restore", "v1.0"]);
    assert!(
        out.contains("Restored") || out.contains("v1.0") || out.contains("restored"),
        "snapshot restore output: {}",
        out
    );

    // Verify pins are back to v1.0 state
    let pins = chub_json(dir, &["pin", "list"]);
    let arr = pins["pins"].as_array().unwrap();
    let ids: Vec<&str> = arr.iter().filter_map(|p| p["id"].as_str()).collect();
    assert!(
        ids.contains(&"openai/chat"),
        "openai/chat should be restored"
    );
    assert!(ids.contains(&"stripe/api"), "stripe/api should be restored");
    assert!(
        !ids.contains(&"axios/api"),
        "axios/api should not be present after restore"
    );
}

// ===========================================================================
// Agent config commands
// ===========================================================================

#[test]
fn agent_config_sync_generates_claude_md() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Add a pin to make the generated config interesting
    chub(dir, &["pin", "add", "openai/chat", "--version", "4.0"]);

    let out = chub(dir, &["agent-config", "sync"]);
    assert!(
        out.contains("sync")
            || out.contains("CLAUDE")
            || out.contains("generated")
            || out.contains("Updated")
            || out.contains("wrote")
            || out.contains("Wrote")
            || out.contains("claude"),
        "agent-config sync output: {}",
        out
    );

    // Check that CLAUDE.md was generated in the project dir
    let claude_md = dir.join("CLAUDE.md");
    assert!(
        claude_md.exists(),
        "CLAUDE.md should be generated by agent-config sync"
    );
    let content = fs::read_to_string(&claude_md).unwrap();
    // Generated CLAUDE.md has project rules header
    assert!(
        content.contains("Project Rules") || content.contains("chub") || content.contains("Chub"),
        "CLAUDE.md should have project rules: {}",
        content
    );
}

// ===========================================================================
// Init command
// ===========================================================================

#[test]
fn init_creates_chub_directory_structure() {
    let guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    // Use a fresh temp dir without .chub/
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    unsafe {
        std::env::set_var("CHUB_PROJECT_DIR", dir);
    }

    let out = chub(dir, &["init"]);
    assert!(
        out.contains("Initialized")
            || out.contains("init")
            || out.contains(".chub")
            || out.contains("Created"),
        "init output: {}",
        out
    );

    // Verify directory structure was created
    assert!(dir.join(".chub").exists(), ".chub/ directory should exist");
    assert!(
        dir.join(".chub/config.yaml").exists(),
        "config.yaml should exist"
    );
    assert!(
        dir.join(".chub/pins.yaml").exists(),
        "pins.yaml should exist"
    );

    drop(guard);
}

// ===========================================================================
// Context doc creation and retrieval
// ===========================================================================

#[test]
fn context_doc_with_frontmatter_is_parsed_correctly() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Create a doc with rich frontmatter
    let chub_dir = dir.join(".chub");
    fs::write(
        chub_dir.join("context/api-guide.md"),
        "---\nname: API Guide\ndescription: \"How to use our REST API\"\ntags: api, rest, guide\n---\n\n# API Guide\n\n## Authentication\n\nUse Bearer tokens for all requests.\n\n## Rate Limiting\n\n100 requests per minute per API key.\n",
    )
    .unwrap();

    // List should include the new doc
    let json = chub_json(dir, &["context", "--list"]);
    let arr = json["docs"].as_array().unwrap();
    let api_guide = arr.iter().find(|d| d["name"] == "API Guide");
    assert!(api_guide.is_some(), "API Guide should be listed: {:?}", arr);
    let guide = api_guide.unwrap();
    assert_eq!(guide["description"], "How to use our REST API");

    // Get should return the content
    let out = chub(dir, &["get", "project/api-guide"]);
    assert!(
        out.contains("Bearer tokens"),
        "should contain content: {}",
        out
    );
    assert!(
        out.contains("Rate Limiting"),
        "should contain sections: {}",
        out
    );
}

// ===========================================================================
// Multiple features together (integration)
// ===========================================================================

#[test]
fn pins_profiles_and_context_work_together() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Set up pins
    chub(
        dir,
        &[
            "pin",
            "add",
            "openai/chat",
            "--lang",
            "python",
            "--version",
            "4.0",
        ],
    );
    chub(dir, &["pin", "add", "stripe/api", "--lang", "javascript"]);

    // Set up profile
    chub(dir, &["profile", "use", "backend"]);

    // Add annotations
    chub(
        dir,
        &[
            "annotate",
            "openai/chat",
            "Use streaming for large responses",
            "--team",
        ],
    );

    // Verify context docs are still accessible
    let ctx = chub(dir, &["get", "project/architecture"]);
    assert!(
        ctx.contains("modular architecture"),
        "context should work alongside pins/profiles"
    );

    // Verify pins are intact
    let pins = chub_json(dir, &["pin", "list"]);
    assert_eq!(pins["pins"].as_array().unwrap().len(), 2);

    // Create a snapshot of this state
    chub(dir, &["snapshot", "create", "full-setup"]);

    // Modify everything
    chub(dir, &["pin", "remove", "stripe/api"]);
    chub(dir, &["profile", "use", "none"]);

    // Restore and verify
    chub(dir, &["snapshot", "restore", "full-setup"]);
    let pins = chub_json(dir, &["pin", "list"]);
    assert_eq!(
        pins["pins"].as_array().unwrap().len(),
        2,
        "pins should be restored"
    );
}

#[test]
fn snapshot_diff_shows_changes() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // Snapshot 1: one pin
    chub(dir, &["pin", "add", "openai/chat", "--version", "3.0"]);
    chub(dir, &["snapshot", "create", "before"]);

    // Snapshot 2: modified pin + new pin
    chub(dir, &["pin", "add", "openai/chat", "--version", "4.0"]);
    chub(dir, &["pin", "add", "stripe/api"]);
    chub(dir, &["snapshot", "create", "after"]);

    let out = chub(dir, &["snapshot", "diff", "before", "after"]);
    // Diff should show some differences
    assert!(!out.is_empty(), "snapshot diff should produce output");
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn pin_remove_nonexistent_is_graceful() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    let out = chub(dir, &["pin", "remove", "nonexistent/doc"]);
    // Should not crash
    assert!(
        !out.is_empty(),
        "should produce some output for nonexistent pin"
    );
}

#[test]
fn profile_use_nonexistent_is_graceful() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    let out = chub(dir, &["profile", "use", "nonexistent"]);
    // Should produce an error or warning, not crash
    assert!(
        !out.is_empty(),
        "should produce output for nonexistent profile"
    );
}

#[test]
fn multiple_context_docs_all_listed() {
    let (tmp, _guard) = setup_project();
    let dir = tmp.path();

    // We already have architecture.md and conventions.md from setup
    // Add a third
    let chub_dir = dir.join(".chub");
    fs::write(
        chub_dir.join("context/testing.md"),
        "---\nname: Testing Guide\ndescription: \"How to write tests\"\ntags: testing\n---\n\n# Testing\n\nRun `cargo test --all`.\n",
    )
    .unwrap();

    let json = chub_json(dir, &["context", "--list"]);
    let arr = json["docs"].as_array().unwrap();
    assert!(arr.len() >= 3, "should have at least 3 docs: {:?}", arr);

    let names: Vec<&str> = arr.iter().filter_map(|d| d["name"].as_str()).collect();
    assert!(names.contains(&"Architecture"));
    assert!(names.contains(&"Conventions"));
    assert!(names.contains(&"Testing Guide"));
}
