//! Comprehensive scan integration tests — inspired by betterleaks test suite.
//!
//! Tests cover: detection accuracy (true positives), false positive prevention,
//! directory scanning, git scanning, output formats, config loading, baseline
//! dedup, path allowlists, and AI agent transcript scanning.

use chub_core::scan::config::ScanConfig;
use chub_core::scan::finding::Finding;
use chub_core::scan::report::{write_csv, write_json, write_sarif, ReportFormat};
use chub_core::scan::scanner::{ScanOptions, Scanner};
use std::io::Write;

// ---------------------------------------------------------------------------
// Table-driven detection tests (betterleaks style)
// ---------------------------------------------------------------------------

/// A test case for secret detection.
struct DetectCase {
    name: &'static str,
    input: &'static str,
    file: &'static str,
    expected_rule: Option<&'static str>, // None = expect no finding
    expected_count: usize,
}

fn run_detect_cases(cases: &[DetectCase]) {
    let scanner = Scanner::new(ScanOptions::default());
    for case in cases {
        let findings = scanner.scan_text(case.input, case.file, None);
        assert_eq!(
            findings.len(),
            case.expected_count,
            "[{}] expected {} findings, got {}: {:?}",
            case.name,
            case.expected_count,
            findings.len(),
            findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
        );
        if let Some(expected_rule) = case.expected_rule {
            assert!(
                findings.iter().any(|f| f.rule_id == expected_rule),
                "[{}] expected rule '{}', found: {:?}",
                case.name,
                expected_rule,
                findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// True positive tests — each provider should be detected
// ---------------------------------------------------------------------------

#[test]
fn detect_cloud_provider_secrets() {
    run_detect_cases(&[
        DetectCase {
            name: "AWS access key",
            input: "aws_access_key_id = AKIAK4JM7NR2PX6SWT3B",
            file: "config.env",
            expected_rule: Some("aws-access-token"),
            expected_count: 1,
        },
        DetectCase {
            name: "AWS secret key (caught by generic/env rules; betterleaks skipReport=true)",
            input: "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMIK7MDENGbPxRfiCYk4Jm8nR2pX",
            file: "config.env",
            expected_rule: None, // any detection is fine
            expected_count: 1,
        },
        DetectCase {
            name: "GCP API key",
            input: "GOOGLE_API_KEY=AIzaSyA1bcDeFgHiJkLmNoPqRsTuVwXyZ012345",
            file: "app.env",
            expected_rule: Some("gcp-api-key"),
            expected_count: 1,
        },
    ]);
}

#[test]
fn detect_github_tokens() {
    run_detect_cases(&[
        DetectCase {
            name: "GitHub PAT",
            input: "GITHUB_TOKEN=ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2",
            file: ".env",
            expected_rule: Some("github-pat"),
            expected_count: 1,
        },
        DetectCase {
            name: "GitHub App token (ghs)",
            input: "token: ghs_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2",
            file: "ci.yml",
            expected_rule: Some("github-app-token"),
            expected_count: 1,
        },
        DetectCase {
            name: "GitLab PAT",
            input: "GITLAB_TOKEN=glpat-abcdef1234567890abcd",
            file: ".env",
            expected_rule: Some("gitlab-pat"),
            expected_count: 1,
        },
    ]);
}

#[test]
fn detect_ai_llm_keys() {
    run_detect_cases(&[
        DetectCase {
            name: "Anthropic API key",
            // sk-ant-api03- + 93 chars of [a-zA-Z0-9_-] + AA (95 suffix total)
            input: "ANTHROPIC_API_KEY=sk-ant-api03-k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2pX6sW9vB3fH7aT1qY5uExAA",
            file: ".env",
            expected_rule: Some("anthropic-api-key"),
            expected_count: 1,
        },
        DetectCase {
            name: "OpenAI project key",
            input: "OPENAI_API_KEY=sk-proj-k4Jm8nR2pX6sW9vB3fH7T3BlbkFJk4Jm8nR2pX6sW9vB3fH7",
            file: ".env",
            expected_rule: Some("openai-api-key"),
            expected_count: 1,
        },
        DetectCase {
            name: "OpenRouter key",
            input: "OPENROUTER_KEY=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",
            file: ".env",
            expected_rule: Some("openrouter-api-key"),
            expected_count: 1,
        },
        DetectCase {
            name: "Groq API key",
            input: "GROQ_KEY=gsk_a1B2c3D4e5F6g7H8i9J0kLmNoPqRsTuVwXyZaBcDeFgHiJkLmNoP",
            file: ".env",
            expected_rule: Some("groq-api-key"),
            expected_count: 1,
        },
        DetectCase {
            name: "HuggingFace token",
            // hf_ + 34 lowercase alpha chars only (no digits)
            input: "HF_TOKEN=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",
            file: ".env",
            expected_rule: Some("huggingface-access-token"),
            expected_count: 1,
        },
        DetectCase {
            name: "Cerebras key",
            input: "CEREBRAS_KEY=csk-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6",
            file: ".env",
            expected_rule: Some("cerebras-api-key"),
            expected_count: 1,
        },
    ]);
}

#[test]
fn detect_devops_tokens() {
    run_detect_cases(&[
        DetectCase {
            name: "DigitalOcean PAT",
            input:
                "DO_TOKEN=dop_v1_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
            file: ".env",
            expected_rule: Some("digitalocean-pat"),
            expected_count: 1,
        },
        DetectCase {
            name: "NPM access token",
            input: "NPM_TOKEN=npm_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8",
            file: ".npmrc",
            expected_rule: Some("npm-access-token"),
            expected_count: 1,
        },
        DetectCase {
            name: "Pulumi token",
            input: "PULUMI_ACCESS_TOKEN=pul-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
            file: ".env",
            expected_rule: Some("pulumi-api-token"),
            expected_count: 1,
        },
        DetectCase {
            name: "Linear API key",
            input: "LINEAR_KEY=lin_api_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
            file: ".env",
            expected_rule: Some("linear-api-key"),
            expected_count: 1,
        },
        DetectCase {
            name: "Shopify access token",
            input: "SHOPIFY=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",
            file: ".env",
            expected_rule: Some("shopify-access-token"),
            expected_count: 1,
        },
    ]);
}

#[test]
fn detect_auth_crypto() {
    run_detect_cases(&[
        DetectCase {
            name: "JWT",
            input: "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
            file: "request.log",
            expected_rule: Some("jwt"),
            expected_count: 1,
        },
        DetectCase {
            name: "Private key header",
            input: "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEAk4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLk4Jm8nR2pX6sW9vB3fH7\n-----END RSA PRIVATE KEY-----",
            file: "key.pem",
            expected_rule: Some("private-key"),
            expected_count: 1,
        },
        DetectCase {
            name: "Database URL with credentials",
            input: "DATABASE_URL=postgres://admin:s3cr3t_P4ss@db.example.com:5432/production",
            file: "config.env",
            expected_rule: Some("database-url"),
            expected_count: 1,
        },
    ]);
}

// ---------------------------------------------------------------------------
// False positive prevention
// ---------------------------------------------------------------------------

#[test]
fn no_false_positives() {
    run_detect_cases(&[
        DetectCase {
            name: "Normal Rust code",
            input: r#"
fn main() {
    let config = Config::new();
    let result = process_data(&config);
    println!("Done: {}", result);
}
"#,
            file: "main.rs",
            expected_rule: None,
            expected_count: 0,
        },
        DetectCase {
            name: "Cargo.toml checksum",
            input: r#"checksum = "e3148f5046208a5d56bcfc03053e3ca6334e51da8dfb19b6cdc8b306fae3283e""#,
            file: "Cargo.lock",
            expected_rule: None,
            expected_count: 0,
        },
        DetectCase {
            name: "Placeholder API key",
            input: r#"api_key = "placeholder""#,
            file: "config.yaml",
            expected_rule: None,
            expected_count: 0,
        },
        DetectCase {
            name: "Template variable",
            input: "SECRET_KEY=${MY_SECRET_VALUE_FROM_VAULT}",
            file: ".env.template",
            expected_rule: None,
            expected_count: 0,
        },
        DetectCase {
            name: "HTML content",
            input: "<html><body><h1>Welcome</h1><p>Normal content.</p></body></html>",
            file: "index.html",
            expected_rule: None,
            expected_count: 0,
        },
        DetectCase {
            name: "Go import statement",
            input: "import \"github.com/stretchr/testify/assert\"",
            file: "main.go",
            expected_rule: None,
            expected_count: 0,
        },
    ]);
}

// ---------------------------------------------------------------------------
// AI agent transcript scanning — the unique chub advantage
// ---------------------------------------------------------------------------

#[test]
fn detect_secrets_in_agent_transcripts() {
    let scanner = Scanner::new(ScanOptions::default());

    // Claude Code transcript JSONL
    let transcript = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"export STRIPE_KEY=sk_live_k4Jm8nR2pX6sW9vB3fH7aT1q && curl https://api.stripe.com"}}]}}
{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"echo AWS_ACCESS_KEY_ID=AKIAK4JM7NR2PX6SWT3B"}}]}}
{"type":"human","message":{"content":[{"type":"text","text":"My API key is sk-proj-k4Jm8nR2pX6sW9vB3fH7T3BlbkFJk4Jm8nR2pX6sW9vB3fH7"}]}}"#;

    let findings = scanner.scan_text(transcript, "transcript.jsonl", None);
    assert!(
        findings.len() >= 3,
        "should find at least 3 secrets in transcript, got {}: {:?}",
        findings.len(),
        findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
    );
}

#[test]
fn detect_secrets_in_cursor_transcript() {
    let scanner = Scanner::new(ScanOptions::default());

    let transcript = r#"[Tool Call: write_file]
Path: .env
Content:
DATABASE_URL=postgres://admin:realpassword@db.prod.example.com:5432/mydb
ANTHROPIC_API_KEY=sk-ant-api03-k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2pX6sW9vB3fH7aT1qY5uExAA
"#;

    let findings = scanner.scan_text(transcript, "session.log", None);
    assert!(
        findings.len() >= 2,
        "should find DB URL and Anthropic key, got: {:?}",
        findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
    );
}

#[test]
fn detect_secrets_in_prompt() {
    let scanner = Scanner::new(ScanOptions::default());

    let prompt = "Please use this Stripe key to test the payment: sk_test_k4Jm8nR2pX6sW9vB3fH7aT1q";
    let findings = scanner.scan_text(prompt, "prompt.txt", None);
    assert!(
        findings.iter().any(|f| f.rule_id == "stripe-access-token"),
        "should detect Stripe key in prompt"
    );
}

// ---------------------------------------------------------------------------
// Finding metadata
// ---------------------------------------------------------------------------

#[test]
fn findings_have_correct_locations() {
    let scanner = Scanner::new(ScanOptions::default());
    let text = "line1\nline2\nAWS_KEY=AKIAK4JM7NR2PX6SWT3B\nline4";
    let findings = scanner.scan_text(text, "test.env", None);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].start_line, 3);
    assert_eq!(findings[0].file, "test.env");
    assert!(!findings[0].fingerprint.is_empty());
    assert!(findings[0].entropy > 0.0);
}

#[test]
fn findings_have_unique_fingerprints() {
    let scanner = Scanner::new(ScanOptions::default());
    let text = "ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
    let f1 = scanner.scan_text(text, "file1.env", None);
    let f2 = scanner.scan_text(text, "file2.env", None);
    assert!(!f1.is_empty());
    assert!(!f2.is_empty());
    // Different file = different fingerprint
    assert_ne!(f1[0].fingerprint, f2[0].fingerprint);
}

#[test]
fn finding_fingerprint_stable() {
    let scanner = Scanner::new(ScanOptions::default());
    let text = "AKIAK4JM7NR2PX6SWT3B";
    let f1 = scanner.scan_text(text, "test.env", None);
    let f2 = scanner.scan_text(text, "test.env", None);
    assert_eq!(f1[0].fingerprint, f2[0].fingerprint);
}

// ---------------------------------------------------------------------------
// Redaction
// ---------------------------------------------------------------------------

#[test]
fn redacted_findings_hide_secrets() {
    let opts = ScanOptions {
        redact_percent: 100,
        ..Default::default()
    };
    let scanner = Scanner::new(opts);
    let text = "AWS_KEY=AKIAK4JM7NR2PX6SWT3B";
    let findings = scanner.scan_text(text, "test.env", None);
    assert!(!findings.is_empty());
    assert!(findings[0].secret.contains('*'));
    assert!(!findings[0].secret.contains("AKIAK4JM7NR2PX6SWT3B"));
}

// ---------------------------------------------------------------------------
// Rule filtering
// ---------------------------------------------------------------------------

#[test]
fn enable_rule_filters_output() {
    let opts = ScanOptions {
        enable_rules: vec!["github-pat".to_string()],
        ..Default::default()
    };
    let scanner = Scanner::new(opts);
    let text = "AWS=AKIAK4JM7NR2PX6SWT3B GH=ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
    let findings = scanner.scan_text(text, "test.env", None);
    assert!(
        findings.iter().all(|f| f.rule_id == "github-pat"),
        "only github-pat should match"
    );
}

// ---------------------------------------------------------------------------
// Baseline deduplication
// ---------------------------------------------------------------------------

#[test]
fn baseline_filters_known_findings() {
    let scanner = Scanner::new(ScanOptions::default());
    let text = "AKIAK4JM7NR2PX6SWT3B";
    let initial = scanner.scan_text(text, "test.env", None);
    assert!(!initial.is_empty());

    // Simulate baseline: write findings to JSON, reload
    let mut buf = Vec::new();
    write_json(&initial, &mut buf).unwrap();
    let baseline_path = std::env::temp_dir().join("chub_test_baseline.json");
    std::fs::write(&baseline_path, &buf).unwrap();

    let opts = ScanOptions {
        baseline_path: Some(baseline_path.to_string_lossy().to_string()),
        ..Default::default()
    };
    let scanner2 = Scanner::new(opts);
    let filtered = scanner2.scan_text(text, "test.env", None);
    assert!(filtered.is_empty(), "baseline should filter known findings");

    // Cleanup
    let _ = std::fs::remove_file(&baseline_path);
}

// ---------------------------------------------------------------------------
// Directory scanning
// ---------------------------------------------------------------------------

#[test]
fn scan_dir_finds_secrets_in_files() {
    let dir = tempfile::tempdir().unwrap();
    let secret_file = dir.path().join("secret.env");
    std::fs::write(&secret_file, "AWS_KEY=AKIAK4JM7NR2PX6SWT3B\n").unwrap();

    let clean_file = dir.path().join("clean.rs");
    std::fs::write(&clean_file, "fn main() { println!(\"hello\"); }\n").unwrap();

    let scanner = Scanner::new(ScanOptions::default());
    let findings = scanner.scan_dir(dir.path());
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "aws-access-token");
    assert!(findings[0].file.contains("secret.env"));
}

#[test]
fn scan_dir_skips_binary_files() {
    let dir = tempfile::tempdir().unwrap();
    let bin_file = dir.path().join("image.png");
    std::fs::write(&bin_file, "AKIAK4JM7NR2PX6SWT3B in binary").unwrap();

    let scanner = Scanner::new(ScanOptions::default());
    let findings = scanner.scan_dir(dir.path());
    assert!(findings.is_empty(), "binary files should be skipped");
}

#[test]
fn scan_dir_skips_large_files() {
    let dir = tempfile::tempdir().unwrap();
    let large_file = dir.path().join("big.env");
    // Create a file just over 1 byte limit
    let opts = ScanOptions {
        max_target_bytes: 100,
        ..Default::default()
    };
    std::fs::write(&large_file, "x".repeat(200) + "\nAKIAK4JM7NR2PX6SWT3B").unwrap();

    let scanner = Scanner::new(opts);
    let findings = scanner.scan_dir(dir.path());
    assert!(findings.is_empty(), "large files should be skipped");
}

#[test]
fn scan_dir_skips_git_directory() {
    let dir = tempfile::tempdir().unwrap();
    let git_dir = dir.path().join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();
    let git_config = git_dir.join("config");
    std::fs::write(&git_config, "AKIAK4JM7NR2PX6SWT3B").unwrap();

    let scanner = Scanner::new(ScanOptions::default());
    let findings = scanner.scan_dir(dir.path());
    assert!(findings.is_empty(), ".git/ should be skipped");
}

// ---------------------------------------------------------------------------
// Stdin scanning
// ---------------------------------------------------------------------------

#[test]
fn scan_stdin() {
    let scanner = Scanner::new(ScanOptions::default());
    let input = b"AKIAK4JM7NR2PX6SWT3B\n";
    let findings = scanner.scan_reader(&input[..], "stdin");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "stdin");
}

// ---------------------------------------------------------------------------
// Output format tests
// ---------------------------------------------------------------------------

#[test]
fn json_report_roundtrips() {
    let scanner = Scanner::new(ScanOptions::default());
    let findings = scanner.scan_text("AKIAK4JM7NR2PX6SWT3B", "test.env", None);

    let mut buf = Vec::new();
    write_json(&findings, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    // Parse back
    let parsed: Vec<Finding> = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.len(), findings.len());
    assert_eq!(parsed[0].rule_id, findings[0].rule_id);
    assert_eq!(parsed[0].fingerprint, findings[0].fingerprint);
}

#[test]
fn csv_report_has_header_and_data() {
    let scanner = Scanner::new(ScanOptions::default());
    let findings = scanner.scan_text("AKIAK4JM7NR2PX6SWT3B", "test.env", None);

    let mut buf = Vec::new();
    write_csv(&findings, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert!(
        lines[0].starts_with("RuleID,"),
        "first line should be header"
    );
    assert!(lines.len() >= 2, "should have header + data rows");
    assert!(lines[1].contains("aws-access-token"));
}

#[test]
fn sarif_report_valid_structure() {
    let scanner = Scanner::new(ScanOptions::default());
    let findings = scanner.scan_text("AKIAK4JM7NR2PX6SWT3B", "test.env", None);

    let mut buf = Vec::new();
    write_sarif(&findings, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    let sarif: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["runs"].is_array());
    assert!(sarif["runs"][0]["tool"]["driver"]["name"] == "chub");
    assert!(sarif["runs"][0]["results"].is_array());
    assert!(sarif["runs"][0]["results"][0]["ruleId"] == "aws-access-token");
}

#[test]
fn empty_findings_produce_valid_output() {
    let findings: Vec<Finding> = Vec::new();

    // JSON
    let mut buf = Vec::new();
    write_json(&findings, &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "[]");

    // CSV
    let mut buf = Vec::new();
    write_csv(&findings, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output.lines().count(), 1); // header only

    // SARIF
    let mut buf = Vec::new();
    write_sarif(&findings, &mut buf).unwrap();
    let sarif: serde_json::Value = serde_json::from_str(&String::from_utf8(buf).unwrap()).unwrap();
    assert!(sarif["runs"][0]["results"].as_array().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Config loading
// ---------------------------------------------------------------------------

#[test]
fn parse_gitleaks_compatible_config() {
    let raw = r#"
title = "My Scan Config"

[allowlist]
description = "global allowlist"
paths = ['''test/.*''', '''fixtures/.*''']
regexes = ['''EXAMPLE''']

[[rules]]
id = "custom-internal-token"
regex = '''INTERNAL_[A-Z0-9]{32}'''
keywords = ["internal_"]
"#;
    let cfg: ScanConfig = toml::from_str(raw).unwrap();
    assert_eq!(cfg.title.unwrap(), "My Scan Config");
    assert_eq!(cfg.allowlist.paths.len(), 2);
    assert_eq!(cfg.allowlist.regexes.len(), 1);
    assert_eq!(cfg.rules.len(), 1);
    assert_eq!(cfg.rules[0].id, "custom-internal-token");
}

#[test]
fn report_format_parsing() {
    assert_eq!(ReportFormat::parse("json"), Some(ReportFormat::Json));
    assert_eq!(ReportFormat::parse("JSON"), Some(ReportFormat::Json));
    assert_eq!(ReportFormat::parse("sarif"), Some(ReportFormat::Sarif));
    assert_eq!(ReportFormat::parse("csv"), Some(ReportFormat::Csv));
    assert_eq!(ReportFormat::parse("xml"), None);
}

#[test]
fn report_format_from_extension() {
    assert_eq!(
        ReportFormat::from_extension("report.json"),
        Some(ReportFormat::Json)
    );
    assert_eq!(
        ReportFormat::from_extension("report.sarif"),
        Some(ReportFormat::Sarif)
    );
    assert_eq!(
        ReportFormat::from_extension("output.csv"),
        Some(ReportFormat::Csv)
    );
}

// ---------------------------------------------------------------------------
// Multiple secrets per file (betterleaks edge case)
// ---------------------------------------------------------------------------

#[test]
fn multiple_secrets_same_file() {
    let scanner = Scanner::new(ScanOptions::default());
    let text = r#"
AWS_KEY=AKIAK4JM7NR2PX6SWT3B
GITHUB_TOKEN=ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2
STRIPE_KEY=sk_live_k4Jm8nR2pX6sW9vB3fH7aT1q
"#;
    let findings = scanner.scan_text(text, "multi.env", None);
    assert!(
        findings.len() >= 3,
        "should find all 3 secrets, got {}: {:?}",
        findings.len(),
        findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
    );

    // Each should have different fingerprints
    let fps: std::collections::HashSet<&str> =
        findings.iter().map(|f| f.fingerprint.as_str()).collect();
    assert_eq!(fps.len(), findings.len(), "fingerprints should be unique");
}

#[test]
fn duplicate_secret_same_line_different_positions() {
    let scanner = Scanner::new(ScanOptions::default());
    // Two AWS keys on the same line
    let text = "keys: AKIAK4JM7NR2PX6SWT3B AKIAVNR2PX6SWT3BK4JM";
    let findings = scanner.scan_text(text, "test.env", None);
    assert!(
        findings.len() >= 2,
        "should detect both keys: {:?}",
        findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Git diff scanning
// ---------------------------------------------------------------------------

#[test]
fn scan_diff_only_added_lines() {
    let scanner = Scanner::new(ScanOptions::default());
    let diff = r#"diff --git a/config.env b/config.env
--- a/config.env
+++ b/config.env
@@ -1,2 +1,3 @@
 NORMAL_VAR=hello
+AWS_KEY=AKIAK4JM7NR2PX6SWT3B
+CLEAN_VAR=world
-REMOVED_SECRET=ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2
"#;
    // Note: scan_diff is not public, but we can test through scan_git behavior
    // or use scan_text with the raw diff
    let findings = scanner.scan_text(diff, "patch.diff", None);
    // The scanner should find the AWS key in the added line
    // and the GH token in the removed line (it scans the whole text)
    assert!(
        findings.iter().any(|f| f.rule_id == "aws-access-token"),
        "should detect AWS key"
    );
}

// ---------------------------------------------------------------------------
// Rule count
// ---------------------------------------------------------------------------

#[test]
fn scanner_has_sufficient_rules() {
    let scanner = Scanner::new(ScanOptions::default());
    assert!(
        scanner.rule_count() >= 60,
        "should have at least 60 rules, got {}",
        scanner.rule_count()
    );
}
