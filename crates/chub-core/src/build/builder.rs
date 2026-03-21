use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use rayon::prelude::*;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::search::bm25;
use crate::types::{DocEntry, Entry, Registry, SearchIndex, SkillEntry};

use super::discovery::{discover_author, load_author_registry};

/// Options for the build process.
#[derive(Debug)]
pub struct BuildOptions {
    pub base_url: Option<String>,
    pub validate_only: bool,
    /// Enable incremental builds using a content hash manifest.
    /// When true, files are only copied if their SHA-256 hash has changed.
    pub incremental: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            base_url: None,
            validate_only: false,
            incremental: true,
        }
    }
}

/// Result of a successful build.
#[derive(Debug)]
pub struct BuildResult {
    pub registry: Registry,
    pub search_index: SearchIndex,
    pub docs_count: usize,
    pub skills_count: usize,
    pub warnings: Vec<String>,
}

/// Build a registry from a content directory.
///
/// Scans top-level directories as author directories.
/// Each author either has a registry.json or we auto-discover DOC.md/SKILL.md files.
pub fn build_registry(content_dir: &Path, opts: &BuildOptions) -> Result<BuildResult> {
    if !content_dir.exists() {
        return Err(Error::ContentDirNotFound(content_dir.to_path_buf()));
    }

    let mut all_docs: Vec<DocEntry> = Vec::new();
    let mut all_skills: Vec<SkillEntry> = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();
    let mut all_errors: Vec<String> = Vec::new();

    // List top-level directories (author directories)
    let mut author_dirs: Vec<(String, std::path::PathBuf)> = Vec::new();
    for entry in fs::read_dir(content_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "dist" || name.starts_with('.') {
            continue;
        }
        author_dirs.push((name, entry.path()));
    }

    for (author_name, author_dir) in &author_dirs {
        let author_registry = author_dir.join("registry.json");

        if author_registry.exists() {
            match load_author_registry(author_dir, author_name) {
                Ok((docs, skills)) => {
                    all_docs.extend(docs);
                    all_skills.extend(skills);
                }
                Err(e) => {
                    all_errors.push(format!("{}/registry.json: {}", author_name, e));
                }
            }
        } else {
            let result = discover_author(author_dir, author_name, content_dir);
            all_docs.extend(result.docs);
            all_skills.extend(result.skills);
            all_warnings.extend(result.warnings);
            all_errors.extend(result.errors);
        }
    }

    // Check for id collisions using HashSet (faster than HashMap)
    let mut seen = HashSet::with_capacity(all_docs.len() + all_skills.len());
    for doc in &all_docs {
        if !seen.insert(&doc.id) {
            all_errors.push(format!("Duplicate doc id '{}'", doc.id));
        }
    }
    for skill in &all_skills {
        if !seen.insert(&skill.id) {
            all_errors.push(format!("Duplicate skill id '{}'", skill.id));
        }
    }

    if !all_errors.is_empty() {
        return Err(Error::BuildErrors(all_errors.join("\n")));
    }

    // Build search index
    let entries: Vec<Entry> = all_docs
        .iter()
        .map(Entry::Doc)
        .chain(all_skills.iter().map(Entry::Skill))
        .collect();
    let search_index = bm25::build_index(&entries);

    let docs_count = all_docs.len();
    let skills_count = all_skills.len();
    let generated = now_iso8601();

    let registry = Registry {
        version: "1.0.0".to_string(),
        generated,
        docs: all_docs,
        skills: all_skills,
        base_url: opts.base_url.clone(),
    };

    Ok(BuildResult {
        docs_count,
        skills_count,
        warnings: all_warnings,
        registry,
        search_index,
    })
}

/// Name of the build manifest file used for incremental builds.
const BUILD_MANIFEST_NAME: &str = ".build-manifest.json";

/// Compute the SHA-256 hex digest of a file's contents.
fn sha256_file(path: &Path) -> std::io::Result<String> {
    let data = fs::read(path)?;
    let hash = Sha256::digest(&data);
    Ok(format!("{:x}", hash))
}

/// Load an existing build manifest from the output directory.
/// Returns an empty map if the file does not exist or cannot be parsed.
fn load_build_manifest(output_dir: &Path) -> HashMap<String, String> {
    let manifest_path = output_dir.join(BUILD_MANIFEST_NAME);
    if let Ok(data) = fs::read_to_string(&manifest_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

/// Save the build manifest to the output directory.
fn save_build_manifest(output_dir: &Path, manifest: &HashMap<String, String>) -> Result<()> {
    use std::io::BufWriter;
    let file = fs::File::create(output_dir.join(BUILD_MANIFEST_NAME))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, manifest)?;
    Ok(())
}

/// Write build results to output directory.
pub fn write_build_output(
    content_dir: &Path,
    output_dir: &Path,
    result: &BuildResult,
) -> Result<()> {
    write_build_output_with_opts(content_dir, output_dir, result, &BuildOptions::default())
}

/// Write build results to output directory with options controlling incremental behavior.
pub fn write_build_output_with_opts(
    content_dir: &Path,
    output_dir: &Path,
    result: &BuildResult,
    opts: &BuildOptions,
) -> Result<()> {
    use std::io::BufWriter;

    fs::create_dir_all(output_dir)?;

    // Write registry.json using buffered writer
    let file = fs::File::create(output_dir.join("registry.json"))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &result.registry)?;

    // Write search-index.json using buffered writer (compact, no pretty-print)
    let file = fs::File::create(output_dir.join("search-index.json"))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &result.search_index)?;

    // Write search-index.bin using bincode serialization
    let bin_data = bincode::serialize(&result.search_index)
        .map_err(|e| Error::BuildErrors(format!("bincode serialization failed: {}", e)))?;
    fs::write(output_dir.join("search-index.bin"), &bin_data)?;

    // Write index.html landing page for the CDN root
    let index_html = generate_index_html(result);
    fs::write(output_dir.join("index.html"), index_html)?;

    // Load existing manifest for incremental builds
    let old_manifest = if opts.incremental {
        load_build_manifest(output_dir)
    } else {
        HashMap::new()
    };
    let mut new_manifest: HashMap<String, String> = HashMap::new();

    // Copy content tree using single walkdir pass with filter_entry for early pruning
    // Phase 1: collect dirs and files
    // Phase 2: batch create dirs, then copy files in parallel
    let mut dirs_to_create = Vec::new();
    let mut files_to_copy: Vec<(std::path::PathBuf, std::path::PathBuf, String)> = Vec::new();

    for entry in walkdir::WalkDir::new(content_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            // Early-prune dist/ and dotfile directories at top level
            if e.depth() == 1 && e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                return name != "dist" && !name.starts_with('.');
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        // Skip registry.json in author root (depth 2: author/registry.json)
        if entry.file_type().is_file() && entry.file_name() == "registry.json" && entry.depth() == 2
        {
            continue;
        }

        let rel = entry.path().strip_prefix(content_dir).unwrap();
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let dest = output_dir.join(rel);

        if entry.file_type().is_dir() {
            dirs_to_create.push(dest);
        } else {
            files_to_copy.push((entry.into_path(), dest, rel_str));
        }
    }

    // Batch create all directories first sequentially (parents must exist before children)
    for dir in &dirs_to_create {
        fs::create_dir_all(dir)?;
    }

    if opts.incremental {
        // Compute hashes and filter unchanged files, then copy in parallel
        let copy_results: Vec<std::result::Result<(String, String), Error>> = files_to_copy
            .par_iter()
            .map(|(src, dest, rel_str)| {
                let hash = sha256_file(src).map_err(|e| {
                    Error::BuildErrors(format!("hash failed for {}: {}", rel_str, e))
                })?;

                // Skip copy if the hash matches the old manifest
                if old_manifest.get(rel_str).map(|h| h.as_str()) == Some(hash.as_str()) {
                    return Ok((rel_str.clone(), hash));
                }

                fs::copy(src, dest).map_err(|e| {
                    Error::BuildErrors(format!("copy failed for {}: {}", rel_str, e))
                })?;
                Ok((rel_str.clone(), hash))
            })
            .collect();

        for res in copy_results {
            let (rel_str, hash) = res?;
            new_manifest.insert(rel_str, hash);
        }

        // Save updated manifest
        save_build_manifest(output_dir, &new_manifest)?;
    } else {
        // Non-incremental: copy all files in parallel using rayon, no manifest
        let copy_results: Vec<std::result::Result<(), Error>> = files_to_copy
            .par_iter()
            .map(|(src, dest, rel_str)| {
                fs::copy(src, dest).map_err(|e| {
                    Error::BuildErrors(format!("copy failed for {}: {}", rel_str, e))
                })?;
                Ok(())
            })
            .collect();

        for res in copy_results {
            res?;
        }
    }

    Ok(())
}

/// Generate an index.html landing page with search for the CDN root.
/// Matches the VitePress website theme (Inter font, sky blue brand colors, dark mode).
fn generate_index_html(result: &BuildResult) -> String {
    // Build a compact JSON catalog for client-side search, including paths for doc viewer
    let mut entries = Vec::new();
    for doc in &result.registry.docs {
        let langs: Vec<serde_json::Value> = doc
            .languages
            .iter()
            .map(|l| {
                let versions: Vec<serde_json::Value> = l
                    .versions
                    .iter()
                    .map(|v| {
                        serde_json::json!({
                            "version": v.version,
                            "path": v.path,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "language": l.language,
                    "recommended": l.recommended_version,
                    "versions": versions,
                })
            })
            .collect();
        let lang_names: Vec<&str> = doc.languages.iter().map(|l| l.language.as_str()).collect();
        entries.push(serde_json::json!({
            "id": doc.id,
            "name": doc.name,
            "description": doc.description,
            "source": doc.source,
            "tags": doc.tags,
            "type": "doc",
            "langNames": lang_names,
            "langs": langs,
        }));
    }
    for skill in &result.registry.skills {
        entries.push(serde_json::json!({
            "id": skill.id,
            "name": skill.name,
            "description": skill.description,
            "source": skill.source,
            "tags": skill.tags,
            "type": "skill",
            "langNames": [],
            "langs": [],
            "path": skill.path,
        }));
    }
    let catalog_json = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());

    let docs_count = result.docs_count;
    let skills_count = result.skills_count;
    let generated = &result.registry.generated;

    let mut languages: Vec<&str> = result
        .registry
        .docs
        .iter()
        .flat_map(|d| d.languages.iter().map(|l| l.language.as_str()))
        .collect();
    languages.sort();
    languages.dedup();

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Chub — Content Registry</title>
<meta name="description" content="Browse {docs} curated API docs and {skills} skills for AI coding agents. Search, filter by language and tags.">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
<style>
:root {{
  --vp-c-brand-1: #0ea5e9;
  --vp-c-brand-2: #0284c7;
  --vp-c-brand-3: #0369a1;
  --vp-c-brand-soft: rgba(14,165,233,0.14);
  --bg: #1b1b1f;
  --bg-alt: #161618;
  --bg-elv: #202127;
  --card: #202127;
  --border: #2e2e32;
  --text: rgba(255,255,245,0.86);
  --muted: rgba(235,235,245,0.6);
  --dim: rgba(235,235,245,0.38);
  --accent: #38bdf8;
  --accent-hover: #0ea5e9;
  --accent-active: #0284c7;
  --accent2: #818cf8;
  --green: #34d399;
  --surface: #161618;
  --code-bg: #161618;
}}
* {{ margin:0; padding:0; box-sizing:border-box }}
body {{
  font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
}}
a {{ color: var(--accent); text-decoration: none; transition: color .15s }}
a:hover {{ color: var(--accent-hover) }}

/* Nav bar */
.navbar {{
  display: flex; align-items: center; justify-content: space-between;
  max-width: 1152px; margin: 0 auto; padding: 0.75rem 1.5rem;
  border-bottom: 1px solid var(--border);
}}
.navbar-brand {{ display: flex; align-items: center; gap: 0.5rem; font-weight: 700; font-size: 1.1rem; color: var(--text) }}
.navbar-brand svg {{ width: 28px; height: 28px }}
.navbar-links {{ display: flex; gap: 1.25rem; font-size: 0.875rem; font-weight: 500 }}
.navbar-links a {{ color: var(--muted) }}
.navbar-links a:hover {{ color: var(--text) }}

.wrap {{ max-width: 1152px; margin: 0 auto; padding: 1.5rem }}
header {{ text-align: center; padding: 2.5rem 0 1.5rem }}
header h1 {{ font-size: 2.25rem; font-weight: 800; letter-spacing: -0.03em }}
header h1 span {{ color: var(--accent) }}
header p {{ color: var(--muted); margin-top: 0.35rem; font-size: 1.05rem }}

.stats {{ display: flex; gap: 1rem; justify-content: center; margin: 1rem 0 2rem; flex-wrap: wrap }}
.stat {{
  background: var(--card); border: 1px solid var(--border); border-radius: 12px;
  padding: 0.75rem 1.5rem; text-align: center; min-width: 110px;
  transition: border-color .2s, box-shadow .2s;
}}
.stat:hover {{ border-color: var(--accent); box-shadow: 0 0 0 1px var(--vp-c-brand-soft) }}
.stat b {{ display: block; font-size: 1.5rem; color: var(--accent); font-weight: 700 }}
.stat small {{ color: var(--muted); font-size: 0.8rem }}

.search-bar {{ position: relative; max-width: 640px; margin: 0 auto 1rem }}
.search-bar input {{
  width: 100%; padding: 0.85rem 1rem 0.85rem 2.85rem;
  background: var(--card); border: 1px solid var(--border); border-radius: 12px;
  color: var(--text); font-size: 1rem; font-family: inherit; outline: none;
  transition: border-color .2s, box-shadow .2s;
}}
.search-bar input:focus {{ border-color: var(--accent); box-shadow: 0 0 0 2px var(--vp-c-brand-soft) }}
.search-bar input::placeholder {{ color: var(--dim) }}
.search-bar svg {{ position: absolute; left: 0.9rem; top: 50%; transform: translateY(-50%); color: var(--dim); width: 18px; height: 18px }}

.filters {{ display: flex; gap: 0.4rem; flex-wrap: wrap; justify-content: center; margin-bottom: 1.5rem }}
.pill {{
  background: var(--card); border: 1px solid var(--border); border-radius: 20px;
  padding: 0.3rem 0.85rem; font-size: 0.8rem; color: var(--muted); cursor: pointer;
  transition: all .15s; user-select: none; font-family: inherit;
}}
.pill:hover, .pill.active {{ background: var(--accent); color: var(--bg); border-color: var(--accent) }}

.results-info {{ color: var(--dim); font-size: 0.85rem; margin-bottom: 0.75rem }}

.grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 0.75rem; margin-bottom: 2rem }}
.entry {{
  background: var(--card); border: 1px solid var(--border); border-radius: 12px;
  padding: 1rem 1.15rem; display: flex; flex-direction: column; cursor: pointer;
  transition: border-color .2s, box-shadow .2s, transform .15s;
}}
.entry:hover {{ border-color: var(--accent); box-shadow: 0 2px 12px rgba(14,165,233,0.08); transform: translateY(-1px) }}
.entry-head {{ display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.35rem }}
.entry-head h3 {{ font-size: 0.95rem; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1 }}
.badge {{ font-size: 0.65rem; padding: 0.15rem 0.5rem; border-radius: 6px; font-weight: 600; text-transform: uppercase; flex-shrink: 0; letter-spacing: 0.02em }}
.badge-doc {{ background: var(--vp-c-brand-soft); color: var(--accent) }}
.badge-skill {{ background: rgba(52,211,153,0.12); color: var(--green) }}
.badge-src {{ background: var(--surface); color: var(--dim); font-size: 0.6rem }}
.entry p {{ color: var(--muted); font-size: 0.82rem; margin-bottom: 0.5rem; flex: 1; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden }}
.entry-meta {{ display: flex; flex-wrap: wrap; gap: 0.3rem; margin-top: auto }}
.tag {{ background: var(--surface); color: var(--dim); padding: 0.1rem 0.5rem; border-radius: 4px; font-size: 0.7rem }}
.lang-tag {{ background: var(--bg-alt); color: var(--accent2); padding: 0.1rem 0.5rem; border-radius: 4px; font-size: 0.7rem; border: 1px solid var(--border) }}
.entry-id {{ font-family: 'SF Mono', 'Fira Code', Consolas, monospace; font-size: 0.75rem; color: var(--dim); margin-bottom: 0.2rem }}

footer {{ text-align: center; padding: 2rem 0; color: var(--dim); font-size: 0.8rem; border-top: 1px solid var(--border) }}
footer .links {{ margin-top: 0.5rem }}
footer .links a {{ color: var(--muted) }}
footer .links a:hover {{ color: var(--accent) }}

.install-hint {{
  max-width: 640px; margin: 0 auto 2.5rem;
  background: var(--code-bg); border: 1px solid var(--border); border-radius: 12px;
  padding: 1rem 1.25rem;
  font-family: 'SF Mono', 'Fira Code', Consolas, monospace; font-size: 0.85rem;
}}
.install-hint .prompt {{ color: var(--dim) }}
.install-hint .cmd {{ color: var(--accent) }}

.empty {{ text-align: center; padding: 3rem; color: var(--muted) }}

.pagination {{ display: flex; justify-content: center; gap: 0.5rem; margin-bottom: 2rem }}
.pagination button {{
  background: var(--card); border: 1px solid var(--border); border-radius: 8px;
  padding: 0.4rem 0.85rem; color: var(--text); font-size: 0.85rem; cursor: pointer;
  font-family: inherit; transition: border-color .15s;
}}
.pagination button:hover {{ border-color: var(--accent) }}
.pagination button:disabled {{ opacity: 0.4; cursor: default }}
.pagination button:disabled:hover {{ border-color: var(--border) }}

/* Doc viewer modal */
.modal-overlay {{
  display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.7);
  z-index: 100; justify-content: center; align-items: flex-start;
  padding: 2rem; overflow-y: auto; backdrop-filter: blur(4px);
}}
.modal-overlay.open {{ display: flex }}
.modal {{
  background: var(--bg); border: 1px solid var(--border); border-radius: 16px;
  max-width: 900px; width: 100%; max-height: 90vh; overflow-y: auto;
  box-shadow: 0 8px 32px rgba(0,0,0,0.5);
}}
.modal-header {{
  position: sticky; top: 0; background: var(--bg); border-bottom: 1px solid var(--border);
  padding: 1rem 1.5rem; display: flex; align-items: center; gap: 0.75rem; z-index: 1;
  border-radius: 16px 16px 0 0;
}}
.modal-header h2 {{ font-size: 1.1rem; font-weight: 700; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap }}
.modal-header select {{
  background: var(--card); border: 1px solid var(--border); border-radius: 8px;
  padding: 0.3rem 0.6rem; color: var(--text); font-size: 0.8rem; font-family: inherit;
  outline: none; cursor: pointer;
}}
.modal-close {{
  background: none; border: 1px solid var(--border); border-radius: 8px;
  width: 36px; height: 36px; color: var(--muted); cursor: pointer; font-size: 1.2rem;
  display: flex; align-items: center; justify-content: center; transition: all .15s;
}}
.modal-close:hover {{ border-color: var(--accent); color: var(--text) }}
.modal-body {{ padding: 1.5rem }}
.modal-body.loading {{ display: flex; align-items: center; justify-content: center; min-height: 200px; color: var(--muted) }}

/* Rendered markdown in modal */
.md h1 {{ font-size: 1.5rem; font-weight: 700; margin: 0 0 0.75rem; border-bottom: 1px solid var(--border); padding-bottom: 0.5rem }}
.md h2 {{ font-size: 1.25rem; font-weight: 600; margin: 1.5rem 0 0.5rem; border-bottom: 1px solid var(--border); padding-bottom: 0.35rem }}
.md h3 {{ font-size: 1.05rem; font-weight: 600; margin: 1.25rem 0 0.4rem }}
.md h4 {{ font-size: 0.95rem; font-weight: 600; margin: 1rem 0 0.35rem }}
.md p {{ margin: 0.5rem 0; line-height: 1.7 }}
.md ul, .md ol {{ margin: 0.5rem 0; padding-left: 1.5rem }}
.md li {{ margin: 0.25rem 0 }}
.md code {{
  background: var(--code-bg); border: 1px solid var(--border); border-radius: 4px;
  padding: 0.15rem 0.35rem; font-size: 0.85em;
  font-family: 'SF Mono', 'Fira Code', Consolas, monospace;
}}
.md pre {{
  background: var(--code-bg); border: 1px solid var(--border); border-radius: 8px;
  padding: 1rem; overflow-x: auto; margin: 0.75rem 0;
  font-family: 'SF Mono', 'Fira Code', Consolas, monospace; font-size: 0.85rem;
  line-height: 1.5;
}}
.md pre code {{ background: none; border: none; padding: 0; font-size: inherit }}
.md blockquote {{
  border-left: 3px solid var(--accent); padding: 0.5rem 1rem; margin: 0.75rem 0;
  color: var(--muted); background: var(--vp-c-brand-soft); border-radius: 0 8px 8px 0;
}}
.md a {{ color: var(--accent) }}
.md a:hover {{ text-decoration: underline }}
.md table {{ border-collapse: collapse; margin: 0.75rem 0; width: 100% }}
.md th, .md td {{ border: 1px solid var(--border); padding: 0.4rem 0.75rem; text-align: left; font-size: 0.9rem }}
.md th {{ background: var(--bg-alt); font-weight: 600 }}
.md hr {{ border: none; border-top: 1px solid var(--border); margin: 1.5rem 0 }}
.md img {{ max-width: 100%; border-radius: 8px }}
</style>
</head>
<body>

<nav class="navbar">
<a href="/" class="navbar-brand">
<svg viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg"><rect width="80" height="80" rx="16" fill="#0ea5e9"/><text x="40" y="55" text-anchor="middle" fill="white" font-size="40" font-weight="800" font-family="Inter,sans-serif">C</text></svg>
Chub Registry
</a>
<div class="navbar-links">
<a href="https://chub.nrl.ai">Docs</a>
<a href="https://chub.nrl.ai/guide/getting-started">Get Started</a>
<a href="https://chub.nrl.ai/reference/cli">CLI Reference</a>
<a href="https://github.com/nrl-ai/chub">GitHub</a>
</div>
</nav>

<div class="wrap">
<header>
<h1><span>Chub</span> Content Registry</h1>
<p>Curated API docs and skills for AI coding agents</p>
</header>

<div class="stats">
<div class="stat"><b>{docs}</b><small>Docs</small></div>
<div class="stat"><b>{skills}</b><small>Skills</small></div>
<div class="stat"><b>{lang_count}</b><small>Languages</small></div>
</div>

<div class="search-bar">
<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
<input type="text" id="q" placeholder="Search docs and skills..." autofocus autocomplete="off">
</div>

<div class="filters" id="filters"></div>

<div class="results-info" id="info"></div>
<div class="grid" id="grid"></div>
<div class="pagination" id="pagination"></div>

<div class="install-hint">
<span class="prompt">$</span> <span class="cmd">npm install -g @nrl-ai/chub</span><br>
<span class="prompt">$</span> <span class="cmd">chub search "stripe payments"</span><br>
<span class="prompt">$</span> <span class="cmd">chub get openai/chat --lang python</span>
</div>

<footer>
Registry built {generated}<br>
<div class="links">
<a href="https://chub.nrl.ai">Website</a> &middot;
<a href="https://github.com/nrl-ai/chub">GitHub</a> &middot;
<a href="https://www.npmjs.com/package/@nrl-ai/chub">npm</a> &middot;
<a href="https://pypi.org/project/chub/">PyPI</a> &middot;
<a href="https://crates.io/crates/chub">crates.io</a> &middot;
<a href="/registry.json">API</a>
</div>
</footer>
</div>

<!-- Doc viewer modal -->
<div class="modal-overlay" id="modal">
<div class="modal">
<div class="modal-header">
<h2 id="modal-title">Loading...</h2>
<select id="modal-lang" style="display:none"></select>
<select id="modal-ver" style="display:none"></select>
<button class="modal-close" id="modal-close">&times;</button>
</div>
<div class="modal-body" id="modal-body"><div class="loading">Loading...</div></div>
</div>
</div>

<script>
const CATALOG={catalog};
const PER_PAGE=60;
let query='',activeLang='',page=0;

const $q=document.getElementById('q'),$grid=document.getElementById('grid'),
      $info=document.getElementById('info'),$filters=document.getElementById('filters'),
      $pg=document.getElementById('pagination'),
      $modal=document.getElementById('modal'),$modalTitle=document.getElementById('modal-title'),
      $modalBody=document.getElementById('modal-body'),$modalClose=document.getElementById('modal-close'),
      $modalLang=document.getElementById('modal-lang'),$modalVer=document.getElementById('modal-ver');

// Build language filter pills
const langs=[...new Set(CATALOG.flatMap(e=>e.langNames))].sort();
langs.forEach(l=>{{
  const b=document.createElement('span');
  b.className='pill';b.textContent=l;
  b.onclick=()=>{{activeLang=activeLang===l?'':l;render();
    document.querySelectorAll('.pill').forEach(p=>p.classList.toggle('active',p.textContent===activeLang))}};
  $filters.appendChild(b);
}});

function score(e,terms){{
  if(!terms.length)return 1;
  let s=0;const id=e.id.toLowerCase(),name=e.name.toLowerCase(),
    desc=e.description.toLowerCase(),tags=e.tags.join(' ').toLowerCase();
  for(const t of terms){{
    if(id.includes(t))s+=10;
    if(name.includes(t))s+=8;
    if(tags.includes(t))s+=4;
    if(desc.includes(t))s+=2;
  }}
  return s;
}}

function render(){{
  const terms=query.toLowerCase().split(/\s+/).filter(t=>t.length>0);
  let filtered=CATALOG.map(e=>({{e,s:score(e,terms)}})).filter(x=>x.s>0);
  if(terms.length)filtered.sort((a,b)=>b.s-a.s);
  if(activeLang)filtered=filtered.filter(x=>x.e.langNames.includes(activeLang));
  const total=filtered.length;
  const maxPage=Math.max(0,Math.ceil(total/PER_PAGE)-1);
  if(page>maxPage)page=maxPage;
  const slice=filtered.slice(page*PER_PAGE,(page+1)*PER_PAGE);
  $info.textContent=total===CATALOG.length?`Showing ${{slice.length}} of ${{total}} entries`:
    `${{total}} result${{total!==1?'s':''}} — showing ${{page*PER_PAGE+1}}-${{Math.min((page+1)*PER_PAGE,total)}}`;
  $grid.innerHTML=slice.map(x=>card(x.e)).join('');
  $pg.innerHTML=total>PER_PAGE?
    `<button onclick="page=Math.max(0,page-1);render()" ${{page===0?'disabled':''}}>Prev</button>`+
    `<span style="color:var(--muted);font-size:.85rem;padding:.4rem">Page ${{page+1}} / ${{maxPage+1}}</span>`+
    `<button onclick="page=Math.min(${{maxPage}},page+1);render()" ${{page>=maxPage?'disabled':''}}>Next</button>`:'';
  // Attach click handlers
  document.querySelectorAll('.entry[data-id]').forEach(el=>{{
    el.onclick=()=>openDoc(el.dataset.id);
  }});
}}

function card(e){{
  const badge=e.type==='skill'?'<span class="badge badge-skill">Skill</span>':'<span class="badge badge-doc">Doc</span>';
  const src=e.source!=='community'?`<span class="badge badge-src">${{e.source}}</span>`:'';
  const langTags=e.langNames.map(l=>`<span class="lang-tag">${{l}}</span>`).join('');
  const tags=e.tags.slice(0,5).map(t=>`<span class="tag">${{t}}</span>`).join('');
  return `<div class="entry" data-id="${{esc(e.id)}}" title="Click to view docs">
<div class="entry-head"><h3>${{esc(e.name)}}</h3>${{badge}}${{src}}</div>
<div class="entry-id">${{esc(e.id)}}</div>
<p>${{esc(e.description)}}</p>
<div class="entry-meta">${{langTags}}${{tags}}</div></div>`;
}}

function esc(s){{return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;')}}

// --- Doc viewer ---
let currentEntry=null;

function openDoc(id){{
  currentEntry=CATALOG.find(e=>e.id===id);
  if(!currentEntry)return;
  $modalTitle.textContent=currentEntry.name;
  $modalBody.innerHTML='<div class="loading">Loading...</div>';
  $modalBody.className='modal-body loading';
  $modal.classList.add('open');
  document.body.style.overflow='hidden';

  if(currentEntry.type==='skill'){{
    $modalLang.style.display='none';
    $modalVer.style.display='none';
    fetchAndRender(currentEntry.path+'/SKILL.md');
  }} else if(currentEntry.langs.length>0){{
    // Show lang selector
    $modalLang.innerHTML=currentEntry.langs.map(l=>`<option value="${{l.language}}">${{l.language}}</option>`).join('');
    $modalLang.style.display='inline-block';
    $modalLang.onchange=()=>updateVersions();
    updateVersions();
  }} else {{
    $modalBody.innerHTML='<div class="loading">No content available.</div>';
  }}
}}

function updateVersions(){{
  const lang=currentEntry.langs.find(l=>l.language===$modalLang.value);
  if(!lang)return;
  if(lang.versions.length>1){{
    $modalVer.innerHTML=lang.versions.map(v=>`<option value="${{v.path}}" ${{v.version===lang.recommended?'selected':''}}>${{v.version}}</option>`).join('');
    $modalVer.style.display='inline-block';
  }} else {{
    $modalVer.style.display='none';
  }}
  const ver=lang.versions.find(v=>v.version===lang.recommended)||lang.versions[0];
  if(ver)fetchAndRender(ver.path+'/DOC.md');
}}

$modalVer.onchange=function(){{fetchAndRender($modalVer.value+'/DOC.md')}};

function fetchAndRender(path){{
  $modalBody.innerHTML='<div class="loading">Loading...</div>';
  $modalBody.className='modal-body loading';
  fetch('/'+path).then(r=>{{
    if(!r.ok)throw new Error(r.status);
    return r.text();
  }}).then(text=>{{
    // Strip YAML frontmatter
    const stripped=text.replace(/^---[\s\S]*?---\s*/,'');
    $modalBody.innerHTML='<div class="md">'+renderMd(stripped)+'</div>';
    $modalBody.className='modal-body';
  }}).catch(()=>{{
    $modalBody.innerHTML='<div class="loading">Failed to load document.</div>';
    $modalBody.className='modal-body loading';
  }});
}}

function closeModal(){{
  $modal.classList.remove('open');
  document.body.style.overflow='';
}}
$modalClose.onclick=closeModal;
$modal.onclick=function(e){{if(e.target===$modal)closeModal()}};
document.addEventListener('keydown',function(e){{if(e.key==='Escape')closeModal()}});

// Simple markdown-to-HTML renderer
function renderMd(src){{
  let html='';
  const lines=src.split('\n');
  let i=0,inCode=false,codeBuf='',codeIndent=false;

  while(i<lines.length){{
    const line=lines[i];

    // Fenced code blocks
    if(!inCode && line.match(/^```/)){{
      inCode=true;codeBuf='';i++;continue;
    }}
    if(inCode){{
      if(line.match(/^```/)){{
        html+='<pre><code>'+esc(codeBuf)+'</code></pre>';
        inCode=false;i++;continue;
      }}
      codeBuf+=line+'\n';i++;continue;
    }}

    // Headings
    const hm=line.match(/^(#{{1,6}})\s+(.*)/);
    if(hm){{html+='<h'+hm[1].length+'>'+inline(hm[2])+'</h'+hm[1].length+'>';i++;continue}}

    // Horizontal rule
    if(line.match(/^(-{{3,}}|\*{{3,}}|_{{3,}})\s*$/)){{html+='<hr>';i++;continue}}

    // Blockquote
    if(line.match(/^>\s?/)){{
      let bq='';
      while(i<lines.length&&lines[i].match(/^>\s?/)){{bq+=lines[i].replace(/^>\s?/,'')+'\n';i++}}
      html+='<blockquote>'+renderMd(bq)+'</blockquote>';continue;
    }}

    // Unordered list
    if(line.match(/^\s*[-*+]\s/)){{
      html+='<ul>';
      while(i<lines.length&&lines[i].match(/^\s*[-*+]\s/)){{
        html+='<li>'+inline(lines[i].replace(/^\s*[-*+]\s/,''))+'</li>';i++;
      }}
      html+='</ul>';continue;
    }}

    // Ordered list
    if(line.match(/^\s*\d+\.\s/)){{
      html+='<ol>';
      while(i<lines.length&&lines[i].match(/^\s*\d+\.\s/)){{
        html+='<li>'+inline(lines[i].replace(/^\s*\d+\.\s/,''))+'</li>';i++;
      }}
      html+='</ol>';continue;
    }}

    // Table
    if(line.includes('|')&&i+1<lines.length&&lines[i+1].match(/^\|?\s*[-:]+/)){{
      const hdrs=parseTRow(line);
      i+=2; // skip header + separator
      html+='<table><thead><tr>'+hdrs.map(h=>'<th>'+inline(h)+'</th>').join('')+'</tr></thead><tbody>';
      while(i<lines.length&&lines[i].includes('|')){{
        const cells=parseTRow(lines[i]);
        html+='<tr>'+cells.map(c=>'<td>'+inline(c)+'</td>').join('')+'</tr>';i++;
      }}
      html+='</tbody></table>';continue;
    }}

    // Empty line
    if(!line.trim()){{i++;continue}}

    // Paragraph
    let para='';
    while(i<lines.length&&lines[i].trim()&&!lines[i].match(/^(#|```|>|\s*[-*+]\s|\s*\d+\.\s|---|\*\*\*|___|\|)/)){{
      para+=(para?' ':'')+lines[i];i++;
    }}
    if(para)html+='<p>'+inline(para)+'</p>';
  }}
  if(inCode)html+='<pre><code>'+esc(codeBuf)+'</code></pre>';
  return html;
}}

function inline(s){{
  return s
    .replace(/!\[([^\]]*)\]\(([^)]+)\)/g,'<img src="$2" alt="$1">')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g,'<a href="$2">$1</a>')
    .replace(/`([^`]+)`/g,'<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g,'<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g,'<em>$1</em>');
}}

function parseTRow(line){{
  return line.replace(/^\|/,'').replace(/\|$/,'').split('|').map(c=>c.trim());
}}

let debounce;
$q.addEventListener('input',()=>{{clearTimeout(debounce);debounce=setTimeout(()=>{{query=$q.value;page=0;render()}},120)}});
render();
</script>
</body>
</html>"##,
        docs = docs_count,
        skills = skills_count,
        lang_count = languages.len(),
        generated = generated,
        catalog = catalog_json,
    )
}

/// Get current time as ISO 8601 string.
fn now_iso8601() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Basic ISO 8601 without external crate
    // For a production build we'd use chrono, but this avoids the dependency
    let secs_per_day = 86400u64;
    let days = now / secs_per_day;
    let time_of_day = now % secs_per_day;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Compute year/month/day from days since epoch
    let (year, month, day) = days_to_date(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day).
pub fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
