use std::error::Error;
use std::fmt;

use crate::julia;

#[derive(Debug)]
pub struct CompressionConfig {
    pub gpu_enabled: bool,
    pub simd_enabled: bool,
    pub pattern_learning: bool,
}

#[derive(Debug)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
}

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

pub fn compress_tokens(
    tokens: &[u32],
    config: Option<CompressionConfig>
) -> Result<(Vec<u8>, CompressionStats), CompressionError> {
    let config = config.unwrap_or(CompressionConfig {
        gpu_enabled: false,
        simd_enabled: true,
        pattern_learning: true,
    });

    let optimized = unsafe {
        julia::julia_optimize_tokens_config(
            tokens.as_ptr(),
            tokens.len() as i64,
            config.gpu_enabled,
            config.simd_enabled,
            config.pattern_learning
        )
    };

    if optimized.is_null() {
        return Err(CompressionError::CompressionFailed("Token optimization failed".into()));
    }

    let optimized = unsafe {
        Vec::from_raw_parts(
            optimized as *mut u32,
            tokens.len(),
            tokens.len()
        )
    };

    let bytes: Vec<u8> = optimized.iter()
        .flat_map(|&token| token.to_le_bytes().to_vec())
        .collect();

    let stats = CompressionStats {
        original_size: tokens.len() * 4,
        compressed_size: bytes.len(),
        compression_ratio: (tokens.len() * 4) as f64 / bytes.len() as f64,
    };

    Ok((bytes, stats))
}

pub fn decompress_tokens(data: &[u8]) -> Result<Vec<u32>, CompressionError> {
    if data.len() % 4 != 0 {
        return Err(CompressionError::InvalidData("Data length must be multiple of 4".into()));
    }

    let tokens: Vec<u32> = data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    Ok(tokens)
}

pub fn compress_batch(
    tokens: &[u32],
    rows: usize,
    cols: usize,
    config: Option<CompressionConfig>
) -> Result<(Vec<u8>, CompressionStats), CompressionError> {
    if tokens.len() != rows * cols {
        return Err(CompressionError::InvalidData(format!(
            "Token count {} does not match dimensions {}x{}",
            tokens.len(), rows, cols
        )));
    }

    let config = config.unwrap_or(CompressionConfig {
        gpu_enabled: false,
        simd_enabled: true,
        pattern_learning: true,
    });

    let compressed = unsafe {
        julia::julia_compress_batch_config(
            tokens.as_ptr(),
            rows as i64,
            cols as i64,
            config.gpu_enabled,
            config.simd_enabled,
            config.pattern_learning
        )
    };

    if compressed.is_null() {
        return Err(CompressionError::CompressionFailed("Batch compression failed".into()));
    }

    let compressed = unsafe {
        Vec::from_raw_parts(
            compressed as *mut u32,
            rows * cols,
            rows * cols
        )
    };

    let bytes: Vec<u8> = compressed.iter()
        .flat_map(|&token| token.to_le_bytes().to_vec())
        .collect();

    let stats = CompressionStats {
        original_size: tokens.len() * 4,
        compressed_size: bytes.len(),
        compression_ratio: (tokens.len() * 4) as f64 / bytes.len() as f64,
    };

    Ok((bytes, stats))
}

pub fn decompress_batch(
    tokens: &[u32],
    rows: usize,
    cols: usize
) -> Result<Vec<u32>, CompressionError> {
    if tokens.len() != rows * cols {
        return Err(CompressionError::InvalidData(format!(
            "Token count {} does not match dimensions {}x{}",
            tokens.len(), rows, cols
        )));
    }

    let decompressed = unsafe {
        julia::julia_decompress_batch(
            tokens.as_ptr(),
            rows as i64,
            cols as i64
        )
    };

    if decompressed.is_null() {
        return Err(CompressionError::DecompressionFailed("Batch decompression failed".into()));
    }

    let decompressed = unsafe {
        Vec::from_raw_parts(
            decompressed as *mut u32,
            rows * cols,
            rows * cols
        )
    };

    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config() {
        let tokens = vec![1, 2, 3, 4, 5];
        let config = CompressionConfig {
            gpu_enabled: false,
            simd_enabled: true,
            pattern_learning: true,
        };
        let result = compress_tokens(&tokens, Some(config));
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_compression() {
        let tokens = vec![1, 2, 3, 4];
        let result = compress_batch(&tokens, 2, 2, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_dimensions() {
        let tokens = vec![1, 2, 3];  // 3 elements
        let result = compress_batch(&tokens, 2, 2, None);  // 2x2=4 elements needed
        assert!(matches!(result, Err(CompressionError::InvalidData(_))));
    }
} 