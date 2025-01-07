use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use std::convert::TryInto;
use std::env;

mod compression;
mod security;
mod julia;

use crate::compression::CompressionConfig;

#[no_mangle]
pub extern "C" fn __rust_julia_init() {
    eprintln!("DEBUG Rust: Initializing Julia runtime");
    
    // This function will be called when the DLL is loaded
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .expect("Failed to get executable directory");
    
    // Temporarily add DLL directory to PATH
    let old_path = env::var_os("PATH").unwrap_or_default();
    let mut new_path = exe_dir.as_os_str().to_owned();
    new_path.push(";");
    new_path.push(&old_path);
    env::set_var("PATH", new_path);
    
    // Get Julia core directory
    let julia_dir = env::var("PROMPTVEIL_CORE_DIR")
        .expect("PROMPTVEIL_CORE_DIR must be set");
    eprintln!("DEBUG Rust: Julia core directory: {}", julia_dir);
    
    // Initialize Julia runtime
    unsafe {
        eprintln!("DEBUG Rust: Calling julia_init");
        julia::julia_init();
        eprintln!("DEBUG Rust: Julia runtime initialized");
    }
}

#[no_mangle]
pub extern "C" fn __rust_julia_cleanup() {
    unsafe {
        eprintln!("DEBUG Rust: Cleaning up Julia runtime");
        julia::julia_cleanup();
        eprintln!("DEBUG Rust: Julia runtime cleaned up");
    }
}

#[pyfunction]
fn compress_tokens(data: &[u8]) -> PyResult<Vec<u8>> {
    let tokens = bytes_to_tokens(data)?;
    
    // Default configuration with GPU enabled
    let config = Some(CompressionConfig {
        gpu_enabled: true,      // Try to use GPU by default
        simd_enabled: true,     // Use SIMD when available
        pattern_learning: true  // Enable pattern learning
    });
    
    compression::compress_tokens(&tokens, config)
        .map(|(compressed, _stats)| compressed)
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
}

#[pyfunction]
fn decompress_tokens(data: &[u8]) -> PyResult<Vec<u8>> {
    compression::decompress_tokens(data)
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
        .map(|tokens| {
            // Convert u32 tokens back to bytes
            tokens.iter()
                .flat_map(|&token| token.to_le_bytes().to_vec())
                .collect()
        })
}

#[pyfunction]
fn compress_batch(data: &[u8], rows: usize, cols: usize) -> PyResult<Vec<u8>> {
    let tokens = bytes_to_tokens(data)?;
    
    // Default configuration with GPU enabled
    let config = Some(CompressionConfig {
        gpu_enabled: true,      // Try to use GPU by default
        simd_enabled: true,     // Use SIMD when available
        pattern_learning: true  // Enable pattern learning
    });
    
    compression::compress_batch(&tokens, rows, cols, config)
        .map(|(compressed, _stats)| compressed)
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
}

#[pyfunction]
fn decompress_batch(data: &[u8], rows: usize, cols: usize) -> PyResult<Vec<u8>> {
    // Convert bytes to u32 tokens
    let tokens: Vec<u32> = data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    compression::decompress_batch(&tokens, rows, cols)
        .map(|decompressed| {
            // Convert u32 tokens back to bytes
            decompressed.iter()
                .flat_map(|&token| token.to_le_bytes().to_vec())
                .collect()
        })
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
}

#[pyfunction]
fn encrypt(data: &[u8], key: &[u8]) -> PyResult<Vec<u8>> {
    let key_array: [u8; 32] = key.try_into()
        .map_err(|_| PyErr::new::<PyIOError, _>("Key must be 32 bytes"))?;

    security::encrypt(data, &key_array)
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
}

#[pyfunction]
fn decrypt(data: &[u8], key: &[u8]) -> PyResult<Vec<u8>> {
    let key_array: [u8; 32] = key.try_into()
        .map_err(|_| PyErr::new::<PyIOError, _>("Key must be 32 bytes"))?;

    security::decrypt(data, &key_array)
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
}

#[pyfunction]
fn generate_key() -> PyResult<Vec<u8>> {
    security::generate_key()
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
        .map(|key| key.to_vec())
}

// Julia FFI functions
#[pyfunction]
fn julia_optimize_tokens_config(
    tokens: Vec<u32>,
    len: i64,
    use_gpu: bool,
    use_simd: bool,
    use_patterns: bool
) -> PyResult<Vec<u32>> {
    let result = unsafe {
        let ptr = tokens.as_ptr();
        let result_ptr = julia::julia_optimize_tokens_config(ptr, len, use_gpu, use_simd, use_patterns);
        Vec::from_raw_parts(result_ptr as *mut u32, len as usize, len as usize)
    };
    Ok(result)
}

#[pyfunction]
fn julia_compress_batch_config(
    tokens: Vec<Vec<u32>>,
    rows: i64,
    cols: i64,
    use_gpu: bool,
    use_simd: bool,
    use_patterns: bool
) -> PyResult<Vec<Vec<u32>>> {
    // Print immediately to see if we reach this point
    eprintln!("DEBUG Rust: Function called");
    
    // Validate input before any processing
    if tokens.is_empty() {
        eprintln!("DEBUG Rust: Empty input tokens");
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Empty input tokens"));
    }
    
    if (rows as usize) != tokens.len() {
        eprintln!("DEBUG Rust: Row count mismatch - expected {}, got {}", rows, tokens.len());
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Row count mismatch - expected {}, got {}", rows, tokens.len())
        ));
    }
    
    if let Some(first_row) = tokens.get(0) {
        if (cols as usize) != first_row.len() {
            eprintln!("DEBUG Rust: Column count mismatch - expected {}, got {}", cols, first_row.len());
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Column count mismatch - expected {}, got {}", cols, first_row.len())
            ));
        }
    }
    
    eprintln!("DEBUG Rust: Input validation passed");
    eprintln!("DEBUG Rust: Input dimensions: {}x{}", rows, cols);
    eprintln!("DEBUG Rust: First row length: {}", tokens.get(0).map_or(0, |row| row.len()));
    eprintln!("DEBUG Rust: Total rows received: {}", tokens.len());
    
    let flat_tokens: Vec<u32> = tokens.into_iter().flatten().collect();
    eprintln!("DEBUG Rust: Flattened array length: {}", flat_tokens.len());
    println!("DEBUG Rust: First few values: {:?}", &flat_tokens.iter().take(10).collect::<Vec<_>>());
    
    let result = unsafe {
        println!("DEBUG Rust: Calling Julia FFI function");
        let ptr = flat_tokens.as_ptr();
        println!("DEBUG Rust: Input pointer: {:?}", ptr);
        let result_ptr = julia::julia_compress_batch_config(ptr, rows, cols, use_gpu, use_simd, use_patterns);
        println!("DEBUG Rust: Result pointer: {:?}", result_ptr);
        
        if result_ptr.is_null() {
            println!("DEBUG Rust: Julia returned null pointer");
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Julia compression returned null pointer"));
        }
        
        println!("DEBUG Rust: Converting result to Vec");
        Vec::from_raw_parts(result_ptr as *mut u32, (rows * cols) as usize, (rows * cols) as usize)
    };
    
    println!("DEBUG Rust: Result length: {}", result.len());
    println!("DEBUG Rust: First few compressed values: {:?}", &result.iter().take(10).collect::<Vec<_>>());
    
    // Convert back to 2D array
    let mut result_2d = Vec::with_capacity(rows as usize);
    for i in 0..(rows as usize) {
        let start = i * (cols as usize);
        let end = start + (cols as usize);
        result_2d.push(result[start..end].to_vec());
    }
    
    println!("DEBUG Rust: Converted back to 2D array");
    println!("DEBUG Rust: First row length: {}", result_2d.get(0).map_or(0, |row| row.len()));
    println!("DEBUG Rust: Total rows: {}", result_2d.len());
    
    Ok(result_2d)
}

#[pyfunction]
fn julia_decompress_batch(
    tokens: Vec<Vec<u32>>,
    rows: i64,
    cols: i64
) -> PyResult<Vec<Vec<u32>>> {
    let flat_tokens: Vec<u32> = tokens.into_iter().flatten().collect();
    let result = unsafe {
        let ptr = flat_tokens.as_ptr();
        let result_ptr = julia::julia_decompress_batch(ptr, rows, cols);
        Vec::from_raw_parts(result_ptr as *mut u32, (rows * cols) as usize, (rows * cols) as usize)
    };
    
    // Convert back to 2D array
    let mut result_2d = Vec::with_capacity(rows as usize);
    for i in 0..(rows as usize) {
        let start = i * (cols as usize);
        let end = start + (cols as usize);
        result_2d.push(result[start..end].to_vec());
    }
    Ok(result_2d)
}

#[pymodule]
fn promptveil_core(_py: Python, m: &PyModule) -> PyResult<()> {
    // Julia FFI functions
    m.add_function(wrap_pyfunction!(julia_optimize_tokens_config, m)?)?;
    m.add_function(wrap_pyfunction!(julia_compress_batch_config, m)?)?;
    m.add_function(wrap_pyfunction!(julia_decompress_batch, m)?)?;
    
    // Original functions
    m.add_function(wrap_pyfunction!(compress_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(compress_batch, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_batch, m)?)?;
    m.add_function(wrap_pyfunction!(encrypt, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt, m)?)?;
    m.add_function(wrap_pyfunction!(generate_key, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let data = b"Hello, World!".repeat(100);
        let compressed = compress_tokens(&data).unwrap();
        let decompressed = decompress_tokens(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_batch_compression() {
        let data = b"Test data for batch compression".repeat(100);
        let rows = 100;
        let cols = 8; // 32 bytes per row (8 tokens of 4 bytes)
        let compressed = compress_batch(&data, rows, cols).unwrap();
        let decompressed = decompress_batch(&compressed, rows, cols).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }
}

// Helper function to convert bytes to tokens
fn bytes_to_tokens(data: &[u8]) -> PyResult<Vec<u32>> {
    if data.len() % 4 != 0 {
        return Err(PyErr::new::<PyIOError, _>("Data length must be multiple of 4"));
    }
    
    Ok(data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect())
} 