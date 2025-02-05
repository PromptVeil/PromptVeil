use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::FormatError;

#[async_trait::async_trait]
pub trait SchemaValidator: Send + Sync {
    async fn validate(&self, data: &[u8]) -> Result<bool, FormatError>;
}

pub struct SchemaRegistry {
    validators: Arc<RwLock<HashMap<String, Box<dyn SchemaValidator>>>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            validators: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_validator<V>(
        &self,
        schema_id: String,
        validator: V,
    ) -> Result<(), FormatError>
    where
        V: SchemaValidator + 'static,
    {
        let mut validators = self.validators.write().await;
        validators.insert(schema_id, Box::new(validator));
        Ok(())
    }

    pub async fn validate(&self, schema_id: &str, data: &[u8]) -> Result<bool, FormatError> {
        let validators = self.validators.read().await;
        let validator = validators
            .get(schema_id)
            .ok_or_else(|| FormatError::SchemaError(format!("No validator found for schema {}", schema_id)))?;

        validator.validate(data).await
    }
}

pub struct JsonSchemaValidator<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonSchemaValidator<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T> SchemaValidator for JsonSchemaValidator<T>
where
    T: for<'de> Deserialize<'de> + Send + Sync,
{
    async fn validate(&self, data: &[u8]) -> Result<bool, FormatError> {
        serde_json::from_slice::<T>(data)
            .map(|_| true)
            .map_err(|e| FormatError::SchemaError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestSchema {
        field1: String,
        field2: i32,
    }

    #[tokio::test]
    async fn test_json_schema_validation() {
        let validator = JsonSchemaValidator::<TestSchema>::new();
        let valid_data = r#"{"field1": "test", "field2": 42}"#.as_bytes();
        let invalid_data = r#"{"field1": "test"}"#.as_bytes();

        assert!(validator.validate(valid_data).await.unwrap());
        assert!(validator.validate(invalid_data).await.is_err());
    }

    #[tokio::test]
    async fn test_schema_registry() {
        let registry = SchemaRegistry::new();
        let validator = JsonSchemaValidator::<TestSchema>::new();
        
        registry.register_validator("test_schema".to_string(), validator).await.unwrap();

        let valid_data = r#"{"field1": "test", "field2": 42}"#.as_bytes();
        let invalid_data = r#"{"field1": "test"}"#.as_bytes();

        assert!(registry.validate("test_schema", valid_data).await.unwrap());
        assert!(registry.validate("test_schema", invalid_data).await.is_err());
        assert!(registry.validate("nonexistent", valid_data).await.is_err());
    }
} 