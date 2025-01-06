use std::error::Error;
use std::fmt;
use std::sync::Once;

static INIT: Once = Once::new();

#[derive(Debug)]
pub enum JuliaError {
    InitializationError(String),
    CallError(String),
    InvalidData(String),
}

impl fmt::Display for JuliaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JuliaError::InitializationError(msg) => write!(f, "Julia initialization error: {}", msg),
            JuliaError::CallError(msg) => write!(f, "Julia call error: {}", msg),
            JuliaError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl Error for JuliaError {}

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
        pattern_learning: bool
    ) -> Result<Vec<u32>, JuliaError> {
        Self::initialize()?;
        
        // Call julia_optimize_tokens with configuration
        let result = unsafe {
            let ptr = tokens.as_ptr();
            let len = tokens.len() as i64;
            let config = ((use_gpu as u32) << 0) |
                        ((use_simd as u32) << 1) |
                        ((pattern_learning as u32) << 2);
                        
            let result_ptr = julia_optimize_tokens_config(
                ptr,
                len,
                config
            );
            
            if result_ptr.is_null() {
                return Err(JuliaError::CallError("Julia optimization failed".into()));
            }
            
            Vec::from_raw_parts(
                result_ptr as *mut u32,
                len as usize,
                len as usize
            )
        };
        
        Ok(result)
    }

    pub fn compress_batch_with_config(
        tokens: &[u32],
        rows: usize,
        cols: usize,
        use_gpu: bool,
        use_simd: bool,
        pattern_learning: bool
    ) -> Result<Vec<u32>, JuliaError> {
        Self::initialize()?;
        
        if tokens.len() != rows * cols {
            return Err(JuliaError::InvalidData(
                format!("Token length {} does not match dimensions {}x{}", tokens.len(), rows, cols)
            ));
        }
        
        // Call julia_compress_batch with configuration
        let result = unsafe {
            let ptr = tokens.as_ptr();
            let config = ((use_gpu as u32) << 0) |
                        ((use_simd as u32) << 1) |
                        ((pattern_learning as u32) << 2);
                        
            let result_ptr = julia_compress_batch_config(
                ptr,
                rows as i64,
                cols as i64,
                config
            );
            
            if result_ptr.is_null() {
                return Err(JuliaError::CallError("Julia compression failed".into()));
            }
            
            Vec::from_raw_parts(
                result_ptr as *mut u32,
                rows * cols,
                rows * cols
            )
        };
        
        Ok(result)
    }

    pub fn decompress_batch(
        tokens: &[u32],
        rows: usize,
        cols: usize
    ) -> Result<Vec<u32>, JuliaError> {
        Self::initialize()?;
        
        // Call julia_decompress_batch
        let result = unsafe {
            let ptr = tokens.as_ptr();
            let result_ptr = julia_decompress_batch(
                ptr,
                rows as i64,
                cols as i64
            );
            
            if result_ptr.is_null() {
                return Err(JuliaError::CallError("Julia decompression failed".into()));
            }
            
            Vec::from_raw_parts(
                result_ptr as *mut u32,
                rows * cols,
                rows * cols
            )
        };
        
        Ok(result)
    }
}

// FFI declarations for Julia functions
extern "C" {
    fn julia_optimize_tokens_config(
        ptr: *const u32,
        len: i64,
        config: u32
    ) -> *mut u32;
    
    fn julia_compress_batch_config(
        ptr: *const u32,
        rows: i64,
        cols: i64,
        config: u32
    ) -> *mut u32;
    
    fn julia_decompress_batch(
        ptr: *const u32,
        rows: i64,
        cols: i64
    ) -> *mut u32;
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