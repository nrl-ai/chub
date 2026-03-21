use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{chub_dir, load_config, SourceConfig};
use crate::types::{Registry, SearchIndex};

/// Default maximum cache size in bytes (100 MB).
const DEFAULT_MAX_CACHE_BYTES: u64 = 100 * 1024 * 1024;

/// Threshold above which cached docs are gzip-compressed.
const GZIP_THRESHOLD: usize = 10 * 1024;

/// Metadata stored alongside each cached source.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceMeta {
    #[serde(rename = "lastUpdated", default)]
    pub last_updated: Option<u64>,
    #[serde(rename = "fullBundle", default)]
    pub full_bundle: bool,
    #[serde(rename = "bundledSeed", default)]
    pub bundled_seed: bool,
}

pub fn get_source_dir(source_name: &str) -> PathBuf {
    chub_dir().join("sources").join(source_name)
}

pub fn get_source_data_dir(source_name: &str) -> PathBuf {
    get_source_dir(source_name).join("data")
}

pub fn get_source_meta_path(source_name: &str) -> PathBuf {
    get_source_dir(source_name).join("meta.json")
}

pub fn get_source_registry_path(source_name: &str) -> PathBuf {
    get_source_dir(source_name).join("registry.json")
}

pub fn get_source_search_index_path(source_name: &str) -> PathBuf {
    get_source_dir(source_name).join("search-index.json")
}

pub fn read_meta(source_name: &str) -> SourceMeta {
    let path = get_source_meta_path(source_name);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn write_meta(source_name: &str, meta: &SourceMeta) {
    let dir = get_source_dir(source_name);
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(
        get_source_meta_path(source_name),
        serde_json::to_string_pretty(meta).unwrap_or_default(),
    );
}

pub fn is_source_cache_fresh(source_name: &str) -> bool {
    let meta = read_meta(source_name);
    let last = match meta.last_updated {
        Some(ts) if ts > 0 => ts,
        _ => return false,
    };
    let config = load_config();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let age_secs = (now.saturating_sub(last)) / 1000;
    age_secs < config.refresh_interval
}

/// Returns true if we should fetch the remote registry for this source.
/// Inverse of fresh check, but also returns true when no registry exists at all.
pub fn should_fetch_remote_registry(source_name: &str) -> bool {
    !is_source_cache_fresh(source_name) || !get_source_registry_path(source_name).exists()
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Load cached/local registry for a single source.
pub fn load_source_registry(source: &SourceConfig) -> Option<Registry> {
    let reg_path = if let Some(ref p) = source.path {
        PathBuf::from(p).join("registry.json")
    } else {
        get_source_registry_path(&source.name)
    };
    if !reg_path.exists() {
        return None;
    }
    let data = fs::read_to_string(&reg_path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Load BM25 search index for a single source.
pub fn load_search_index(source: &SourceConfig) -> Option<SearchIndex> {
    // For local sources, look in the source path
    if let Some(ref p) = source.path {
        let index_path = PathBuf::from(p).join("search-index.json");
        if index_path.exists() {
            return fs::read_to_string(&index_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok());
        }
        return None;
    }
    // For remote sources, check the per-source search index file
    let index_path = get_source_search_index_path(&source.name);
    if !index_path.exists() {
        return None;
    }
    fs::read_to_string(&index_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Cache stats for display.
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub exists: bool,
    pub sources: Vec<SourceStat>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SourceStat {
    #[serde(rename = "local")]
    Local { name: String, path: String },
    #[serde(rename = "remote")]
    Remote {
        name: String,
        #[serde(rename = "hasRegistry")]
        has_registry: bool,
        #[serde(rename = "lastUpdated")]
        last_updated: Option<String>,
        #[serde(rename = "fullBundle")]
        full_bundle: bool,
        #[serde(rename = "fileCount")]
        file_count: usize,
        #[serde(rename = "dataSize")]
        data_size: u64,
    },
}

pub fn get_cache_stats() -> CacheStats {
    let chub = chub_dir();
    if !chub.exists() {
        return CacheStats {
            exists: false,
            sources: vec![],
        };
    }

    let config = load_config();
    let mut sources = Vec::new();

    for source in &config.sources {
        if let Some(ref p) = source.path {
            sources.push(SourceStat::Local {
                name: source.name.clone(),
                path: p.clone(),
            });
            continue;
        }

        let meta = read_meta(&source.name);
        let data_dir = get_source_data_dir(&source.name);
        let (file_count, data_size) = dir_stats(&data_dir);

        let last_updated = meta.last_updated.map(|ts| {
            // Convert millis to ISO 8601
            let secs = ts / 1000;
            let days = secs / 86400;
            let tod = secs % 86400;
            let (y, m, d) = crate::util::days_to_date(days);
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z",
                y,
                m,
                d,
                tod / 3600,
                (tod % 3600) / 60,
                tod % 60
            )
        });

        sources.push(SourceStat::Remote {
            name: source.name.clone(),
            has_registry: get_source_registry_path(&source.name).exists(),
            last_updated,
            full_bundle: meta.full_bundle,
            file_count,
            data_size,
        });
    }

    CacheStats {
        exists: true,
        sources,
    }
}

fn dir_stats(dir: &Path) -> (usize, u64) {
    let mut count = 0usize;
    let mut size = 0u64;
    if dir.exists() {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                count += 1;
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    (count, size)
}

/// Clear the cache (preserves config.yaml).
pub fn clear_cache() {
    let chub = chub_dir();
    let config_path = chub.join("config.yaml");
    let config_content = fs::read_to_string(&config_path).ok();

    let _ = fs::remove_dir_all(&chub);

    if let Some(content) = config_content {
        let _ = fs::create_dir_all(&chub);
        let _ = fs::write(&config_path, content);
    }
}

/// Save a fetched registry to the source cache.
pub fn save_source_registry(source_name: &str, data: &str) {
    let dir = get_source_dir(source_name);
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(get_source_registry_path(source_name), data);
}

/// Update the last_updated timestamp for a source.
pub fn touch_source_meta(source_name: &str) {
    let mut meta = read_meta(source_name);
    meta.last_updated = Some(now_millis());
    write_meta(source_name, &meta);
}

/// Save a fetched doc to the source data cache.
/// Content larger than 10 KB is gzip-compressed (saved as `.gz`).
pub fn save_cached_doc(source_name: &str, doc_path: &str, content: &str) {
    let base_path = get_source_data_dir(source_name).join(doc_path);
    if let Some(parent) = base_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if content.len() > GZIP_THRESHOLD {
        let gz_path = PathBuf::from(format!("{}.gz", base_path.display()));
        if let Ok(file) = fs::File::create(&gz_path) {
            let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
            let _ = encoder.write_all(content.as_bytes());
            let _ = encoder.finish();
            // Remove uncompressed version if it exists
            let _ = fs::remove_file(&base_path);
            return;
        }
    }
    let _ = fs::write(&base_path, content);
}

/// Read a cached doc if it exists (handles both plain and gzip-compressed).
pub fn read_cached_doc(source_name: &str, doc_path: &str) -> Option<String> {
    let base_path = get_source_data_dir(source_name).join(doc_path);

    // Check for gzip-compressed version first
    let gz_path = PathBuf::from(format!("{}.gz", base_path.display()));
    if gz_path.exists() {
        if let Ok(file) = fs::File::open(&gz_path) {
            let mut decoder = flate2::read::GzDecoder::new(file);
            let mut content = String::new();
            if decoder.read_to_string(&mut content).is_ok() {
                return Some(content);
            }
        }
    }

    // Fall back to plain file
    fs::read_to_string(&base_path).ok()
}

/// Evict cached data from the oldest sources until total cache size is under the limit.
/// Returns the number of bytes freed.
pub fn evict_lru_cache(max_bytes: Option<u64>) -> u64 {
    let max = max_bytes.unwrap_or(DEFAULT_MAX_CACHE_BYTES);
    let config = load_config();
    let chub = chub_dir();

    if !chub.exists() {
        return 0;
    }

    // Collect (source_name, data_size, last_updated) for remote sources
    let mut source_stats: Vec<(String, u64, u64)> = Vec::new();
    let mut total_size: u64 = 0;

    for source in &config.sources {
        if source.path.is_some() {
            continue;
        }
        let data_dir = get_source_data_dir(&source.name);
        let (_, size) = dir_stats(&data_dir);
        let meta = read_meta(&source.name);
        let last = meta.last_updated.unwrap_or(0);
        total_size += size;
        source_stats.push((source.name.clone(), size, last));
    }

    if total_size <= max {
        return 0;
    }

    // Sort by last_updated ascending (oldest first)
    source_stats.sort_by_key(|s| s.2);

    let mut freed: u64 = 0;
    for (name, size, _) in &source_stats {
        if total_size - freed <= max {
            break;
        }
        let data_dir = get_source_data_dir(name);
        if data_dir.exists() {
            let _ = fs::remove_dir_all(&data_dir);
            freed += size;
        }
    }

    freed
}

/// Check if any source has a registry available.
pub fn has_any_registry() -> bool {
    let config = load_config();
    for source in &config.sources {
        if let Some(ref p) = source.path {
            if PathBuf::from(p).join("registry.json").exists() {
                return true;
            }
        } else if get_source_registry_path(&source.name).exists() {
            return true;
        }
    }
    false
}
