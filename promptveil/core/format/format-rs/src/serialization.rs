use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::{FormatError, FormatProvider, Message};

#[derive(Clone)]
pub struct JsonProvider<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonProvider<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: Serialize + DeserializeOwned + Send + Sync> FormatProvider for JsonProvider<T> {
    type Input = Message<T>;
    type Output = Message<T>;

    async fn serialize(&self, data: &Self::Input) -> Result<Vec<u8>, FormatError> {
        serde_json::to_vec(data)
            .map_err(|e| FormatError::SerializationError(e.to_string()))
    }

    async fn deserialize(&self, data: &[u8]) -> Result<Self::Output, FormatError> {
        serde_json::from_slice(data)
            .map_err(|e| FormatError::DeserializationError(e.to_string()))
    }

    async fn validate_schema(&self, data: &[u8]) -> Result<bool, FormatError> {
        serde_json::from_slice::<Message<T>>(data)
            .map(|_| true)
            .map_err(|e| FormatError::SchemaError(e.to_string()))
    }
}

#[derive(Clone)]
pub struct BincodeProvider<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> BincodeProvider<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: Serialize + DeserializeOwned + Send + Sync> FormatProvider for BincodeProvider<T> {
    type Input = Message<T>;
    type Output = Message<T>;

    async fn serialize(&self, data: &Self::Input) -> Result<Vec<u8>, FormatError> {
        bincode::serialize(data)
            .map_err(|e| FormatError::SerializationError(e.to_string()))
    }

    async fn deserialize(&self, data: &[u8]) -> Result<Self::Output, FormatError> {
        bincode::deserialize(data)
            .map_err(|e| FormatError::DeserializationError(e.to_string()))
    }

    async fn validate_schema(&self, data: &[u8]) -> Result<bool, FormatError> {
        bincode::deserialize::<Message<T>>(data)
            .map(|_| true)
            .map_err(|e| FormatError::SchemaError(e.to_string()))
    }
}

#[derive(Clone)]
pub struct MessagePackProvider<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> MessagePackProvider<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: Serialize + DeserializeOwned + Send + Sync> FormatProvider for MessagePackProvider<T> {
    type Input = Message<T>;
    type Output = Message<T>;

    async fn serialize(&self, data: &Self::Input) -> Result<Vec<u8>, FormatError> {
        rmp_serde::to_vec(data)
            .map_err(|e| FormatError::SerializationError(e.to_string()))
    }

    async fn deserialize(&self, data: &[u8]) -> Result<Self::Output, FormatError> {
        rmp_serde::from_slice(data)
            .map_err(|e| FormatError::DeserializationError(e.to_string()))
    }

    async fn validate_schema(&self, data: &[u8]) -> Result<bool, FormatError> {
        rmp_serde::from_slice::<Message<T>>(data)
            .map(|_| true)
            .map_err(|e| FormatError::SchemaError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::MessageMetadata;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        field1: String,
        field2: i32,
    }

    fn create_test_message() -> Message<TestData> {
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

    #[tokio::test]
    async fn test_json_provider() {
        let provider = JsonProvider::<TestData>::new();
        let message = create_test_message();

        let serialized = provider.serialize(&message).await.unwrap();
        let deserialized = provider.deserialize(&serialized).await.unwrap();

        assert_eq!(message.content, deserialized.content);
    }

    #[tokio::test]
    async fn test_bincode_provider() {
        let provider = BincodeProvider::<TestData>::new();
        let message = create_test_message();

        let serialized = provider.serialize(&message).await.unwrap();
        let deserialized = provider.deserialize(&serialized).await.unwrap();

        assert_eq!(message.content, deserialized.content);
    }

    #[tokio::test]
    async fn test_messagepack_provider() {
        let provider = MessagePackProvider::<TestData>::new();
        let message = create_test_message();

        let serialized = provider.serialize(&message).await.unwrap();
        let deserialized = provider.deserialize(&serialized).await.unwrap();

        assert_eq!(message.content, deserialized.content);
    }
} 