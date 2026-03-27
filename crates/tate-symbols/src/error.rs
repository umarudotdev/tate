use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum SymbolError {
    #[error("unsupported language: {ext}")]
    UnsupportedLanguage { ext: String },
    #[error("symbol `{name}` not found in {}", path.display())]
    SymbolNotFound {
        path: PathBuf,
        name: String,
        found: Vec<String>,
    },
    #[error("failed to parse {}", path.display())]
    ParseFailed { path: PathBuf },
    #[error("failed to read {}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
