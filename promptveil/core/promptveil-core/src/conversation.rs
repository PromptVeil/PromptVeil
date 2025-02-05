use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::embeddings::{EmbeddingManager, EmbeddingError};
use index_rs::{TextIndex, VectorIndex, IndexError, IndexConfig};

#[derive(Error, Debug)]
pub enum ConversationError {
    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] EmbeddingError),

    #[error("Index error: {0}")]
    IndexError(#[from] IndexError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub metadata: Option<serde_json::Value>,
}

pub struct ConversationManager {
    embedding_manager: Arc<EmbeddingManager>,
    text_index: Arc<RwLock<TextIndex>>,
    vector_index: Arc<RwLock<VectorIndex>>,
}

impl ConversationManager {
    pub async fn new(config: IndexConfig) -> Result<Self, ConversationError> {
        let embedding_manager = Arc::new(EmbeddingManager::new().await?);
        let text_index = Arc::new(RwLock::new(TextIndex::new(config.clone())?));
        let vector_index = Arc::new(RwLock::new(VectorIndex::new(config)?));

        Ok(Self {
            embedding_manager,
            text_index,
            vector_index,
        })
    }

    pub async fn add_conversation(&self, conversation: Conversation) -> Result<(), ConversationError> {
        // Validate conversation
        if conversation.messages.is_empty() {
            return Err(ConversationError::InvalidInput("Empty conversation".to_string()));
        }

        // Concatenate all messages for embedding
        let full_text = conversation.messages
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n");

        // Generate embedding
        let embedding = self.embedding_manager.get_embeddings(&full_text).await?;

        // Add to both indices
        {
            let mut text_index = self.text_index.write().await;
            text_index.add_document(&conversation.id, &full_text)?;
        }

        {
            let mut vector_index = self.vector_index.write().await;
            vector_index.add_vector(&conversation.id, embedding)?;
        }

        Ok(())
    }

    pub async fn search_text(&self, query: &str, limit: usize) -> Result<Vec<(String, f32)>, ConversationError> {
        let text_index = self.text_index.read().await;
        let results = text_index.search(query, limit)?;
        Ok(results)
    }

    pub async fn search_semantic(&self, query: &str, limit: usize) -> Result<Vec<(String, f32)>, ConversationError> {
        // Generate query embedding
        let query_embedding = self.embedding_manager.get_embeddings(query).await?;

        // Search vector index
        let vector_index = self.vector_index.read().await;
        let results = vector_index.search_similar(&query_embedding, limit)?;
        Ok(results)
    }

    pub async fn hybrid_search(&self, query: &str, text_weight: f32, semantic_weight: f32, limit: usize) 
        -> Result<Vec<(String, f32)>, ConversationError> 
    {
        if !((0.0..=1.0).contains(&text_weight) && (0.0..=1.0).contains(&semantic_weight)) {
            return Err(ConversationError::InvalidInput("Weights must be between 0 and 1".to_string()));
        }

        let text_results = self.search_text(query, limit).await?;
        let semantic_results = self.search_semantic(query, limit).await?;

        // Combine and normalize scores
        let mut combined_scores: std::collections::HashMap<String, f32> = std::collections::HashMap::new();

        // Add text search scores
        for (id, score) in text_results {
            combined_scores.insert(id, score * text_weight);
        }

        // Add semantic search scores
        for (id, score) in semantic_results {
            combined_scores
                .entry(id)
                .and_modify(|e| *e += score * semantic_weight)
                .or_insert(score * semantic_weight);
        }

        // Sort by combined score
        let mut results: Vec<_> = combined_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    pub async fn clear_indices(&self) -> Result<(), ConversationError> {
        {
            let mut text_index = self.text_index.write().await;
            text_index.clear()?;
        }
        {
            let mut vector_index = self.vector_index.write().await;
            vector_index.clear()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_conversation(id: &str, content: &str) -> Conversation {
        Conversation {
            id: id.to_string(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: content.to_string(),
                    timestamp: Utc::now().timestamp(),
                }
            ],
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_conversation_operations() {
        let config = IndexConfig::default();
        let manager = ConversationManager::new(config).await.unwrap();

        // Add test conversations
        let conv1 = create_test_conversation("1", "This is a test conversation about AI");
        let conv2 = create_test_conversation("2", "Another conversation about programming");
        
        manager.add_conversation(conv1).await.unwrap();
        manager.add_conversation(conv2).await.unwrap();

        // Test text search
        let text_results = manager.search_text("AI", 5).await.unwrap();
        assert!(!text_results.is_empty());
        assert_eq!(text_results[0].0, "1");

        // Test semantic search
        let semantic_results = manager.search_semantic("artificial intelligence", 5).await.unwrap();
        assert!(!semantic_results.is_empty());

        // Test hybrid search
        let hybrid_results = manager.hybrid_search("AI programming", 0.5, 0.5, 5).await.unwrap();
        assert!(!hybrid_results.is_empty());

        // Test clear indices
        manager.clear_indices().await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_weights() {
        let config = IndexConfig::default();
        let manager = ConversationManager::new(config).await.unwrap();

        let result = manager.hybrid_search("test", 1.5, 0.5, 5).await;
        assert!(matches!(result, Err(ConversationError::InvalidInput(_))));
    }
} 