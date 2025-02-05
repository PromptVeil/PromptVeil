use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Text index error: {0}")]
    TextIndex(String),

    #[error("Vector index error: {0}")]
    VectorIndex(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid vector dimensions: expected {expected}, got {actual}")]
    InvalidVectorDimensions {
        expected: usize,
        actual: usize,
    },

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Index initialization error: {0}")]
    InitializationError(String),

    #[error("Index operation error: {0}")]
    OperationError(String),
} 