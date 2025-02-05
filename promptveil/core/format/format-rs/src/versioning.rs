use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::{FormatError, Message};

#[async_trait::async_trait]
pub trait VersionConverter<T, U> {
    async fn convert(&self, data: T) -> Result<U, FormatError>;
}

pub struct VersionRegistry<T> {
    current_version: String,
    converters: Arc<RwLock<HashMap<String, Box<dyn VersionConverter<Message<T>, Message<T>> + Send + Sync>>>>,
}

impl<T> VersionRegistry<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(current_version: String) -> Self {
        Self {
            current_version,
            converters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_converter<C>(
        &self,
        from_version: String,
        converter: C,
    ) -> Result<(), FormatError>
    where
        C: VersionConverter<Message<T>, Message<T>> + Send + Sync + 'static,
    {
        let mut converters = self.converters.write().await;
        converters.insert(from_version, Box::new(converter));
        Ok(())
    }

    pub async fn convert_to_current(&self, message: Message<T>) -> Result<Message<T>, FormatError> {
        if message.metadata.version == self.current_version {
            return Ok(message);
        }

        let converters = self.converters.read().await;
        let converter = converters
            .get(&message.metadata.version)
            .ok_or_else(|| {
                FormatError::VersionError(format!(
                    "No converter found for version {}",
                    message.metadata.version
                ))
            })?;

        converter.convert(message).await
    }

    pub fn get_current_version(&self) -> String {
        self.current_version.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        field: String,
    }

    struct TestConverter;

    #[async_trait::async_trait]
    impl VersionConverter<Message<TestData>, Message<TestData>> for TestConverter {
        async fn convert(&self, mut data: Message<TestData>) -> Result<Message<TestData>, FormatError> {
            data.metadata.version = "2.0".to_string();
            data.content.field = format!("converted_{}", data.content.field);
            Ok(data)
        }
    }

    #[tokio::test]
    async fn test_version_conversion() {
        let registry = VersionRegistry::new("2.0".to_string());
        registry
            .register_converter("1.0".to_string(), TestConverter)
            .await
            .unwrap();

        let message = Message {
            metadata: crate::MessageMetadata {
                version: "1.0".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                format_type: "test".to_string(),
            },
            content: TestData {
                field: "test".to_string(),
            },
        };

        let converted = registry.convert_to_current(message.clone()).await.unwrap();
        assert_eq!(converted.metadata.version, "2.0");
        assert_eq!(converted.content.field, "converted_test");
    }

    #[tokio::test]
    async fn test_same_version() {
        let registry = VersionRegistry::new("2.0".to_string());
        let message = Message {
            metadata: crate::MessageMetadata {
                version: "2.0".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                format_type: "test".to_string(),
            },
            content: TestData {
                field: "test".to_string(),
            },
        };

        let converted = registry.convert_to_current(message.clone()).await.unwrap();
        assert_eq!(converted, message);
    }

    #[tokio::test]
    async fn test_missing_converter() {
        let registry = VersionRegistry::new("2.0".to_string());
        let message = Message {
            metadata: crate::MessageMetadata {
                version: "1.0".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                format_type: "test".to_string(),
            },
            content: TestData {
                field: "test".to_string(),
            },
        };

        let result = registry.convert_to_current(message).await;
        assert!(matches!(result, Err(FormatError::VersionError(_))));
    }
} 