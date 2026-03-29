//! Secret redaction for transcripts and checkpoints.
//!
//! Scans text for known secret patterns (API keys, tokens, credentials) and
//! replaces them with `[REDACTED:<rule-id>]`.
//!
//! ## Attribution
//!
//! Rule definitions (261 patterns) are loaded from **betterleaks** TOML config
//! (<https://github.com/nicosrm/betterleaks>), which is itself an enhanced fork
//! of **gitleaks** (<https://github.com/gitleaks/gitleaks>). Both projects are
//! MIT-licensed. The embedded `betterleaks.toml` file in `references/` is used
//! via `include_str!()` at compile time.
//!
//! Key techniques borrowed from betterleaks/gitleaks:
//! - **Keyword pre-filtering**: fast keyword check before expensive regex matches.
//! - **Shannon entropy scoring**: filters low-entropy false positives.
//! - **Stopword suppression**: rejects matches containing placeholder/test words.
//! - **Per-rule allowlists**: rule-specific regex/stopword exclusions.
//! - **Regex patterns**: Go RE2 patterns adapted for the Rust `regex` crate
//!   (inline flag toggles `(?-i:...)` stripped, `\x60` hex escapes resolved).
//!
//! Additional chub-specific features:
//! - Base64 decoding pass to catch encoded secrets in CI configs
//! - Generic-vs-specific rule priority deduplication
//! - Custom rules for database URLs, bearer tokens, env files, Azure keys

use std::collections::HashMap;
use std::sync::OnceLock;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use regex::{Regex, RegexBuilder};

use crate::scan::config::ScanConfig;

// ---------------------------------------------------------------------------
// Rule definition
// ---------------------------------------------------------------------------

/// A single secret detection rule.
struct Rule {
    /// Unique identifier (e.g. "aws-access-token").
    id: String,
    /// Human-readable description.
    description: String,
    /// Regex pattern string (compiled lazily on first use).
    pattern: String,
    /// Lazily compiled regex. Compiled on first match attempt.
    compiled: OnceLock<Option<Regex>>,
    /// Which capture group contains the secret (0 = whole match).
    secret_group: usize,
    /// Lowercase keywords — at least one must appear in text (quick pre-filter).
    keywords: Vec<String>,
    /// Minimum Shannon entropy for the matched secret. 0.0 disables the check.
    min_entropy: f64,
    /// Generic/catch-all rules have lower priority when overlapping with specific rules.
    generic: bool,
    /// Per-rule allowlist regexes (compiled lazily).
    allowlist_patterns: Vec<String>,
    /// Lazily compiled per-rule allowlist regexes.
    compiled_allowlists: OnceLock<Vec<Regex>>,
    /// Per-rule stopwords.
    rule_stopwords: Vec<String>,
    /// Skip this rule in reports (composite-only rules).
    #[allow(dead_code)]
    skip_report: bool,
}

impl Rule {
    /// Get the compiled regex, compiling on first access.
    fn regex(&self) -> Option<&Regex> {
        self.compiled
            .get_or_init(|| {
                let fixed = fixup_regex_pattern(&self.pattern);
                RegexBuilder::new(&fixed)
                    .size_limit(50 * 1024 * 1024)
                    .build()
                    .ok()
            })
            .as_ref()
    }

    /// Get compiled per-rule allowlist regexes.
    fn allowlists(&self) -> &[Regex] {
        self.compiled_allowlists.get_or_init(|| {
            self.allowlist_patterns
                .iter()
                .filter_map(|p| Regex::new(p).ok())
                .collect()
        })
    }
}

/// Fix regex patterns from betterleaks TOML for Rust regex crate compatibility.
/// The betterleaks patterns use Go RE2 extensions that Rust regex doesn't support:
/// - `\x60` (backtick hex escape) → literal backtick
/// - `(?-i:...)` (inline flag toggle) → approximate without flag toggle
/// - `[:alnum:]` POSIX classes → character ranges
fn fixup_regex_pattern(pattern: &str) -> String {
    let mut s = pattern.replace("\\x60", "`");
    // Remove (?-i:...) wrappers — Rust regex doesn't support inline flag toggles.
    // Simple approach: strip (?-i: and its matching ) to make it case-sensitive by default.
    // This is a slight loss in specificity but avoids compile failures.
    while let Some(start) = s.find("(?-i:") {
        // Find matching closing paren
        let inner_start = start + 5;
        let mut depth = 1;
        let mut end = inner_start;
        for (i, c) in s[inner_start..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = inner_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth == 0 {
            let inner = &s[inner_start..end];
            let inner_owned = inner.to_string();
            s = format!("{}{}{}", &s[..start], inner_owned, &s[end + 1..]);
        } else {
            break; // malformed, stop processing
        }
    }
    // Translate POSIX character classes to Rust regex equivalents.
    // These appear in betterleaks rules but are not supported by the Rust regex crate.
    s = s.replace("[[:alnum:]]", "[a-zA-Z0-9]");
    s = s.replace("[[:alpha:]]", "[a-zA-Z]");
    s = s.replace("[[:digit:]]", "[0-9]");
    s = s.replace("[[:lower:]]", "[a-z]");
    s = s.replace("[[:upper:]]", "[A-Z]");
    s = s.replace("[[:space:]]", "[\\s]");
    s = s.replace("[[:xdigit:]]", "[0-9a-fA-F]");
    s = s.replace("[[:print:]]", "[\\x20-\\x7e]");
    s = s.replace("[[:punct:]]", "[!-/:-@\\[-`{-~]");
    // Also handle variants without outer brackets: [:alnum:] inside an existing class
    // e.g. [[:alnum:]_-] — replace the inner POSIX class within a character class
    s = s.replace("[:alnum:]", "a-zA-Z0-9");
    s = s.replace("[:alpha:]", "a-zA-Z");
    s = s.replace("[:digit:]", "0-9");
    s = s.replace("[:lower:]", "a-z");
    s = s.replace("[:upper:]", "A-Z");
    s = s.replace("[:xdigit:]", "0-9a-fA-F");
    s
}

// ---------------------------------------------------------------------------
// Stopwords — substrings that indicate a match is a false positive
// ---------------------------------------------------------------------------

/// Common false positive substrings. If a matched "secret" is composed mostly
/// of these words or contains them as substrings, it's likely not a real secret.
/// Sourced from betterleaks' word-based filtering approach.
const STOPWORDS: &[&str] = &[
    // Explicit placeholders
    "placeholder",
    "your_api",
    "your-api",
    "your_key",
    "your-key",
    "your_token",
    "your-token",
    "your_secret",
    "your-secret",
    "insert_here",
    "insert-here",
    "insert_your",
    "insert-your",
    "change_me",
    "changeme",
    "replace_me",
    "replaceme",
    "xxxxxxxxxxxxxxxx", // 16+ x's
    // Documentation / template markers
    "${",
    "{{",
];

/// Check if a matched secret looks like a false positive based on stopwords.
fn contains_stopword(secret: &str) -> bool {
    let lower = secret.to_lowercase();
    STOPWORDS.iter().any(|sw| lower.contains(sw))
}

// ---------------------------------------------------------------------------
// Built-in rules (sourced from betterleaks / gitleaks patterns)
// ---------------------------------------------------------------------------

/// Base64 chunk detection regex — compiled once per process.
static BASE64_CHUNK_RE: OnceLock<Regex> = OnceLock::new();

fn get_base64_chunk_regex() -> &'static Regex {
    BASE64_CHUNK_RE.get_or_init(|| Regex::new(r"\b[A-Za-z0-9+/]{20,}={0,3}\b").unwrap())
}

/// Embedded default rules in betterleaks-compatible TOML format.
/// Parsed once at first use and cached for the lifetime of the process.
const DEFAULT_RULES_TOML: &str = include_str!("../../scan/rules.toml");

static DEFAULT_CONFIG: OnceLock<ScanConfig> = OnceLock::new();

fn load_default_config() -> &'static ScanConfig {
    DEFAULT_CONFIG.get_or_init(|| {
        toml::from_str(DEFAULT_RULES_TOML).expect("embedded default rules TOML must parse")
    })
}

/// Return the IDs of built-in rules that have `tokenEfficiency = true` in rules.toml.
pub fn token_efficiency_rule_ids() -> Vec<String> {
    load_default_config()
        .rules
        .iter()
        .filter(|r| r.token_efficiency)
        .map(|r| r.id.clone())
        .collect()
}

fn built_in_rules() -> Vec<Rule> {
    let config = load_default_config();
    let mut rules: Vec<Rule> = Vec::with_capacity(config.rules.len() + 10);

    // Set of "generic/catch-all" rule IDs — these should have lower priority
    // when overlapping with specific rules (sorted last on tie-break).
    let generic_ids: &[&str] = &["generic-api-key", "curl-auth-header", "curl-auth-user"];

    for rc in &config.rules {
        if rc.skip_report {
            continue; // Skip composite-only rules (e.g. aws-secret-access-key)
        }
        if let Some(ref regex_str) = rc.regex {
            // Collect per-rule allowlist regexes and stopwords
            let mut allowlist_patterns = Vec::new();
            let mut rule_stopwords = Vec::new();
            for al in &rc.allowlists {
                allowlist_patterns.extend(al.regexes.clone());
                rule_stopwords.extend(al.stopwords.clone());
            }

            rules.push(Rule {
                id: rc.id.clone(),
                description: rc.description.clone().unwrap_or_default(),
                pattern: regex_str.clone(),
                compiled: OnceLock::new(),
                // Betterleaks convention: group 1 wraps the secret, suffix is
                // non-capturing. Default to 1 so we don't include boundary chars.
                secret_group: rc.secret_group.unwrap_or(1),
                keywords: rc.keywords.iter().map(|k| k.to_lowercase()).collect(),
                min_entropy: rc.entropy.unwrap_or(0.0),
                generic: generic_ids.contains(&rc.id.as_str()),
                allowlist_patterns,
                compiled_allowlists: OnceLock::new(),
                rule_stopwords,
                skip_report: rc.skip_report,
            });
        }
    }

    // Add custom rules not covered by betterleaks
    add_custom_rules(&mut rules);

    rules
}

/// Additional rules specific to chub that are not in the betterleaks ruleset.
fn add_custom_rules(rules: &mut Vec<Rule>) {
    #[allow(clippy::type_complexity)]
    let custom: &[(&str, &str, &str, usize, &[&str], f64, bool)] = &[
        (
            "database-url",
            "Database Connection URL detected",
            r#"(?i)((?:postgres|mysql|mongodb|redis|amqp|mssql)://[^\s'"]{10,})"#,
            1,
            &[
                "://", "postgres", "mysql", "mongodb", "redis", "amqp", "mssql",
            ],
            0.0,
            false,
        ),
        (
            "generic-secret-assignment",
            "Generic secret assignment detected",
            r#"(?i)(?:api[_-]?key|api[_-]?secret|access[_-]?token|auth[_-]?token|secret[_-]?key|client[_-]?secret|app[_-]?secret|private[_-]?key)\s*[=:]\s*['"]([a-zA-Z0-9/+=_\-]{16,})['"]\s*"#,
            1,
            &[
                "api_key",
                "api-key",
                "api_secret",
                "api-secret",
                "access_token",
                "access-token",
                "auth_token",
                "auth-token",
                "secret_key",
                "secret-key",
                "client_secret",
                "client-secret",
                "app_secret",
                "app-secret",
                "private_key",
                "private-key",
            ],
            3.5,
            true,
        ),
        (
            "env-file-secret",
            "Environment file secret detected",
            r##"(?m)^[A-Z][A-Z0-9_]*(?:SECRET|TOKEN|KEY|PASSWORD|CREDENTIAL|API_KEY|APIKEY|AUTH)\s*=\s*['"]?([^\s'"#]{8,})['"]?"##,
            1,
            &[
                "secret",
                "token",
                "key",
                "password",
                "credential",
                "api_key",
                "apikey",
                "auth",
            ],
            3.5,
            true,
        ),
        (
            "bearer-token",
            "Bearer token detected",
            r#"(?i)(?:authorization|bearer)\s*[:=]\s*['"]?(?:Bearer\s+)?([a-zA-Z0-9._\-]{20,})['"]?"#,
            1,
            &["authorization", "bearer"],
            3.5,
            true,
        ),
        (
            "azure-storage-key",
            "Azure Storage Key detected",
            r#"(?i)(?:account.?key|storage.?key|azure.?key)\s*[=:]\s*['"]?([A-Za-z0-9+/]{86}==)['"]?"#,
            1,
            &["azure", "account_key", "storage_key"],
            4.0,
            false,
        ),
    ];

    for &(id, desc, pattern, secret_group, kws, entropy, generic) in custom {
        rules.push(Rule {
            id: id.to_string(),
            description: desc.to_string(),
            pattern: pattern.to_string(),
            compiled: OnceLock::new(),
            secret_group,
            keywords: kws.iter().map(|k| k.to_string()).collect(),
            min_entropy: entropy,
            generic,
            allowlist_patterns: Vec::new(),
            compiled_allowlists: OnceLock::new(),
            rule_stopwords: Vec::new(),
            skip_report: false,
        });
    }
}

// ---------------------------------------------------------------------------
// Shannon entropy
// ---------------------------------------------------------------------------

/// Calculate Shannon entropy (bits per character) for a string.
/// Public wrapper for use by the scan module.
pub fn shannon_entropy_pub(s: &str) -> f64 {
    shannon_entropy(s)
}

/// Calculate Shannon entropy (bits per character) for a string.
fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut freq: HashMap<u8, usize> = HashMap::new();
    for &b in s.as_bytes() {
        *freq.entry(b).or_insert(0) += 1;
    }
    let len = s.len() as f64;
    freq.values().fold(0.0_f64, |acc, &count| {
        let p = count as f64 / len;
        acc - p * p.log2()
    })
}

// ---------------------------------------------------------------------------
// Base64 decoding for encoded secrets
// ---------------------------------------------------------------------------

/// Attempt to decode base64-encoded chunks in text and return the decoded
/// content alongside the original. This catches secrets that have been
/// base64-encoded (e.g. in CI configs, docker secrets, etc.).
fn decode_base64_chunks(text: &str) -> Option<String> {
    // Match standalone base64 strings (at least 20 chars, padded)
    let re = match Regex::new(r"\b([A-Za-z0-9+/]{20,}={0,3})\b") {
        Ok(r) => r,
        Err(_) => return None,
    };

    let mut decoded_parts: Vec<String> = Vec::new();
    let mut found_any = false;

    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            let candidate = m.as_str();
            // Try standard base64 decode
            if let Ok(bytes) = base64_decode(candidate) {
                if let Ok(s) = std::str::from_utf8(&bytes) {
                    // Only keep if it looks like it could contain a secret
                    // (has some printable ASCII content, not binary garbage)
                    if s.len() >= 8 && s.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) {
                        decoded_parts.push(s.to_string());
                        found_any = true;
                    }
                }
            }
        }
    }

    if found_any {
        Some(decoded_parts.join("\n"))
    } else {
        None
    }
}

/// Simple base64 decoder (no external dependency needed).
fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    let input = input.trim_end_matches('=');
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for c in input.bytes() {
        let val = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err(()),
        };
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    Ok(output)
}

// ---------------------------------------------------------------------------
// Redaction engine
// ---------------------------------------------------------------------------

/// A single redaction finding.
#[derive(Debug, Clone)]
pub struct RedactFinding {
    /// Rule ID that matched.
    pub rule_id: String,
    /// Byte offset of the start of the redacted region in the original text.
    pub start: usize,
    /// Byte offset of the end (exclusive) of the redacted region.
    pub end: usize,
}

/// Result of redacting a piece of text.
#[derive(Debug, Clone)]
pub struct RedactResult {
    /// The text with secrets replaced by `[REDACTED:<rule-id>]`.
    pub text: String,
    /// Findings (one per redacted secret).
    pub findings: Vec<RedactFinding>,
}

/// Configuration for the redaction engine.
#[derive(Debug, Clone, Default)]
pub struct RedactConfig {
    /// If true, redaction is disabled entirely (pass-through).
    pub disabled: bool,
    /// Extra regex patterns to match (id, pattern string).
    pub extra_patterns: Vec<(String, String)>,
    /// Allowlist: regexes — if a matched secret matches any of these, skip it.
    pub allowlist_regexes: Vec<String>,
    /// Allowlist: file paths — if the source path matches, skip redaction entirely.
    pub allowlist_paths: Vec<String>,
    /// If true, skip base64 decoding pass (slightly faster, less thorough).
    pub skip_base64_decode: bool,
}

impl From<&crate::config::RedactionConfig> for RedactConfig {
    fn from(cfg: &crate::config::RedactionConfig) -> Self {
        Self {
            disabled: cfg.disabled,
            extra_patterns: cfg
                .extra_patterns
                .iter()
                .map(|p| (p.id.clone(), p.pattern.clone()))
                .collect(),
            allowlist_regexes: cfg.allowlist.clone(),
            allowlist_paths: Vec::new(),
            skip_base64_decode: false,
        }
    }
}

/// The redaction engine. Compile once and reuse across many texts.
pub struct Redactor {
    rules: Vec<Rule>,
    allowlist_regexes: Vec<Regex>,
    allowlist_path_regexes: Vec<Regex>,
    extra_stopwords: Vec<String>,
    disabled: bool,
    skip_base64_decode: bool,
    /// Aho-Corasick automaton over all unique lowercase keywords (all rules).
    /// Single O(n) pass over content finds which keywords are present.
    keyword_ac: AhoCorasick,
    /// `keyword_ac` pattern index → list of rule indices that need this keyword.
    keyword_to_rules: Vec<Vec<usize>>,
    /// Rule indices for rules that have no keywords (must always be evaluated).
    no_keyword_rules: Vec<usize>,
}

impl Redactor {
    /// Build the Aho-Corasick keyword automaton and keyword→rules index.
    ///
    /// Returns `(ac, keyword_to_rules, no_keyword_rules)`:
    /// - `ac`: automaton over all unique lowercase keywords
    /// - `keyword_to_rules[pattern_id]`: rule indices that require pattern_id's keyword
    /// - `no_keyword_rules`: rule indices with no keywords (always evaluated)
    fn build_keyword_index(rules: &[Rule]) -> (AhoCorasick, Vec<Vec<usize>>, Vec<usize>) {
        // Deduplicate keywords while preserving their AC pattern index → keyword string mapping
        let mut keyword_list: Vec<String> = Vec::new();
        let mut keyword_index: HashMap<String, usize> = HashMap::new();
        let mut keyword_to_rules: Vec<Vec<usize>> = Vec::new();
        let mut no_keyword_rules: Vec<usize> = Vec::new();

        for (rule_idx, rule) in rules.iter().enumerate() {
            if rule.keywords.is_empty() {
                no_keyword_rules.push(rule_idx);
                continue;
            }
            for kw in &rule.keywords {
                let ac_idx = *keyword_index.entry(kw.clone()).or_insert_with(|| {
                    let idx = keyword_list.len();
                    keyword_list.push(kw.clone());
                    keyword_to_rules.push(Vec::new());
                    idx
                });
                keyword_to_rules[ac_idx].push(rule_idx);
            }
        }

        // Use Standard match kind so that keywords which are prefixes of other
        // keywords (e.g. "api" vs "api-") are both reported when they share a
        // start position, ensuring every rule's regex gets a chance to run.
        let ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::Standard)
            .build(&keyword_list)
            .unwrap_or_else(|_| {
                // Fallback: empty AC (all rules will fall into no_keyword_rules path)
                AhoCorasick::new(Vec::<String>::new()).unwrap()
            });

        (ac, keyword_to_rules, no_keyword_rules)
    }

    /// Create a new redactor with default built-in rules.
    pub fn new() -> Self {
        let config = load_default_config();
        let allowlist_regexes = config
            .allowlist
            .regexes
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        let extra_stopwords = config.allowlist.stopwords.clone();
        let rules = built_in_rules();
        let (keyword_ac, keyword_to_rules, no_keyword_rules) = Self::build_keyword_index(&rules);
        Self {
            rules,
            allowlist_regexes,
            allowlist_path_regexes: Vec::new(),
            extra_stopwords,
            disabled: false,
            skip_base64_decode: false,
            keyword_ac,
            keyword_to_rules,
            no_keyword_rules,
        }
    }

    /// Create a redactor from configuration.
    pub fn from_config(config: &RedactConfig) -> Self {
        let default_config = load_default_config();
        let mut rules = built_in_rules();

        // Add extra patterns from config
        for (id, pattern) in &config.extra_patterns {
            rules.push(Rule {
                id: id.clone(),
                description: format!("{} detected", id),
                pattern: pattern.clone(),
                compiled: OnceLock::new(),
                secret_group: 0,
                keywords: Vec::new(),
                min_entropy: 0.0,
                generic: false,
                allowlist_patterns: Vec::new(),
                compiled_allowlists: OnceLock::new(),
                rule_stopwords: Vec::new(),
                skip_report: false,
            });
        }

        // Merge default + user allowlist regexes
        let mut allowlist_regexes: Vec<Regex> = default_config
            .allowlist
            .regexes
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        allowlist_regexes.extend(
            config
                .allowlist_regexes
                .iter()
                .filter_map(|p| Regex::new(p).ok()),
        );

        let allowlist_path_regexes = config
            .allowlist_paths
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        let extra_stopwords = default_config.allowlist.stopwords.clone();
        let (keyword_ac, keyword_to_rules, no_keyword_rules) = Self::build_keyword_index(&rules);

        Self {
            rules,
            allowlist_regexes,
            allowlist_path_regexes,
            extra_stopwords,
            disabled: config.disabled,
            skip_base64_decode: config.skip_base64_decode,
            keyword_ac,
            keyword_to_rules,
            no_keyword_rules,
        }
    }

    /// Check if a file path is allowlisted (skip redaction entirely).
    pub fn is_path_allowlisted(&self, path: &str) -> bool {
        self.allowlist_path_regexes
            .iter()
            .any(|re| re.is_match(path))
    }

    /// Redact secrets from the given text.
    pub fn redact(&self, text: &str) -> RedactResult {
        if self.disabled || text.is_empty() {
            return RedactResult {
                text: text.to_string(),
                findings: Vec::new(),
            };
        }

        // Scan the original text
        let mut findings = self.scan_text(text);

        // Also scan base64-decoded chunks for encoded secrets
        if !self.skip_base64_decode {
            if let Some(decoded) = decode_base64_chunks(text) {
                let decoded_findings = self.scan_text(&decoded);
                if !decoded_findings.is_empty() {
                    // For encoded secrets, we need to find the base64 chunk in original text
                    // and redact that instead. We report them as additional findings.
                    let b64_re = get_base64_chunk_regex();
                    {
                        for m in b64_re.find_iter(text) {
                            let chunk = m.as_str();
                            if let Ok(bytes) = base64_decode(chunk) {
                                if let Ok(decoded_str) = std::str::from_utf8(&bytes) {
                                    // Check if any decoded finding came from this chunk
                                    for df in &decoded_findings {
                                        if decoded_str
                                            .contains(text.get(df.0..df.1).unwrap_or_default())
                                            || decoded_str.len() >= 8
                                        {
                                            // Check: does the decoded chunk actually contain
                                            // a secret? Re-scan just this chunk.
                                            let chunk_findings = self.scan_text(decoded_str);
                                            if !chunk_findings.is_empty() {
                                                findings.push((
                                                    m.start(),
                                                    m.end(),
                                                    chunk_findings[0].2,
                                                    false,
                                                ));
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if findings.is_empty() {
            return RedactResult {
                text: text.to_string(),
                findings: Vec::new(),
            };
        }

        // Remove generic findings that overlap with any specific finding.
        // This handles the case where generic-api-key captures "key: sk_test_..."
        // at [0..37] while stripe-access-token captures just "sk_test_..." at [5..37].
        let specific: Vec<(usize, usize)> = findings
            .iter()
            .filter(|f| !f.3)
            .map(|f| (f.0, f.1))
            .collect();
        if !specific.is_empty() {
            findings.retain(|f| {
                if !f.3 {
                    return true;
                } // keep all specific
                  // Drop generic finding if any specific finding overlaps with it
                !specific.iter().any(|s| f.0 < s.1 && s.0 < f.1)
            });
        }

        // Sort: by start offset, then specific before generic, then longest match first
        findings.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.3.cmp(&b.3)) // false (specific) < true (generic)
                .then(b.1.cmp(&a.1)) // longer match first
        });

        // Build redacted text, merging overlapping ranges
        let mut result = String::with_capacity(text.len());
        let mut result_findings = Vec::new();
        let mut cursor = 0usize;

        for (start, end, rule_id, _generic) in &findings {
            if *start < cursor {
                // Overlapping with a previous redaction — skip
                continue;
            }
            result.push_str(&text[cursor..*start]);
            result.push_str(&format!("[REDACTED:{}]", rule_id));
            result_findings.push(RedactFinding {
                rule_id: rule_id.to_string(),
                start: *start,
                end: *end,
            });
            cursor = *end;
        }
        result.push_str(&text[cursor..]);

        RedactResult {
            text: result,
            findings: result_findings,
        }
    }

    /// Look up the human-readable description for a rule ID.
    pub fn rule_description(&self, rule_id: &str) -> String {
        self.rules
            .iter()
            .find(|r| r.id == rule_id)
            .and_then(|r| {
                if r.description.is_empty() {
                    None
                } else {
                    Some(r.description.clone())
                }
            })
            .unwrap_or_else(|| format!("{} detected", rule_id))
    }

    /// Internal: scan text and collect raw findings.
    ///
    /// Uses Aho-Corasick for a single O(n) keyword pass over the text to
    /// identify which rules are candidates, then only runs regexes for those.
    /// This replaces the previous O(rules × text_length) keyword loop.
    fn scan_text<'a>(&'a self, text: &str) -> Vec<(usize, usize, &'a str, bool)> {
        let mut all_findings: Vec<(usize, usize, &str, bool)> = Vec::new();

        // Phase 1: single Aho-Corasick pass to find which rules are candidates.
        // Build a bitset of rule indices to check.
        let mut rule_candidates = vec![false; self.rules.len()];

        for m in self.keyword_ac.find_overlapping_iter(text) {
            for &rule_idx in &self.keyword_to_rules[m.pattern().as_usize()] {
                rule_candidates[rule_idx] = true;
            }
        }
        // Rules with no keywords always run.
        for &rule_idx in &self.no_keyword_rules {
            rule_candidates[rule_idx] = true;
        }

        // Phase 2: run regexes only for candidate rules.
        for (rule_idx, is_candidate) in rule_candidates.iter().enumerate() {
            if !is_candidate {
                continue;
            }
            let rule = &self.rules[rule_idx];
            let regex = match rule.regex() {
                Some(r) => r,
                None => continue,
            };
            for cap in regex.captures_iter(text) {
                let m = match cap.get(rule.secret_group).or_else(|| cap.get(0)) {
                    Some(m) => m,
                    None => continue,
                };
                let secret = m.as_str();

                // Stopword check — skip if it looks like a placeholder/example
                if contains_stopword(secret) {
                    continue;
                }
                // Extra stopwords from TOML config (global)
                if !self.extra_stopwords.is_empty() {
                    let lower_secret = secret.to_lowercase();
                    if self
                        .extra_stopwords
                        .iter()
                        .any(|sw| lower_secret.contains(sw.as_str()))
                    {
                        continue;
                    }
                }
                // Per-rule stopwords
                if !rule.rule_stopwords.is_empty() {
                    let lower_secret = secret.to_lowercase();
                    if rule
                        .rule_stopwords
                        .iter()
                        .any(|sw| lower_secret.contains(sw.as_str()))
                    {
                        continue;
                    }
                }

                // Entropy check
                if rule.min_entropy > 0.0 && shannon_entropy(secret) < rule.min_entropy {
                    continue;
                }

                // Global allowlist check
                if self.allowlist_regexes.iter().any(|al| al.is_match(secret)) {
                    continue;
                }

                // Per-rule allowlist check
                if rule.allowlists().iter().any(|al| al.is_match(secret)) {
                    continue;
                }

                all_findings.push((m.start(), m.end(), rule.id.as_str(), rule.generic));
            }
        }

        all_findings
    }

    /// Convenience: redact and return just the text.
    pub fn redact_text(&self, text: &str) -> String {
        self.redact(text).text
    }

    /// Return the number of loaded rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn redactor() -> Redactor {
        Redactor::new()
    }

    // --- TOML loading ---

    #[test]
    fn toml_rules_load_correctly() {
        let config = load_default_config();
        assert!(
            config.rules.len() >= 250,
            "TOML should have 250+ rules, got {}",
            config.rules.len()
        );
    }

    #[test]
    fn all_rules_compile() {
        let r = redactor();
        let mut failed = Vec::new();
        for rule in &r.rules {
            if rule.regex().is_none() {
                let fixed = fixup_regex_pattern(&rule.pattern);
                let err = Regex::new(&fixed)
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_default();
                failed.push(format!("{}: {} | err: {}", rule.id, fixed, err));
            }
        }
        assert!(
            failed.is_empty(),
            "{} rules failed to compile:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    // --- Entropy ---

    #[test]
    fn entropy_empty_string() {
        assert_eq!(shannon_entropy(""), 0.0);
    }

    #[test]
    fn entropy_single_char() {
        assert_eq!(shannon_entropy("aaaa"), 0.0);
    }

    #[test]
    fn entropy_high_for_random() {
        let e = shannon_entropy("aB3kL9mNpQ2rStUvWxYz");
        assert!(e > 3.5, "entropy {e} should be > 3.5");
    }

    #[test]
    fn entropy_low_for_repeated() {
        let e = shannon_entropy("aaaaabbbbb");
        assert!(e < 1.5, "entropy {e} should be < 1.5");
    }

    // --- Stopwords ---

    #[test]
    fn stopword_rejects_placeholder() {
        assert!(contains_stopword("your_api_key_here"));
        assert!(contains_stopword("PLACEHOLDER_VALUE"));
        assert!(contains_stopword("changeme_please"));
    }

    #[test]
    fn stopword_allows_real_secret() {
        assert!(!contains_stopword("sk-ant-api03-aB3kL9mNpQ2rStUv"));
        assert!(!contains_stopword("AKIAK4JM7NR2PX6SWT3B"));
    }

    // --- Base64 decoding ---

    #[test]
    fn base64_decode_simple() {
        let decoded = base64_decode("aGVsbG8gd29ybGQ=").unwrap();
        assert_eq!(std::str::from_utf8(&decoded).unwrap(), "hello world");
    }

    #[test]
    fn base64_chunks_detected() {
        let encoded = "c2tfbGl2ZV9hQmNEZUZnSGlKa0xtTm9QcVJzVHVWd1g=";
        let decoded = decode_base64_chunks(&format!("config: {encoded}"));
        assert!(decoded.is_some());
        assert!(decoded.unwrap().contains("sk_live_"));
    }

    // --- AWS ---

    #[test]
    fn detect_aws_access_key() {
        let r = redactor();
        // Use a realistic non-EXAMPLE AWS key (AKIA + 16 chars from [A-Z2-7])
        let text = "aws_access_key_id = AKIAK4JM7NR2PX6SWT3B";
        let result = r.redact(text);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].rule_id, "aws-access-token");
        assert!(result.text.contains("[REDACTED:aws-access-token]"));
    }

    #[test]
    fn detect_aws_secret_key() {
        let r = redactor();
        // Note: betterleaks marks aws-secret-access-key as skipReport=true (composite-only).
        // Our generic/custom rules should still catch it via env-file-secret or generic patterns.
        let text = "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMIK7MDENGbPxRfiCYk4Jm8nR2pX";
        let result = r.redact(text);
        assert!(
            !result.findings.is_empty(),
            "should detect AWS secret: {:?}",
            result.findings
        );
    }

    // --- GitHub ---
    // Note: test tokens use high-entropy random chars to avoid betterleaks stopwords

    #[test]
    fn detect_github_pat() {
        let r = redactor();
        //                         123456789012345678901234567890123456  (36 chars)
        let text = "token: ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "github-pat"),
            "should detect github-pat: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_github_fine_grained() {
        let r = redactor();
        //                                                                                      82 chars
        let suffix =
            "k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4Jm8nR2pX";
        assert_eq!(suffix.len(), 82, "suffix must be exactly 82 chars");
        let token = format!("github_pat_{suffix}");
        let text = format!("GITHUB_TOKEN={token}");
        let result = r.redact(&text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "github-fine-grained-pat"),
            "should detect github-fine-grained-pat: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_github_app_token() {
        let r = redactor();
        //                           123456789012345678901234567890123456  (36 chars)
        let text = "header: ghs_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "github-app-token"),
            "should detect github-app-token: {:?}",
            result.findings
        );
    }

    // --- GitLab ---

    #[test]
    fn detect_gitlab_pat() {
        let r = redactor();
        // glpat- + 20 word chars
        let text = "GITLAB_TOKEN=glpat-k4Jm8nR2pX6sW9vB3fH7";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "gitlab-pat"),
            "should detect gitlab-pat: {:?}",
            result.findings
        );
    }

    // --- AI / LLM ---

    #[test]
    fn detect_anthropic_key() {
        let r = redactor();
        // sk-ant-api03- + 93 chars of [a-zA-Z0-9_-] + literal "AA" (95 chars total)
        let suffix = "k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2pX6sW9vB3fH7aT1qY5uExAA";
        assert_eq!(suffix.len(), 95);
        let text = format!("KEY=sk-ant-api03-{}", suffix);
        let result = r.redact(&text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "anthropic-api-key"),
            "should detect Anthropic key: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_openai_key() {
        let r = redactor();
        // betterleaks: sk-proj- + 20 chars + T3BlbkFJ + 20 chars
        let text = "OPENAI_API_KEY=sk-proj-k4Jm8nR2pX6sW9vB3fH7T3BlbkFJk4Jm8nR2pX6sW9vB3fH7";
        let result = r.redact(&text);
        assert!(
            result.findings.iter().any(|f| f.rule_id.contains("openai")),
            "should detect OpenAI key: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_huggingface_token() {
        let r = redactor();
        // betterleaks: hf_ + 34 lowercase alpha chars
        let text = "HF_TOKEN=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";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id.contains("huggingface")),
            "should detect HuggingFace token: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_openrouter_key() {
        let r = redactor();
        let text = "OPENROUTER_KEY=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";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "openrouter-api-key"),
            "should detect OpenRouter key: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_xai_key() {
        let r = redactor();
        // xai- + 70-120 chars
        let text =
            "key: xai-k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "xai-api-key"),
            "should detect xAI key: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_cerebras_key() {
        let r = redactor();
        let text = "CEREBRAS_KEY=csk-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "cerebras-api-key"),
            "should detect Cerebras key: {:?}",
            result.findings
        );
    }

    // --- Payment ---

    #[test]
    fn detect_stripe_key() {
        let r = redactor();
        let text = "STRIPE_KEY=sk_live_k4Jm8nR2pX6sW9vB3fH7aT1q";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id.contains("stripe")),
            "should detect Stripe key: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_stripe_test_key() {
        let r = redactor();
        let text = "key: sk_test_k4Jm8nR2pX6sW9vB3fH7aT1q";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id.contains("stripe")),
            "should detect Stripe test key: {:?}",
            result.findings
        );
    }

    // --- Communication ---

    #[test]
    fn detect_slack_bot_token() {
        let r = redactor();
        let text = "SLACK_TOKEN=xoxb-1234567890-1234567890-k4Jm8nR2pX6sW9vB3fH7aT1q";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "slack-bot-token"),
            "should detect Slack bot token: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_slack_webhook() {
        let r = redactor();
        // betterleaks: hooks.slack.com/(services|workflows|triggers)/ + 43-56 base64 chars
        let text =
            "url: https://hooks.slack.com/services/T0k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4JmR";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id.contains("slack")),
            "should detect Slack webhook: {:?}",
            result.findings
        );
    }

    // --- Email ---

    #[test]
    fn detect_sendgrid_key() {
        let r = redactor();
        // betterleaks: SG. + 66 chars
        //                     123456789012345678901234567890123456789012345678901234567890123456  (66 chars)
        let text = "key: SG.k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2k4Jm8nR2pX6sW9vB3fH7aT1qnR2pX6";
        let result = r.redact(&text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id.contains("sendgrid")),
            "should detect SendGrid key: {:?}",
            result.findings
        );
    }

    // --- Auth / Crypto ---

    #[test]
    fn detect_jwt() {
        let r = redactor();
        let text =
            "token: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "jwt"),
            "should detect JWT: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_private_key() {
        let r = redactor();
        // betterleaks requires content between BEGIN/END markers (64+ chars)
        let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEAk4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLk4Jm8nR2pX6sW9vB3fH7\n-----END RSA PRIVATE KEY-----";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "private-key"),
            "should detect private key: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_ec_private_key() {
        let r = redactor();
        let text = "-----BEGIN EC PRIVATE KEY-----\nMIIEowIBAAKCAQEAk4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLk4Jm8nR2pX6sW9vB3fH7\n-----END EC PRIVATE KEY-----";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "private-key"),
            "should detect EC private key: {:?}",
            result.findings
        );
    }

    // --- Database ---

    #[test]
    fn detect_postgres_url() {
        let r = redactor();
        let text = "DATABASE_URL=postgres://user:secretpass@localhost:5432/mydb";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "database-url"),
            "should detect database URL: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_mongodb_url() {
        let r = redactor();
        let text = "MONGO_URI=mongodb://admin:password123@mongo.example.com:27017/db";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "database-url"),
            "should detect MongoDB URL: {:?}",
            result.findings
        );
    }

    // --- DevOps / Infrastructure ---

    #[test]
    fn detect_digitalocean_pat() {
        let r = redactor();
        let text =
            "DO_TOKEN=dop_v1_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "digitalocean-pat"),
            "should detect DigitalOcean PAT: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_npm_token() {
        let r = redactor();
        let text = "NPM_TOKEN=npm_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "npm-access-token"),
            "should detect npm token: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_pulumi_token() {
        let r = redactor();
        let text = "PULUMI_ACCESS_TOKEN=pul-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "pulumi-api-token"),
            "should detect Pulumi token: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_linear_api_key() {
        let r = redactor();
        let text = "LINEAR_KEY=lin_api_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "linear-api-key"),
            "should detect Linear API key: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_shopify_token() {
        let r = redactor();
        let text = "SHOPIFY=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";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "shopify-access-token"),
            "should detect Shopify token: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_grafana_service_token() {
        let r = redactor();
        // glsa_ + 32 alnum + _ + 8 hex
        let text = "GRAFANA=glsa_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cDx_a1b2c3d4";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "grafana-service-account-token"),
            "should detect Grafana SA token: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_databricks_token() {
        let r = redactor();
        let text = "DB_TOKEN=dapia1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6";
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id == "databricks-api-token"),
            "should detect Databricks token: {:?}",
            result.findings
        );
    }

    // --- Generic patterns ---

    #[test]
    fn detect_generic_api_key_assignment() {
        let r = redactor();
        let text = r#"api_key = "k4Jm8nR2pX6sW9vB3fH7aT1q""#;
        let result = r.redact(text);
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.rule_id.contains("generic") || f.rule_id.contains("api-key")),
            "should detect generic secret: {:?}",
            result.findings
        );
    }

    #[test]
    fn detect_env_file_secret() {
        let r = redactor();
        let text = "MY_API_KEY=sk_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLk4Jm";
        let result = r.redact(text);
        assert!(
            !result.findings.is_empty(),
            "should detect env-style secret"
        );
    }

    #[test]
    fn detect_bearer_token() {
        let r = redactor();
        let text = r#"Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"#;
        let result = r.redact(text);
        assert!(!result.findings.is_empty(), "should detect bearer token");
    }

    // --- False positive reduction ---

    #[test]
    fn no_false_positive_placeholder() {
        let r = redactor();
        let text = "api_key = 'placeholder'";
        let result = r.redact(text);
        assert!(
            result.findings.is_empty(),
            "placeholder should not be flagged: {:?}",
            result.findings
        );
    }

    #[test]
    fn no_false_positive_example() {
        let r = redactor();
        let text = "api_key = 'your_api_key_here'";
        let result = r.redact(text);
        assert!(
            result.findings.is_empty(),
            "example value should not be flagged: {:?}",
            result.findings
        );
    }

    #[test]
    fn no_false_positive_normal_code() {
        let r = redactor();
        let text = r#"
fn main() {
    let config = Config::new();
    let result = process_data(&config);
    println!("Done: {}", result);
}
"#;
        let result = r.redact(text);
        assert!(
            result.findings.is_empty(),
            "normal code should not be flagged: {:?}",
            result.findings
        );
    }

    #[test]
    fn no_false_positive_cargo_checksum() {
        let r = redactor();
        let text =
            r#"checksum = "e3148f5046208a5d56bcfc03053e3ca6334e51da8dfb19b6cdc8b306fae3283e""#;
        let result = r.redact(text);
        assert!(
            result.findings.is_empty(),
            "Cargo checksum should not be flagged: {:?}",
            result.findings
        );
    }

    #[test]
    fn no_false_positive_stopword_env_template() {
        let r = redactor();
        let text = "SECRET_KEY=${MY_SECRET_VALUE_FROM_VAULT}";
        let result = r.redact(text);
        assert!(
            result.findings.is_empty(),
            "template variable should not be flagged: {:?}",
            result.findings
        );
    }

    #[test]
    fn no_false_positive_changeme() {
        let r = redactor();
        let text = r#"api_key = "change_me_please_update""#;
        let result = r.redact(text);
        assert!(
            result.findings.is_empty(),
            "changeme should not be flagged: {:?}",
            result.findings
        );
    }

    // --- Allowlist ---

    #[test]
    fn allowlist_regex_no_match_still_detects() {
        let config = RedactConfig {
            allowlist_regexes: vec!["^XXXXX".to_string()],
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let text = "key: AKIAK4JM7NR2PX6SWT3B";
        let result = r.redact(text);
        assert!(!result.findings.is_empty(), "should still detect the key");
    }

    #[test]
    fn allowlist_regex_matches_secret() {
        let config = RedactConfig {
            allowlist_regexes: vec!["K4JM7".to_string()],
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let text = "key: AKIAK4JM7NR2PX6SWT3B";
        let result = r.redact(text);
        assert!(
            result.findings.is_empty(),
            "allowlisted secret should be skipped: {:?}",
            result.findings
        );
    }

    #[test]
    fn allowlist_path_skips_file() {
        let config = RedactConfig {
            allowlist_paths: vec![r"\.test\.js$".to_string(), r"fixtures/".to_string()],
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        assert!(r.is_path_allowlisted("src/auth.test.js"));
        assert!(r.is_path_allowlisted("fixtures/secrets.json"));
        assert!(!r.is_path_allowlisted("src/auth.rs"));
    }

    // --- Disabled ---

    #[test]
    fn disabled_returns_unchanged() {
        let config = RedactConfig {
            disabled: true,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let text = "ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
        let result = r.redact(text);
        assert_eq!(result.text, text);
        assert!(result.findings.is_empty());
    }

    // --- Extra patterns ---

    #[test]
    fn extra_pattern_from_config() {
        let config = RedactConfig {
            extra_patterns: vec![(
                "custom-secret".to_string(),
                r"MYSECRET_[a-z]{10}".to_string(),
            )],
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let text = "value: MYSECRET_kjmrnpxswv";
        let result = r.redact(text);
        assert!(
            result.findings.iter().any(|f| f.rule_id == "custom-secret"),
            "custom rule should match: {:?}",
            result.findings
        );
    }

    // --- Multiple secrets ---

    #[test]
    fn multiple_secrets_in_one_text() {
        let r = redactor();
        // AWS key (non-EXAMPLE) + github pat (36 chars after ghp_)
        let text = "AWS_KEY=AKIAK4JM7NR2PX6SWT3B, GITHUB=ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2";
        let result = r.redact(text);
        assert!(
            result.findings.len() >= 2,
            "should find at least 2 secrets: {:?}",
            result.findings
        );
        assert!(result.text.contains("[REDACTED:aws-access-token]"));
        assert!(result.text.contains("[REDACTED:github-pat]"));
    }

    // --- Overlapping matches ---

    #[test]
    fn overlapping_matches_handled() {
        let r = redactor();
        let text = "AKIAK4JM7NR2PX6SWT3B";
        let result = r.redact(text);
        assert_eq!(result.findings.len(), 1);
    }

    // --- Empty / no-op ---

    #[test]
    fn empty_text_returns_empty() {
        let r = redactor();
        let result = r.redact("");
        assert_eq!(result.text, "");
        assert!(result.findings.is_empty());
    }

    #[test]
    fn clean_text_returns_unchanged() {
        let r = redactor();
        let text = "Hello world! This is normal text without any secrets.";
        let result = r.redact(text);
        assert_eq!(result.text, text);
        assert!(result.findings.is_empty());
    }

    // --- Rule count ---

    #[test]
    fn has_built_in_rules() {
        let r = redactor();
        assert!(
            r.rule_count() >= 250,
            "should have at least 250 rules (261 betterleaks + custom), got {}",
            r.rule_count()
        );
    }

    // --- Redact multi-line transcript excerpt ---

    #[test]
    fn redact_transcript_excerpt() {
        let r = redactor();
        let text = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"export STRIPE_KEY=sk_live_k4Jm8nR2pX6sW9vB && curl https://api.stripe.com"}}]}}"#;
        let result = r.redact(text);
        assert!(
            !result.findings.is_empty(),
            "should find secrets in transcript JSON"
        );
        assert!(!result.text.contains("sk_live_"));
    }
}
