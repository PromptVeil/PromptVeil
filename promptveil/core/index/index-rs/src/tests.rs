#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_config() -> IndexConfig {
        IndexConfig {
            vector_dim: 4,  // Smaller dimension for testing
            max_elements: 100,
            ef_construction: 10,
            m: 8,
            text_analyzer: "default".to_string(),
            text_index_memory: 1_000_000,  // 1MB for testing
            enable_highlighting: true,
        }
    }

    async fn create_test_index() -> (Index, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let config = create_test_config();
        let index = Index::new_with_config(path.clone(), config).await.unwrap();
        (index, path)
    }

    #[tokio::test]
    async fn test_basic_operations() {
        let (index, _path) = create_test_index().await;
        
        // Test data
        let id = "test1";
        let text = "This is a test document with some specific words for searching.";
        let vector = vec![0.1, 0.2, 0.3, 0.4];

        // Add conversation
        index.add_conversation(id, text, vector.clone()).await.unwrap();

        // Search text
        let results = index.search_text("specific words", 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].conversation_id, id);
        assert!(results[0].score > 0.0);
        assert!(!results[0].snippet.is_empty());
        assert!(!results[0].highlights.is_empty());

        // Search similar
        let similar = index.search_similar(vector, 5).await.unwrap();
        assert!(!similar.is_empty());
        assert_eq!(similar[0].conversation_id, id);
        assert!(similar[0].similarity > 0.9); // Should be very similar to itself

        // Delete conversation
        index.delete_conversation(id).await.unwrap();

        // Verify deletion
        let results = index.search_text("specific words", 5).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_documents() {
        let (index, _path) = create_test_index().await;

        // Add multiple documents
        for i in 0..5 {
            let id = format!("test{}", i);
            let text = format!("Document {} with unique content for testing multiple documents", i);
            let vector = vec![i as f32 * 0.1, (i + 1) as f32 * 0.1, (i + 2) as f32 * 0.1, (i + 3) as f32 * 0.1];
            index.add_conversation(&id, &text, vector).await.unwrap();
        }

        // Search text across all documents
        let results = index.search_text("unique content", 10).await.unwrap();
        assert_eq!(results.len(), 5);

        // Search similar vectors
        let query_vector = vec![0.1, 0.2, 0.3, 0.4]; // Similar to first document
        let similar = index.search_similar(query_vector, 5).await.unwrap();
        assert_eq!(similar.len(), 5);
        
        // Verify ordering by similarity
        for i in 1..similar.len() {
            assert!(similar[i-1].similarity >= similar[i].similarity);
        }
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let config = create_test_config();

        // Create and populate index
        {
            let index = Index::new_with_config(path.clone(), config.clone()).await.unwrap();
            index.add_conversation(
                "test1",
                "Persistent document",
                vec![0.1, 0.2, 0.3, 0.4]
            ).await.unwrap();
        }

        // Create new index instance and verify data persists
        {
            let index = Index::new_with_config(path.clone(), config).await.unwrap();
            let results = index.search_text("Persistent", 5).await.unwrap();
            assert!(!results.is_empty());
            assert_eq!(results[0].conversation_id, "test1");

            let similar = index.search_similar(vec![0.1, 0.2, 0.3, 0.4], 5).await.unwrap();
            assert!(!similar.is_empty());
            assert_eq!(similar[0].conversation_id, "test1");
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let (index, _path) = create_test_index().await;

        // Test invalid vector dimensions
        let result = index.add_conversation(
            "test1",
            "Test document",
            vec![0.1, 0.2] // Wrong dimensions
        ).await;
        assert!(matches!(result, Err(IndexError::InvalidVectorDimensions { .. })));

        // Test deleting non-existent document
        let result = index.delete_conversation("nonexistent").await;
        assert!(matches!(result, Err(IndexError::DocumentNotFound(_))));
    }

    #[tokio::test]
    async fn test_clear() {
        let (index, _path) = create_test_index().await;

        // Add some documents
        for i in 0..3 {
            let id = format!("test{}", i);
            let text = format!("Document {}", i);
            let vector = vec![i as f32 * 0.1, (i + 1) as f32 * 0.1, (i + 2) as f32 * 0.1, (i + 3) as f32 * 0.1];
            index.add_conversation(&id, &text, vector).await.unwrap();
        }

        // Verify documents exist
        assert!(!index.search_text("Document", 5).await.unwrap().is_empty());

        // Clear index
        index.clear().await.unwrap();

        // Verify all documents are removed
        assert!(index.search_text("Document", 5).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_highlighting() {
        let (index, _path) = create_test_index().await;

        // Add document with specific phrases
        index.add_conversation(
            "test1",
            "This is a test document containing some important phrases. \
             The important information should be highlighted. \
             Multiple occurrences of important terms should all be found.",
            vec![0.1, 0.2, 0.3, 0.4]
        ).await.unwrap();

        // Search with terms that should be highlighted
        let results = index.search_text("important phrases", 5).await.unwrap();
        assert!(!results.is_empty());
        assert!(!results[0].highlights.is_empty());

        // Verify highlights contain the search terms
        for highlight in &results[0].highlights {
            assert!(
                highlight.to_lowercase().contains("important") ||
                highlight.to_lowercase().contains("phrases")
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (index, _path) = create_test_index().await;
        let index = Arc::new(index);

        let mut handles = Vec::new();

        // Spawn multiple tasks to add documents concurrently
        for i in 0..10 {
            let index = index.clone();
            let handle = tokio::spawn(async move {
                let id = format!("test{}", i);
                let text = format!("Concurrent document {}", i);
                let vector = vec![i as f32 * 0.1, (i + 1) as f32 * 0.1, (i + 2) as f32 * 0.1, (i + 3) as f32 * 0.1];
                index.add_conversation(&id, &text, vector).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all additions to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all documents were added
        let results = index.search_text("Concurrent", 20).await.unwrap();
        assert_eq!(results.len(), 10);
    }
} 