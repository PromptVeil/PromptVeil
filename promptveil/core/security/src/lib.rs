use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use std::convert::TryInto;
use std::env;
use std::path::Path;

mod compression;
mod security;
mod julia;

#[no_mangle]
pub extern "C" fn __rust_julia_init() {
    // Esta função será chamada quando a DLL for carregada
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .expect("Failed to get executable directory");
    
    // Adiciona o diretório da DLL ao PATH temporariamente
    let old_path = env::var_os("PATH").unwrap_or_default();
    let mut new_path = exe_dir.as_os_str().to_owned();
    new_path.push(";");
    new_path.push(&old_path);
    env::set_var("PATH", new_path);
}


#[pyfunction]
fn compress_tokens(data: &[u8]) -> PyResult<Vec<u8>> {
    // Convert bytes to u32 tokens (assuming 4 bytes per token)
    let tokens: Vec<u32> = data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    compression::compress_tokens(&tokens)
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
fn compress_batch(data: &[u8], chunk_size: usize) -> PyResult<Vec<u8>> {
    // Convert bytes to u32 tokens (assuming 4 bytes per token)
    let tokens: Vec<u32> = data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    let rows = data.len() / chunk_size;
    let cols = chunk_size / 4;

    compression::compress_batch(&tokens, rows, cols)
        .map(|compressed| {
            // Convert u32 tokens back to bytes
            compressed.iter()
                .flat_map(|&token| token.to_le_bytes().to_vec())
                .collect()
        })
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

#[pymodule]
fn promptveil_core(_py: Python, m: &PyModule) -> PyResult<()> {
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
        let chunk_size = 16;
        let rows = data.len() / chunk_size;
        let cols = chunk_size / 4;
        let compressed = compress_batch(&data, chunk_size).unwrap();
        let decompressed = decompress_batch(&compressed, rows, cols).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }
} 