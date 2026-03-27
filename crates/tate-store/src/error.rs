use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("database corrupted, recreating")]
    Corrupted,
    #[error("failed to write: {0}")]
    Write(#[source] rusqlite::Error),
    #[error("failed to read: {0}")]
    Read(#[source] rusqlite::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum DeckFileError {
    #[error("deck file not found: {0}")]
    NotFound(PathBuf),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("invalid config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid regex pattern: {pattern}")]
    InvalidPattern { pattern: String },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
