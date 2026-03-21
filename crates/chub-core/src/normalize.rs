use std::collections::HashMap;
use std::sync::LazyLock;

static ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("js", "javascript"),
        ("ts", "typescript"),
        ("py", "python"),
        ("rb", "ruby"),
        ("cs", "csharp"),
    ])
});

static DISPLAY: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("javascript", "js"),
        ("typescript", "ts"),
        ("python", "py"),
        ("ruby", "rb"),
        ("csharp", "cs"),
    ])
});

/// Normalize a language name: resolve aliases (js→javascript, etc.) and lowercase.
pub fn normalize_language(lang: &str) -> String {
    let lower = lang.to_lowercase();
    ALIASES
        .get(lower.as_str())
        .map(|s| s.to_string())
        .unwrap_or(lower)
}

/// Get the short display form of a language name (javascript→js, etc.).
pub fn display_language(lang: &str) -> &str {
    DISPLAY.get(lang).copied().unwrap_or(lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_aliases() {
        assert_eq!(normalize_language("py"), "python");
        assert_eq!(normalize_language("js"), "javascript");
        assert_eq!(normalize_language("ts"), "typescript");
        assert_eq!(normalize_language("rb"), "ruby");
        assert_eq!(normalize_language("cs"), "csharp");
    }

    #[test]
    fn test_normalize_case_insensitive() {
        assert_eq!(normalize_language("PY"), "python");
        assert_eq!(normalize_language("JavaScript"), "javascript");
    }

    #[test]
    fn test_normalize_unknown() {
        assert_eq!(normalize_language("rust"), "rust");
        assert_eq!(normalize_language("go"), "go");
    }

    #[test]
    fn test_display_language() {
        assert_eq!(display_language("javascript"), "js");
        assert_eq!(display_language("typescript"), "ts");
        assert_eq!(display_language("python"), "py");
        assert_eq!(display_language("ruby"), "rb");
        assert_eq!(display_language("csharp"), "cs");
        assert_eq!(display_language("rust"), "rust");
    }
}
