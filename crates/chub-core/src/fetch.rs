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

    let data = res
        .text()
        .await
        .map_err(|e| Error::Config(format!("Failed to read registry body: {}", e)))?;

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

    let bytes = res
        .bytes()
        .await
        .map_err(|e| Error::Config(format!("Failed to read bundle body: {}", e)))?;

    let source_dir = get_source_dir(source_name);
    fs::create_dir_all(&source_dir)?;

    let tmp_path = source_dir.join("bundle.tar.gz");
    fs::write(&tmp_path, &bytes)?;

    // Extract tar.gz
    let data_dir = get_source_data_dir(source_name);
    fs::create_dir_all(&data_dir)?;

    let file = fs::File::open(&tmp_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(&data_dir)?;

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

    let content = res
        .text()
        .await
        .map_err(|e| Error::Config(format!("Failed to read body: {}", e)))?;

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
