use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Index error: {0}")]
    Index(#[from] index_rs::IndexError),

    #[error("Security error: {0}")]
    Security(#[from] security_rs::SecurityError),

    #[error("Format error: {0}")]
    Format(#[from] format_rs::FormatError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Core initialization error: {0}")]
    InitializationError(String),

    #[error("Operation error: {0}")]
    OperationError(String),
} 