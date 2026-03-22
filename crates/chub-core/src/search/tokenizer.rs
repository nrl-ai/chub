use std::collections::HashSet;
use std::sync::LazyLock;

/// Stop words — must match the JS implementation exactly.
static STOP_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "have", "in", "is",
        "it", "its", "of", "on", "or", "that", "the", "to", "was", "were", "will", "with", "this",
        "but", "not", "you", "your", "can", "do", "does", "how", "if", "may", "no", "so", "than",
        "too", "very", "just", "about", "into", "over", "such", "then", "them", "these", "those",
        "through", "under", "use", "using", "used",
    ])
});

/// Check if a token is searchable: length > 1 char (or all digits) and not a stop word.
/// Matches JS `isSearchableToken`.
pub fn is_searchable_token(token: &str) -> bool {
    (token.chars().count() > 1 || token.chars().all(|c| c.is_ascii_digit()))
        && !STOP_WORDS.contains(token)
}

/// Compact an identifier by lowercasing and removing all non-alphanumeric chars.
/// Matches JS `compactIdentifier`.
pub fn compact_identifier(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Split where alpha meets digit and vice versa, inserting spaces.
/// "auth0" → "auth 0", "v2api" → "v 2 api"
fn split_alpha_numeric(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 4);
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        result.push(chars[i]);
        if i + 1 < chars.len() {
            let cur_alpha = chars[i].is_ascii_alphabetic();
            let cur_digit = chars[i].is_ascii_digit();
            let next_alpha = chars[i + 1].is_ascii_alphabetic();
            let next_digit = chars[i + 1].is_ascii_digit();
            if (cur_alpha && next_digit) || (cur_digit && next_alpha) {
                result.push(' ');
            }
        }
    }
    result
}

/// Tokenize text into lowercase terms with stop word removal.
/// Must produce identical output to the JS tokenize() function.
///
/// Algorithm: lowercase → replace non-alphanumeric (except spaces and hyphens) with space
///          → split on whitespace/hyphens → filter searchable tokens
pub fn tokenize(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let lower = text.to_lowercase();

    // Replace non-[a-z0-9\s-] with space (matching JS regex /[^a-z0-9\s-]/g)
    let cleaned: String = lower
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect();

    // Split on whitespace and hyphens (matching JS /[\s-]+/)
    cleaned
        .split(|c: char| c.is_ascii_whitespace() || c == '-')
        .filter(|t| !t.is_empty() && is_searchable_token(t))
        .map(|t| t.to_string())
        .collect()
}

/// Tokenize identifiers more aggressively than free text.
/// Splits on `/`, `_`, `.`, `-`, spaces and also splits alpha/numeric boundaries.
/// Matches JS `tokenizeIdentifier`.
pub fn tokenize_identifier(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens: HashSet<String> = HashSet::new();

    // Start with normal tokenize results
    for t in tokenize(text) {
        tokens.insert(t);
    }

    let raw = text;
    let compact = compact_identifier(raw);

    // Split by / and by /_.\ -
    let mut segments: HashSet<String> = HashSet::new();
    for seg in raw.split('/') {
        let c = compact_identifier(seg);
        if !c.is_empty() {
            segments.insert(c);
        }
    }
    for seg in raw.split(&['/', '_', '.', ' ', '-'][..]) {
        let c = compact_identifier(seg);
        if !c.is_empty() {
            segments.insert(c);
        }
    }

    // Add the full compact form
    if is_searchable_token(&compact) {
        tokens.insert(compact.clone());
    }

    // Add alpha-numeric split of compact form
    for t in tokenize(&split_alpha_numeric(&compact)) {
        tokens.insert(t);
    }

    // Process each segment
    for segment in &segments {
        if is_searchable_token(segment) {
            tokens.insert(segment.clone());
        }
        for t in tokenize(&split_alpha_numeric(segment)) {
            tokens.insert(t);
        }
    }

    tokens.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let tokens = tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_stop_word_removal() {
        let tokens = tokenize("the quick and easy way to do it");
        // "the", "and", "to", "do", "it" are stop words; "way" is 3 chars (kept)
        assert_eq!(tokens, vec!["quick", "easy", "way"]);
    }

    #[test]
    fn test_punctuation_removal() {
        let tokens = tokenize("hello, world! foo-bar (baz)");
        assert_eq!(tokens, vec!["hello", "world", "foo", "bar", "baz"]);
    }

    #[test]
    fn test_single_char_filtered() {
        let tokens = tokenize("a b c de fg");
        // "a" is stop word and single char; "b", "c" are single chars
        assert_eq!(tokens, vec!["de", "fg"]);
    }

    #[test]
    fn test_empty_input() {
        assert!(tokenize("").is_empty());
    }

    #[test]
    fn test_only_stop_words() {
        let tokens = tokenize("the and is are");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_compact_identifier() {
        assert_eq!(compact_identifier("node-fetch"), "nodefetch");
        assert_eq!(compact_identifier("@scope/pkg"), "scopepkg");
        assert_eq!(compact_identifier("Auth0"), "auth0");
    }

    #[test]
    fn test_tokenize_identifier() {
        let tokens = tokenize_identifier("node-fetch");
        assert!(tokens.contains(&"node".to_string()));
        assert!(tokens.contains(&"fetch".to_string()));
        assert!(tokens.contains(&"nodefetch".to_string()));
    }

    #[test]
    fn test_tokenize_identifier_alpha_numeric_split() {
        let tokens = tokenize_identifier("auth0/sdk");
        assert!(tokens.contains(&"auth".to_string()));
        assert!(tokens.contains(&"sdk".to_string()));
        assert!(tokens.contains(&"auth0".to_string()));
    }
}
