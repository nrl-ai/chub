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
fn generate_index_html(result: &BuildResult) -> String {
    // Build a compact JSON catalog for client-side search
    let mut entries = Vec::new();
    for doc in &result.registry.docs {
        let langs: Vec<&str> = doc.languages.iter().map(|l| l.language.as_str()).collect();
        let versions: Vec<&str> = doc
            .languages
            .iter()
            .flat_map(|l| l.versions.iter().map(|v| v.version.as_str()))
            .collect();
        entries.push(serde_json::json!({
            "id": doc.id,
            "name": doc.name,
            "description": doc.description,
            "source": doc.source,
            "tags": doc.tags,
            "type": "doc",
            "languages": langs,
            "versions": versions,
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
            "languages": [],
            "versions": [],
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
<style>
:root{{--bg:#0f172a;--card:#1e293b;--border:#334155;--text:#e2e8f0;--muted:#94a3b8;--dim:#64748b;--accent:#38bdf8;--accent2:#818cf8;--green:#34d399;--surface:#0f172a}}
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:var(--bg);color:var(--text);line-height:1.5}}
a{{color:var(--accent);text-decoration:none}}a:hover{{text-decoration:underline}}
.wrap{{max-width:1024px;margin:0 auto;padding:1.5rem}}
header{{text-align:center;padding:2.5rem 0 1.5rem}}
header h1{{font-size:2rem;font-weight:800;letter-spacing:-.025em}}
header h1 span{{color:var(--accent)}}
header p{{color:var(--muted);margin-top:.25rem;font-size:1.05rem}}
.stats{{display:flex;gap:1.5rem;justify-content:center;margin:1.25rem 0 2rem;flex-wrap:wrap}}
.stat{{background:var(--card);border:1px solid var(--border);border-radius:8px;padding:.75rem 1.25rem;text-align:center;min-width:100px}}
.stat b{{display:block;font-size:1.4rem;color:var(--accent);font-weight:700}}
.stat small{{color:var(--muted);font-size:.8rem}}
.search-bar{{position:relative;max-width:600px;margin:0 auto 1rem}}
.search-bar input{{width:100%;padding:.75rem 1rem .75rem 2.75rem;background:var(--card);border:1px solid var(--border);border-radius:8px;color:var(--text);font-size:1rem;outline:none;transition:border-color .15s}}
.search-bar input:focus{{border-color:var(--accent)}}
.search-bar svg{{position:absolute;left:.85rem;top:50%;transform:translateY(-50%);color:var(--muted);width:18px;height:18px}}
.filters{{display:flex;gap:.5rem;flex-wrap:wrap;justify-content:center;margin-bottom:1.5rem}}
.pill{{background:var(--card);border:1px solid var(--border);border-radius:20px;padding:.3rem .85rem;font-size:.8rem;color:var(--muted);cursor:pointer;transition:all .15s;user-select:none}}
.pill:hover,.pill.active{{background:var(--accent);color:var(--bg);border-color:var(--accent)}}
.results-info{{color:var(--dim);font-size:.85rem;margin-bottom:.75rem}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:.75rem;margin-bottom:2rem}}
.entry{{background:var(--card);border:1px solid var(--border);border-radius:8px;padding:1rem;transition:border-color .15s;display:flex;flex-direction:column}}
.entry:hover{{border-color:var(--accent)}}
.entry-head{{display:flex;align-items:center;gap:.5rem;margin-bottom:.4rem}}
.entry-head h3{{font-size:.95rem;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1}}
.badge{{font-size:.65rem;padding:.15rem .45rem;border-radius:4px;font-weight:600;text-transform:uppercase;flex-shrink:0}}
.badge-doc{{background:#1e3a5f;color:var(--accent)}}
.badge-skill{{background:#1a2e3d;color:var(--green)}}
.badge-src{{background:var(--surface);color:var(--muted);font-size:.6rem}}
.entry p{{color:var(--muted);font-size:.82rem;margin-bottom:.5rem;flex:1;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}}
.entry-meta{{display:flex;flex-wrap:wrap;gap:.3rem;margin-top:auto}}
.tag{{background:var(--surface);color:var(--dim);padding:.1rem .45rem;border-radius:3px;font-size:.7rem}}
.lang-tag{{background:#1e293b;color:var(--accent2);padding:.1rem .45rem;border-radius:3px;font-size:.7rem;border:1px solid #334155}}
.entry-id{{font-family:'SF Mono',Consolas,monospace;font-size:.75rem;color:var(--dim);margin-bottom:.25rem}}
footer{{text-align:center;padding:2rem 0;color:var(--dim);font-size:.8rem;border-top:1px solid var(--border)}}
footer .links{{margin-top:.5rem}}
.install-hint{{max-width:600px;margin:0 auto 2rem;background:var(--card);border:1px solid var(--border);border-radius:8px;padding:1rem 1.25rem;font-family:'SF Mono',Consolas,monospace;font-size:.85rem}}
.install-hint .prompt{{color:var(--dim)}}
.install-hint .cmd{{color:var(--accent)}}
.empty{{text-align:center;padding:3rem;color:var(--muted)}}
.pagination{{display:flex;justify-content:center;gap:.5rem;margin-bottom:2rem}}
.pagination button{{background:var(--card);border:1px solid var(--border);border-radius:6px;padding:.4rem .85rem;color:var(--text);font-size:.85rem;cursor:pointer}}
.pagination button:hover{{border-color:var(--accent)}}
.pagination button:disabled{{opacity:.4;cursor:default}}
.pagination button:disabled:hover{{border-color:var(--border)}}
</style>
</head>
<body>
<div class="wrap">
<header>
<h1><span>Chub</span> Content Registry</h1>
<p>Curated API docs for AI coding agents</p>
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
<a href="https://github.com/nrl-ai/chub">GitHub</a> · <a href="https://chub.nrl.ai">Docs</a> · <a href="https://www.npmjs.com/package/@nrl-ai/chub">npm</a> · <a href="https://pypi.org/project/chub/">PyPI</a> · <a href="/registry.json">API</a>
</div>
</footer>
</div>

<script>
const CATALOG={catalog};
const PER_PAGE=60;
let query='',activeLang='',page=0;

const $q=document.getElementById('q'),$grid=document.getElementById('grid'),
      $info=document.getElementById('info'),$filters=document.getElementById('filters'),
      $pg=document.getElementById('pagination');

// Build language filter pills
const langs=[...new Set(CATALOG.flatMap(e=>e.languages))].sort();
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
  if(activeLang)filtered=filtered.filter(x=>x.e.languages.includes(activeLang));
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
}}

function card(e){{
  const badge=e.type==='skill'?'<span class="badge badge-skill">Skill</span>':'<span class="badge badge-doc">Doc</span>';
  const src=e.source!=='community'?`<span class="badge badge-src">${{e.source}}</span>`:'';
  const langs=e.languages.map(l=>`<span class="lang-tag">${{l}}</span>`).join('');
  const tags=e.tags.slice(0,5).map(t=>`<span class="tag">${{t}}</span>`).join('');
  return `<div class="entry">
<div class="entry-head"><h3>${{esc(e.name)}}</h3>${{badge}}${{src}}</div>
<div class="entry-id">${{esc(e.id)}}</div>
<p>${{esc(e.description)}}</p>
<div class="entry-meta">${{langs}}${{tags}}</div></div>`;
}}

function esc(s){{return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')}}

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
