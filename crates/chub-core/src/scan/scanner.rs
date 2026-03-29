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
    /// Scan only diff lines per commit (fast, matches gitleaks behaviour).
    /// When false, scans full blob content of every unique file (thorough but slow).
    pub diff_only: bool,
    /// Run CEL validation on findings that have a `validate` expression.
    /// Makes live HTTP/API calls — disabled by default, opt-in only.
    pub validate: bool,
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
            diff_only: true,
            validate: false,
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
    /// Rule IDs that have `tokenEfficiency = true` in their config.
    token_efficiency_rules: HashSet<String>,
    /// Rule ID → CEL validate expression (rules that have `validate = "..."` set).
    validate_exprs: std::collections::HashMap<String, String>,
}

impl Scanner {
    /// Create a scanner with default rules and given options.
    pub fn new(options: ScanOptions) -> Self {
        let (redactor, scan_config) = build_redactor(&options);
        let baseline_fingerprints = load_baseline(options.baseline_path.as_deref());
        let path_allowlist = build_path_allowlist(&options, scan_config.as_ref());
        let token_efficiency_rules = build_token_efficiency_set(scan_config.as_ref());
        let validate_exprs = build_validate_exprs(scan_config.as_ref());

        Self {
            redactor,
            options,
            baseline_fingerprints,
            path_allowlist,
            _scan_config: scan_config,
            token_efficiency_rules,
            validate_exprs,
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
        self.scan_text_with(&self.redactor, text, file, commit_info)
    }

    /// Like `scan_text` but uses an explicit redactor (enables per-thread redactor clones
    /// to eliminate `regex-automata` CachePool mutex contention in parallel scans).
    fn scan_text_with(
        &self,
        redactor: &Redactor,
        text: &str,
        file: &str,
        commit_info: Option<&CommitInfo>,
    ) -> Vec<Finding> {
        let result = redactor.redact(text);
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

            // BPE token-efficiency filter (betterleaks-compatible): drop findings that look
            // like natural language rather than random secrets.
            if self.token_efficiency_rules.contains(&rf.rule_id)
                && super::token_filter::fails_token_efficiency_filter(secret)
            {
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
                validation_status: None,
                validation_reason: None,
            };

            // CEL validation: call the live API to confirm the secret is active.
            if self.options.validate {
                if let Some(expr) = self.validate_exprs.get(&rf.rule_id) {
                    let vr = super::cel_validate::evaluate_validate(
                        &rf.rule_id,
                        expr,
                        secret,
                        &std::collections::HashMap::new(),
                    );
                    finding.validation_status = Some(vr.status);
                    finding.validation_reason = vr.reason;
                }
            }

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

        // Parallel scan with rayon (per-thread Redactor clones to avoid CachePool contention)
        let n = rayon::current_num_threads().max(1);
        let redactors: Vec<Redactor> = (0..n).map(|_| self.redactor.clone()).collect();
        files
            .par_iter()
            .flat_map(|(path, rel_path)| {
                let idx = rayon::current_thread_index().unwrap_or(0) % n;
                std::fs::read_to_string(path)
                    .map(|content| self.scan_text_with(&redactors[idx], &content, rel_path, None))
                    .unwrap_or_default()
            })
            .collect()
    }

    /// Scan git history.
    ///
    /// Strategy:
    /// - `diff_only = true` (default): parallel `git log -p --no-walk --stdin` workers,
    ///   matching the betterleaks ParallelGit design. Falls back to gix if git unavailable.
    /// - `diff_only = false`: full-blob scan via gix with OID deduplication (thorough mode).
    /// - `staged_only`: use git CLI diff --cached.
    pub fn scan_git(&self, repo: &Path, log_opts: Option<&str>, staged_only: bool) -> Vec<Finding> {
        if staged_only {
            return self.scan_git_cli(repo, log_opts, staged_only);
        }
        if self.options.diff_only {
            // Parallel workers (betterleaks ParallelGit design): partition commits across
            // min(cpu, 4) OS threads each running its own `git log --no-walk --stdin`.
            // Falls back to single subprocess, then gix tree-diff.
            if let Some(findings) = self.scan_git_log_parallel(repo, log_opts) {
                return findings;
            }
            if let Some(findings) = self.scan_git_log_single(repo, log_opts) {
                return findings;
            }
            return self.scan_git_history_diff(repo).unwrap_or_default();
        }
        // Full-blob mode: gix with OID dedup.
        self.scan_git_history_gix(repo)
            .unwrap_or_else(|_| self.scan_git_cli(repo, log_opts, staged_only))
    }

    /// Parallel git history scan — betterleaks ParallelGit design.
    ///
    /// Phase A (I/O, parallel OS threads): partition commits across min(cpu, 4) workers,
    ///   each running its own `git log -p --no-walk --stdin` process. Each worker streams
    ///   output into a Vec of (file, content, commit_info) batches. Workers run on real OS
    ///   threads so blocking I/O doesn't starve the rayon thread pool.
    ///
    /// Single git log subprocess — simple approach, faster for small repos.
    fn scan_git_log_single(&self, repo: &Path, log_opts: Option<&str>) -> Option<Vec<Finding>> {
        use rayon::prelude::*;
        let mut args = vec![
            "log".to_string(),
            "-p".to_string(),
            "-U0".to_string(),
            "--diff-filter=ACDM".to_string(),
            "--format=%H%n%an%n%ae%n%aI%n%s%n---COMMIT_END---".to_string(),
        ];
        if let Some(opts) = log_opts {
            args.extend(opts.split_whitespace().map(String::from));
        }
        let out = std::process::Command::new("git")
            .args(&args)
            .current_dir(repo)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let log_output = String::from_utf8_lossy(&out.stdout);
        let batches =
            collect_git_log_batches_from_reader(std::io::Cursor::new(log_output.as_bytes()));
        let max_bytes = if self.options.max_target_bytes > 0 {
            self.options.max_target_bytes as usize
        } else {
            usize::MAX
        };
        let filtered: Vec<_> = batches
            .into_iter()
            .filter(|(f, c, _)| {
                !self.is_path_ignored(f)
                    && !is_binary_extension(Path::new(f))
                    && c.len() <= max_bytes
            })
            .collect();

        // Precompile all rule regexes once on this thread, then clone per rayon thread.
        // Cloning an initialized Regex shares the compiled automaton (Arc) but creates a
        // fresh independent CachePool — eliminates regex-automata Pool<Cache> mutex contention.
        self.redactor.precompile_rules();
        let n = rayon::current_num_threads().max(1);
        let redactors: Vec<Redactor> = (0..n).map(|_| self.redactor.clone()).collect();

        Some(
            filtered
                .par_iter()
                .flat_map(|(file, content, ci)| {
                    let idx = rayon::current_thread_index().unwrap_or(0) % n;
                    self.scan_text_with(&redactors[idx], content, file, ci.as_ref())
                })
                .collect(),
        )
    }

    /// Phase B (CPU, rayon): merge all batches and run parallel regex scan.
    fn scan_git_log_parallel(&self, repo: &Path, log_opts: Option<&str>) -> Option<Vec<Finding>> {
        use rayon::prelude::*;

        // Step 1: enumerate all SHAs
        let rev_out = std::process::Command::new("git")
            .args(["rev-list", "HEAD"])
            .current_dir(repo)
            .output()
            .ok()?;
        if !rev_out.status.success() {
            return None;
        }
        let rev_str = String::from_utf8_lossy(&rev_out.stdout);
        let shas: Vec<String> = rev_str.lines().map(String::from).collect();
        if shas.is_empty() {
            return Some(Vec::new());
        }

        // Step 2: partition across workers (min(cpu, 4), like betterleaks)
        let num_workers = std::thread::available_parallelism()
            .map(|n| n.get().min(4))
            .unwrap_or(2)
            .min(shas.len());

        let chunk_size = shas.len().div_ceil(num_workers);
        let chunks: Vec<Vec<String>> = shas.chunks(chunk_size).map(|c| c.to_vec()).collect();

        let repo_path = repo.to_path_buf();
        let log_opts_owned = log_opts.map(String::from);

        // Phase A: I/O — collect batches in parallel OS threads (no rayon thread blocking)
        let thread_handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                let repo_clone = repo_path.clone();
                let opts_clone = log_opts_owned.clone();
                std::thread::spawn(move || {
                    collect_git_log_batches(&repo_clone, &chunk, opts_clone.as_deref())
                        .unwrap_or_default()
                })
            })
            .collect();

        // Overlap: precompile all rule regexes on THIS thread while the git I/O workers run.
        // The rayon thread pool is idle during Phase A, so we use this dead time for
        // parallel regex compilation. Workers take ~380ms and precompile ~18ms; they overlap.
        // Critical path = max(I/O, precompile) + scan, instead of I/O + precompile + scan.
        self.redactor.precompile_rules();

        // Pre-create per-thread Redactor clones NOW (after precompile so each clone inherits
        // populated OnceLocks and fresh independent CachePools — no mutex contention).
        let n = rayon::current_num_threads().max(1);
        let redactors: Vec<Redactor> = (0..n).map(|_| self.redactor.clone()).collect();

        // Collect all batches from workers; filter ignored paths on the main thread
        let max_bytes = if self.options.max_target_bytes > 0 {
            self.options.max_target_bytes as usize
        } else {
            usize::MAX
        };
        let mut all_batches: Vec<(String, String, Option<CommitInfo>)> = Vec::new();
        for handle in thread_handles {
            if let Ok(batches) = handle.join() {
                for batch in batches {
                    if !self.is_path_ignored(&batch.0)
                        && !is_binary_extension(Path::new(&batch.0))
                        && batch.1.len() <= max_bytes
                    {
                        all_batches.push(batch);
                    }
                }
            }
        }

        // Phase B: CPU — parallel regex scan.
        // Clone per rayon thread for independent CachePools (no mutex contention).
        let findings: Vec<Finding> = all_batches
            .par_iter()
            .flat_map(|(file, content, ci)| {
                let idx = rayon::current_thread_index().unwrap_or(0) % n;
                self.scan_text_with(&redactors[idx], content, file, ci.as_ref())
            })
            .collect();

        Some(findings)
    }

    /// Diff-based git history scan with OID deduplication.
    ///
    /// **Phase 1** — walk all commits, tree-diff each against its parent to find
    ///   added/modified blobs. Skip any blob OID already seen in a newer commit.
    ///   Collect (commit_info, path, content) for unique new-blob OIDs only.
    ///   Deletions are skipped (we care about introduced secrets, not removed ones).
    ///
    /// **Phase 2** — parallel regex scanning across all collected targets with rayon.
    ///
    /// Tree-diff keeps the scan set small (only changed files); OID dedup avoids
    /// re-scanning the same content that persists across many commits.
    fn scan_git_history_diff(&self, repo_path: &Path) -> Result<Vec<Finding>, String> {
        use gix::bstr::ByteSlice;
        use gix::diff::tree::recorder::{Change as RecChange, Location};
        use gix::objs::TreeRefIter;
        use rayon::prelude::*;

        let mut repo = gix::open(repo_path).map_err(|e| format!("gix open: {}", e))?;
        repo.object_cache_size_if_unset(32 * 1024 * 1024);

        let head_id = repo.head_id().map_err(|e| format!("head_id: {}", e))?;
        let walk = repo
            .rev_walk([head_id.detach()])
            .all()
            .map_err(|e| format!("rev_walk: {}", e))?;

        // Phase 1: collect unique (ci, path, content) targets via tree-diff + OID dedup.
        let mut targets: Vec<(CommitInfo, String, String)> = Vec::new();
        let mut seen_oids: HashSet<gix::ObjectId> = HashSet::new();

        // Reusable per-loop buffers.
        let mut diff_state = gix::diff::tree::State::default();
        let mut recorder =
            gix::diff::tree::Recorder::default().track_location(Some(Location::Path));
        let mut old_tree_buf: Vec<u8> = Vec::new();
        let mut new_tree_buf: Vec<u8> = Vec::new();

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

            let new_tree_id = match commit.tree() {
                Ok(t) => t.id,
                Err(_) => continue,
            };
            let old_tree_id = commit
                .parent_ids()
                .next()
                .map(|pid| pid.detach())
                .and_then(|pid| {
                    repo.find_object(pid)
                        .ok()
                        .and_then(|obj| obj.try_into_commit().ok())
                        .and_then(|p| p.tree().ok().map(|t| t.id))
                })
                .unwrap_or_else(|| gix::ObjectId::empty_tree(repo.object_hash()));

            let old_iter = if old_tree_id.is_empty_tree() {
                TreeRefIter::from_bytes(b"")
            } else {
                match repo.find_object(old_tree_id) {
                    Ok(obj) => {
                        old_tree_buf.clear();
                        old_tree_buf.extend_from_slice(&obj.data);
                        TreeRefIter::from_bytes(&old_tree_buf)
                    }
                    Err(_) => continue,
                }
            };
            match repo.find_object(new_tree_id) {
                Ok(obj) => {
                    new_tree_buf.clear();
                    new_tree_buf.extend_from_slice(&obj.data);
                }
                Err(_) => continue,
            }
            let new_iter = TreeRefIter::from_bytes(&new_tree_buf);

            recorder.records.clear();
            let _ = gix::diff::tree(
                old_iter,
                new_iter,
                &mut diff_state,
                &repo.objects,
                &mut recorder,
            );

            for record in &recorder.records {
                let (oid, path, entry_mode) = match record {
                    RecChange::Addition {
                        oid,
                        path,
                        entry_mode,
                        ..
                    } => (oid, path, entry_mode),
                    RecChange::Modification {
                        oid,
                        path,
                        entry_mode,
                        ..
                    } => (oid, path, entry_mode),
                    RecChange::Deletion { .. } => continue,
                };
                if !entry_mode.is_blob() {
                    continue;
                }
                let path_str = path.to_str_lossy().into_owned();
                if self.is_path_ignored(&path_str)
                    || is_binary_extension(Path::new(&path_str))
                    || seen_oids.contains(oid)
                {
                    continue;
                }
                if let Ok(blob) = repo.find_object(*oid) {
                    if let Ok(content) = std::str::from_utf8(&blob.data) {
                        seen_oids.insert(*oid);
                        targets.push((ci.clone(), path_str, content.to_string()));
                    }
                }
            }
        }

        // Phase 2: parallel regex scanning (per-thread Redactor clones).
        let n = rayon::current_num_threads().max(1);
        let redactors: Vec<Redactor> = (0..n).map(|_| self.redactor.clone()).collect();
        let findings = targets
            .par_iter()
            .flat_map(|(ci, path, content)| {
                let idx = rayon::current_thread_index().unwrap_or(0) % n;
                self.scan_text_with(&redactors[idx], content, path, Some(ci))
            })
            .collect();

        Ok(findings)
    }

    /// Two-phase git history scan optimised for maximum throughput.
    ///
    /// **Phase 1** — walk all commits sequentially via gix. For each commit's tree,
    ///   **recursively** walk all blob entries in the tree (including subdirectories).
    ///   Skip any blob OID already seen in a prior commit — each unique file content is
    ///   scanned at most once regardless of how many commits contain it.
    ///   Collect (commit_info, path, content) for unique blobs.
    ///
    /// **Phase 2** — parallel regex scanning across all collected targets with rayon.
    ///
    /// Avoids `for_each_to_obtain_tree` (tree diffing) overhead; OID deduplication
    /// achieves the same "only new content" invariant without per-commit diff cost.
    fn scan_git_history_gix(&self, repo_path: &Path) -> Result<Vec<Finding>, String> {
        use rayon::prelude::*;

        let mut repo = gix::open(repo_path).map_err(|e| format!("gix open: {}", e))?;
        repo.object_cache_size_if_unset(8 * 1024 * 1024);

        let head_id = repo.head_id().map_err(|e| format!("head_id: {}", e))?;
        let walk = repo
            .rev_walk([head_id.detach()])
            .all()
            .map_err(|e| format!("rev_walk: {}", e))?;

        // Phase 1: walk all commit trees recursively, collect unique blobs
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

            // Recursively walk all blobs in this commit's tree.
            // OID dedup skips blobs already scanned from a newer commit.
            self.walk_tree_blobs(&repo, tree.id, "", &ci, &mut targets, &mut seen_oids);
        }

        // Phase 2: parallel regex scanning (per-thread Redactor clones).
        let n = rayon::current_num_threads().max(1);
        let redactors: Vec<Redactor> = (0..n).map(|_| self.redactor.clone()).collect();
        let findings = targets
            .par_iter()
            .flat_map(|(ci, path, content)| {
                let idx = rayon::current_thread_index().unwrap_or(0) % n;
                self.scan_text_with(&redactors[idx], content, path, Some(ci))
            })
            .collect();

        Ok(findings)
    }

    /// Recursively walk a git tree, collecting (commit_info, path, content) for
    /// blobs not yet seen. Traverses into subtrees (directories) recursively.
    fn walk_tree_blobs(
        &self,
        repo: &gix::Repository,
        tree_oid: gix::ObjectId,
        prefix: &str,
        ci: &CommitInfo,
        targets: &mut Vec<(CommitInfo, String, String)>,
        seen_oids: &mut HashSet<gix::ObjectId>,
    ) {
        let tree_obj = match repo.find_object(tree_oid) {
            Ok(o) => o,
            Err(_) => return,
        };
        let tree = match tree_obj.try_into_tree() {
            Ok(t) => t,
            Err(_) => return,
        };

        for entry in tree.iter().flatten() {
            let name = entry.filename().to_str_lossy();
            let full_path = if prefix.is_empty() {
                name.into_owned()
            } else {
                format!("{}/{}", prefix, name)
            };

            if entry.mode().is_blob() {
                let oid = entry.object_id();
                if seen_oids.contains(&oid)
                    || self.is_path_ignored(&full_path)
                    || is_binary_extension(Path::new(&full_path))
                {
                    continue;
                }
                if let Ok(blob) = repo.find_object(oid) {
                    if let Ok(content) = std::str::from_utf8(&blob.data) {
                        seen_oids.insert(oid);
                        targets.push((ci.clone(), full_path, content.to_string()));
                    }
                }
            } else if entry.mode().is_tree() {
                // Recurse into subdirectory
                self.walk_tree_blobs(repo, entry.object_id(), &full_path, ci, targets, seen_oids);
            }
        }
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
    ///
    /// Batches added lines per file before calling `scan_text`, which avoids
    /// per-line `scan_text` call overhead on large diffs.
    fn scan_diff(&self, diff: &str, default_commit: Option<&CommitInfo>) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut current_file = String::new();
        let mut batch = String::new();

        let flush = |file: &str, batch: &mut String, findings: &mut Vec<Finding>| {
            if !batch.is_empty() && !file.is_empty() {
                findings.extend(self.scan_text(batch, file, default_commit));
                batch.clear();
            }
        };

        for line in diff.lines() {
            if let Some(rest) = line.strip_prefix("+++ b/") {
                flush(&current_file.clone(), &mut batch, &mut findings);
                current_file = rest.to_string();
            } else if line.starts_with('+')
                && !line.starts_with("+++")
                && !current_file.is_empty()
                && !self.is_path_ignored(&current_file)
            {
                batch.push_str(&line[1..]);
                batch.push('\n');
            }
        }
        flush(&current_file, &mut batch, &mut findings);

        findings
    }

    /// Parse `git log -p` output: collect (file, added_lines, commit_info) batches,
    /// then scan all batches in parallel with rayon.
    fn parse_git_log(&self, log_output: &str) -> Vec<Finding> {
        use rayon::prelude::*;

        // Phase 1: collect batches (sequential parse)
        let mut batches: Vec<(String, String, Option<CommitInfo>)> = Vec::new();
        let mut current_commit: Option<CommitInfo> = None;
        let mut current_file = String::new();
        let mut batch = String::new();
        let mut in_header = false;
        let mut header_line = 0u8;

        macro_rules! flush {
            () => {
                if !batch.is_empty() && !current_file.is_empty() {
                    batches.push((
                        current_file.clone(),
                        std::mem::take(&mut batch),
                        current_commit.clone(),
                    ));
                }
            };
        }

        for line in log_output.lines() {
            if line == "---COMMIT_END---" {
                flush!();
                current_file.clear();
                in_header = true;
                header_line = 0;
                current_commit = None;
                continue;
            }

            if in_header && header_line < 5 {
                match header_line {
                    0 => {
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

            if let Some(rest) = line.strip_prefix("+++ b/") {
                flush!();
                current_file = rest.to_string();
            } else if line.starts_with('+')
                && !line.starts_with("+++")
                && !current_file.is_empty()
                && !self.is_path_ignored(&current_file)
            {
                batch.push_str(&line[1..]);
                batch.push('\n');
            }
        }
        flush!();

        // Phase 2: parallel regex scan across all batches
        batches
            .par_iter()
            .flat_map(|(file, content, ci)| self.scan_text(content, file, ci.as_ref()))
            .collect()
    }

    /// Return the number of rules loaded.
    pub fn rule_count(&self) -> usize {
        self.redactor.rule_count()
    }
}

/// Spawn a `git log -p --no-walk --stdin` process for a specific set of commit SHAs,
/// stream the patch output, and return all (file, added_lines, commit_info) batches.
///
/// Runs on a plain OS thread (not rayon) so blocking I/O doesn't starve the thread pool.
fn collect_git_log_batches(
    repo: &Path,
    shas: &[String],
    log_opts: Option<&str>,
) -> Option<Vec<(String, String, Option<CommitInfo>)>> {
    use std::io::{BufReader, Write};

    let mut args = vec![
        "log".to_string(),
        "-p".to_string(),
        "-U0".to_string(),
        "--no-walk".to_string(),
        "--stdin".to_string(),
        "--diff-filter=ACDM".to_string(),
        "--format=%H%n%an%n%ae%n%aI%n%s%n---COMMIT_END---".to_string(),
    ];
    if let Some(opts) = log_opts {
        args.extend(opts.split_whitespace().map(String::from));
    }

    let mut child = std::process::Command::new("git")
        .args(&args)
        .current_dir(repo)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .ok()?;

    // Write SHAs to stdin in a background thread (avoid deadlock if stdout fills)
    let mut stdin = child.stdin.take()?;
    let shas_copy: Vec<String> = shas.to_vec();
    std::thread::spawn(move || {
        for sha in &shas_copy {
            let _ = writeln!(stdin, "{}", sha);
        }
    });

    let stdout = child.stdout.take()?;
    let reader = BufReader::new(stdout);
    let batches = collect_git_log_batches_from_reader(reader);

    let _ = child.wait();
    Some(batches)
}

/// Parse `git log -p` output from a reader into (file, added_lines, commit_info) batches.
fn collect_git_log_batches_from_reader<R: std::io::BufRead>(
    reader: R,
) -> Vec<(String, String, Option<CommitInfo>)> {
    let mut batches: Vec<(String, String, Option<CommitInfo>)> = Vec::new();
    let mut current_commit: Option<CommitInfo> = None;
    let mut current_file = String::new();
    let mut batch = String::new();
    let mut in_header = false;
    let mut header_line = 0u8;

    macro_rules! flush {
        () => {
            if !batch.is_empty() && !current_file.is_empty() {
                batches.push((
                    current_file.clone(),
                    std::mem::take(&mut batch),
                    current_commit.clone(),
                ));
            }
        };
    }

    let mut line_buf = String::new();
    let mut reader = reader;
    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let line = line_buf.trim_end_matches('\n').trim_end_matches('\r');

        if line == "---COMMIT_END---" {
            flush!();
            current_file.clear();
            in_header = true;
            header_line = 0;
            current_commit = None;
            continue;
        }

        if in_header && header_line < 5 {
            match header_line {
                0 => {
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

        if let Some(rest) = line.strip_prefix("+++ b/") {
            flush!();
            current_file = rest.to_string();
        } else if line.starts_with('+') && !line.starts_with("+++") && !current_file.is_empty() {
            batch.push_str(&line[1..]);
            batch.push('\n');
        }
    }
    flush!();
    batches
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

/// Collect rule IDs that have `token_efficiency = true` from the merged (built-in + user) rule set.
/// The built-in rules are parsed from `redact.rs` defaults, but `token_efficiency` is a config-only
/// flag, so we read from `ScanConfig` (user config) plus the built-in `rules.toml` via `redact`.
/// Collect rule ID → CEL validate expression from the merged rule set.
fn build_validate_exprs(
    scan_config: Option<&ScanConfig>,
) -> std::collections::HashMap<String, String> {
    use crate::team::tracking::redact::validate_exprs_map;
    let mut map = validate_exprs_map();
    // User config can add or override validate expressions
    if let Some(cfg) = scan_config {
        for rule in &cfg.rules {
            if let Some(ref expr) = rule.validate {
                map.insert(rule.id.clone(), expr.clone());
            }
        }
    }
    map
}

fn build_token_efficiency_set(scan_config: Option<&ScanConfig>) -> HashSet<String> {
    use crate::team::tracking::redact::token_efficiency_rule_ids;
    let mut set: HashSet<String> = token_efficiency_rule_ids().iter().cloned().collect();
    // User config can override / extend
    if let Some(cfg) = scan_config {
        for rule in &cfg.rules {
            if rule.token_efficiency {
                set.insert(rule.id.clone());
            }
        }
    }
    set
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
