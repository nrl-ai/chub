//! Core scanning engine — scans text fragments and produces Findings.
//!
//! Wraps the redaction engine's rule set but outputs structured Findings
//! with line/column locations, fingerprints, and git metadata.

use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use gix::bstr::ByteSlice;
use regex::Regex;

use super::config::ScanConfig;
use super::finding::Finding;
use crate::team::tracking::redact::{RedactConfig, Redactor};

/// Options controlling scanner behavior.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Config file path (explicit).
    pub config_path: Option<String>,
    /// Baseline report path (findings in baseline are skipped).
    pub baseline_path: Option<String>,
    /// Maximum file size to scan (bytes). 0 = unlimited.
    pub max_target_bytes: u64,
    /// Redaction percentage (0-100) for output.
    pub redact_percent: u8,
    /// Only enable these rule IDs (empty = all).
    pub enable_rules: Vec<String>,
    /// Follow symlinks when scanning directories.
    pub follow_symlinks: bool,
    /// Path allowlist regexes from .gitleaksignore or config.
    pub ignore_paths: Vec<String>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            config_path: None,
            baseline_path: None,
            max_target_bytes: 10 * 1024 * 1024, // 10 MB default
            redact_percent: 0,
            enable_rules: Vec::new(),
            follow_symlinks: false,
            ignore_paths: Vec::new(),
        }
    }
}

/// The scanner — holds a compiled Redactor and config.
pub struct Scanner {
    redactor: Redactor,
    options: ScanOptions,
    baseline_fingerprints: HashSet<String>,
    path_allowlist: Vec<Regex>,
    _scan_config: Option<ScanConfig>,
}

impl Scanner {
    /// Create a scanner with default rules and given options.
    pub fn new(options: ScanOptions) -> Self {
        let (redactor, scan_config) = build_redactor(&options);
        let baseline_fingerprints = load_baseline(options.baseline_path.as_deref());
        let path_allowlist = build_path_allowlist(&options, scan_config.as_ref());

        Self {
            redactor,
            options,
            baseline_fingerprints,
            path_allowlist,
            _scan_config: scan_config,
        }
    }

    /// Scan a single text fragment (e.g., file contents, stdin, diff chunk).
    /// Returns findings with line/column locations.
    pub fn scan_text(
        &self,
        text: &str,
        file: &str,
        commit_info: Option<&CommitInfo>,
    ) -> Vec<Finding> {
        let result = self.redactor.redact(text);
        if result.findings.is_empty() {
            return Vec::new();
        }

        let mut findings = Vec::new();
        for rf in &result.findings {
            // Filter by enabled rules
            if !self.options.enable_rules.is_empty()
                && !self.options.enable_rules.iter().any(|r| r == &rf.rule_id)
            {
                continue;
            }

            // Compute line/column from byte offset
            let (start_line, start_col) = byte_offset_to_line_col(text, rf.start);
            let (end_line, end_col) = byte_offset_to_line_col(text, rf.end);

            let secret = &text[rf.start..rf.end];
            let match_text = extract_match_line(text, rf.start, rf.end);

            let commit = commit_info.map(|c| c.hash.as_str()).unwrap_or("");
            let fingerprint = Finding::compute_fingerprint(&rf.rule_id, file, secret, commit);

            // Baseline check
            if self.baseline_fingerprints.contains(&fingerprint) {
                continue;
            }

            let entropy = crate::team::tracking::redact::shannon_entropy_pub(secret);

            let mut finding = Finding {
                rule_id: rf.rule_id.clone(),
                description: rule_description(&rf.rule_id),
                start_line,
                end_line,
                start_column: start_col,
                end_column: end_col,
                match_text,
                secret: secret.to_string(),
                file: file.to_string(),
                symlink_file: String::new(),
                commit: commit.to_string(),
                entropy,
                author: commit_info.map(|c| c.author.clone()).unwrap_or_default(),
                email: commit_info.map(|c| c.email.clone()).unwrap_or_default(),
                date: commit_info.map(|c| c.date.clone()).unwrap_or_default(),
                message: commit_info.map(|c| c.message.clone()).unwrap_or_default(),
                tags: Vec::new(),
                fingerprint,
            };

            // Apply redaction to output if requested
            if self.options.redact_percent > 0 {
                finding = finding.redacted(self.options.redact_percent);
            }

            findings.push(finding);
        }

        findings
    }

    /// Scan a directory recursively. Uses rayon for parallel file scanning.
    pub fn scan_dir(&self, dir: &Path) -> Vec<Finding> {
        use rayon::prelude::*;

        let walker = if self.options.follow_symlinks {
            walkdir::WalkDir::new(dir).follow_links(true)
        } else {
            walkdir::WalkDir::new(dir)
        };

        // Collect eligible file paths first, then scan in parallel.
        let files: Vec<(PathBuf, String)> = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                if is_binary_extension(path) {
                    return None;
                }
                if let Ok(meta) = std::fs::metadata(path) {
                    if self.options.max_target_bytes > 0
                        && meta.len() > self.options.max_target_bytes
                    {
                        return None;
                    }
                }
                let rel_path = path
                    .strip_prefix(dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if self.is_path_ignored(&rel_path) {
                    return None;
                }
                Some((path.to_path_buf(), rel_path))
            })
            .collect();

        // Parallel scan with rayon
        files
            .par_iter()
            .flat_map(|(path, rel_path)| {
                std::fs::read_to_string(path)
                    .map(|content| self.scan_text(&content, rel_path, None))
                    .unwrap_or_default()
            })
            .collect()
    }

    /// Scan git history using native gix library.
    /// Falls back to `git` CLI if gix fails (e.g. unsupported repo format).
    pub fn scan_git(&self, repo: &Path, log_opts: Option<&str>, staged_only: bool) -> Vec<Finding> {
        self.scan_git_gix(repo, log_opts, staged_only)
            .unwrap_or_else(|_| self.scan_git_cli(repo, log_opts, staged_only))
    }

    /// Native gix-based git scanning.
    fn scan_git_gix(
        &self,
        repo_path: &Path,
        _log_opts: Option<&str>,
        staged_only: bool,
    ) -> Result<Vec<Finding>, String> {
        // Staged scanning uses CLI (gix index API is complex, and staged scanning
        // is typically fast since it's just a few files).
        if staged_only {
            return Err("use CLI for staged".to_string());
        }
        self.scan_git_history_gix(repo_path)
    }

    /// Two-phase git history scan optimised for maximum throughput.
    ///
    /// **Phase 1** — walk all commits sequentially via gix. For each commit's tree,
    ///   walk all blob entries directly (no tree-diff computation). Skip any blob OID
    ///   already seen in a prior commit — this is equivalent to only scanning the first
    ///   version of each unique file content, avoiding re-scanning unchanged files.
    ///   Collect (commit_info, path, content) for unique blobs.
    ///
    /// **Phase 2** — parallel regex scanning across all collected targets with rayon.
    ///
    /// Avoiding `for_each_to_obtain_tree` (tree diffing) eliminates significant per-commit
    /// overhead; the OID deduplication achieves the same "only new content" property.
    fn scan_git_history_gix(&self, repo_path: &Path) -> Result<Vec<Finding>, String> {
        use rayon::prelude::*;

        let mut repo = gix::open(repo_path).map_err(|e| format!("gix open: {}", e))?;
        repo.object_cache_size_if_unset(8 * 1024 * 1024);

        let head_id = repo.head_id().map_err(|e| format!("head_id: {}", e))?;
        let walk = repo
            .rev_walk([head_id.detach()])
            .all()
            .map_err(|e| format!("rev_walk: {}", e))?;

        // Phase 1: walk all commit trees, collect unique blobs
        let mut targets: Vec<(CommitInfo, String, String)> = Vec::new();
        let mut seen_oids: HashSet<gix::ObjectId> = HashSet::new();

        for info_result in walk {
            let info = match info_result {
                Ok(i) => i,
                Err(_) => continue,
            };
            let commit = match info.object() {
                Ok(c) => c,
                Err(_) => continue,
            };
            let ci = extract_commit_info(&commit);
            let tree = match commit.tree() {
                Ok(t) => t,
                Err(_) => continue,
            };

            // Walk all blob entries in this commit's tree directly.
            // OID dedup ensures each unique file content is only scanned once.
            for entry in tree.iter().flatten() {
                if !entry.mode().is_blob() {
                    continue;
                }
                let oid = entry.object_id();
                if seen_oids.contains(&oid) {
                    continue;
                }
                let path_str = entry.filename().to_str_lossy().to_string();
                if self.is_path_ignored(&path_str) || is_binary_extension(Path::new(&path_str)) {
                    continue;
                }
                if let Ok(blob) = repo.find_object(oid) {
                    if let Ok(content) = std::str::from_utf8(&blob.data) {
                        seen_oids.insert(oid);
                        targets.push((ci.clone(), path_str, content.to_string()));
                    }
                }
            }
        }

        // Phase 2: parallel regex scanning
        let findings = targets
            .par_iter()
            .flat_map(|(ci, path, content)| self.scan_text(content, path, Some(ci)))
            .collect();

        Ok(findings)
    }

    /// Fallback: scan git using CLI (when gix fails).
    fn scan_git_cli(&self, repo: &Path, log_opts: Option<&str>, staged_only: bool) -> Vec<Finding> {
        let mut findings = Vec::new();

        if staged_only {
            let output = std::process::Command::new("git")
                .args(["diff", "--cached", "-U0", "--diff-filter=ACM"])
                .current_dir(repo)
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    let diff = String::from_utf8_lossy(&out.stdout);
                    findings.extend(self.scan_diff(&diff, None));
                }
            }
        } else {
            let mut args = vec![
                "log".to_string(),
                "-p".to_string(),
                "--diff-filter=ACM".to_string(),
                "--format=%H%n%an%n%ae%n%aI%n%s%n---COMMIT_END---".to_string(),
            ];
            if let Some(opts) = log_opts {
                args.extend(opts.split_whitespace().map(String::from));
            }
            let output = std::process::Command::new("git")
                .args(&args)
                .current_dir(repo)
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    let log_output = String::from_utf8_lossy(&out.stdout);
                    findings.extend(self.parse_git_log(&log_output));
                }
            }
        }

        findings
    }

    /// Scan from a reader (stdin or pipe).
    pub fn scan_reader<R: Read>(&self, reader: R, label: &str) -> Vec<Finding> {
        let mut content = String::new();
        let mut buf_reader = std::io::BufReader::new(reader);
        if buf_reader.read_to_string(&mut content).is_err() {
            return Vec::new();
        }
        self.scan_text(&content, label, None)
    }

    /// Check if a path should be ignored.
    fn is_path_ignored(&self, path: &str) -> bool {
        // Built-in ignores
        if path.starts_with(".git/") || path == ".git" {
            return true;
        }
        self.path_allowlist.iter().any(|re| re.is_match(path))
    }

    /// Parse unified diff output and scan added lines.
    fn scan_diff(&self, diff: &str, default_commit: Option<&CommitInfo>) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut current_file = String::new();

        for line in diff.lines() {
            if let Some(rest) = line.strip_prefix("+++ b/") {
                current_file = rest.to_string();
            } else if line.starts_with('+') && !line.starts_with("+++") {
                // Added line — scan it
                let content = &line[1..];
                if !current_file.is_empty() && !self.is_path_ignored(&current_file) {
                    let file_findings = self.scan_text(content, &current_file, default_commit);
                    findings.extend(file_findings);
                }
            }
        }

        findings
    }

    /// Parse `git log -p` output with commit metadata.
    fn parse_git_log(&self, log_output: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut current_commit: Option<CommitInfo> = None;
        let mut diff_lines = Vec::new();
        let mut in_header = false;
        let mut header_line = 0u8;

        for line in log_output.lines() {
            if line == "---COMMIT_END---" {
                // Process accumulated diff for this commit
                if !diff_lines.is_empty() {
                    let diff_text = diff_lines.join("\n");
                    findings.extend(self.scan_diff(&diff_text, current_commit.as_ref()));
                    diff_lines.clear();
                }
                in_header = true;
                header_line = 0;
                current_commit = None;
                continue;
            }

            if in_header && header_line < 5 {
                match header_line {
                    0 => {
                        // Commit hash (40 hex chars)
                        if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
                            current_commit = Some(CommitInfo {
                                hash: line.to_string(),
                                author: String::new(),
                                email: String::new(),
                                date: String::new(),
                                message: String::new(),
                            });
                        }
                    }
                    1 => {
                        if let Some(ref mut c) = current_commit {
                            c.author = line.to_string();
                        }
                    }
                    2 => {
                        if let Some(ref mut c) = current_commit {
                            c.email = line.to_string();
                        }
                    }
                    3 => {
                        if let Some(ref mut c) = current_commit {
                            c.date = line.to_string();
                        }
                    }
                    4 => {
                        if let Some(ref mut c) = current_commit {
                            c.message = line.to_string();
                        }
                        in_header = false;
                    }
                    _ => {}
                }
                header_line += 1;
                continue;
            }

            diff_lines.push(line);
        }

        // Process any remaining diff
        if !diff_lines.is_empty() {
            let diff_text = diff_lines.join("\n");
            findings.extend(self.scan_diff(&diff_text, current_commit.as_ref()));
        }

        findings
    }

    /// Return the number of rules loaded.
    pub fn rule_count(&self) -> usize {
        self.redactor.rule_count()
    }
}

/// Git commit metadata.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub message: String,
}

/// Extract commit metadata from a gix Commit object.
fn extract_commit_info(commit: &gix::Commit<'_>) -> CommitInfo {
    let hash = commit.id().to_hex().to_string();
    let (author_name, author_email, date) = commit
        .author()
        .map(|a| {
            let name = a.name.to_str_lossy().into_owned();
            let email = a.email.to_str_lossy().into_owned();
            let date = a
                .time()
                .map(|t| t.format(gix::date::time::format::ISO8601_STRICT))
                .unwrap_or_default();
            (name, email, date)
        })
        .unwrap_or_default();
    let message = commit
        .message()
        .ok()
        .map(|m| m.summary().to_str_lossy().into_owned())
        .unwrap_or_default();
    CommitInfo {
        hash,
        author: author_name,
        email: author_email,
        date,
        message,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a Redactor from scan options + config file.
fn build_redactor(options: &ScanOptions) -> (Redactor, Option<ScanConfig>) {
    let target_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = ScanConfig::find_config(options.config_path.as_deref(), &target_dir);

    let scan_config = config_path.and_then(|p| ScanConfig::from_file(&p).ok());

    let mut redact_config = RedactConfig::default();

    if let Some(ref cfg) = scan_config {
        // Add global allowlist regexes
        redact_config.allowlist_regexes = cfg.allowlist.regexes.clone();

        // Add extra rules from config
        for rule in &cfg.rules {
            if let Some(ref regex) = rule.regex {
                redact_config
                    .extra_patterns
                    .push((rule.id.clone(), regex.clone()));
            }
        }
    }

    // Add ignore paths from options
    redact_config.allowlist_paths = options.ignore_paths.clone();

    // Scanner operates on raw text — skip expensive base64 decode pass
    redact_config.skip_base64_decode = true;

    (Redactor::from_config(&redact_config), scan_config)
}

/// Build path allowlist from config + .gitleaksignore.
fn build_path_allowlist(options: &ScanOptions, scan_config: Option<&ScanConfig>) -> Vec<Regex> {
    let mut patterns: Vec<String> = options.ignore_paths.clone();

    if let Some(cfg) = scan_config {
        patterns.extend(cfg.allowlist.paths.clone());
    }

    // Load .gitleaksignore / .betterleaksignore / .chub-scan-ignore
    for name in [".chub-scan-ignore", ".betterleaksignore", ".gitleaksignore"] {
        if let Ok(content) = std::fs::read_to_string(name) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    patterns.push(line.to_string());
                }
            }
            break; // Use the first ignore file found
        }
    }

    patterns.iter().filter_map(|p| Regex::new(p).ok()).collect()
}

/// Load baseline fingerprints from a JSON report.
fn load_baseline(path: Option<&str>) -> HashSet<String> {
    let mut fps = HashSet::new();
    if let Some(p) = path {
        if let Ok(content) = std::fs::read_to_string(p) {
            if let Ok(findings) = serde_json::from_str::<Vec<Finding>>(&content) {
                for f in findings {
                    fps.insert(f.fingerprint);
                }
            }
        }
    }
    fps
}

/// Convert byte offset to (line, column) — both 1-based.
fn byte_offset_to_line_col(text: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(text.len());
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Extract the line containing the match for the Match field.
fn extract_match_line(text: &str, start: usize, end: usize) -> String {
    // Find line boundaries around the match
    let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[end..]
        .find('\n')
        .map(|i| end + i)
        .unwrap_or(text.len());
    text[line_start..line_end].to_string()
}

/// Human-readable description for a rule ID.
/// Delegates to the shared Redactor instance which has descriptions from the
/// betterleaks TOML for all 261 rules.
fn rule_description(rule_id: &str) -> String {
    static REDACTOR: OnceLock<crate::team::tracking::redact::Redactor> = OnceLock::new();
    REDACTOR
        .get_or_init(crate::team::tracking::redact::Redactor::new)
        .rule_description(rule_id)
}

/// Check if a file extension suggests binary content.
fn is_binary_extension(path: &Path) -> bool {
    const BINARY_EXTS: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp", "avif", "mp3", "mp4", "avi",
        "mov", "wav", "flac", "ogg", "webm", "zip", "tar", "gz", "bz2", "xz", "zst", "7z", "rar",
        "exe", "dll", "so", "dylib", "o", "a", "lib", "pdf", "doc", "docx", "xls", "xlsx", "ppt",
        "pptx", "woff", "woff2", "ttf", "otf", "eot", "class", "pyc", "pyo", "wasm", "db",
        "sqlite", "sqlite3",
    ];
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| BINARY_EXTS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_offset_to_line_col_basic() {
        let text = "line1\nline2\nline3";
        assert_eq!(byte_offset_to_line_col(text, 0), (1, 1));
        assert_eq!(byte_offset_to_line_col(text, 5), (1, 6));
        assert_eq!(byte_offset_to_line_col(text, 6), (2, 1));
        assert_eq!(byte_offset_to_line_col(text, 12), (3, 1));
    }

    #[test]
    fn byte_offset_to_line_col_empty() {
        assert_eq!(byte_offset_to_line_col("", 0), (1, 1));
    }

    #[test]
    fn extract_match_line_basic() {
        let text = "first line\nsecret: AKIA123\nthird line";
        let line = extract_match_line(text, 19, 26);
        assert_eq!(line, "secret: AKIA123");
    }

    #[test]
    fn is_binary_ext() {
        assert!(is_binary_extension(Path::new("image.png")));
        assert!(is_binary_extension(Path::new("archive.tar.gz")));
        assert!(!is_binary_extension(Path::new("code.rs")));
        assert!(!is_binary_extension(Path::new("config.yaml")));
    }

    #[test]
    fn scanner_scans_text() {
        let scanner = Scanner::new(ScanOptions::default());
        let text = "AWS_KEY=AKIAK4JM7NR2PX6SWT3B";
        let findings = scanner.scan_text(text, "test.env", None);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].rule_id, "aws-access-token");
        assert_eq!(findings[0].file, "test.env");
        assert_eq!(findings[0].start_line, 1);
        assert!(!findings[0].fingerprint.is_empty());
    }

    #[test]
    fn scanner_clean_text_no_findings() {
        let scanner = Scanner::new(ScanOptions::default());
        let text = "Hello, this is normal code without secrets.";
        let findings = scanner.scan_text(text, "clean.rs", None);
        assert!(findings.is_empty());
    }

    #[test]
    fn scanner_multiline_locations() {
        let scanner = Scanner::new(ScanOptions::default());
        let text = "line1\nline2\nAWS_KEY=AKIAK4JM7NR2PX6SWT3B\nline4";
        let findings = scanner.scan_text(text, "test.env", None);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].start_line, 3);
    }

    #[test]
    fn scanner_baseline_filters() {
        let scanner = Scanner::new(ScanOptions::default());
        let text = "AWS_KEY=AKIAIOSFODNN7EXAMPLE";
        let findings = scanner.scan_text(text, "test.env", None);
        assert!(!findings.is_empty());

        // Now scan with baseline containing that fingerprint
        let mut opts = ScanOptions::default();
        // We'll manually set the fingerprint instead of a file
        let fp = findings[0].fingerprint.clone();
        let mut scanner2 = Scanner::new(opts);
        scanner2.baseline_fingerprints.insert(fp);
        let findings2 = scanner2.scan_text(text, "test.env", None);
        assert!(findings2.is_empty(), "baseline should filter the finding");
    }

    #[test]
    fn scanner_rule_filtering() {
        let opts = ScanOptions {
            enable_rules: vec!["github-pat".to_string()],
            ..Default::default()
        };
        let scanner = Scanner::new(opts);
        let text = "AWS_KEY=AKIAIOSFODNN7EXAMPLE ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234";
        let findings = scanner.scan_text(text, "test.env", None);
        // Only github-pat should be found, not aws
        assert!(findings.iter().all(|f| f.rule_id == "github-pat"));
    }

    #[test]
    fn scanner_redact_output() {
        let opts = ScanOptions {
            redact_percent: 100,
            ..Default::default()
        };
        let scanner = Scanner::new(opts);
        let text = "AWS_KEY=AKIAIOSFODNN7EXAMPLE";
        let findings = scanner.scan_text(text, "test.env", None);
        assert!(!findings.is_empty());
        assert!(findings[0].secret.contains('*'));
        assert!(!findings[0].secret.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn scanner_git_ignore_path() {
        let scanner = Scanner::new(ScanOptions::default());
        assert!(scanner.is_path_ignored(".git/config"));
        assert!(scanner.is_path_ignored(".git"));
        assert!(!scanner.is_path_ignored("src/main.rs"));
    }

    #[test]
    fn scan_diff_finds_added_secrets() {
        let scanner = Scanner::new(ScanOptions::default());
        let diff = r#"diff --git a/test.env b/test.env
--- /dev/null
+++ b/test.env
@@ -0,0 +1 @@
+AWS_KEY=AKIAIOSFODNN7EXAMPLE
"#;
        let findings = scanner.scan_diff(diff, None);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].file, "test.env");
    }

    #[test]
    fn scan_diff_ignores_removed_lines() {
        let scanner = Scanner::new(ScanOptions::default());
        let diff = r#"diff --git a/test.env b/test.env
--- a/test.env
+++ b/test.env
@@ -1 +0,0 @@
-AWS_KEY=AKIAIOSFODNN7EXAMPLE
"#;
        let findings = scanner.scan_diff(diff, None);
        assert!(findings.is_empty(), "removed lines should not be scanned");
    }
}
