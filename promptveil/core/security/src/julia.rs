use std::error::Error;
use std::fmt;
use std::sync::Once;

static INIT: Once = Once::new();

#[derive(Debug)]
pub enum JuliaError {
    InitializationError(String),
    CompressionError(String),
    DecompressionError(String),
}

impl fmt::Display for JuliaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JuliaError::InitializationError(msg) => write!(f, "Julia initialization error: {}", msg),
            JuliaError::CompressionError(msg) => write!(f, "Julia compression error: {}", msg),
            JuliaError::DecompressionError(msg) => write!(f, "Julia decompression error: {}", msg),
        }
    }
}

impl Error for JuliaError {}

extern "C" {
    fn julia_optimize_tokens_config(
        tokens: *const u32,
        len: i64,
        use_gpu: bool,
        use_simd: bool,
        use_patterns: bool
    ) -> *mut u32;

    fn julia_compress_batch_config(
        tokens: *const u32,
        rows: i64,
        cols: i64,
        use_gpu: bool,
        use_simd: bool,
        use_patterns: bool
    ) -> *mut u32;

    fn julia_decompress_batch(
        tokens: *const u32,
        rows: i64,
        cols: i64
    ) -> *mut u32;
}

pub struct JuliaInterface;

impl JuliaInterface {
    pub fn initialize() -> Result<(), JuliaError> {
        INIT.call_once(|| {
            // Initialize Julia runtime and load PromptVeilCore
            // This would be implemented in the actual build
        });
        Ok(())
    }

    pub fn optimize_tokens_with_config(
        tokens: &[u32],
        use_gpu: bool,
        use_simd: bool,
        use_patterns: bool
    ) -> Result<Vec<u32>, JuliaError> {
        let result = unsafe {
            julia_optimize_tokens_config(
                tokens.as_ptr(),
                tokens.len() as i64,
                use_gpu,
                use_simd,
                use_patterns
            )
        };
        
        if result.is_null() {
            return Err(JuliaError::CompressionError("Julia optimization failed".into()));
        }
        
        let optimized = unsafe {
            Vec::from_raw_parts(
                result,
                tokens.len(),
                tokens.len()
            )
        };
        
        Ok(optimized)
    }

    pub fn compress_batch_with_config(
        tokens: &[u32],
        rows: usize,
        cols: usize,
        use_gpu: bool,
        use_simd: bool,
        use_patterns: bool
    ) -> Result<Vec<u32>, JuliaError> {
        let result = unsafe {
            julia_compress_batch_config(
                tokens.as_ptr(),
                rows as i64,
                cols as i64,
                use_gpu,
                use_simd,
                use_patterns
            )
        };
        
        if result.is_null() {
            return Err(JuliaError::CompressionError("Julia compression failed".into()));
        }
        
        let compressed = unsafe {
            Vec::from_raw_parts(
                result,
                rows * cols,
                rows * cols
            )
        };
        
        Ok(compressed)
    }

    pub fn decompress_batch(
        tokens: &[u32],
        rows: usize,
        cols: usize
    ) -> Result<Vec<u32>, JuliaError> {
        let result = unsafe {
            julia_decompress_batch(
                tokens.as_ptr(),
                rows as i64,
                cols as i64
            )
        };
        
        if result.is_null() {
            return Err(JuliaError::DecompressionError("Julia decompression failed".into()));
        }
        
        let decompressed = unsafe {
            Vec::from_raw_parts(
                result,
                rows * cols,
                rows * cols
            )
        };
        
        Ok(decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_julia_initialization() {
        assert!(JuliaInterface::initialize().is_ok());
    }

    #[test]
    fn test_optimize_tokens_config() {
        let tokens = vec![1, 2, 3, 4, 5];
        let result = JuliaInterface::optimize_tokens_with_config(
            &tokens,
            false,  // CPU only
            true,   // Use SIMD
            true    // Use pattern learning
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_compress_batch_config() {
        let tokens = vec![1, 2, 3, 4];
        let result = JuliaInterface::compress_batch_with_config(
            &tokens,
            2,      // rows
            2,      // cols
            false,  // CPU only
            true,   // Use SIMD
            true    // Use pattern learning
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_dimensions() {
        let tokens = vec![1, 2, 3];  // 3 elements
        let result = JuliaInterface::compress_batch_with_config(
            &tokens,
            2,      // rows
            2,      // cols (2x2=4 elements needed)
            false,
            true,
            true
        );
        assert!(matches!(result, Err(JuliaError::InvalidData(_))));
    }
} 