use std::fs;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::cache::{
    get_source_data_dir, get_source_dir, get_source_registry_path, get_source_search_index_path,
    read_cached_doc, read_meta, save_cached_doc, save_source_registry,
    should_fetch_remote_registry, write_meta,
};
use crate::config::{load_config, SourceConfig};
use crate::error::{Error, Result};

const FETCH_TIMEOUT_SECS: u64 = 30;

/// Maximum size for registry.json downloads (50 MB).
const MAX_REGISTRY_SIZE: usize = 50 * 1024 * 1024;
/// Maximum size for bundle.tar.gz downloads (500 MB).
const MAX_BUNDLE_SIZE: usize = 500 * 1024 * 1024;
/// Maximum size for individual doc downloads (10 MB).
const MAX_DOC_SIZE: usize = 10 * 1024 * 1024;

/// Fetch registry for a single remote source.
pub async fn fetch_remote_registry(source: &SourceConfig, force: bool) -> Result<()> {
    if !force && !should_fetch_remote_registry(&source.name) {
        return Ok(());
    }

    let url = format!(
        "{}/registry.json",
        source.url.as_deref().unwrap_or_default()
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| Error::Config(format!("HTTP client error: {}", e)))?;

    let res = client.get(&url).send().await.map_err(|e| {
        Error::Config(format!(
            "Failed to fetch registry from {}: {}",
            source.name, e
        ))
    })?;

    if !res.status().is_success() {
        return Err(Error::Config(format!(
            "Failed to fetch registry from {}: {} {}",
            source.name,
            res.status().as_u16(),
            res.status().canonical_reason().unwrap_or("")
        )));
    }

    let data = read_response_limited(res, MAX_REGISTRY_SIZE, "registry").await?;

    save_source_registry(&source.name, &data);
    crate::cache::touch_source_meta(&source.name);
    Ok(())
}

/// Fetch registries for all configured sources.
pub async fn fetch_all_registries(force: bool) -> Vec<FetchError> {
    let config = load_config();
    let mut errors = Vec::new();

    for source in &config.sources {
        if source.path.is_some() {
            continue;
        }
        if let Err(e) = fetch_remote_registry(source, force).await {
            errors.push(FetchError {
                source: source.name.clone(),
                error: e.to_string(),
            });
        }
    }

    errors
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FetchError {
    pub source: String,
    pub error: String,
}

/// Download full bundle for a remote source.
pub async fn fetch_full_bundle(source_name: &str) -> Result<()> {
    let config = load_config();
    let source = config
        .sources
        .iter()
        .find(|s| s.name == source_name)
        .ok_or_else(|| Error::Config(format!("Source \"{}\" not found", source_name)))?;

    if source.path.is_some() {
        return Err(Error::Config(format!(
            "Source \"{}\" is not a remote source.",
            source_name
        )));
    }

    let url = format!(
        "{}/bundle.tar.gz",
        source.url.as_deref().unwrap_or_default()
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| Error::Config(format!("HTTP client error: {}", e)))?;

    let res = client.get(&url).send().await.map_err(|e| {
        Error::Config(format!(
            "Failed to fetch bundle from {}: {}",
            source_name, e
        ))
    })?;

    if !res.status().is_success() {
        return Err(Error::Config(format!(
            "Failed to fetch bundle from {}: {} {}",
            source_name,
            res.status().as_u16(),
            res.status().canonical_reason().unwrap_or("")
        )));
    }

    let bytes = read_response_bytes_limited(res, MAX_BUNDLE_SIZE, "bundle").await?;

    let source_dir = get_source_dir(source_name);
    fs::create_dir_all(&source_dir)?;

    // Use a unique temp file name to avoid predictable-name attacks
    let tmp_name = format!("bundle.{}.tar.gz", std::process::id());
    let tmp_path = source_dir.join(&tmp_name);
    fs::write(&tmp_path, &bytes)?;

    // Extract tar.gz with path validation
    let data_dir = get_source_data_dir(source_name);
    fs::create_dir_all(&data_dir)?;

    let file = fs::File::open(&tmp_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    // Validate each entry path before extraction to prevent path traversal
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let entry_path = entry.path()?.to_path_buf();
        let entry_str = entry_path.to_string_lossy();

        // Reject absolute paths, paths with "..", and paths with backslashes
        if entry_path.is_absolute() || entry_str.contains("..") || entry_str.contains('\\') {
            return Err(Error::Config(format!(
                "Malicious tar entry rejected: \"{}\"",
                entry_str
            )));
        }

        let target = data_dir.join(&entry_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        entry.unpack(&target)?;
    }

    // Copy registry.json from extracted bundle if present
    let extracted_registry = data_dir.join("registry.json");
    if extracted_registry.exists() {
        let reg_data = fs::read_to_string(&extracted_registry)?;
        fs::write(get_source_registry_path(source_name), &reg_data)?;
    }

    // Copy search-index.json from extracted bundle if present
    let extracted_search_index = data_dir.join("search-index.json");
    if extracted_search_index.exists() {
        let idx_data = fs::read_to_string(&extracted_search_index)?;
        fs::write(get_source_search_index_path(source_name), &idx_data)?;
    } else {
        let _ = fs::remove_file(get_source_search_index_path(source_name));
    }

    // Update meta
    let mut meta = read_meta(source_name);
    meta.last_updated = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    );
    meta.full_bundle = true;
    write_meta(source_name, &meta);

    // Clean up temp file
    let _ = fs::remove_file(&tmp_path);

    Ok(())
}

/// Fetch a single doc. Source must have name + (url or path).
pub async fn fetch_doc(source: &SourceConfig, doc_path: &str) -> Result<String> {
    // Local source: read directly
    if let Some(ref local_path) = source.path {
        let full_path = PathBuf::from(local_path).join(doc_path);
        if !full_path.exists() {
            return Err(Error::NotFound(format!(
                "File not found: {}",
                full_path.display()
            )));
        }
        return Ok(fs::read_to_string(&full_path)?);
    }

    // Remote source: check cache first
    if let Some(content) = read_cached_doc(&source.name, doc_path) {
        return Ok(content);
    }

    // Fetch from CDN
    let url = format!("{}/{}", source.url.as_deref().unwrap_or_default(), doc_path);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| Error::Config(format!("HTTP client error: {}", e)))?;

    let res = client.get(&url).send().await.map_err(|e| {
        Error::Config(format!(
            "Failed to fetch {} from {}: {}",
            doc_path, source.name, e
        ))
    })?;

    if !res.status().is_success() {
        return Err(Error::Config(format!(
            "Failed to fetch {} from {}: {} {}",
            doc_path,
            source.name,
            res.status().as_u16(),
            res.status().canonical_reason().unwrap_or("")
        )));
    }

    let content = read_response_limited(res, MAX_DOC_SIZE, "doc").await?;

    // Cache locally
    save_cached_doc(&source.name, doc_path, &content);

    Ok(content)
}

/// Fetch all files in an entry directory. Returns vec of (filename, content).
pub async fn fetch_doc_full(
    source: &SourceConfig,
    base_path: &str,
    files: &[String],
) -> Result<Vec<(String, String)>> {
    let mut results = Vec::new();
    for file in files {
        let file_path = format!("{}/{}", base_path, file);
        let content = fetch_doc(source, &file_path).await?;
        results.push((file.clone(), content));
    }
    Ok(results)
}

/// Read a text response body with a size limit.
async fn read_response_limited(
    res: reqwest::Response,
    max_bytes: usize,
    kind: &str,
) -> Result<String> {
    // Check Content-Length header first (if present)
    if let Some(len) = res.content_length() {
        if len as usize > max_bytes {
            return Err(Error::Config(format!(
                "Response too large for {} ({} bytes, max {})",
                kind, len, max_bytes
            )));
        }
    }

    let bytes = read_response_bytes_limited(res, max_bytes, kind).await?;
    String::from_utf8(bytes)
        .map_err(|_| Error::Config(format!("Invalid UTF-8 in {} response", kind)))
}

/// Read a binary response body with a size limit.
async fn read_response_bytes_limited(
    res: reqwest::Response,
    max_bytes: usize,
    kind: &str,
) -> Result<Vec<u8>> {
    // Check Content-Length header first (if present)
    if let Some(len) = res.content_length() {
        if len as usize > max_bytes {
            return Err(Error::Config(format!(
                "Response too large for {} ({} bytes, max {})",
                kind, len, max_bytes
            )));
        }
    }

    let bytes = res
        .bytes()
        .await
        .map_err(|e| Error::Config(format!("Failed to read {} body: {}", kind, e)))?;

    if bytes.len() > max_bytes {
        return Err(Error::Config(format!(
            "Response too large for {} ({} bytes, max {})",
            kind,
            bytes.len(),
            max_bytes
        )));
    }

    Ok(bytes.to_vec())
}

/// Verify fetched content against an expected SHA-256 hash.
/// Returns Ok(content) if hash matches or no hash was provided.
/// Returns Err if hash mismatch (content tampering detected).
pub fn verify_content_hash(
    content: &str,
    expected_hash: Option<&str>,
    doc_path: &str,
) -> Result<()> {
    if let Some(expected) = expected_hash {
        let actual = format!("{:x}", Sha256::digest(content.as_bytes()));
        if actual != expected {
            return Err(Error::Config(format!(
                "Content integrity check failed for \"{}\": expected hash {}, got {}",
                doc_path, expected, actual
            )));
        }
    }
    Ok(())
}

/// Ensure at least one registry is available.
pub async fn ensure_registry() -> Result<()> {
    if crate::cache::has_any_registry() {
        // Auto-refresh stale remote registries (best-effort)
        let config = load_config();
        for source in &config.sources {
            if source.path.is_some() {
                continue;
            }
            if should_fetch_remote_registry(&source.name) {
                let _ = fetch_remote_registry(source, false).await;
            }
        }
        return Ok(());
    }

    // No registries at all — must download from remote
    let errors = fetch_all_registries(true).await;
    if !errors.is_empty() && !crate::cache::has_any_registry() {
        return Err(Error::Config(format!(
            "Failed to fetch registries: {}",
            errors
                .iter()
                .map(|e| format!("{}: {}", e.source, e.error))
                .collect::<Vec<_>>()
                .join("; ")
        )));
    }

    Ok(())
}
