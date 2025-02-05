use std::path::PathBuf;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

// Re-export core modules
pub use index_rs;
pub use security_rs;
pub use format_rs;

mod error;
pub use error::CoreError;

#[cfg(feature = "extension-module")]
mod python;

mod embeddings;
mod conversation;

#[cfg(test)]
mod tests;

pub use embeddings::{EmbeddingManager, EmbeddingError};
pub use conversation::{ConversationManager, ConversationError, Conversation, Message};
pub use index_rs::{IndexConfig, IndexError};

// Re-export types that are commonly used
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Conversation error: {0}")]
    ConversationError(#[from] ConversationError),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] EmbeddingError),
    
    #[error("Index error: {0}")]
    IndexError(#[from] IndexError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationData {
    pub id: String,
    pub messages: Vec<Message>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: f64,
    pub updated_at: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub security: SecurityConfig,
    pub index: IndexConfig,
    pub format: FormatConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub key_rotation_days: i32,
    pub encryption_enabled: bool,
    pub hardware_acceleration: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormatConfig {
    pub compression_enabled: bool,
    pub compression_level: i32,
}

pub struct PromptVeilCore {
    index: index_rs::Index,
    security: security_rs::SecurityManager,
    format: format_rs::FormatManager,
    base_path: PathBuf,
    config: Config,
}

impl PromptVeilCore {
    pub async fn new(base_path: PathBuf, config: Config) -> Result<Self, CoreError> {
        // Ensure directories exist
        let index_path = base_path.join("index");
        let security_path = base_path.join("security");
        let format_path = base_path.join("format");

        std::fs::create_dir_all(&index_path)?;
        std::fs::create_dir_all(&security_path)?;
        std::fs::create_dir_all(&format_path)?;

        // Initialize components with configuration
        let index = index_rs::Index::new(
            index_path,
            config.index.vector_dim,
            config.index.max_elements,
            config.index.ef_construction,
            config.index.m,
        ).await?;

        let security = security_rs::SecurityManager::new(
            security_path,
            config.security.encryption_enabled,
            config.security.hardware_acceleration,
            config.security.key_rotation_days,
        )?;

        let format = format_rs::FormatManager::new(
            format_path,
            config.format.compression_enabled,
            config.format.compression_level,
        )?;

        Ok(Self {
            index,
            security,
            format,
            base_path,
            config,
        })
    }

    pub async fn add_conversation(&self, conversation: ConversationData) -> Result<(), CoreError> {
        // 1. Format the conversation data
        let formatted_data = self.format.format_conversation(&conversation)?;

        // 2. Encrypt sensitive data
        let encrypted_data = self.security.encrypt_data(&formatted_data)?;

        // 3. Compute vector representation
        let vector = self.compute_vector(&conversation)?;

        // 4. Index the conversation
        self.index.add_conversation(
            &conversation.id,
            &conversation.messages.iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n"),
            vector,
        ).await?;

        // 5. Store encrypted data
        self.format.store_conversation(&conversation.id, &encrypted_data)?;

        Ok(())
    }

    pub async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), CoreError> {
        // 1. Get existing conversation
        let mut conversation = self.get_conversation(conversation_id).await?;

        // 2. Add new message
        conversation.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata,
        });
        conversation.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        // 3. Update conversation
        self.add_conversation(conversation).await
    }

    pub async fn save_conversations(&self, conversations: Vec<ConversationData>) -> Result<(), CoreError> {
        for conversation in conversations {
            self.add_conversation(conversation).await?;
        }
        Ok(())
    }

    pub async fn search_text(&self, query: &str, limit: usize) -> Result<Vec<index_rs::SearchResult>, CoreError> {
        self.index.search_text(query, limit).await.map_err(CoreError::from)
    }

    pub async fn search_similar(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<index_rs::SimilarityMatch>, CoreError> {
        self.index.search_similar(vector, limit).await.map_err(CoreError::from)
    }

    pub async fn get_conversation(&self, id: &str) -> Result<ConversationData, CoreError> {
        // 1. Retrieve encrypted data
        let encrypted_data = self.format.get_conversation(id)?;

        // 2. Decrypt data
        let decrypted_data = self.security.decrypt_data(&encrypted_data)?;

        // 3. Parse formatted data
        let conversation = self.format.parse_conversation(&decrypted_data)?;

        Ok(conversation)
    }

    pub async fn delete_conversation(&self, id: &str) -> Result<(), CoreError> {
        // 1. Remove from index
        self.index.delete_conversation(id).await?;

        // 2. Delete encrypted data
        self.format.delete_conversation(id)?;

        Ok(())
    }

    pub async fn clear(&self) -> Result<(), CoreError> {
        // 1. Clear index
        self.index.clear().await?;

        // 2. Clear all stored data
        self.format.clear()?;

        Ok(())
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn update_config(&mut self, config: Config) -> Result<(), CoreError> {
        // TODO: Implement configuration update logic
        self.config = config;
        Ok(())
    }

    fn compute_vector(&self, conversation: &ConversationData) -> Result<Vec<f32>, CoreError> {
        // TODO: Implement proper vector computation
        Ok(vec![0.0; self.config.index.vector_dim])
    }
} 