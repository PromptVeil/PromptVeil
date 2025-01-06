use std::error::Error;
use std::fmt;
use crate::julia::JuliaInterface;

#[derive(Debug)]
pub enum CompressionError {
    JuliaError(String),
    InvalidInput(String),
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompressionError::JuliaError(msg) => write!(f, "Julia error: {}", msg),
            CompressionError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl Error for CompressionError {}

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub gpu_enabled: bool,
    pub simd_enabled: bool,
    pub pattern_learning: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            gpu_enabled: true,
            simd_enabled: true,
            pattern_learning: true,
        }
    }
}

pub struct CompressionStats {
    pub compression_ratio: f64,
    pub time_ms: f64,
    pub use_gpu: bool
}

pub fn compress_tokens(tokens: &[u32], config: Option<CompressionConfig>) -> Result<(Vec<u8>, CompressionStats), CompressionError> {
    let config = config.unwrap_or_default();
    let use_gpu = config.gpu_enabled;
    let start_time = std::time::Instant::now();
    
    // Validate input
    if tokens.is_empty() {
        return Err(CompressionError::InvalidInput("Empty token sequence".into()));
    }
    
    // Optimize tokens via Julia
    let optimized = match JuliaInterface::optimize_tokens_with_config(
        tokens,
        use_gpu,
        config.simd_enabled,
        config.pattern_learning
    ) {
        Ok(tokens) => tokens,
        Err(e) => return Err(CompressionError::JuliaError(e.to_string())),
    };
    
    // Convert to bytes with compression metadata
    let mut compressed = Vec::with_capacity(optimized.len() * 4 + 16);
    
    // Add compression header (version, flags, etc.)
    compressed.extend_from_slice(&[0x01u8, 0x00, 0x00, 0x00]); // Version 1
    let flags = ((use_gpu as u32) << 0) | 
                ((config.simd_enabled as u32) << 1) |
                ((config.pattern_learning as u32) << 2);
    compressed.extend_from_slice(&flags.to_le_bytes());
    
    // Add compressed data
    compressed.extend(optimized.iter()
        .flat_map(|&token| token.to_le_bytes().to_vec()));
    
    // Calculate statistics
    let stats = CompressionStats {
        compression_ratio: compressed.len() as f64 / (tokens.len() * 4) as f64,
        time_ms: start_time.elapsed().as_millis() as f64,
        use_gpu,
    };
    
    Ok((compressed, stats))
}

pub fn decompress_tokens(data: &[u8]) -> Result<Vec<u32>, CompressionError> {
    if data.len() < 8 {
        return Err(CompressionError::InvalidInput("Invalid compressed data".into()));
    }
    
    // Read header
    let version = u32::from_le_bytes(data[0..4].try_into().unwrap());
    if version != 1 {
        return Err(CompressionError::InvalidInput(format!("Unsupported version: {}", version)));
    }
    
    let flags = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let _used_gpu = (flags & 1) != 0;
    let _used_simd = (flags & 2) != 0;
    let _used_pattern_learning = (flags & 4) != 0;
    
    // Extract token data
    let tokens: Vec<u32> = data[8..].chunks_exact(4)
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
    let config = config.unwrap_or_default();
    let use_gpu = config.gpu_enabled;
    let start_time = std::time::Instant::now();
    
    // Validate input
    if tokens.len() != rows * cols {
        return Err(CompressionError::InvalidInput(
            format!("Token length {} does not match dimensions {}x{}", tokens.len(), rows, cols)
        ));
    }
    
    // Compress batch via Julia
    let compressed = match JuliaInterface::compress_batch_with_config(
        tokens,
        rows,
        cols,
        use_gpu,
        config.simd_enabled,
        config.pattern_learning
    ) {
        Ok(tokens) => tokens,
        Err(e) => return Err(CompressionError::JuliaError(e.to_string())),
    };
    
    // Convert to bytes
    let mut bytes = Vec::with_capacity(compressed.len() * 4);
    bytes.extend(compressed.iter()
        .flat_map(|&token| token.to_le_bytes().to_vec()));
    
    // Calculate statistics
    let stats = CompressionStats {
        compression_ratio: bytes.len() as f64 / (tokens.len() * 4) as f64,
        time_ms: start_time.elapsed().as_millis() as f64,
        use_gpu,
    };
    
    Ok((bytes, stats))
}

pub fn decompress_batch(
    tokens: &[u32],
    rows: usize,
    cols: usize
) -> Result<Vec<u32>, CompressionError> {
    // Validate input dimensions
    if tokens.is_empty() {
        return Err(CompressionError::InvalidInput("Empty token sequence".into()));
    }
    
    // Decompress via Julia
    match JuliaInterface::decompress_batch(tokens, rows, cols) {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(CompressionError::JuliaError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_decompression() {
        let tokens = vec![1, 2, 3, 4, 5];
        let (compressed, stats) = compress_tokens(&tokens, None).unwrap();
        let decompressed = decompress_tokens(&compressed).unwrap();
        
        assert_eq!(tokens, decompressed);
        assert!(stats.compression_ratio > 0.0);
        assert!(stats.time_ms >= 0.0);
    }

    #[test]
    fn test_batch_compression() {
        let tokens = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let rows = 2;
        let cols = 4;
        
        let (compressed, stats) = compress_batch(&tokens, rows, cols, None).unwrap();
        let decompressed = decompress_batch(&compressed, rows, cols).unwrap();
        
        assert_eq!(tokens, decompressed);
        assert!(stats.compression_ratio > 0.0);
        assert!(stats.time_ms >= 0.0);
    }
    
    #[test]
    fn test_compression_config() {
        let tokens = vec![1, 2, 3, 4, 5];
        let config = CompressionConfig {
            gpu_enabled: false,
            simd_enabled: true,
            pattern_learning: true,
        };
        
        let (compressed, stats) = compress_tokens(&tokens, Some(config)).unwrap();
        assert!(!stats.use_gpu);
        
        let decompressed = decompress_tokens(&compressed).unwrap();
        assert_eq!(tokens, decompressed);
    }
    
    #[test]
    fn test_invalid_input() {
        let result = compress_tokens(&[], None);
        assert!(matches!(result, Err(CompressionError::InvalidInput(_))));
        
        let result = compress_batch(&[1, 2, 3], 2, 2, None);
        assert!(matches!(result, Err(CompressionError::InvalidInput(_))));
    }
} 