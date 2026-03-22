use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use sha2::{Digest, Sha256};

use crate::error::Result;
use crate::frontmatter::parse_frontmatter;
use crate::types::{DocEntry, LanguageEntry, SkillEntry, VersionEntry};

/// Compute SHA-256 hex digest of file contents.
fn sha256_content(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

/// Result of discovering entries in an author directory.
#[derive(Debug, Default)]
pub struct DiscoveryResult {
    pub docs: Vec<DocEntry>,
    pub skills: Vec<SkillEntry>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Get relative path as a forward-slash string.
fn pathdiff(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Info about a directory collected during the single-pass walk.
#[derive(Debug, Default)]
struct DirInfo {
    files: Vec<String>,
    total_size: u64,
}

/// Process an author directory with auto-discovery of DOC.md/SKILL.md files.
/// Uses a single-pass walk to discover entries AND collect file info simultaneously.
pub fn discover_author(
    author_dir: &Path,
    author_name: &str,
    content_dir: &Path,
) -> DiscoveryResult {
    let mut result = DiscoveryResult::default();

    // Phase 1: Single walk — collect entry files AND per-directory file info
    struct EntryInfo {
        path: PathBuf,
        rel_path: String,
        is_skill: bool,
        dir_key: PathBuf, // parent directory (canonical key)
    }

    let mut entries: Vec<EntryInfo> = Vec::new();
    let mut dir_infos: HashMap<PathBuf, DirInfo> = HashMap::new();

    for entry in WalkDir::new(author_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();
        let file_path = entry.path();
        let parent = file_path.parent().unwrap_or(author_dir);

        // Collect file info for the parent directory
        let dir_info = dir_infos.entry(parent.to_path_buf()).or_default();
        dir_info.files.push(pathdiff(file_path, parent));
        dir_info.total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);

        // Check if this is an entry file
        if file_name == "DOC.md" || file_name == "SKILL.md" {
            entries.push(EntryInfo {
                path: file_path.to_path_buf(),
                rel_path: pathdiff(file_path, content_dir),
                is_skill: file_name == "SKILL.md",
                dir_key: parent.to_path_buf(),
            });
        }
    }

    // Phase 2: Process discovered entries using pre-collected dir info
    let mut docs: HashMap<String, DocBuilder> = HashMap::new();
    let mut skills: HashMap<String, SkillEntry> = HashMap::new();

    for ef in &entries {
        let content = match fs::read_to_string(&ef.path) {
            Ok(c) => c,
            Err(e) => {
                result.errors.push(format!("{}: {}", ef.rel_path, e));
                continue;
            }
        };

        let (fm, _body) = parse_frontmatter(&content);

        let name = match &fm.name {
            Some(n) => n.clone(),
            None => {
                result
                    .errors
                    .push(format!("{}: missing 'name' in frontmatter", ef.rel_path));
                continue;
            }
        };

        if fm.description.is_none() {
            result.warnings.push(format!(
                "{}: missing 'description' in frontmatter",
                ef.rel_path
            ));
        }

        let meta = &fm.metadata;
        let description = fm.description.clone().unwrap_or_default();
        let source = meta
            .source
            .clone()
            .unwrap_or_else(|| "community".to_string());
        let tags: Vec<String> = meta
            .tags
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let updated_on = meta.updated_on.clone().unwrap_or_else(today_fallback);
        let revision = meta.revision.clone();

        let entry_path = pathdiff(&ef.dir_key, content_dir);
        let dir_info = &dir_infos[&ef.dir_key];
        let files = dir_info.files.clone();
        let size = dir_info.total_size;

        if meta.source.is_none() {
            result.warnings.push(format!(
                "{}: missing 'metadata.source', defaulting to 'community'",
                ef.rel_path
            ));
        }

        if ef.is_skill {
            if skills.contains_key(&name) {
                result
                    .errors
                    .push(format!("{}: duplicate skill name '{}'", ef.rel_path, name));
                continue;
            }
            let content_hash = Some(sha256_content(content.as_bytes()));
            skills.insert(
                name.clone(),
                SkillEntry {
                    id: format!("{}/{}", author_name, name),
                    name,
                    description,
                    source,
                    tags,
                    path: entry_path,
                    files,
                    size,
                    last_updated: updated_on,
                    revision,
                    content_hash,
                },
            );
        } else {
            // Doc — needs language and version
            let languages: Option<Vec<String>> = meta
                .languages
                .as_ref()
                .map(|l| l.split(',').map(|s| s.trim().to_lowercase()).collect());
            let versions: Option<Vec<String>> = meta
                .versions
                .as_ref()
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect());

            let has_langs = languages.as_ref().is_some_and(|l| !l.is_empty());
            let has_vers = versions.as_ref().is_some_and(|v| !v.is_empty());

            if !has_langs {
                result.errors.push(format!(
                    "{}: missing 'metadata.languages' in frontmatter",
                    ef.rel_path
                ));
                continue;
            }
            if !has_vers {
                result.errors.push(format!(
                    "{}: missing 'metadata.versions' in frontmatter",
                    ef.rel_path
                ));
                continue;
            }

            let languages = languages.unwrap();
            let versions = versions.unwrap();

            let doc = docs.entry(name.clone()).or_insert_with(|| DocBuilder {
                description: description.clone(),
                source: source.clone(),
                tags: tags.clone(),
                languages: HashMap::new(),
            });

            let content_hash = Some(sha256_content(content.as_bytes()));
            for lang in &languages {
                let lang_versions = doc.languages.entry(lang.clone()).or_default();
                for ver in &versions {
                    lang_versions.push(VersionEntry {
                        version: ver.clone(),
                        path: entry_path.clone(),
                        files: files.clone(),
                        size,
                        last_updated: updated_on.clone(),
                        revision: revision.clone(),
                        content_hash: content_hash.clone(),
                    });
                }
            }
        }
    }

    // Convert docs map to array
    for (name, doc_builder) in docs {
        let mut languages = Vec::new();
        for (lang, mut versions) in doc_builder.languages {
            versions.sort_by(|a, b| human_sort_desc(&b.version, &a.version));
            let recommended = versions
                .first()
                .map(|v| v.version.clone())
                .unwrap_or_default();
            languages.push(LanguageEntry {
                language: lang,
                versions,
                recommended_version: recommended,
            });
        }
        result.docs.push(DocEntry {
            id: format!("{}/{}", author_name, name),
            name,
            description: doc_builder.description,
            source: doc_builder.source,
            tags: doc_builder.tags,
            languages,
        });
    }

    result.skills = skills.into_values().collect();
    result
}

/// Helper struct for building docs from multiple entry files.
struct DocBuilder {
    description: String,
    source: String,
    tags: Vec<String>,
    languages: HashMap<String, Vec<VersionEntry>>,
}

/// Descending version sort using numeric-aware comparison.
fn human_sort_desc(a: &str, b: &str) -> std::cmp::Ordering {
    let seg_a = segmentize(a);
    let seg_b = segmentize(b);

    for (sa, sb) in seg_a.iter().zip(seg_b.iter()) {
        let ord = match (sa, sb) {
            (Segment::Num(na), Segment::Num(nb)) => na.cmp(nb),
            (Segment::Text(ta), Segment::Text(tb)) => ta.cmp(tb),
            (Segment::Num(_), Segment::Text(_)) => std::cmp::Ordering::Less,
            (Segment::Text(_), Segment::Num(_)) => std::cmp::Ordering::Greater,
        };
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    seg_a.len().cmp(&seg_b.len())
}

enum Segment {
    Num(u64),
    Text(String),
}

fn segmentize(s: &str) -> Vec<Segment> {
    let mut segs = Vec::new();
    let mut chars = s.chars().peekable();
    while chars.peek().is_some() {
        if chars.peek().unwrap().is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            segs.push(Segment::Num(num_str.parse().unwrap_or(0)));
        } else {
            let mut text = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    break;
                }
                text.push(c);
                chars.next();
            }
            segs.push(Segment::Text(text));
        }
    }
    segs
}

/// Fallback date for missing updated-on.
fn today_fallback() -> String {
    crate::util::today_date()
}

/// Load an author's registry.json and prefix paths with author name.
pub fn load_author_registry(
    author_dir: &Path,
    author_name: &str,
) -> Result<(Vec<DocEntry>, Vec<SkillEntry>)> {
    let registry_path = author_dir.join("registry.json");
    let raw = fs::read_to_string(&registry_path)?;
    let author_reg: crate::types::AuthorRegistry = serde_json::from_str(&raw)?;

    let mut docs = Vec::new();
    for doc in author_reg.docs {
        let id = doc
            .id
            .unwrap_or_else(|| format!("{}/{}", author_name, doc.name));
        let id = if id.contains('/') {
            id
        } else {
            format!("{}/{}", author_name, id)
        };

        let languages = doc
            .languages
            .unwrap_or_default()
            .into_iter()
            .map(|mut lang| {
                for ver in &mut lang.versions {
                    ver.path = format!("{}/{}", author_name, ver.path);
                }
                lang
            })
            .collect();

        docs.push(DocEntry {
            id,
            name: doc.name,
            description: doc.description.unwrap_or_default(),
            source: doc.source.unwrap_or_else(|| "community".to_string()),
            tags: doc.tags.unwrap_or_default(),
            languages,
        });
    }

    let mut skills = Vec::new();
    for skill in author_reg.skills {
        let id = skill
            .id
            .unwrap_or_else(|| format!("{}/{}", author_name, skill.name));
        let id = if id.contains('/') {
            id
        } else {
            format!("{}/{}", author_name, id)
        };

        skills.push(SkillEntry {
            id,
            name: skill.name,
            description: skill.description.unwrap_or_default(),
            source: skill.source.unwrap_or_else(|| "community".to_string()),
            tags: skill.tags.unwrap_or_default(),
            path: format!("{}/{}", author_name, skill.path),
            files: skill.files.unwrap_or_default(),
            size: skill.size.unwrap_or(0),
            last_updated: skill.last_updated.unwrap_or_default(),
            revision: None,
            content_hash: None,
        });
    }

    Ok((docs, skills))
}
