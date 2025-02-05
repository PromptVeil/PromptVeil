#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_config() -> Config {
        Config {
            security: SecurityConfig {
                key_rotation_days: 30,
                encryption_enabled: true,
                hardware_acceleration: true,
            },
            index: IndexConfig {
                vector_dim: 384, // matches all-MiniLM-L6-v2 dimension
                max_elements: 1000,
                ef_construction: 100,
                m: 16,
            },
            format: FormatConfig {
                compression_enabled: true,
                compression_level: 6,
            },
        }
    }

    fn create_test_message(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now().timestamp(),
        }
    }

    fn create_test_conversation(id: &str, messages: Vec<Message>) -> Conversation {
        Conversation {
            id: id.to_string(),
            messages,
            metadata: None,
        }
    }

    async fn setup_test_core() -> (PromptVeilCore, tempfile::TempDir) {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let config = create_test_config();
        let core = PromptVeilCore::new(temp_dir.path().to_path_buf(), config)
            .await
            .expect("Failed to create core");
        (core, temp_dir)
    }

    #[tokio::test]
    async fn test_core_initialization() {
        let (core, _temp_dir) = setup_test_core().await;
        assert!(core.get_config().index.vector_dim == 384);
    }

    #[tokio::test]
    async fn test_conversation_operations() {
        let (core, _temp_dir) = setup_test_core().await;

        // Create test conversation
        let messages = vec![
            create_test_message("user", "Hello, how does machine learning work?"),
            create_test_message("assistant", "Machine learning is a type of artificial intelligence..."),
        ];
        let conversation = create_test_conversation("test1", messages);

        // Test adding conversation
        core.add_conversation(conversation.clone())
            .await
            .expect("Failed to add conversation");

        // Test retrieving conversation
        let retrieved = core.get_conversation("test1")
            .await
            .expect("Failed to retrieve conversation");
        assert_eq!(retrieved.id, "test1");
        assert_eq!(retrieved.messages.len(), 2);

        // Test text search
        let text_results = core.search_text("machine learning", 5)
            .await
            .expect("Failed to perform text search");
        assert!(!text_results.is_empty());
        assert_eq!(text_results[0].conversation_id, "test1");

        // Test semantic search
        let semantic_results = core.search_semantic("AI and neural networks", 5)
            .await
            .expect("Failed to perform semantic search");
        assert!(!semantic_results.is_empty());

        // Test hybrid search
        let hybrid_results = core.hybrid_search("artificial intelligence", 0.5, 0.5, 5)
            .await
            .expect("Failed to perform hybrid search");
        assert!(!hybrid_results.is_empty());

        // Test deleting conversation
        core.delete_conversation("test1")
            .await
            .expect("Failed to delete conversation");

        // Verify deletion
        let result = core.get_conversation("test1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let (core, _temp_dir) = setup_test_core().await;

        // Create multiple test conversations
        let conversations: Vec<_> = (0..5)
            .map(|i| {
                let messages = vec![
                    create_test_message("user", &format!("Test message {}", i)),
                    create_test_message("assistant", &format!("Test response {}", i)),
                ];
                create_test_conversation(&format!("batch{}", i), messages)
            })
            .collect();

        // Test batch save
        core.save_conversations(conversations.clone())
            .await
            .expect("Failed to save conversations in batch");

        // Verify all conversations were saved
        for conv in conversations {
            let retrieved = core.get_conversation(&conv.id)
                .await
                .expect("Failed to retrieve conversation");
            assert_eq!(retrieved.messages.len(), 2);
        }

        // Test clear
        core.clear()
            .await
            .expect("Failed to clear conversations");

        // Verify all conversations were deleted
        for i in 0..5 {
            let result = core.get_conversation(&format!("batch{}", i)).await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let (core, _temp_dir) = setup_test_core().await;

        // Test invalid conversation ID
        let result = core.get_conversation("nonexistent").await;
        assert!(matches!(result, Err(CoreError::NotFound(_))));

        // Test invalid search parameters
        let result = core.hybrid_search("test", 1.5, 0.5, 5).await;
        assert!(matches!(result, Err(CoreError::InvalidInput(_))));

        // Test empty conversation
        let empty_conv = create_test_conversation("empty", vec![]);
        let result = core.add_conversation(empty_conv).await;
        assert!(matches!(result, Err(CoreError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_config_updates() {
        let (mut core, _temp_dir) = setup_test_core().await;

        let new_config = Config {
            security: SecurityConfig {
                key_rotation_days: 60,
                encryption_enabled: false,
                hardware_acceleration: false,
            },
            ..create_test_config()
        };

        core.update_config(new_config.clone())
            .expect("Failed to update config");

        let updated_config = core.get_config();
        assert_eq!(updated_config.security.key_rotation_days, 60);
        assert_eq!(updated_config.security.encryption_enabled, false);
    }

    #[tokio::test]
    async fn test_conversation_metadata() {
        let (core, _temp_dir) = setup_test_core().await;

        let metadata = serde_json::json!({
            "tags": ["test", "metadata"],
            "priority": 1,
            "custom": {
                "field": "value"
            }
        });

        let mut conversation = create_test_conversation("metadata_test", vec![
            create_test_message("user", "Test message with metadata"),
        ]);
        conversation.metadata = Some(metadata.clone());

        core.add_conversation(conversation)
            .await
            .expect("Failed to add conversation with metadata");

        let retrieved = core.get_conversation("metadata_test")
            .await
            .expect("Failed to retrieve conversation");

        assert!(retrieved.metadata.is_some());
        assert_eq!(retrieved.metadata.unwrap(), metadata);
    }
} 