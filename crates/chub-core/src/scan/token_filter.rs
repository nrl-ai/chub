//! BPE-based token efficiency filter — port of betterleaks `failsTokenEfficiencyFilter`.
//!
//! Algorithm: compute `len(string) / len(bpe_tokens)`. High ratio → natural language
//! (false positive). Low ratio → random/rare string → likely a real secret.
//!
//! Reference: betterleaks detect/detect.go `failsTokenEfficiencyFilter`, commit 8d86729.

use std::collections::HashSet;
use std::sync::OnceLock;

/// Word list extracted from betterleaks/words/words.go (auto-generated from wordfreq, Zipf ≥ 2.5).
static WORDS_TXT: &str = include_str!("words.txt");

static WORD_SET: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn word_set() -> &'static HashSet<&'static str> {
    WORD_SET.get_or_init(|| WORDS_TXT.lines().filter(|l| !l.is_empty()).collect())
}

/// Returns true if `text` contains a dictionary word of at least `min_len` chars as a substring.
/// Mirrors `words.HasMatchInList(word, minLen)` from betterleaks.
fn has_word_match(text: &str, min_len: usize) -> bool {
    let text_lower = text.to_lowercase();
    let bytes = text_lower.as_bytes();
    let n = bytes.len();
    if n < min_len {
        return false;
    }
    let set = word_set();
    for start in 0..=(n - min_len) {
        for end in (start + min_len)..=n {
            // All words in the list are ASCII so from_utf8 never fails in practice
            if let Ok(sub) = std::str::from_utf8(&bytes[start..end]) {
                if set.contains(sub) {
                    return true;
                }
            }
        }
    }
    false
}

static BPE: OnceLock<Option<tiktoken_rs::CoreBPE>> = OnceLock::new();

fn get_bpe() -> Option<&'static tiktoken_rs::CoreBPE> {
    BPE.get_or_init(|| tiktoken_rs::cl100k_base().ok()).as_ref()
}

/// Returns `true` if `secret` should be filtered out (looks like natural language, not a real
/// secret). Returning `true` means the finding is a false positive and should be dropped.
///
/// Mirrors betterleaks `failsTokenEfficiencyFilter` exactly:
/// - Reject if contains a 5+ char dictionary word (always natural language)
/// - Otherwise compute `len / bpe_tokens`. If ≥ threshold it looks like natural language.
///   Threshold is 2.5 normally, 2.1 for short secrets that contain a 4-char word.
pub fn fails_token_efficiency_filter(secret: &str) -> bool {
    let bpe = match get_bpe() {
        Some(b) => b,
        None => return false, // tokenizer unavailable — skip filter, don't suppress findings
    };

    // For short secrets containing newlines, strip them so word detection works correctly
    let analyzed: std::borrow::Cow<str> = if secret.len() < 20 && secret.contains(['\n', '\r']) {
        std::borrow::Cow::Owned(secret.replace(['\n', '\r'], ""))
    } else {
        std::borrow::Cow::Borrowed(secret)
    };

    let tokens = bpe.encode_ordinary(&analyzed);
    if tokens.is_empty() {
        return false;
    }

    // Any 5+ char dictionary word → natural language → false positive
    if has_word_match(&analyzed, 5) {
        return true;
    }

    let threshold = if analyzed.len() < 12 {
        // Stricter for short strings, but only if they also contain a 4-char word
        if has_word_match(&analyzed, 4) {
            2.1_f64
        } else {
            2.5_f64
        }
    } else {
        2.5_f64
    };

    let efficiency = analyzed.len() as f64 / tokens.len() as f64;
    efficiency >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_set_loads() {
        assert!(word_set().len() > 30_000, "expected ~33k words");
    }

    #[test]
    fn has_word_match_basics() {
        assert!(has_word_match("password", 5)); // "password" itself is in the word list
        assert!(has_word_match("mysecretpassword", 5)); // contains "secret", "password"
        assert!(has_word_match("example_secret_here", 5)); // contains "example", "secret"
                                                           // Short passwords / random strings without long dictionary words
        assert!(!has_word_match("a1b2c3d4e5f6", 5));
        assert!(!has_word_match("xK9mP2", 4)); // too short and not a word
    }

    #[test]
    fn filter_rejects_natural_language() {
        // Contains common English word → should be filtered (natural language)
        assert!(fails_token_efficiency_filter("examplepassword"));
        assert!(fails_token_efficiency_filter("letmein123"));
        assert!(fails_token_efficiency_filter("mysecretpassword"));
    }

    #[test]
    fn filter_keeps_random_secrets() {
        // High-entropy random strings → should NOT be filtered (are real secrets)
        assert!(!fails_token_efficiency_filter("aK9mP2xL5nQ8wR3z"));
        assert!(!fails_token_efficiency_filter("4a3f8b2c1d9e7f6a5b0c3d2e1f"));
        // UUID-like
        assert!(!fails_token_efficiency_filter(
            "a1b2c3d4e5f67890abcdef1234567890"
        ));
    }

    #[test]
    fn filter_empty_string_is_kept() {
        // Empty string has no tokens; filter should not suppress
        assert!(!fails_token_efficiency_filter(""));
    }

    #[test]
    fn has_word_match_min_len_four() {
        // 4-char words should be found when min_len == 4
        assert!(has_word_match("test", 4));
        assert!(has_word_match("keytest", 4));
        // But not when string is shorter than min_len
        assert!(!has_word_match("abc", 4));
    }

    #[test]
    fn has_word_match_case_insensitive() {
        // Word list is lowercase; input should be lowercased before matching
        assert!(has_word_match("PASSWORD", 5));
        assert!(has_word_match("MySecret", 5));
    }

    #[test]
    fn filter_api_key_format_kept() {
        // Typical API key patterns (no real words, high entropy) → kept
        assert!(!fails_token_efficiency_filter(
            "sk-proj-xK9mP2xL5nQ8wR3zAbCdEfGh"
        ));
        assert!(!fails_token_efficiency_filter(
            "ghp_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789"
        ));
    }

    #[test]
    fn filter_newline_stripped_for_short_secrets() {
        // Short secrets with embedded newlines — newlines stripped before analysis
        // "pass\nword" → "password" which contains "password" (5+ chars) → filtered
        assert!(fails_token_efficiency_filter("pass\nword"));
    }
}
