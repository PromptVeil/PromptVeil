use jlrs::prelude::*;
use pyo3::prelude::*;
use pyo3::exceptions::{PyIOError, PyRuntimeError};
use std::convert::TryInto;
use std::env;
use std::sync::Once;
use jlrs::ccall::CCall;
use jlrs::data::managed::array::ArrayRef;
use jlrs::data::layout::bool::Bool;

mod compression;
mod security;
mod layouts;  // Generated layouts from Julia

use crate::layouts::{CompressionConfig as JuliaConfig, CompressedResult as JuliaResult};

// Initialize Julia runtime once
static mut JULIA_HANDLE: Option<LocalHandle> = None;
static INIT: Once = Once::new();

fn init_julia() -> Result<(), String> {
    unsafe {
        INIT.call_once(|| {
    eprintln!("DEBUG Rust: Initializing Julia runtime");
    
            // Set up environment
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .ok_or("Failed to get executable directory")?;
    
    let old_path = env::var_os("PATH").unwrap_or_default();
    let mut new_path = exe_dir.as_os_str().to_owned();
    new_path.push(";");
    new_path.push(&old_path);
    env::set_var("PATH", new_path);
    
            // Initialize Julia runtime
            let handle = Builder::new().start_local()
                .map_err(|e| format!("Failed to start Julia: {}", e))?;
            
            // Load PromptVeilCore module
            handle.local_scope::<_, 1>(|mut frame| -> JlrsResult<()> {
                let module = Module::main(&frame);
                module.function(&mut frame, "using")?
                    .call1(&mut frame, Value::new(&mut frame, "PromptVeilCore"))?;
                Ok(())
            })?;
            
            JULIA_HANDLE = Some(handle);
        eprintln!("DEBUG Rust: Julia runtime initialized");
        });
        Ok(())
    }
}

fn get_julia() -> Result<&'static mut LocalHandle, String> {
    unsafe {
        init_julia()?;
        JULIA_HANDLE.as_mut().ok_or_else(|| "Julia runtime not initialized".to_string())
    }
}

// Clean up Julia runtime on drop
struct JuliaCleanup;

impl Drop for JuliaCleanup {
    fn drop(&mut self) {
        unsafe {
            if let Some(handle) = JULIA_HANDLE.take() {
        eprintln!("DEBUG Rust: Cleaning up Julia runtime");
                drop(handle);
        eprintln!("DEBUG Rust: Julia runtime cleaned up");
            }
        }
    }
}

// Static instance to ensure cleanup
static mut CLEANUP: Option<JuliaCleanup> = Some(JuliaCleanup);

julia_module! {
    become promptveil_init_fn;

    fn optimize_tokens(
        tokens: ArrayRef<'_, '_>,
        config: CompressionConfig
    ) -> CompressedResult<'_, '_>;

    fn compress_batch(
        tokens: ArrayRef<'_, '_>,
        rows: i64,
        cols: i64,
        config: CompressionConfig
    ) -> CompressedResult<'_, '_>;
}

#[pymodule]
fn promptveil_core(_py: Python, m: &PyModule) -> PyResult<()> {
    // Initialize Julia on module load
    init_julia().map_err(|e| PyRuntimeError::new_err(e))?;
    
    m.add_function(wrap_pyfunction!(compress_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(encrypt, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt, m)?)?;
    m.add_function(wrap_pyfunction!(generate_key, m)?)?;
    Ok(())
}

// Compression functions using JLRS
#[pyfunction]
fn compress_tokens(data: &[u8], use_gpu: bool, use_simd: bool, use_patterns: bool) -> PyResult<Vec<u8>> {
    let tokens = bytes_to_tokens(data)?;
    let julia = get_julia().map_err(|e| PyRuntimeError::new_err(e))?;
    
    julia.local_scope::<_, 3>(|mut frame| -> JlrsResult<Vec<u8>> {
        // Create config struct using generated layout
        let config = Value::new(&mut frame, JuliaConfig {
            use_gpu,
            use_simd,
            use_patterns,
        });
        
        // Get PromptVeilCore module and function
        let module = Module::main(&frame).submodule(&mut frame, "PromptVeilCore")?;
        let func = module.function(&mut frame, "optimize_tokens")?;
        
        // Convert tokens to Julia array
        let tokens_array = Value::new(&mut frame, tokens);
        
        // Call function
        let result = unsafe { 
            func.call2(&mut frame, tokens_array, config)
                .into_jlrs_result()?
        };
        
        // Convert result back using generated layout
        let compressed_result: JuliaResult = result.unbox()?;
        Ok(compressed_result.data.iter()
            .flat_map(|&token| token.to_le_bytes().to_vec())
            .collect())
    })
    .map_err(|e| PyRuntimeError::new_err(format!("Julia error: {}", e)))
}

#[pyfunction]
fn decompress_tokens(data: &[u8]) -> PyResult<Vec<u8>> {
    let tokens = bytes_to_tokens(data)?;
    let julia = get_julia().map_err(|e| PyRuntimeError::new_err(e))?;
    
    julia.local_scope::<_, 2>(|mut frame| -> JlrsResult<Vec<u8>> {
        // Get PromptVeilCore module and function
        let module = Module::main(&frame).submodule(&mut frame, "PromptVeilCore")?;
        let func = module.function(&mut frame, "decompress_tokens")?;
        
        // Convert tokens to Julia array
        let tokens_array = Value::new(&mut frame, tokens);
        
        // Call function
        let result = unsafe {
            func.call1(&mut frame, tokens_array)
                .into_jlrs_result()?
        };
        
        // Convert result back
        let decompressed: Vec<u32> = result.unbox()?;
        Ok(decompressed.iter()
                .flat_map(|&token| token.to_le_bytes().to_vec())
            .collect())
        })
    .map_err(|e| PyRuntimeError::new_err(format!("Julia error: {}", e)))
}

// Helper function to convert bytes to tokens
fn bytes_to_tokens(data: &[u8]) -> PyResult<Vec<u32>> {
    if data.len() % 4 != 0 {
        return Err(PyIOError::new_err("Data length must be multiple of 4"));
    }
    
    Ok(data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect())
}

// Security functions remain unchanged
#[pyfunction]
fn encrypt(data: &[u8], key: &[u8]) -> PyResult<Vec<u8>> {
    let key_array: [u8; 32] = key.try_into()
        .map_err(|_| PyIOError::new_err("Key must be 32 bytes"))?;

    security::encrypt(data, &key_array)
        .map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pyfunction]
fn decrypt(data: &[u8], key: &[u8]) -> PyResult<Vec<u8>> {
    let key_array: [u8; 32] = key.try_into()
        .map_err(|_| PyIOError::new_err("Key must be 32 bytes"))?;

    security::decrypt(data, &key_array)
        .map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pyfunction]
fn generate_key() -> PyResult<Vec<u8>> {
    security::generate_key()
        .map_err(|e| PyIOError::new_err(e.to_string()))
        .map(|key| key.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let data = b"Hello, World!".repeat(100);
        let compressed = compress_tokens(&data, true, true, true).unwrap();
        let decompressed = decompress_tokens(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_julia_errors() {
        // Test invalid data length
        let data = vec![1, 2, 3]; // Not multiple of 4
        assert!(compress_tokens(&data, false, false, false).is_err());
    }
} 