#[cfg(test)]
mod tests {
    use crate::{
        FormatError, FormatManager, FormatProvider, Message, MessageMetadata,
        JsonProvider, BincodeProvider,
    };
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};
    use serde_json::json;

    // Test Data Structures
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        field1: String,
        field2: i32,
        field3: Option<Vec<String>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct ComplexData {
        nested: TestData,
        array: Vec<i32>,
        map: std::collections::HashMap<String, String>,
    }

    fn create_test_metadata() -> MessageMetadata {
        MessageMetadata {
            version: "1.0".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            format_type: "test".to_string(),
        }
    }

    // Basic Serialization Tests
    #[tokio::test]
    async fn test_json_serialization() {
        let manager = FormatManager::new(JsonProvider::<TestData>::new());
        let test_data = Message {
            metadata: create_test_metadata(),
            content: TestData {
                field1: "test".to_string(),
                field2: 42,
                field3: Some(vec!["a".to_string(), "b".to_string()]),
            },
        };

        let serialized = manager.serialize(&test_data).await.unwrap();
        let deserialized = manager.deserialize(&serialized).await.unwrap();

        assert_eq!(test_data.content, deserialized.content);
        assert_eq!(test_data.metadata.version, deserialized.metadata.version);
    }

    #[tokio::test]
    async fn test_binary_serialization() {
        let manager = FormatManager::new(BincodeProvider::<ComplexData>::new());
        let mut map = std::collections::HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());

        let test_data = Message {
            metadata: create_test_metadata(),
            content: ComplexData {
                nested: TestData {
                    field1: "nested".to_string(),
                    field2: 100,
                    field3: None,
                },
                array: vec![1, 2, 3],
                map,
            },
        };

        let serialized = manager.serialize(&test_data).await.unwrap();
        let deserialized = manager.deserialize(&serialized).await.unwrap();

        assert_eq!(test_data.content.nested, deserialized.content.nested);
        assert_eq!(test_data.content.array, deserialized.content.array);
        assert_eq!(test_data.content.map, deserialized.content.map);
    }

    // Schema Validation Tests
    #[tokio::test]
    async fn test_schema_validation() {
        let manager = FormatManager::new(JsonProvider::<TestData>::new());
        let test_data = Message {
            metadata: create_test_metadata(),
            content: TestData {
                field1: "test".to_string(),
                field2: 42,
                field3: None,
            },
        };

        let serialized = manager.serialize(&test_data).await.unwrap();
        assert!(manager.validate(&serialized).await.unwrap());
    }

    #[tokio::test]
    async fn test_invalid_schema() {
        let manager = FormatManager::new(JsonProvider::<TestData>::new());
        let invalid_data = json!({
            "metadata": {
                "version": "1.0",
                "timestamp": 123456789,
                "format_type": "test"
            },
            "content": {
                "invalid_field": "this should fail"
            }
        });

        let serialized = serde_json::to_vec(&invalid_data).unwrap();
        assert!(manager.validate(&serialized).await.is_err());
    }

    // Error Handling Tests
    #[tokio::test]
    async fn test_deserialization_error() {
        let manager = FormatManager::new(JsonProvider::<TestData>::new());
        let invalid_data = b"this is not valid json";
        
        let result = manager.deserialize(invalid_data).await;
        assert!(matches!(result, Err(FormatError::DeserializationError(_))));
    }

    #[tokio::test]
    async fn test_version_mismatch() {
        let manager = FormatManager::new(JsonProvider::<TestData>::new());
        let test_data = Message {
            metadata: MessageMetadata {
                version: "2.0".to_string(), // Different version
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                format_type: "test".to_string(),
            },
            content: TestData {
                field1: "test".to_string(),
                field2: 42,
                field3: None,
            },
        };

        let serialized = manager.serialize(&test_data).await.unwrap();
        let deserialized = manager.deserialize(&serialized).await.unwrap();
        
        assert_ne!(deserialized.metadata.version, "1.0");
        assert_eq!(deserialized.metadata.version, "2.0");
    }

    // Edge Cases
    #[tokio::test]
    async fn test_empty_data() {
        let manager = FormatManager::new(JsonProvider::<TestData>::new());
        let test_data = Message {
            metadata: create_test_metadata(),
            content: TestData {
                field1: "".to_string(),
                field2: 0,
                field3: Some(vec![]),
            },
        };

        let serialized = manager.serialize(&test_data).await.unwrap();
        let deserialized = manager.deserialize(&serialized).await.unwrap();

        assert_eq!(test_data.content, deserialized.content);
    }

    #[tokio::test]
    async fn test_large_data() {
        let manager = FormatManager::new(BincodeProvider::<ComplexData>::new());
        let mut large_array = Vec::with_capacity(10000);
        for i in 0..10000 {
            large_array.push(i);
        }

        let test_data = Message {
            metadata: create_test_metadata(),
            content: ComplexData {
                nested: TestData {
                    field1: "large".to_string(),
                    field2: 999999,
                    field3: Some(vec!["large".repeat(1000)]),
                },
                array: large_array,
                map: std::collections::HashMap::new(),
            },
        };

        let serialized = manager.serialize(&test_data).await.unwrap();
        let deserialized = manager.deserialize(&serialized).await.unwrap();

        assert_eq!(test_data.content.array, deserialized.content.array);
        assert_eq!(test_data.content.nested.field3, deserialized.content.nested.field3);
    }

    // Concurrent Operations
    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager = FormatManager::new(JsonProvider::<TestData>::new());
        let test_data = Message {
            metadata: create_test_metadata(),
            content: TestData {
                field1: "concurrent".to_string(),
                field2: 42,
                field3: Some(vec!["test".to_string()]),
            },
        };

        let mut handles = Vec::new();
        for _ in 0..10 {
            let manager = manager.clone();
            let data = test_data.clone();
            
            handles.push(tokio::spawn(async move {
                let serialized = manager.serialize(&data).await.unwrap();
                let deserialized = manager.deserialize(&serialized).await.unwrap();
                assert_eq!(data.content, deserialized.content);
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    // File I/O Tests
    #[tokio::test]
    async fn test_pveil_file_io() {
        use std::fs;

        let provider = JsonProvider::<TestData>::new();
        let manager = FormatManager::new(provider);
        let test_data = Message {
            metadata: create_test_metadata(),
            content: TestData {
                field1: "file_test".to_string(),
                field2: 42,
                field3: Some(vec!["test".to_string()]),
            },
        };

        // Create a temporary file path
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test.pveil");

        // Serialize and write to file
        let serialized = manager.serialize(&test_data).await.unwrap();
        fs::write(&file_path, &serialized).unwrap();

        // Read from file and deserialize
        let file_content = fs::read(&file_path).unwrap();
        let deserialized = manager.deserialize(&file_content).await.unwrap();

        // Clean up
        fs::remove_file(&file_path).unwrap();

        // Verify
        assert_eq!(test_data.content, deserialized.content);
        assert_eq!(test_data.metadata.version, deserialized.metadata.version);
    }

    #[tokio::test]
    async fn test_pveil_file_concurrent_io() {
        use tokio::fs as tokio_fs;

        let provider = JsonProvider::<TestData>::new();
        let manager = FormatManager::new(provider);
        let base_data = Message {
            metadata: create_test_metadata(),
            content: TestData {
                field1: "concurrent_file_test".to_string(),
                field2: 42,
                field3: Some(vec!["test".to_string()]),
            },
        };

        let temp_dir = std::env::temp_dir();
        let mut handles = Vec::new();

        // Spawn multiple tasks to write and read files concurrently
        for i in 0..5 {
            let manager = manager.clone();
            let data = base_data.clone();
            let temp_dir = temp_dir.clone();
            
            handles.push(tokio::spawn(async move {
                let file_path = temp_dir.join(format!("test_{}.pveil", i));
                
                // Write file
                let serialized = manager.serialize(&data).await.unwrap();
                tokio_fs::write(&file_path, &serialized).await.unwrap();

                // Read file
                let file_content = tokio_fs::read(&file_path).await.unwrap();
                let deserialized = manager.deserialize(&file_content).await.unwrap();

                // Clean up
                tokio_fs::remove_file(&file_path).await.unwrap();

                // Verify
                assert_eq!(data.content, deserialized.content);
                assert_eq!(data.metadata.version, deserialized.metadata.version);
            }));
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_pveil_file_large_data() {
        use std::fs;

        let provider = BincodeProvider::<ComplexData>::new();
        let manager = FormatManager::new(provider);
        let mut large_array = Vec::with_capacity(10000);
        for i in 0..10000 {
            large_array.push(i);
        }

        let test_data = Message {
            metadata: create_test_metadata(),
            content: ComplexData {
                nested: TestData {
                    field1: "large_file".to_string(),
                    field2: 999999,
                    field3: Some(vec!["large".repeat(1000)]),
                },
                array: large_array,
                map: std::collections::HashMap::new(),
            },
        };

        // Create a temporary file path
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_large.pveil");

        // Serialize and write to file
        let serialized = manager.serialize(&test_data).await.unwrap();
        fs::write(&file_path, &serialized).unwrap();

        // Read from file and deserialize
        let file_content = fs::read(&file_path).unwrap();
        let deserialized = manager.deserialize(&file_content).await.unwrap();

        // Clean up
        fs::remove_file(&file_path).unwrap();

        // Verify
        assert_eq!(test_data.content.nested, deserialized.content.nested);
        assert_eq!(test_data.content.array, deserialized.content.array);
        assert_eq!(test_data.content.map, deserialized.content.map);
    }
} 