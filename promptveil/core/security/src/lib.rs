#![allow(clippy::all)]
use jlrs::prelude::*;
use pyo3::prelude::*;
use pyo3::exceptions::{PyIOError, PyRuntimeError};
use std::convert::TryInto;
use std::env;
use std::sync::Once;

mod compression;
mod security;
mod layouts;

use crate::layouts::{CompressionConfig, CompressedResult};

// Initialize Julia runtime once
static INIT: Once = Once::new();
static mut JULIA_HANDLE: Option<Runtime> = None;

fn init_julia() -> PyResult<()> {
    INIT.call_once(|| unsafe {
        let runtime = Runtime::init().expect("Failed to initialize Julia runtime");
        JULIA_HANDLE = Some(runtime);
    });
    Ok(())
}

fn get_julia() -> PyResult<&'static Runtime> {
    unsafe {
        JULIA_HANDLE.as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("Julia not initialized"))
    }
}

#[julia_module]
mod promptveil_core {
    use super::*;
    
    #[ccall]
    fn optimize_tokens<'a>(tokens: TypedArray<'a, 'a, u32>, config: Value<'a, 'a>) 
        -> JlrsResult<CompressedResult<'a, 'a>> {
        let module = Module::main(&frame).submodule("PromptVeilCore")?;
        let func = module.function("optimize_tokens")?;
        
        unsafe {
            func.call2(tokens, config).into_jlrs_result()
        }
    }

    #[ccall]
    fn compress_batch<'a>(tokens: TypedArray<'a, 'a, u32>, rows: i64, cols: i64, config: Value<'a, 'a>)
        -> JlrsResult<CompressedResult<'a, 'a>> {
        let module = Module::main(&frame).submodule("PromptVeilCore")?;
        let func = module.function("compress_batch")?;
        
        unsafe {
            func.call4(tokens, rows, cols, config).into_jlrs_result()
        }
    }
}

#[pymodule]
fn promptveil_core(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    init_julia()?;
    
    m.add_function(wrap_pyfunction!(optimize_tokens_py, m)?)?;
    m.add_function(wrap_pyfunction!(compress_batch_py, m)?)?;
    m.add_function(wrap_pyfunction!(encrypt, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt, m)?)?;
    m.add_function(wrap_pyfunction!(generate_key, m)?)?;
    Ok(())
}

#[pyfunction]
fn optimize_tokens_py(py: Python<'_>, tokens: Vec<u32>, use_gpu: bool, use_simd: bool, use_patterns: bool) -> PyResult<(Vec<u32>, usize)> {
    let config = CompressionConfig {
        use_gpu,
        use_simd,
        use_patterns,
    };

    py.allow_threads(|| {
        let ccall = CCall::new()?;
        ccall.scope(|frame| {
            let tokens_array = TypedArray::from_slice(&frame, &tokens, tokens.len())?;
            let config_val = Value::new(&frame, config)?;
            
            let result = promptveil_core::optimize_tokens(&frame, tokens_array, config_val)?;
            Ok((result.data.to_vec(), result.size))
        })
    })
}

#[pyfunction]
fn compress_batch_py(py: Python<'_>, tokens: Vec<u32>, rows: i64, cols: i64, use_gpu: bool, use_simd: bool, use_patterns: bool) -> PyResult<(Vec<u32>, usize)> {
    let config = CompressionConfig {
        use_gpu,
        use_simd,
        use_patterns,
    };

    py.allow_threads(|| {
        let ccall = CCall::new()?;
        ccall.scope(|frame| {
            let tokens_array = TypedArray::from_slice(&frame, &tokens, (rows * cols) as usize)?;
            let config_val = Value::new(&frame, config)?;
            
            let result = promptveil_core::compress_batch(&frame, tokens_array, rows, cols, config_val)?;
            Ok((result.data.to_vec(), result.size))
        })
    })
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
    fn test_julia_errors() {
        // Test invalid data length
        let data = vec![1, 2, 3]; // Not multiple of 4
        assert!(bytes_to_tokens(&data).is_err());
    }
}