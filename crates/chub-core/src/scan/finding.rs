//! Finding — a detected secret with location and metadata.

use serde::{Deserialize, Serialize};

/// A single detected secret finding, compatible with gitleaks/betterleaks output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Finding {
    /// Rule ID that matched (e.g. "aws-access-token").
    #[serde(rename = "RuleID")]
    pub rule_id: String,
    /// Human-readable description of the rule.
    pub description: String,
    /// Starting line number (1-based).
    pub start_line: usize,
    /// Ending line number (1-based).
    pub end_line: usize,
    /// Starting column number (1-based).
    pub start_column: usize,
    /// Ending column number (1-based).
    pub end_column: usize,
    /// The full regex match.
    #[serde(rename = "Match")]
    pub match_text: String,
    /// The extracted secret value (may be redacted in output).
    pub secret: String,
    /// Source file path.
    pub file: String,
    /// Symlink target (if file is a symlink).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub symlink_file: String,
    /// Git commit hash (empty for directory/stdin scans).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub commit: String,
    /// Shannon entropy of the secret.
    pub entropy: f64,
    /// Git commit author.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    /// Git commit author email.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub email: String,
    /// Git commit date.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub date: String,
    /// Git commit message.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
    /// Tags associated with the rule.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Unique fingerprint for deduplication.
    pub fingerprint: String,
    /// Validation status from CEL validation (present only when `--validate` is
    /// enabled).  One of: `"valid"`, `"invalid"`, `"revoked"`, `"unknown"`,
    /// `"error"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_status: Option<String>,
    /// Human-readable reason accompanying a non-`"valid"` validation status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_reason: Option<String>,
}

impl Finding {
    /// Compute a fingerprint for deduplication.
    /// Format: sha256(rule_id:file:secret:commit)[..16]
    pub fn compute_fingerprint(rule_id: &str, file: &str, secret: &str, commit: &str) -> String {
        use sha2::{Digest, Sha256};
        let input = format!("{}:{}:{}:{}", rule_id, file, secret, commit);
        let hash = Sha256::digest(input.as_bytes());
        hex::encode(&hash[..8])
    }

    /// Redact the secret in this finding (replace middle with asterisks).
    pub fn redacted(&self, percent: u8) -> Self {
        let mut f = self.clone();
        if percent == 0 || self.secret.is_empty() {
            return f;
        }
        let len = self.secret.len();
        let redact_count = (len as f64 * percent as f64 / 100.0).ceil() as usize;
        let keep_start = (len - redact_count) / 2;
        let keep_end = len - redact_count - keep_start;
        let mut redacted = String::with_capacity(len);
        redacted.push_str(&self.secret[..keep_start]);
        for _ in 0..redact_count {
            redacted.push('*');
        }
        redacted.push_str(&self.secret[len - keep_end..]);
        f.secret = redacted;
        // Also redact in match_text
        f.match_text = f.match_text.replace(&self.secret, &f.secret);
        f
    }
}

/// Hex encoding helper (avoid adding a dep just for this).
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_deterministic() {
        let fp1 = Finding::compute_fingerprint("aws", "main.py", "AKIA123", "abc123");
        let fp2 = Finding::compute_fingerprint("aws", "main.py", "AKIA123", "abc123");
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn fingerprint_differs_on_different_input() {
        let fp1 = Finding::compute_fingerprint("aws", "main.py", "AKIA123", "abc");
        let fp2 = Finding::compute_fingerprint("aws", "main.py", "AKIA456", "abc");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn redact_100_percent() {
        let f = Finding {
            rule_id: "test".into(),
            description: String::new(),
            start_line: 1,
            end_line: 1,
            start_column: 1,
            end_column: 10,
            match_text: "key=secret123".into(),
            secret: "secret123".into(),
            file: "test.txt".into(),
            symlink_file: String::new(),
            commit: String::new(),
            entropy: 3.0,
            author: String::new(),
            email: String::new(),
            date: String::new(),
            message: String::new(),
            tags: vec![],
            fingerprint: "abc".into(),
            validation_status: None,
            validation_reason: None,
        };
        let redacted = f.redacted(100);
        assert!(!redacted.secret.contains("secret123"));
        assert!(redacted.secret.contains('*'));
    }

    #[test]
    fn redact_0_percent_unchanged() {
        let f = Finding {
            rule_id: "test".into(),
            description: String::new(),
            start_line: 1,
            end_line: 1,
            start_column: 1,
            end_column: 10,
            match_text: "secret123".into(),
            secret: "secret123".into(),
            file: "test.txt".into(),
            symlink_file: String::new(),
            commit: String::new(),
            entropy: 3.0,
            author: String::new(),
            email: String::new(),
            date: String::new(),
            message: String::new(),
            tags: vec![],
            fingerprint: "abc".into(),
            validation_status: None,
            validation_reason: None,
        };
        let redacted = f.redacted(0);
        assert_eq!(redacted.secret, "secret123");
    }
}
