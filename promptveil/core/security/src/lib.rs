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
static mut JULIA_HANDLE: Option<CCall<'static>> = None;

fn init_julia() -> PyResult<()> {
    INIT.call_once(|| unsafe {
        let mut frame = StackFrame::new();
        let ccall = CCall::new(&mut frame);
        JULIA_HANDLE = Some(ccall);
    });
    Ok(())
}

fn get_julia() -> PyResult<&'static CCall<'static>> {
    unsafe {
        JULIA_HANDLE.as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("Julia not initialized"))
    }
}

julia_module! {
    become promptveil_core;

    #[ccall]
    fn optimize_tokens(tokens: TypedArray<'_, '_, u32>, use_gpu: bool, use_simd: bool, use_patterns: bool) 
        -> JlrsResult<CompressedResult<'_, '_>> {
        let module = Module::main(&frame).submodule(&mut frame, "PromptVeilCore")?;
        let func = module.function(&mut frame, "optimize_tokens")?;
        
        let config = CompressionConfig {
            use_gpu: Bool::new(use_gpu),
            use_simd: Bool::new(use_simd),
            use_patterns: Bool::new(use_patterns),
        };

        unsafe {
            func.call2(&mut frame, tokens, config)
                .into_jlrs_result()?
                .unbox()
        }
    }

    #[ccall]
    fn compress_batch(tokens: TypedArray<'_, '_, u32>, rows: i64, cols: i64, use_gpu: bool, use_simd: bool, use_patterns: bool) 
        -> JlrsResult<CompressedResult<'_, '_>> {
        let module = Module::main(&frame).submodule(&mut frame, "PromptVeilCore")?;
        let func = module.function(&mut frame, "compress_batch")?;
        
        let config = CompressionConfig {
            use_gpu: Bool::new(use_gpu),
            use_simd: Bool::new(use_simd),
            use_patterns: Bool::new(use_patterns),
        };

        unsafe {
            func.call4(&mut frame, tokens, rows, cols, config)
                .into_jlrs_result()?
                .unbox()
        }
    }
}

#[pyfunction]
pub fn optimize_tokens(
    tokens: Vec<u32>,
    use_gpu: bool,
    use_simd: bool,
    use_patterns: bool,
) -> PyResult<(Vec<u32>, usize)> {
    let julia = get_julia()?;

    julia.scope(|mut frame| {
        let tokens_array = TypedArray::from_slice(&mut frame, &tokens)?;
        
        let result = unsafe {
            promptveil_core::optimize_tokens(&mut frame, tokens_array, use_gpu, use_simd, use_patterns)?
        };

        Ok((result.data().to_vec(), result.size() as usize))
    }).map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
}

#[pyfunction]
pub fn compress_batch(
    tokens: Vec<u32>,
    rows: i64,
    cols: i64,
    use_gpu: bool,
    use_simd: bool,
    use_patterns: bool,
) -> PyResult<(Vec<u32>, usize)> {
    let julia = get_julia()?;

    julia.scope(|mut frame| {
        let tokens_array = TypedArray::from_slice(&mut frame, &tokens)?;
        
        let result = unsafe {
            promptveil_core::compress_batch(&mut frame, tokens_array, rows, cols, use_gpu, use_simd, use_patterns)?
        };

        Ok((result.data().to_vec(), result.size() as usize))
    }).map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
}

#[pymodule]
fn promptveil_core(_py: Python, m: &PyModule) -> PyResult<()> {
    init_julia()?;
    
    m.add_function(wrap_pyfunction!(optimize_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(compress_batch, m)?)?;
    m.add_function(wrap_pyfunction!(encrypt, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt, m)?)?;
    m.add_function(wrap_pyfunction!(generate_key, m)?)?;
    Ok(())
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