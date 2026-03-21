use crate::types::{Frontmatter, FrontmatterMetadata};

/// Parse YAML frontmatter from markdown content.
/// Returns (frontmatter, body) where frontmatter contains parsed YAML attributes.
pub fn parse_frontmatter(content: &str) -> (Frontmatter, &str) {
    let content = content.trim_start_matches('\u{feff}'); // strip BOM

    // Normalize \r\n to \n so we only need one set of patterns.
    let normalized = content.replace("\r\n", "\n");

    if !normalized.starts_with("---\n") {
        return (Frontmatter::default(), content);
    }

    let after_first = &normalized[4..];

    // Find the closing --- delimiter
    let (yaml_end, body_start) = if after_first.starts_with("---\n") || after_first == "---" {
        (0, "---\n".len().min(after_first.len()))
    } else if let Some(pos) = after_first.find("\n---\n") {
        (pos, pos + "\n---\n".len())
    } else if after_first.ends_with("\n---") {
        (after_first.len() - "\n---".len(), after_first.len())
    } else {
        return (Frontmatter::default(), content);
    };

    let yaml_str = &after_first[..yaml_end];
    let body = &after_first[body_start..];

    let fm = parse_yaml_frontmatter(yaml_str);
    // Return a slice of the *original* content with the same body text.
    // The body in the normalized string starts at offset (4 + body_start).
    // We need to find the equivalent position in the original content.
    let original_body = find_original_body(content, body);
    (fm, original_body)
}

/// Map the normalized body back to a slice of the original content.
fn find_original_body<'a>(original: &'a str, normalized_body: &str) -> &'a str {
    if normalized_body.is_empty() {
        return "";
    }
    // The body is everything after the second "---" delimiter line and its newline.
    // Search in the original for the closing delimiter.
    let after_open = if original.starts_with("---\r\n") {
        5
    } else {
        4 // "---\n"
    };
    let search = &original[after_open..];

    // Check empty frontmatter first: closing --- immediately after opening
    let empty_patterns: &[&str] = &["---\r\n", "---\n"];
    for pat in empty_patterns {
        if search.starts_with(pat) {
            let start = after_open + pat.len();
            return &original[start..];
        }
    }

    // Find closing delimiter: \n---\n, \r\n---\r\n, \n---\r\n, \r\n---\n, or trailing \n---, \r\n---
    let patterns: &[&str] = &["\r\n---\r\n", "\r\n---\n", "\n---\r\n", "\n---\n"];
    for pat in patterns {
        if let Some(pos) = search.find(pat) {
            let start = after_open + pos + pat.len();
            return &original[start..];
        }
    }
    // Trailing --- with no final newline
    for pat in &["\r\n---", "\n---"] {
        if search.ends_with(pat) {
            return "";
        }
    }
    ""
}

/// Extract a string value from a YAML mapping by key.
fn yaml_get_str(mapping: &serde_yaml::Mapping, key: &str) -> Option<String> {
    mapping
        .get(serde_yaml::Value::String(key.to_string()))
        .and_then(|v| match v {
            serde_yaml::Value::String(s) => Some(s.clone()),
            serde_yaml::Value::Number(n) => Some(n.to_string()),
            serde_yaml::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        })
}

fn parse_yaml_frontmatter(yaml_str: &str) -> Frontmatter {
    let value: serde_yaml::Value = match serde_yaml::from_str(yaml_str) {
        Ok(v) => v,
        Err(_) => return Frontmatter::default(),
    };

    let mapping = match value.as_mapping() {
        Some(m) => m,
        None => return Frontmatter::default(),
    };

    let metadata = mapping
        .get(serde_yaml::Value::String("metadata".to_string()))
        .and_then(|v| v.as_mapping())
        .map(|meta| FrontmatterMetadata {
            languages: yaml_get_str(meta, "languages"),
            versions: yaml_get_str(meta, "versions"),
            source: yaml_get_str(meta, "source"),
            tags: yaml_get_str(meta, "tags"),
            updated_on: yaml_get_str(meta, "updated-on"),
        })
        .unwrap_or_default();

    Frontmatter {
        name: yaml_get_str(mapping, "name"),
        description: yaml_get_str(mapping, "description"),
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_frontmatter() {
        let content = r#"---
name: test-lib
description: "A test library"
metadata:
  languages: "javascript,python"
  versions: "1.0.0,2.0.0"
  source: official
  tags: "test,example"
  updated-on: "2025-01-01"
---
# Test Library

Some content here.
"#;
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.name.as_deref(), Some("test-lib"));
        assert_eq!(fm.description.as_deref(), Some("A test library"));
        assert_eq!(fm.metadata.languages.as_deref(), Some("javascript,python"));
        assert_eq!(fm.metadata.versions.as_deref(), Some("1.0.0,2.0.0"));
        assert_eq!(fm.metadata.source.as_deref(), Some("official"));
        assert_eq!(fm.metadata.tags.as_deref(), Some("test,example"));
        assert_eq!(fm.metadata.updated_on.as_deref(), Some("2025-01-01"));
        assert!(body.contains("# Test Library"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a markdown file\n\nNo frontmatter here.";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.name.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\nBody content";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.name.is_none());
        assert_eq!(body, "Body content");
    }

    #[test]
    fn test_frontmatter_no_metadata() {
        let content = "---\nname: simple\ndescription: \"No metadata block\"\n---\nBody";
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.name.as_deref(), Some("simple"));
        assert!(fm.metadata.languages.is_none());
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_frontmatter_crlf_line_endings() {
        let content =
            "---\r\nname: crlf-test\r\ndescription: \"CRLF file\"\r\n---\r\nBody with CRLF";
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.name.as_deref(), Some("crlf-test"));
        assert_eq!(body, "Body with CRLF");
    }

    #[test]
    fn test_frontmatter_with_bom() {
        let content = "\u{feff}---\nname: bom-test\n---\nBody";
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.name.as_deref(), Some("bom-test"));
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_frontmatter_numeric_value() {
        let content = "---\nname: test\nmetadata:\n  versions: 2\n---\nBody";
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.name.as_deref(), Some("test"));
        // Numeric YAML value should be converted to string
        assert_eq!(fm.metadata.versions.as_deref(), Some("2"));
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_frontmatter_no_trailing_newline() {
        let content = "---\nname: trail\n---";
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.name.as_deref(), Some("trail"));
        assert_eq!(body, "");
    }

    #[test]
    fn test_frontmatter_whitespace_values() {
        let content = "---\nname: \"  \"\ndescription: \"\"\n---\nBody";
        let (fm, _body) = parse_frontmatter(content);
        assert_eq!(fm.name.as_deref(), Some("  "));
        assert_eq!(fm.description.as_deref(), Some(""));
    }
}
