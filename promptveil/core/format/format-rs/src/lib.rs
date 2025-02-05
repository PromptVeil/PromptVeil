use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

mod serialization;
mod versioning;
mod schema;

pub use serialization::{JsonProvider, BincodeProvider, MessagePackProvider};

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Serialization failed: {0}")]
    SerializationError(String),
    #[error("Deserialization failed: {0}")]
    DeserializationError(String),
    #[error("Version mismatch: {0}")]
    VersionError(String),
    #[error("Schema validation failed: {0}")]
    SchemaError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageMetadata {
    pub version: String,
    pub timestamp: i64,
    pub format_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message<T> {
    pub metadata: MessageMetadata,
    pub content: T,
}

#[async_trait]
pub trait FormatProvider: Send + Sync {
    type Input;
    type Output;

    async fn serialize(&self, data: &Self::Input) -> Result<Vec<u8>, FormatError>;
    async fn deserialize(&self, data: &[u8]) -> Result<Self::Output, FormatError>;
    async fn validate_schema(&self, data: &[u8]) -> Result<bool, FormatError>;
}

#[derive(Clone)]
pub struct FormatManager<P> {
    provider: P,
}

impl<P: FormatProvider> FormatManager<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub async fn serialize(&self, data: &P::Input) -> Result<Vec<u8>, FormatError> {
        debug!("Serializing data");
        self.provider.serialize(data).await
    }

    pub async fn deserialize(&self, data: &[u8]) -> Result<P::Output, FormatError> {
        debug!("Deserializing data");
        self.provider.deserialize(data).await
    }

    pub async fn validate(&self, data: &[u8]) -> Result<bool, FormatError> {
        debug!("Validating schema");
        self.provider.validate_schema(data).await
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_utils {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub field1: String,
        pub field2: i32,
    }

    pub fn create_test_message() -> Message<TestData> {
        Message {
            metadata: MessageMetadata {
                version: "1.0".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                format_type: "test".to_string(),
            },
            content: TestData {
                field1: "test".to_string(),
                field2: 42,
            },
        }
    }
}

#[cfg(test)]
mod internal_tests {
    use super::*;
    use test_utils::*;

    #[tokio::test]
    async fn test_format_manager() {
        let provider = JsonProvider::<TestData>::new();
        let manager = FormatManager::new(provider);
        let message = create_test_message();

        let serialized = manager.serialize(&message).await.unwrap();
        let deserialized = manager.deserialize(&serialized).await.unwrap();

        assert_eq!(message.content, deserialized.content);
        assert!(manager.validate(&serialized).await.unwrap());
    }
} 