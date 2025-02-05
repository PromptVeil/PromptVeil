mod error;

use async_trait::async_trait;
use error::{CompressionError, Result};
use jlrs::prelude::*;
use tracing::info;

const INIT_SCRIPT: &str = r#"
import Pkg
# Desenvolver o pacote local em vez de criar um novo
Pkg.develop(path=joinpath(@__DIR__, "..", "TokenCompression.jl"))
Pkg.update()  # Atualizar todas as dependências
Pkg.resolve()  # Resolver conflitos de versão
using TokenCompression
"#;

/// TokenCompressor provides an async interface to TokenCompression.jl
pub struct TokenCompressor {
    handle: AsyncHandle,
    _thread: std::thread::JoinHandle<()>,
}

/// Compression task for processing token sequences
struct CompressionTask {
    tokens: Vec<u32>,
    batch_size: Option<usize>,
}

/// Batch compression task for processing multiple sequences
struct BatchCompressionTask {
    tokens: Vec<Vec<u32>>,
    batch_size: usize,
}

/// Init task for setting up Julia environment
struct InitTask;

#[async_trait(?Send)]
impl AsyncTask for CompressionTask {
    type Output = Result<Vec<u32>>;

    async fn run<'frame>(&mut self, mut frame: AsyncGcFrame<'frame>) -> Self::Output {
        unsafe {
            // Calcular o comprimento antes do empréstimo mutável
            let tokens_len = self.tokens.len();
            
            // Criar o array Julia diretamente como TypedArray
            let tokens = TypedArray::<u32>::from_slice(&mut frame, &mut self.tokens[..], &[tokens_len])
                .map_err(|e| CompressionError::JuliaError(e.to_string()))?
                .expect("invalid size"); // Desembrulhar o Result interno

            let value_tokens = tokens.as_value();

            // Obter o módulo e a função
            let main_module = Module::main(&frame);
            let module = main_module.submodule(&mut frame, "TokenCompression")
                .map_err(|e| CompressionError::JuliaError(e.to_string()))?;
            
            let optimize_tokens = module.function(&mut frame, "optimize_tokens")
                .map_err(|e| CompressionError::JuliaError(e.to_string()))?;

            // Chamar a função Julia com o array
            let result = optimize_tokens.call1(&mut frame, value_tokens)
                .map_err(|e| CompressionError::JuliaError(format!("{:?}", e)))?;

            // Converter o resultado de volta usando cast
            let array = result.cast::<TypedArray<u32>>()
                .map_err(|_| CompressionError::InvalidTokens("Failed to convert result".into()))?;

            // Extrair os dados do array usando bits_data
            let data = array.bits_data();
            Ok(data.as_slice().to_vec())
        }
    }
}

#[async_trait(?Send)]
impl AsyncTask for BatchCompressionTask {
    type Output = Result<Vec<Vec<u32>>>;

    async fn run<'frame>(&mut self, mut frame: AsyncGcFrame<'frame>) -> Self::Output {
        unsafe {
            // Calculate dimensions
            let n_sequences = self.tokens.len();
            let max_len = self.tokens.iter().map(|seq| seq.len()).max().unwrap_or(0);
            
            // Create flat array with padding
            let mut flat_tokens = Vec::with_capacity(n_sequences * max_len);
            for seq in &self.tokens {
                let mut padded_seq = seq.clone();
                padded_seq.resize(max_len, 0);  // Pad with zeros to max_len
                flat_tokens.extend(padded_seq);
            }

            // Create Julia matrix with correct dimensions
            let tokens_matrix = TypedArray::<u32>::from_slice(&mut frame, &mut flat_tokens[..], &[n_sequences, max_len])
                .map_err(|e| CompressionError::JuliaError(e.to_string()))?
                .expect("invalid size");

            let value_tokens_matrix = tokens_matrix.as_value();

            // Get module and function
            let main_module = Module::main(&frame);
            let module = main_module.submodule(&mut frame, "TokenCompression")
                .map_err(|e| CompressionError::JuliaError(e.to_string()))?;
            
            let compress_batch = module.function(&mut frame, "compress_batch")
                .map_err(|e| CompressionError::JuliaError(e.to_string()))?;

            // Call Julia function with matrix
            let result = compress_batch.call1(&mut frame, value_tokens_matrix)
                .map_err(|e| CompressionError::JuliaError(format!("{:?}", e)))?;

            // Convert result back
            let array = result.cast::<TypedArray<u32>>()
                .map_err(|_| CompressionError::InvalidTokens("Failed to convert result".into()))?;

            // Extract data from array
            let data = array.bits_data();
            let data_slice = data.as_slice();
            
            // Convert back to Vec<Vec<u32>>
            let mut results = Vec::with_capacity(n_sequences);
            let compressed_max_len = data_slice.len() / n_sequences;
            
            for i in 0..n_sequences {
                let start = i * compressed_max_len;
                let end = start + compressed_max_len;
                let seq = &data_slice[start..end];
                // Find the end of the actual sequence (before padding zeros)
                let len = seq.iter().position(|&x| x == 0).unwrap_or(compressed_max_len);
                results.push(seq[..len].to_vec());
            }
            
            Ok(results)
        }
    }
}

#[async_trait(?Send)]
impl AsyncTask for InitTask {
    type Output = Result<()>;

    async fn run<'frame>(&mut self, mut frame: AsyncGcFrame<'frame>) -> Self::Output {
        unsafe {
            Value::eval_string(&mut frame, INIT_SCRIPT)
                .into_jlrs_result()
                .map_err(|e| CompressionError::InitError(e.to_string()))?;
            Ok(())
        }
    }
}

impl TokenCompressor {
    /// Create a new TokenCompressor instance
    pub fn new() -> Result<Self> {
        let (handle, thread) = Builder::new()
            .async_runtime(Tokio::<3>::new(false))
            .spawn()
            .map_err(|e| CompressionError::InitError(e.to_string()))?;

        // Load local TokenCompression.jl
        handle
            .task(InitTask)
            .try_dispatch()
            .map_err(|_| CompressionError::InitError("Failed to dispatch init task".into()))?
            .blocking_recv()
            .map_err(|_| CompressionError::InitError("Failed to receive init result".into()))??;

        Ok(Self { handle, _thread: thread })
    }

    /// Compress a single token sequence
    pub async fn compress_tokens(&self, tokens: Vec<u32>) -> Result<Vec<u32>> {
        let task = CompressionTask {
            tokens,
            batch_size: None,
        };

        self.handle
            .task(task)
            .try_dispatch()
            .map_err(|_| CompressionError::AsyncError("Failed to dispatch compression task".into()))?
            .await
            .map_err(|_| CompressionError::AsyncError("Failed to receive compression result".into()))?
    }

    /// Compress multiple token sequences in batch
    pub async fn compress_batch(&self, tokens: Vec<Vec<u32>>, batch_size: usize) -> Result<Vec<Vec<u32>>> {
        let task = BatchCompressionTask {
            tokens,
            batch_size,
        };

        self.handle
            .task(task)
            .try_dispatch()
            .map_err(|_| CompressionError::AsyncError("Failed to dispatch batch task".into()))?
            .await
            .map_err(|_| CompressionError::AsyncError("Failed to receive batch result".into()))?
    }

    /// Decompress a single token sequence
    pub async fn decompress_tokens(&self, _data: &[u8]) -> Result<Vec<u32>> {
        // Implementação temporária
        Ok(vec![42])
    }
}

impl Drop for TokenCompressor {
    fn drop(&mut self) {
        info!("Shutting down TokenCompressor");
    }
}

#[cfg(test)]
mod tests;
