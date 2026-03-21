//! Integration tests for team features.
//! All tests use isolated temp directories via CHUB_PROJECT_DIR env var
//! to avoid reading/writing the repo's .chub/ directory.
//!
//! Because CHUB_PROJECT_DIR is a process-global env var and Rust runs tests
//! in parallel, we use a mutex to serialize all tests that depend on it.

use std::fs;
use std::sync::Mutex;

/// Global mutex to serialize tests that use CHUB_PROJECT_DIR env var.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Set up an isolated .chub/ directory in a temp dir and point CHUB_PROJECT_DIR to it.
/// Returns the temp dir (kept alive by the caller) and the mutex guard.
fn setup_isolated_project() -> (tempfile::TempDir, std::sync::MutexGuard<'static, ()>) {
    let guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = tempfile::tempdir().unwrap();
    let chub_dir = tmp.path().join(".chub");
    fs::create_dir_all(chub_dir.join("annotations")).unwrap();
    fs::create_dir_all(chub_dir.join("context")).unwrap();
    fs::create_dir_all(chub_dir.join("profiles")).unwrap();
    fs::create_dir_all(chub_dir.join("snapshots")).unwrap();
    fs::create_dir_all(chub_dir.join("bundles")).unwrap();

    // Write minimal config
    fs::write(chub_dir.join("config.yaml"), "# test config\n").unwrap();

    // Write empty pins
    fs::write(chub_dir.join("pins.yaml"), "pins: []\n").unwrap();

    // Write a base profile
    fs::write(
        chub_dir.join("profiles").join("base.yaml"),
        "name: Base\ndescription: \"Test base profile\"\nrules:\n  - \"test rule\"\ncontext: []\n",
    )
    .unwrap();

    // Write a child profile
    fs::write(
        chub_dir.join("profiles").join("backend.yaml"),
        "name: Backend\nextends: base\ndescription: \"Backend profile\"\nrules:\n  - \"backend rule\"\npins:\n  - openai/chat\ncontext:\n  - arch.md\n",
    )
    .unwrap();

    // Write a context doc
    fs::write(
        chub_dir.join("context").join("arch.md"),
        "---\nname: Architecture\ndescription: \"Test architecture doc\"\ntags: architecture, test\n---\n\n# Architecture\n\nTest content.\n",
    )
    .unwrap();

    // Point the env var to our temp dir
    unsafe {
        std::env::set_var("CHUB_PROJECT_DIR", tmp.path());
    }

    (tmp, guard)
}

// ==================== PINS ====================

#[test]
fn pins_empty_by_default() {
    let (_tmp, _guard) = setup_isolated_project();
    let pins = chub_core::team::pins::list_pins();
    assert!(pins.is_empty());
}

#[test]
fn pins_add_and_list() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::pins::add_pin(
        "openai/chat",
        Some("python".to_string()),
        Some("4.0".to_string()),
        Some("test reason".to_string()),
        None,
    )
    .unwrap();

    let pins = chub_core::team::pins::list_pins();
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].id, "openai/chat");
    assert_eq!(pins[0].lang.as_deref(), Some("python"));
    assert_eq!(pins[0].version.as_deref(), Some("4.0"));
    assert_eq!(pins[0].reason.as_deref(), Some("test reason"));
}

#[test]
fn pins_add_multiple_and_remove() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::pins::add_pin("openai/chat", None, None, None, None).unwrap();
    chub_core::team::pins::add_pin("stripe/api", None, None, None, None).unwrap();
    assert_eq!(chub_core::team::pins::list_pins().len(), 2);

    let removed = chub_core::team::pins::remove_pin("openai/chat").unwrap();
    assert!(removed);
    assert_eq!(chub_core::team::pins::list_pins().len(), 1);
    assert_eq!(chub_core::team::pins::list_pins()[0].id, "stripe/api");

    let not_found = chub_core::team::pins::remove_pin("nonexistent").unwrap();
    assert!(!not_found);
}

#[test]
fn pins_get_specific() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::pins::add_pin(
        "openai/chat",
        Some("python".to_string()),
        Some("4.0".to_string()),
        None,
        None,
    )
    .unwrap();

    let pin = chub_core::team::pins::get_pin("openai/chat");
    assert!(pin.is_some());
    assert_eq!(pin.unwrap().version.as_deref(), Some("4.0"));

    let missing = chub_core::team::pins::get_pin("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn pins_update_existing() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::pins::add_pin("openai/chat", None, Some("3.0".to_string()), None, None)
        .unwrap();
    chub_core::team::pins::add_pin(
        "openai/chat",
        Some("python".to_string()),
        Some("4.0".to_string()),
        None,
        None,
    )
    .unwrap();

    let pins = chub_core::team::pins::list_pins();
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].version.as_deref(), Some("4.0"));
    assert_eq!(pins[0].lang.as_deref(), Some("python"));
}

// ==================== PROFILES ====================

#[test]
fn profiles_list() {
    let (_tmp, _guard) = setup_isolated_project();
    let profiles = chub_core::team::profiles::list_profiles();
    assert!(profiles.len() >= 2);

    let names: Vec<&str> = profiles.iter().map(|(n, _)| n.as_str()).collect();
    assert!(names.contains(&"base"));
    assert!(names.contains(&"backend"));
}

#[test]
fn profiles_load_base() {
    let (_tmp, _guard) = setup_isolated_project();
    let profile = chub_core::team::profiles::load_profile("base").unwrap();
    assert_eq!(profile.name, "Base");
    assert!(profile.extends.is_none());
    assert_eq!(profile.rules.len(), 1);
    assert_eq!(profile.rules[0], "test rule");
}

#[test]
fn profiles_resolve_with_inheritance() {
    let (_tmp, _guard) = setup_isolated_project();
    let resolved = chub_core::team::profiles::resolve_profile("backend").unwrap();
    assert_eq!(resolved.name, "backend");

    // Should inherit base rules + own rules
    assert!(resolved.rules.contains(&"test rule".to_string()));
    assert!(resolved.rules.contains(&"backend rule".to_string()));

    // Should have backend pins
    assert!(resolved.pins.contains(&"openai/chat".to_string()));

    // Should have backend context
    assert!(resolved.context.contains(&"arch.md".to_string()));
}

#[test]
fn profiles_set_and_get_active() {
    let (_tmp, _guard) = setup_isolated_project();

    // No active profile initially
    let active = chub_core::team::profiles::get_active_profile();
    assert!(active.is_none());

    // Set active profile
    chub_core::team::profiles::set_active_profile(Some("base")).unwrap();
    let active = chub_core::team::profiles::get_active_profile();
    assert_eq!(active.as_deref(), Some("base"));

    // Clear active profile
    chub_core::team::profiles::set_active_profile(None).unwrap();
    let active = chub_core::team::profiles::get_active_profile();
    assert!(active.is_none());
}

#[test]
fn profiles_circular_inheritance_detected() {
    let (_tmp, _guard) = setup_isolated_project();
    let chub_dir = _tmp.path().join(".chub");

    // Create circular: a extends b, b extends a
    fs::write(
        chub_dir.join("profiles").join("a.yaml"),
        "name: A\nextends: b\nrules: []\n",
    )
    .unwrap();
    fs::write(
        chub_dir.join("profiles").join("b.yaml"),
        "name: B\nextends: a\nrules: []\n",
    )
    .unwrap();

    let result = chub_core::team::profiles::resolve_profile("a");
    assert!(result.is_err());
}

// ==================== CONTEXT ====================

#[test]
fn context_list_docs() {
    let (_tmp, _guard) = setup_isolated_project();
    let docs = chub_core::team::context::list_context_docs();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "Architecture");
    assert_eq!(docs[0].description, "Test architecture doc");
    assert!(docs[0].tags.contains(&"architecture".to_string()));
}

#[test]
fn context_get_doc() {
    let (_tmp, _guard) = setup_isolated_project();
    let result = chub_core::team::context::get_context_doc("arch");
    assert!(result.is_some());
    let (doc, content) = result.unwrap();
    assert_eq!(doc.name, "Architecture");
    assert!(content.contains("# Architecture"));
}

#[test]
fn context_get_nonexistent() {
    let (_tmp, _guard) = setup_isolated_project();
    let result = chub_core::team::context::get_context_doc("nonexistent");
    assert!(result.is_none());
}

// ==================== TEAM ANNOTATIONS ====================

#[test]
fn team_annotations_empty_by_default() {
    let (_tmp, _guard) = setup_isolated_project();
    let anns = chub_core::team::team_annotations::list_team_annotations();
    assert!(anns.is_empty());
}

#[test]
fn team_annotations_write_and_read() {
    let (_tmp, _guard) = setup_isolated_project();

    let result = chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "test note",
        "alice",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );
    assert!(result.is_some());

    let ann = chub_core::team::team_annotations::read_team_annotation("openai/chat");
    assert!(ann.is_some());
    let ann = ann.unwrap();
    assert_eq!(ann.id, "openai/chat");
    assert_eq!(ann.notes.len(), 1);
    assert_eq!(ann.notes[0].author, "alice");
    assert_eq!(ann.notes[0].note, "test note");
}

#[test]
fn team_annotations_append() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "note 1",
        "alice",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );
    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "note 2",
        "bob",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );

    let ann = chub_core::team::team_annotations::read_team_annotation("openai/chat").unwrap();
    assert_eq!(ann.notes.len(), 2);
    assert_eq!(ann.notes[0].author, "alice");
    assert_eq!(ann.notes[1].author, "bob");
}

#[test]
fn team_annotations_merged() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "team note",
        "alice",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );

    let merged = chub_core::team::team_annotations::get_merged_annotation("openai/chat");
    assert!(merged.is_some());
    assert!(merged.unwrap().contains("team note"));
}

#[test]
fn team_annotations_pin_notice() {
    let notice = chub_core::team::team_annotations::get_pin_notice(
        Some("4.0"),
        Some("python"),
        Some("use streaming API"),
    );
    assert!(notice.contains("[Team pin]"));
    assert!(notice.contains("v4.0"));
    assert!(notice.contains("python"));
    assert!(notice.contains("use streaming API"));
}

// ==================== TEAM ANNOTATION CLEAR ====================

#[test]
fn clear_team_annotation_works() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "test note",
        "alice",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );

    let cleared = chub_core::team::team_annotations::clear_team_annotation("openai/chat");
    assert!(cleared);

    let ann = chub_core::team::team_annotations::read_team_annotation("openai/chat");
    assert!(ann.is_none());
}

#[test]
fn clear_team_annotation_missing_returns_false() {
    let (_tmp, _guard) = setup_isolated_project();
    let cleared = chub_core::team::team_annotations::clear_team_annotation("nonexistent/entry");
    assert!(!cleared);
}

// ==================== STRUCTURED ANNOTATION KINDS ====================

#[test]
fn structured_annotations_issue_kind() {
    let (_tmp, _guard) = setup_isolated_project();

    let result = chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "tool_choice='none' silently ignores tools",
        "bob",
        chub_core::annotations::AnnotationKind::Issue,
        Some("high".to_string()),
    );
    assert!(result.is_some());

    let ann = chub_core::team::team_annotations::read_team_annotation("openai/chat").unwrap();
    assert_eq!(ann.issues.len(), 1);
    assert_eq!(ann.notes.len(), 0);
    assert_eq!(ann.fixes.len(), 0);
    assert_eq!(ann.issues[0].author, "bob");
    assert_eq!(ann.issues[0].severity.as_deref(), Some("high"));
    assert!(ann.issues[0].note.contains("tool_choice"));
}

#[test]
fn structured_annotations_fix_kind() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "Pass tool_choice='auto' instead",
        "bob",
        chub_core::annotations::AnnotationKind::Fix,
        None,
    );

    let ann = chub_core::team::team_annotations::read_team_annotation("openai/chat").unwrap();
    assert_eq!(ann.fixes.len(), 1);
    assert_eq!(ann.issues.len(), 0);
    assert_eq!(ann.fixes[0].severity, None);
}

#[test]
fn structured_annotations_practice_kind() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "Always set max_tokens explicitly",
        "alice",
        chub_core::annotations::AnnotationKind::Practice,
        None,
    );

    let ann = chub_core::team::team_annotations::read_team_annotation("openai/chat").unwrap();
    assert_eq!(ann.practices.len(), 1);
    assert_eq!(ann.practices[0].author, "alice");
}

#[test]
fn structured_annotations_mixed_kinds() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::team_annotations::write_team_annotation(
        "stripe/api",
        "idempotency_key ignored with confirm=true",
        "alice",
        chub_core::annotations::AnnotationKind::Issue,
        Some("high".to_string()),
    );
    chub_core::team::team_annotations::write_team_annotation(
        "stripe/api",
        "Use separate create then confirm calls",
        "alice",
        chub_core::annotations::AnnotationKind::Fix,
        None,
    );
    chub_core::team::team_annotations::write_team_annotation(
        "stripe/api",
        "Always use two-step create+confirm in production",
        "alice",
        chub_core::annotations::AnnotationKind::Practice,
        None,
    );
    chub_core::team::team_annotations::write_team_annotation(
        "stripe/api",
        "Python SDK auto-retries on 429",
        "bob",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );

    let ann = chub_core::team::team_annotations::read_team_annotation("stripe/api").unwrap();
    assert_eq!(ann.issues.len(), 1);
    assert_eq!(ann.fixes.len(), 1);
    assert_eq!(ann.practices.len(), 1);
    assert_eq!(ann.notes.len(), 1);
}

#[test]
fn structured_annotations_merged_format() {
    let (_tmp, _guard) = setup_isolated_project();

    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "tool_choice='none' breaks tools",
        "bob",
        chub_core::annotations::AnnotationKind::Issue,
        Some("high".to_string()),
    );
    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "Use tool_choice='auto'",
        "bob",
        chub_core::annotations::AnnotationKind::Fix,
        None,
    );

    let merged = chub_core::team::team_annotations::get_merged_annotation("openai/chat").unwrap();
    assert!(merged.contains("[Team issue (high)"));
    assert!(merged.contains("[Team fix"));
    assert!(merged.contains("tool_choice='none' breaks tools"));
    assert!(merged.contains("Use tool_choice='auto'"));
}

#[test]
fn structured_annotations_severity_only_on_issues() {
    let (_tmp, _guard) = setup_isolated_project();

    // severity should be ignored for non-issue kinds
    chub_core::team::team_annotations::write_team_annotation(
        "openai/chat",
        "a practice note",
        "alice",
        chub_core::annotations::AnnotationKind::Practice,
        Some("high".to_string()),
    );

    let ann = chub_core::team::team_annotations::read_team_annotation("openai/chat").unwrap();
    assert_eq!(ann.practices.len(), 1);
    assert_eq!(ann.practices[0].severity, None); // severity stripped for non-issues
}

#[test]
fn annotation_policy_in_agent_config() {
    let (_tmp, _guard) = setup_isolated_project();
    let chub_dir = _tmp.path().join(".chub");

    std::fs::write(
        chub_dir.join("config.yaml"),
        r#"
agent_rules:
  global:
    - "Use TypeScript strict mode"
  modules: {}
  include_pins: false
  include_context: false
  include_annotation_policy: true
  targets:
    - claude.md
"#,
    )
    .unwrap();

    let rules = chub_core::team::agent_config::load_agent_rules().unwrap();
    let content = chub_core::team::agent_config::generate_config(&rules);

    assert!(content.contains("Annotation Policy"));
    assert!(content.contains("kind=\"issue\""));
    assert!(content.contains("kind=\"fix\""));
    assert!(content.contains("kind=\"practice\""));
    assert!(content.contains("Annotate after confirming"));
}

#[test]
fn annotation_policy_disabled_by_default() {
    let (_tmp, _guard) = setup_isolated_project();
    let chub_dir = _tmp.path().join(".chub");

    std::fs::write(
        chub_dir.join("config.yaml"),
        r#"
agent_rules:
  global: []
  modules: {}
  targets:
    - claude.md
"#,
    )
    .unwrap();

    let rules = chub_core::team::agent_config::load_agent_rules().unwrap();
    let content = chub_core::team::agent_config::generate_config(&rules);

    assert!(!content.contains("Annotation Policy"));
}

// ==================== PERSONAL ANNOTATION SEMANTICS ====================
// These tests use ENV_MUTEX because personal annotations read CHUB_DIR from the environment.

#[test]
fn personal_annotation_overwrites_previous() {
    // Personal annotations use overwrite semantics: a second write replaces the first.
    // This is intentional and differs from team annotations (which append).
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("CHUB_DIR", tmp.path());
    }
    chub_core::annotations::write_annotation(
        "test/entry",
        "first note",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );
    chub_core::annotations::write_annotation(
        "test/entry",
        "second note",
        chub_core::annotations::AnnotationKind::Note,
        None,
    );
    let ann = chub_core::annotations::read_annotation("test/entry").unwrap();
    assert_eq!(
        ann.note, "second note",
        "write_annotation must overwrite, not append"
    );
    unsafe {
        std::env::remove_var("CHUB_DIR");
    }
}

#[test]
fn personal_annotation_stores_kind_and_severity() {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("CHUB_DIR", tmp.path());
    }
    chub_core::annotations::write_annotation(
        "test/entry",
        "broken param",
        chub_core::annotations::AnnotationKind::Issue,
        Some("high".to_string()),
    );
    let ann = chub_core::annotations::read_annotation("test/entry").unwrap();
    assert_eq!(ann.kind, chub_core::annotations::AnnotationKind::Issue);
    assert_eq!(ann.severity.as_deref(), Some("high"));
    unsafe {
        std::env::remove_var("CHUB_DIR");
    }
}

#[test]
fn personal_annotation_severity_stripped_for_non_issues() {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("CHUB_DIR", tmp.path());
    }
    chub_core::annotations::write_annotation(
        "test/entry",
        "a practice",
        chub_core::annotations::AnnotationKind::Practice,
        Some("high".to_string()),
    );
    let ann = chub_core::annotations::read_annotation("test/entry").unwrap();
    assert_eq!(
        ann.severity, None,
        "severity must be None for non-issue kinds"
    );
    unsafe {
        std::env::remove_var("CHUB_DIR");
    }
}

// ==================== SNAPSHOTS ====================

#[test]
fn snapshots_create_and_list() {
    let (_tmp, _guard) = setup_isolated_project();

    // Add a pin first
    chub_core::team::pins::add_pin("openai/chat", None, Some("4.0".to_string()), None, None)
        .unwrap();

    let snap = chub_core::team::snapshots::create_snapshot("v1").unwrap();
    assert_eq!(snap.name, "v1");
    assert_eq!(snap.pins.len(), 1);
    assert_eq!(snap.pins[0].id, "openai/chat");

    let list = chub_core::team::snapshots::list_snapshots();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].0, "v1");
}

#[test]
fn snapshots_restore() {
    let (_tmp, _guard) = setup_isolated_project();

    // Create snapshot with one pin
    chub_core::team::pins::add_pin("openai/chat", None, None, None, None).unwrap();
    chub_core::team::snapshots::create_snapshot("v1").unwrap();

    // Add another pin
    chub_core::team::pins::add_pin("stripe/api", None, None, None, None).unwrap();
    assert_eq!(chub_core::team::pins::list_pins().len(), 2);

    // Restore snapshot
    chub_core::team::snapshots::restore_snapshot("v1").unwrap();
    assert_eq!(chub_core::team::pins::list_pins().len(), 1);
    assert_eq!(chub_core::team::pins::list_pins()[0].id, "openai/chat");
}

#[test]
fn snapshots_diff() {
    let (_tmp, _guard) = setup_isolated_project();

    // Snapshot with one pin
    chub_core::team::pins::add_pin("openai/chat", None, Some("3.0".to_string()), None, None)
        .unwrap();
    chub_core::team::snapshots::create_snapshot("v1").unwrap();

    // Modify pins and create second snapshot
    chub_core::team::pins::add_pin("openai/chat", None, Some("4.0".to_string()), None, None)
        .unwrap();
    chub_core::team::pins::add_pin("stripe/api", None, None, None, None).unwrap();
    chub_core::team::snapshots::create_snapshot("v2").unwrap();

    let diffs = chub_core::team::snapshots::diff_snapshots("v1", "v2").unwrap();
    assert_eq!(diffs.len(), 2); // one changed, one added
}

#[test]
fn snapshots_not_found() {
    let (_tmp, _guard) = setup_isolated_project();
    let result = chub_core::team::snapshots::restore_snapshot("nonexistent");
    assert!(result.is_err());
}

// ==================== DETECT ====================

#[test]
fn detect_npm_dependencies() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"dependencies": {"express": "^4.18.0", "lodash": "4.17.21"}, "devDependencies": {"jest": "^29.0.0"}}"#,
    )
    .unwrap();

    let deps = chub_core::team::detect::detect_dependencies(tmp.path());
    assert!(deps.len() >= 3);
    assert!(deps.iter().any(|d| d.name == "express"));
    assert!(deps.iter().any(|d| d.name == "lodash"));
    assert!(deps.iter().any(|d| d.name == "jest"));
    assert!(deps.iter().all(|d| d.language == "javascript"));
}

#[test]
fn detect_cargo_dependencies() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
tempfile = "3"
"#,
    )
    .unwrap();

    let deps = chub_core::team::detect::detect_dependencies(tmp.path());
    assert!(deps
        .iter()
        .any(|d| d.name == "serde" && d.version.as_deref() == Some("1.0")));
    assert!(deps.iter().any(|d| d.name == "tokio"));
    assert!(deps.iter().any(|d| d.name == "tempfile"));
    assert!(deps.iter().all(|d| d.language == "rust"));
}

#[test]
fn detect_requirements_txt() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("requirements.txt"),
        "flask==2.3.0\nrequests>=2.28\nnumpy\n# comment\n",
    )
    .unwrap();

    let deps = chub_core::team::detect::detect_dependencies(tmp.path());
    assert_eq!(deps.len(), 3);
    assert!(deps.iter().any(|d| d.name == "flask"));
    assert!(deps.iter().any(|d| d.name == "requests"));
    assert!(deps.iter().any(|d| d.name == "numpy"));
}

#[test]
fn detect_go_mod() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("go.mod"),
        "module example.com/myapp\n\ngo 1.21\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n\tgithub.com/lib/pq v1.10.9\n)\n",
    )
    .unwrap();

    let deps = chub_core::team::detect::detect_dependencies(tmp.path());
    assert!(deps.iter().any(|d| d.name == "gin"));
    assert!(deps.iter().any(|d| d.name == "pq"));
}

#[test]
fn detect_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let deps = chub_core::team::detect::detect_dependencies(tmp.path());
    assert!(deps.is_empty());
}

#[test]
fn detect_match_deps_to_docs() {
    let deps = vec![chub_core::team::detect::DetectedDep {
        name: "openai".to_string(),
        version: Some("1.0".to_string()),
        source_file: "package.json".to_string(),
        language: "javascript".to_string(),
    }];

    let doc_ids = vec![("openai/chat".to_string(), "OpenAI Chat".to_string())];

    let matches = chub_core::team::detect::match_deps_to_docs(&deps, &doc_ids);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].doc_id, "openai/chat");
    assert_eq!(matches[0].confidence, 1.0);
}

// ==================== FRESHNESS ====================

#[test]
fn freshness_no_pins() {
    let (_tmp, _guard) = setup_isolated_project();
    let results = chub_core::team::freshness::check_freshness(_tmp.path());
    assert!(results.is_empty());
}

#[test]
fn freshness_pin_no_deps() {
    let (_tmp, _guard) = setup_isolated_project();
    chub_core::team::pins::add_pin("openai/chat", None, Some("4.0".to_string()), None, None)
        .unwrap();

    let results = chub_core::team::freshness::check_freshness(_tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].status,
        chub_core::team::freshness::FreshnessStatus::Unknown
    );
}

// ==================== AGENT CONFIG ====================

#[test]
fn agent_config_generate_content() {
    let (_tmp, _guard) = setup_isolated_project();
    let chub_dir = _tmp.path().join(".chub");

    // Write config with agent_rules
    fs::write(
        chub_dir.join("config.yaml"),
        r#"
agent_rules:
  global:
    - "Use TypeScript strict mode"
  modules: {}
  include_pins: true
  include_context: true
  targets:
    - claude.md
"#,
    )
    .unwrap();

    let rules = chub_core::team::agent_config::load_agent_rules().unwrap();
    let content = chub_core::team::agent_config::generate_config(&rules);

    assert!(content.contains("# Project Rules"));
    assert!(content.contains("Use TypeScript strict mode"));
    assert!(content.contains("Project Context"));
}

// ==================== ANALYTICS ====================

#[test]
fn analytics_record_and_stats() {
    // Analytics uses chub_dir() (personal), not project dir, so we set CHUB_DIR
    // Still needs mutex since it uses env var
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("CHUB_DIR", tmp.path());
    }

    chub_core::team::analytics::record_fetch("openai/chat", Some("test-agent"));
    chub_core::team::analytics::record_fetch("openai/chat", None);
    chub_core::team::analytics::record_fetch("stripe/api", None);

    let stats = chub_core::team::analytics::get_stats(30);
    assert_eq!(stats.total_fetches, 3);
    assert!(stats
        .most_fetched
        .iter()
        .any(|(id, count)| id == "openai/chat" && *count == 2));
    assert!(stats
        .most_fetched
        .iter()
        .any(|(id, count)| id == "stripe/api" && *count == 1));

    unsafe {
        std::env::remove_var("CHUB_DIR");
    }
}

// ==================== PROJECT INIT ====================

#[test]
fn project_init_creates_structure() {
    let (_tmp, _guard) = setup_isolated_project();
    let config = chub_core::team::project::load_project_config();
    assert!(config.is_some());
}
