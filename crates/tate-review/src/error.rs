use tate_store::error::{DeckFileError, StorageError};

#[derive(thiserror::Error, Debug)]
pub enum ReviewError {
    #[error("{0}")]
    Storage(#[from] StorageError),
    #[error("{0}")]
    DeckFile(#[from] DeckFileError),
    #[error("{0}")]
    Other(String),
}
