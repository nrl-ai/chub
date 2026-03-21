//! Integration tests for build command parity with the JS implementation.
//! Mirrors the e2e.test.js BUILD section and build.test.js.

use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/chub-cli
    // Fixtures are at tests/fixtures (workspace root)
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
}

fn chub_binary() -> PathBuf {
    // Binary is built to the workspace target dir: chub-rs/target/debug/chub
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("debug")
        .join("chub");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

fn chub_json(args: &[&str]) -> serde_json::Value {
    let output = Command::new(chub_binary())
        .args(args)
        .arg("--json")
        .output()
        .expect("Failed to execute chub");
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!(
            "Failed to parse JSON output: {}\nstdout: {}\nstderr: {}",
            e,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn chub_run(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(chub_binary())
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("Failed to execute chub");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

// ===== BUILD TESTS =====

#[test]
fn build_produces_registry_json() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");

    let (stdout, _stderr, success) = chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);
    assert!(success, "Build should succeed");
    assert!(output_dir.join("registry.json").exists());
    assert!(output_dir.join("search-index.json").exists());
    assert!(stdout.contains("Built:"));
}

#[test]
fn build_registry_has_correct_counts() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    let result = chub_json(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    assert_eq!(result["docs"], 3, "Expected 3 docs");
    assert_eq!(result["skills"], 1, "Expected 1 skill");
}

#[test]
fn build_copies_content_files() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    // Check that DOC.md and references/advanced.md are copied
    assert!(output_dir.join("acme/docs/widgets/DOC.md").exists());
    assert!(output_dir
        .join("acme/docs/widgets/references/advanced.md")
        .exists());
    assert!(output_dir
        .join("multilang/docs/client/python/DOC.md")
        .exists());
    assert!(output_dir
        .join("testskills/skills/deploy/SKILL.md")
        .exists());
}

#[test]
fn build_validate_only() {
    let (stdout, _stderr, success) =
        chub_run(&["build", fixtures_dir().to_str().unwrap(), "--validate-only"]);
    assert!(success);
    assert!(stdout.contains("3 docs"));
    assert!(stdout.contains("1 skills"));
}

#[test]
fn build_validate_only_json() {
    let result = chub_json(&["build", fixtures_dir().to_str().unwrap(), "--validate-only"]);
    assert_eq!(result["docs"], 3);
    assert_eq!(result["skills"], 1);
    assert!(result["warnings"].is_number());
}

#[test]
fn build_errors_on_missing_content_dir() {
    let (_stdout, stderr, success) = chub_run(&["build", "/nonexistent/path"]);
    assert!(!success, "Should fail on missing dir");
    assert!(
        stderr.contains("not found") || stderr.contains("Content directory"),
        "Should mention missing directory: {stderr}"
    );
}

#[test]
fn build_json_includes_docs_and_skills() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    let result = chub_json(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);
    assert!(result.get("docs").is_some());
    assert!(result.get("skills").is_some());
    assert!(result.get("output").is_some());
}

// ===== REGISTRY STRUCTURE TESTS =====

#[test]
fn build_registry_structure_is_correct() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    let registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("registry.json")).unwrap())
            .unwrap();

    assert_eq!(registry["version"], "1.0.0");
    assert!(registry["generated"].is_string());
    assert!(registry["docs"].is_array());
    assert!(registry["skills"].is_array());
}

#[test]
fn build_registry_docs_have_correct_structure() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    let registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("registry.json")).unwrap())
            .unwrap();

    for doc in registry["docs"].as_array().unwrap() {
        assert!(doc["id"].is_string(), "Doc should have id");
        assert!(doc["name"].is_string(), "Doc should have name");
        assert!(
            doc["description"].is_string(),
            "Doc should have description"
        );
        assert!(doc["source"].is_string(), "Doc should have source");
        assert!(doc["tags"].is_array(), "Doc should have tags array");
        assert!(
            doc["languages"].is_array(),
            "Doc should have languages array"
        );

        // Check language structure
        for lang in doc["languages"].as_array().unwrap() {
            assert!(lang["language"].is_string());
            assert!(lang["recommendedVersion"].is_string());
            assert!(lang["versions"].is_array());

            for ver in lang["versions"].as_array().unwrap() {
                assert!(ver["version"].is_string());
                assert!(ver["path"].is_string());
                assert!(ver["files"].is_array());
                assert!(ver["size"].is_number());
                assert!(ver["lastUpdated"].is_string());
                // Paths should use forward slashes
                assert!(
                    !ver["path"].as_str().unwrap().contains('\\'),
                    "Paths should use forward slashes: {}",
                    ver["path"]
                );
            }
        }
    }
}

#[test]
fn build_registry_skills_have_correct_structure() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    let registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("registry.json")).unwrap())
            .unwrap();

    for skill in registry["skills"].as_array().unwrap() {
        assert!(skill["id"].is_string());
        assert!(skill["name"].is_string());
        assert!(skill["description"].is_string());
        assert!(skill["source"].is_string());
        assert!(skill["tags"].is_array());
        assert!(skill["path"].is_string());
        assert!(skill["files"].is_array());
        assert!(skill["size"].is_number());
        assert!(skill["lastUpdated"].is_string());
        // Paths should use forward slashes
        assert!(!skill["path"].as_str().unwrap().contains('\\'));
    }
}

// ===== VERSION TESTS =====

#[test]
fn build_groups_multi_version_docs_correctly() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    let registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("registry.json")).unwrap())
            .unwrap();

    let versioned = registry["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "acme/versioned-api")
        .expect("Should find acme/versioned-api");

    let js_lang = versioned["languages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["language"] == "javascript")
        .expect("Should have javascript language");

    let versions = js_lang["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2, "Should have 2 versions");

    // Versions should be sorted descending
    assert_eq!(versions[0]["version"], "2.0.0");
    assert_eq!(versions[1]["version"], "1.0.0");

    // Recommended should be latest
    assert_eq!(js_lang["recommendedVersion"], "2.0.0");
}

#[test]
fn build_handles_multi_language_docs() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    let registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("registry.json")).unwrap())
            .unwrap();

    let multilang = registry["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "multilang/client")
        .expect("Should find multilang/client");

    let languages: Vec<&str> = multilang["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["language"].as_str().unwrap())
        .collect();

    assert!(languages.contains(&"javascript"));
    assert!(languages.contains(&"python"));
    assert!(languages.contains(&"go"));
    assert_eq!(languages.len(), 3);
}

// ===== SEARCH INDEX TESTS =====

#[test]
fn build_produces_valid_search_index() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    let index: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(output_dir.join("search-index.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(index["version"], "1.0.0");
    assert_eq!(index["algorithm"], "bm25");
    assert_eq!(index["totalDocs"], 4); // 3 docs + 1 skill
    assert_eq!(index["params"]["k1"], 1.5);
    assert_eq!(index["params"]["b"], 0.75);
    assert!(index["avgFieldLengths"]["name"].is_number());
    assert!(index["avgFieldLengths"]["description"].is_number());
    assert!(index["avgFieldLengths"]["tags"].is_number());
    assert!(index["idf"].is_object());
    assert_eq!(index["documents"].as_array().unwrap().len(), 4);
}

// ===== BASE URL TEST =====

#[test]
fn build_includes_base_url_when_specified() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
        "--base-url",
        "https://cdn.example.com/v1",
    ]);

    let registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("registry.json")).unwrap())
            .unwrap();

    assert_eq!(registry["base_url"], "https://cdn.example.com/v1");
}

#[test]
fn build_omits_base_url_when_not_specified() {
    let tmp = tempfile::tempdir().unwrap();
    let output_dir = tmp.path().join("dist");
    chub_run(&[
        "build",
        fixtures_dir().to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ]);

    let registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("registry.json")).unwrap())
            .unwrap();

    assert!(registry.get("base_url").is_none());
}
