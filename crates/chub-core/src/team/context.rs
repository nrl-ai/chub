use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::team::project::project_chub_dir;

/// A custom project context document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDoc {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Filename (relative to .chub/context/)
    pub file: String,
}

/// Frontmatter for a context doc.
#[derive(Debug, Clone, Default, Deserialize)]
struct ContextFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tags: Option<String>,
}

fn context_dir() -> Option<PathBuf> {
    project_chub_dir().map(|d| d.join("context"))
}

/// Parse frontmatter from a markdown context doc.
fn parse_context_frontmatter(content: &str) -> (ContextFrontmatter, String) {
    if !content.starts_with("---") {
        return (ContextFrontmatter::default(), content.to_string());
    }
    let rest = &content[3..];
    if let Some(end) = rest.find("\n---") {
        let yaml_str = &rest[..end];
        let body = &rest[end + 4..];
        let fm: ContextFrontmatter = serde_yaml::from_str(yaml_str).unwrap_or_default();
        (fm, body.trim_start_matches('\n').to_string())
    } else {
        (ContextFrontmatter::default(), content.to_string())
    }
}

/// Discover all context docs in `.chub/context/`.
pub fn discover_context_docs() -> Vec<ContextDoc> {
    let dir = match context_dir() {
        Some(d) if d.exists() => d,
        _ => return vec![],
    };

    let mut docs = Vec::new();

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path
            .extension()
            .map(|e| e == "md" || e == "markdown")
            .unwrap_or(false)
        {
            continue;
        }

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, _body) = parse_context_frontmatter(&content);
        let name = fm.name.unwrap_or_else(|| stem.clone());
        let description = fm.description.unwrap_or_default();
        let tags = fm
            .tags
            .map(|t| {
                t.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        docs.push(ContextDoc {
            name,
            description,
            tags,
            file: filename,
        });
    }

    docs.sort_by(|a, b| a.name.cmp(&b.name));
    docs
}

/// Get a specific context doc by name (stem or name field).
pub fn get_context_doc(name: &str) -> Option<(ContextDoc, String)> {
    let dir = context_dir()?;

    // Try exact filename first
    let md_path = dir.join(format!("{}.md", name));
    if md_path.exists() {
        let content = fs::read_to_string(&md_path).ok()?;
        let (fm, _body) = parse_context_frontmatter(&content);
        let doc = ContextDoc {
            name: fm.name.unwrap_or_else(|| name.to_string()),
            description: fm.description.unwrap_or_default(),
            tags: fm
                .tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            file: format!("{}.md", name),
        };
        return Some((doc, content));
    }

    // Search by name field
    for doc in discover_context_docs() {
        if doc.name.to_lowercase() == name.to_lowercase() {
            let full_path = dir.join(&doc.file);
            let content = fs::read_to_string(&full_path).ok()?;
            return Some((doc, content));
        }
    }

    None
}

/// List context docs (name and description only).
pub fn list_context_docs() -> Vec<ContextDoc> {
    discover_context_docs()
}
