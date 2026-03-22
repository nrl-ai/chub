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

/// Static assets for the CDN index page, embedded at compile time.
const INDEX_TEMPLATE: &str = include_str!("static/index.html");
const INDEX_STYLE: &str = include_str!("static/style.css");
const INDEX_SCRIPT: &str = include_str!("static/script.js");

/// Generate an index.html landing page with search for the CDN root.
/// Matches the VitePress website theme (Inter font, sky blue brand colors, dark/light toggle).
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

    INDEX_TEMPLATE
        .replace("{style}", INDEX_STYLE)
        .replace("{script}", INDEX_SCRIPT)
        .replace("{docs}", &docs_count.to_string())
        .replace("{skills}", &skills_count.to_string())
        .replace("{lang_count}", &languages.len().to_string())
        .replace("{generated}", generated)
        .replace("{catalog}", &catalog_json)
}

/// Get current time as ISO 8601 string.
fn now_iso8601() -> String {
    crate::util::now_iso8601()
}

/// Convert days since Unix epoch to (year, month, day).
/// Delegates to [`crate::util::days_to_date`] — kept here for backward compatibility.
pub fn days_to_date(days: u64) -> (u64, u64, u64) {
    crate::util::days_to_date(days)
}
