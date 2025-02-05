use std::path::PathBuf;
use tantivy::{
    schema::{Schema, STORED, TEXT, Value},
    Index, IndexWriter, TantivyDocument, ReloadPolicy,
    collector::TopDocs,
    query::QueryParser,
    Term,
    tokenizer::{TextAnalyzer, SimpleTokenizer},
    TantivyError,
};
use std::sync::RwLock;
use crate::{IndexError, IndexConfig, SearchResult};

impl From<TantivyError> for IndexError {
    fn from(err: TantivyError) -> Self {
        IndexError::TextIndex(err.to_string())
    }
}

pub struct TextIndex {
    index: Index,
    writer: RwLock<IndexWriter>,
    schema: Schema,
    config: IndexConfig,
}

impl TextIndex {
    pub fn new(path: PathBuf, config: IndexConfig) -> Result<Self, IndexError> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_dir(&path, schema.clone())
            .map_err(|e| IndexError::InitializationError(e.to_string()))?;

        // Configure text analyzer based on config
        match config.text_analyzer.as_str() {
            "simple" => {
                index.tokenizers()
                    .register("simple", TextAnalyzer::from(SimpleTokenizer::default()));
            },
            // Add more analyzers as needed
            _ => {} // Use default analyzer
        }

        let writer = index
            .writer(config.text_index_memory)
            .map_err(|e| IndexError::InitializationError(e.to_string()))?;

        Ok(Self {
            index,
            writer: RwLock::new(writer),
            schema,
            config,
        })
    }

    pub fn add_document(&self, id: &str, content: &str) -> Result<(), IndexError> {
        let mut doc = TantivyDocument::default();
        doc.add_text(self.schema.get_field("id").unwrap(), id);
        doc.add_text(self.schema.get_field("content").unwrap(), content);

        let mut writer = self.writer.write().unwrap();
        writer.add_document(doc)?;
        writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query_text: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        let reader = self.index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| IndexError::TextIndex(e.to_string()))?;

        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.schema.get_field("content").unwrap()],
        );

        let query = query_parser
            .parse_query(query_text)
            .map_err(|e| IndexError::TextIndex(e.to_string()))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| IndexError::TextIndex(e.to_string()))?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| IndexError::TextIndex(e.to_string()))?;

            let id = retrieved_doc
                .get_first(self.schema.get_field("id").unwrap())
                .ok_or_else(|| IndexError::TextIndex("Missing document ID".to_string()))?
                .as_str()
                .ok_or_else(|| IndexError::TextIndex("Invalid document ID type".to_string()))?;

            let content = retrieved_doc
                .get_first(self.schema.get_field("content").unwrap())
                .ok_or_else(|| IndexError::TextIndex("Missing document content".to_string()))?
                .as_str()
                .ok_or_else(|| IndexError::TextIndex("Invalid document content type".to_string()))?;

            let snippet = content[..std::cmp::min(200, content.len())].to_string();
            let highlights = if self.config.enable_highlighting {
                let terms: Vec<&str> = query_text
                    .split_whitespace()
                    .collect();
                
                let mut highlights = Vec::new();
                for term in terms {
                    if let Some(idx) = content.to_lowercase().find(&term.to_lowercase()) {
                        let start = idx.saturating_sub(30);
                        let end = (idx + term.len() + 30).min(content.len());
                        let highlight = content[start..end].to_string();
                        highlights.push(highlight);
                    }
                }
                highlights
            } else {
                vec![]
            };

            results.push(SearchResult {
                conversation_id: id.to_string(),
                score,
                snippet,
                highlights,
            });
        }

        Ok(results)
    }

    pub fn delete_document(&self, id: &str) -> Result<(), IndexError> {
        let mut writer = self.writer.write().unwrap();
        let term = Term::from_field_text(self.schema.get_field("id").unwrap(), id);
        writer.delete_term(term);
        writer.commit()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), IndexError> {
        let mut writer = self.writer.write().unwrap();
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_text_index_basic_operations() -> Result<(), IndexError> {
        let dir = tempdir().unwrap();
        let config = IndexConfig::default();
        let index = TextIndex::new(dir.path().to_path_buf(), config)?;

        // Test document addition
        index.add_document("doc1", "This is a test document about Rust programming")?;
        index.add_document("doc2", "Another document about Python programming")?;

        // Test search - both documents contain "programming"
        let results = index.search("programming", 10)?;
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.conversation_id == "doc1"));
        assert!(results.iter().any(|r| r.conversation_id == "doc2"));

        // Test more specific search
        let results = index.search("Rust programming", 10)?;
        assert!(results.iter().any(|r| r.conversation_id == "doc1"));
        assert!(results.iter().find(|r| r.conversation_id == "doc1").unwrap().score >
               results.iter().find(|r| r.conversation_id == "doc2").unwrap_or(&results[0]).score);

        // Test document deletion
        index.delete_document("doc1")?;
        let results = index.search("Rust programming", 10)?;
        assert!(!results.iter().any(|r| r.conversation_id == "doc1"));

        // Test clear
        index.clear()?;
        let results = index.search("programming", 10)?;
        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[test]
    fn test_text_index_highlighting() -> Result<(), IndexError> {
        let dir = tempdir().unwrap();
        let mut config = IndexConfig::default();
        config.enable_highlighting = true;
        let index = TextIndex::new(dir.path().to_path_buf(), config)?;

        index.add_document("doc1", "The quick brown fox jumps over the lazy dog")?;

        let results = index.search("quick fox", 10)?;
        assert_eq!(results.len(), 1);
        assert!(!results[0].highlights.is_empty());
        assert!(results[0].highlights.iter().any(|h| h.contains("quick")));
        assert!(results[0].highlights.iter().any(|h| h.contains("fox")));

        Ok(())
    }
} 