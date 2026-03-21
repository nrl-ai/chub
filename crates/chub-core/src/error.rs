use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Content directory not found: {0}")]
    ContentDirNotFound(PathBuf),

    #[error("Missing frontmatter field '{field}' in {path}")]
    MissingFrontmatter { field: String, path: String },

    #[error("Duplicate entry id '{0}'")]
    DuplicateId(String),

    #[error("Build errors:\n{0}")]
    BuildErrors(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Entry not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
