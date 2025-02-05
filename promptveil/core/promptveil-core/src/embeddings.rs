use std::sync::Arc;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel,
};
use moka::future::Cache;
use rayon::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),

    #[error("Failed to generate embeddings: {0}")]
    GenerationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

pub struct EmbeddingManager {
    model: Arc<SentenceEmbeddingsModel>,
    cache: Cache<String, Vec<f32>>,
}

impl EmbeddingManager {
    pub async fn new() -> Result<Self, EmbeddingError> {
        // Initialize the model with all-MiniLM-L6-v2 (good balance of speed and quality)
        let model = SentenceEmbeddingsBuilder::remote(
            rust_bert::resources::RemoteResource::from_pretrained("sentence-transformers/all-MiniLM-L6-v2")
        )
        .create_model()
        .map_err(|e| EmbeddingError::ModelLoadError(e.to_string()))?;

        // Create cache with reasonable defaults
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(std::time::Duration::from_secs(3600)) // 1 hour
            .build();

        Ok(Self {
            model: Arc::new(model),
            cache,
        })
    }

    pub async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text".to_string()));
        }

        // Check cache first
        if let Some(cached) = self.cache.get(text).await {
            return Ok(cached);
        }

        // Generate embeddings
        let embeddings = self.model
            .encode(&[text])
            .map_err(|e| EmbeddingError::GenerationError(e.to_string()))?;

        if embeddings.is_empty() {
            return Err(EmbeddingError::GenerationError("No embeddings generated".to_string()));
        }

        let vector = embeddings[0].clone();
        
        // Cache the result
        self.cache.insert(text.to_string(), vector.clone()).await;

        Ok(vector)
    }

    pub async fn get_embeddings_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Filter out texts that are already cached
        let (cached, to_compute): (Vec<_>, Vec<_>) = texts
            .iter()
            .enumerate()
            .partition(|(_, text)| self.cache.contains_key(*text));

        // Get cached embeddings
        let mut results = vec![Vec::new(); texts.len()];
        for (idx, text) in cached {
            if let Some(embedding) = self.cache.get(text).await {
                results[idx] = embedding;
            }
        }

        // Compute missing embeddings
        if !to_compute.is_empty() {
            let texts_to_compute: Vec<_> = to_compute.iter().map(|(_, text)| *text).collect();
            let new_embeddings = self.model
                .encode(&texts_to_compute)
                .map_err(|e| EmbeddingError::GenerationError(e.to_string()))?;

            // Cache and store results
            for ((idx, text), embedding) in to_compute.iter().zip(new_embeddings.iter()) {
                self.cache.insert((*text).to_string(), embedding.clone()).await;
                results[*idx] = embedding.clone();
            }
        }

        Ok(results)
    }

    pub fn get_dimension(&self) -> usize {
        384 // all-MiniLM-L6-v2 dimension
    }

    pub async fn clear_cache(&self) {
        self.cache.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_embedding_generation() {
        let manager = EmbeddingManager::new().await.unwrap();
        let text = "This is a test sentence.";
        let embeddings = manager.get_embeddings(text).await.unwrap();
        
        assert_eq!(embeddings.len(), manager.get_dimension());
        
        // Test caching
        let cached_embeddings = manager.get_embeddings(text).await.unwrap();
        assert_eq!(embeddings, cached_embeddings);
    }

    #[tokio::test]
    async fn test_batch_embedding() {
        let manager = EmbeddingManager::new().await.unwrap();
        let texts = vec![
            "First sentence.".to_string(),
            "Second sentence.".to_string(),
            "Third sentence.".to_string(),
        ];

        let embeddings = manager.get_embeddings_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), texts.len());
        for embedding in embeddings {
            assert_eq!(embedding.len(), manager.get_dimension());
        }
    }

    #[tokio::test]
    async fn test_invalid_input() {
        let manager = EmbeddingManager::new().await.unwrap();
        let result = manager.get_embeddings("").await;
        assert!(matches!(result, Err(EmbeddingError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let manager = EmbeddingManager::new().await.unwrap();
        let text = "Cache test sentence.";
        
        // Generate and cache embeddings
        let embeddings = manager.get_embeddings(text).await.unwrap();
        assert!(manager.cache.contains_key(text));

        // Clear cache
        manager.clear_cache().await;
        assert!(!manager.cache.contains_key(text));
    }
} 