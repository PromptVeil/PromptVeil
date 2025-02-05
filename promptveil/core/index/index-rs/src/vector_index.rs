use std::path::PathBuf;
use std::sync::RwLock;
use std::collections::HashMap;
use hnsw_rs::prelude::*;
use crate::{IndexError, IndexConfig};

#[derive(Clone)]
struct EuclideanDistance;

impl Distance<f32> for EuclideanDistance {
    fn eval(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum = 0.0;
        for (x, y) in a.iter().zip(b.iter()) {
            let diff = x - y;
            sum += diff * diff;
        }
        sum.sqrt()
    }
}

pub struct VectorIndex<'a> {
    index: RwLock<Hnsw<'a, f32, EuclideanDistance>>,
    id_map: RwLock<HashMap<String, usize>>,
    vectors: RwLock<HashMap<String, Vec<f32>>>,  // Cache of vectors
    config: IndexConfig,
}

impl<'a> VectorIndex<'a> {
    pub fn new(_path: PathBuf, config: &IndexConfig) -> Result<Self, IndexError> {
        let distance = EuclideanDistance;
        let index = Hnsw::new(
            config.m,             // max_nb_connection
            config.max_elements,  // max_elements
            3,                    // max_layer (default value)
            config.ef_construction,
            distance,
        );

        let instance = Self {
            index: RwLock::new(index),
            id_map: RwLock::new(HashMap::new()),
            vectors: RwLock::new(HashMap::new()),
            config: config.clone(),
        };

        Ok(instance)
    }

    pub fn add_vector(&self, id: String, vector: Vec<f32>) -> Result<(), IndexError> {
        if vector.len() != self.config.vector_dim {
            return Err(IndexError::InvalidVectorDimensions {
                expected: self.config.vector_dim,
                got: vector.len(),
            });
        }

        let index = self.index.write().map_err(|e| 
            IndexError::VectorIndex(format!("Failed to acquire write lock: {}", e)))?;
        
        let mut id_map = self.id_map.write().map_err(|e|
            IndexError::VectorIndex(format!("Failed to acquire write lock for id_map: {}", e)))?;

        let mut vectors = self.vectors.write().map_err(|e|
            IndexError::VectorIndex(format!("Failed to acquire write lock for vectors: {}", e)))?;

        let point_id = id_map.len();
        index.insert((&vector, point_id));
        id_map.insert(id.clone(), point_id);
        vectors.insert(id, vector);
        Ok(())
    }

    pub fn search_similar(&self, vector: Vec<f32>, k: usize) -> Result<Vec<(String, f32)>, IndexError> {
        if vector.len() != self.config.vector_dim {
            return Err(IndexError::InvalidVectorDimensions {
                expected: self.config.vector_dim,
                got: vector.len(),
            });
        }
        
        let index = self.index.read().map_err(|e| 
            IndexError::VectorIndex(format!("Failed to acquire read lock: {}", e)))?;

        let id_map = self.id_map.read().map_err(|e|
            IndexError::VectorIndex(format!("Failed to acquire read lock for id_map: {}", e)))?;

        let ef_search = k * 2; // Use a larger ef_search for better recall
        let neighbors = index.search(&vector, k, ef_search);

        let mut results = Vec::with_capacity(neighbors.len());
        for neighbor in neighbors {
            if let Some((id, _)) = id_map.iter().find(|(_, &idx)| idx == neighbor.d_id) {
                results.push((id.clone(), neighbor.distance));
            }
        }

        Ok(results)
    }

    pub fn delete_vector(&self, id: &str) -> Result<(), IndexError> {
        let mut id_map = self.id_map.write().map_err(|e|
            IndexError::VectorIndex(format!("Failed to acquire write lock for id_map: {}", e)))?;

        let mut vectors = self.vectors.write().map_err(|e|
            IndexError::VectorIndex(format!("Failed to acquire write lock for vectors: {}", e)))?;

        if let Some(_) = id_map.remove(id) {
            vectors.remove(id);

            let mut index = self.index.write().map_err(|e| 
                IndexError::VectorIndex(format!("Failed to acquire write lock: {}", e)))?;

            // Create a new index with the same parameters
            let distance = EuclideanDistance;
            *index = Hnsw::new(
                self.config.m,
                self.config.max_elements,
                3,  // max_layer (default value)
                self.config.ef_construction,
                distance,
            );

            // Reinsert all vectors except the deleted one
            for (id, vector) in vectors.iter() {
                let point_id = id_map.len();
                index.insert((vector, point_id));
                id_map.insert(id.clone(), point_id);
            }

            Ok(())
        } else {
            Err(IndexError::DocumentNotFound(id.to_string()))
        }
    }

    pub fn clear(&self) -> Result<(), IndexError> {
        let mut index = self.index.write().map_err(|e| 
            IndexError::VectorIndex(format!("Failed to acquire write lock: {}", e)))?;

        let mut id_map = self.id_map.write().map_err(|e|
            IndexError::VectorIndex(format!("Failed to acquire write lock for id_map: {}", e)))?;

        let mut vectors = self.vectors.write().map_err(|e|
            IndexError::VectorIndex(format!("Failed to acquire write lock for vectors: {}", e)))?;

        // Create a new empty index with the same parameters
        let distance = EuclideanDistance;
        *index = Hnsw::new(
            self.config.m,
            self.config.max_elements,
            3,  // max_layer (default value)
            self.config.ef_construction,
            distance,
        );

        id_map.clear();
        vectors.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_vector(values: &[f32]) -> Vec<f32> {
        values.to_vec()
    }

    #[test]
    fn test_vector_index_basic_operations() -> Result<(), IndexError> {
        let dir = tempdir().unwrap();
        let mut config = IndexConfig::default();
        config.vector_dim = 3;
        config.max_elements = 10;
        config.m = 5;
        config.ef_construction = 10;

        let index = VectorIndex::new(dir.path().to_path_buf(), &config)?;

        // Test vector addition
        let vec1 = create_test_vector(&[1.0, 0.0, 0.0]);
        let vec2 = create_test_vector(&[0.0, 1.0, 0.0]);
        let vec3 = create_test_vector(&[0.0, 0.0, 1.0]);

        index.add_vector("vec1".to_string(), vec1.clone())?;
        index.add_vector("vec2".to_string(), vec2.clone())?;
        index.add_vector("vec3".to_string(), vec3)?;

        // Test similarity search
        let results = index.search_similar(vec1, 2)?;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "vec1"); // Most similar should be itself
        assert!(results[0].1 < results[1].1); // First result should have smaller distance

        // Test vector deletion
        index.delete_vector("vec1")?;
        let results = index.search_similar(vec2.clone(), 2)?;
        assert_eq!(results.len(), 2);
        assert!(!results.iter().any(|(id, _)| id == "vec1")); // vec1 should not be in results

        // Test clear
        index.clear()?;
        let results = index.search_similar(vec2, 1)?;
        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[test]
    fn test_vector_dimension_validation() {
        let dir = tempdir().unwrap();
        let mut config = IndexConfig::default();
        config.vector_dim = 3;
        
        let index = VectorIndex::new(dir.path().to_path_buf(), &config).unwrap();

        // Test with correct dimensions
        let correct_vec = create_test_vector(&[1.0, 0.0, 0.0]);
        assert!(index.add_vector("correct".to_string(), correct_vec).is_ok());

        // Test with wrong dimensions
        let wrong_vec = create_test_vector(&[1.0, 0.0, 0.0, 0.0]);
        let result = index.add_vector("wrong".to_string(), wrong_vec);
        assert!(matches!(result, 
            Err(IndexError::InvalidVectorDimensions { expected: 3, got: 4 })));
    }
} 