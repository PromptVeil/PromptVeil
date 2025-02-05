use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Julia runtime error: {0}")]
    JuliaError(String),

    #[error("Failed to initialize Julia runtime: {0}")]
    InitError(String),

    #[error("Invalid token sequence: {0}")]
    InvalidTokens(String),

    #[error("GPU error: {0}")]
    GpuError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Batch processing error: {0}")]
    BatchError(String),

    #[error("Async task error: {0}")]
    AsyncError(String),
}

pub type Result<T> = std::result::Result<T, CompressionError>; 