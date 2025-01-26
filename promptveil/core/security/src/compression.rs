use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CompressionError {
    InvalidData(String),
    CompressionFailed(String),
    DecompressionFailed(String),
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompressionError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            CompressionError::CompressionFailed(msg) => write!(f, "Compression failed: {}", msg),
            CompressionError::DecompressionFailed(msg) => write!(f, "Decompression failed: {}", msg),
        }
    }
}

impl Error for CompressionError {} 