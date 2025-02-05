use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::types::{PyDict, PyList};
use std::path::PathBuf;
use tokio::runtime::Runtime;

use crate::{
    PromptVeilCore, ConversationData, Message,
    Config, SecurityConfig, IndexConfig, FormatConfig,
    CoreError,
};

/// Python wrapper for PromptVeilCore
#[pyclass(name = "PromptVeilCore")]
struct PyPromptVeilCore {
    core: PromptVeilCore,
    runtime: Runtime,
}

#[pymethods]
impl PyPromptVeilCore {
    #[new]
    fn new(base_path: String, config: Option<&PyDict>) -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        let config = config.map(|d| {
            Python::with_gil(|py| {
                d.extract::<Config>()
                    .map_err(|e| PyValueError::new_err(format!("Invalid config: {}", e)))
            })
        }).transpose()?.unwrap_or_else(|| Config {
            security: SecurityConfig {
                key_rotation_days: 30,
                encryption_enabled: true,
                hardware_acceleration: true,
            },
            index: IndexConfig {
                vector_dim: 768,
                max_elements: 100_000,
                ef_construction: 200,
                m: 16,
            },
            format: FormatConfig {
                compression_enabled: true,
                compression_level: 6,
            },
        });

        let core = runtime.block_on(async {
            PromptVeilCore::new(PathBuf::from(base_path), config)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to initialize core: {}", e)))
        })?;

        Ok(Self { core, runtime })
    }

    fn add_conversation(
        &self,
        py: Python<'_>,
        conversation: &PyDict,
    ) -> PyResult<&PyAny> {
        let conversation = Python::with_gil(|py| {
            conversation.extract::<ConversationData>()
                .map_err(|e| PyValueError::new_err(format!("Invalid conversation data: {}", e)))
        })?;

        let core = &self.core;
        let fut = async move {
            core.add_conversation(conversation)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to add conversation: {}", e)))
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn add_message(
        &self,
        py: Python<'_>,
        conversation_id: String,
        role: String,
        content: String,
        metadata: Option<&PyDict>,
    ) -> PyResult<&PyAny> {
        let metadata = metadata.map(|d| {
            Python::with_gil(|py| {
                d.extract::<serde_json::Value>()
                    .map_err(|e| PyValueError::new_err(format!("Invalid metadata: {}", e)))
            })
        }).transpose()?;

        let core = &self.core;
        let fut = async move {
            core.add_message(&conversation_id, &role, &content, metadata)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to add message: {}", e)))
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn save_conversations(
        &self,
        py: Python<'_>,
        conversations: Vec<&PyDict>,
    ) -> PyResult<&PyAny> {
        let conversations = conversations.into_iter()
            .map(|d| {
                Python::with_gil(|py| {
                    d.extract::<ConversationData>()
                        .map_err(|e| PyValueError::new_err(format!("Invalid conversation data: {}", e)))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let core = &self.core;
        let fut = async move {
            core.save_conversations(conversations)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to save conversations: {}", e)))
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn search_text(
        &self,
        py: Python<'_>,
        query: String,
        limit: usize,
    ) -> PyResult<&PyAny> {
        let core = &self.core;
        let fut = async move {
            let results = core.search_text(&query, limit)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Search failed: {}", e)))?;

            Python::with_gil(|py| {
                let py_results = PyList::empty(py);
                for result in results {
                    let dict = PyDict::new(py);
                    dict.set_item("conversation_id", result.conversation_id)?;
                    dict.set_item("score", result.score)?;
                    dict.set_item("snippet", result.snippet)?;
                    dict.set_item("highlights", result.highlights)?;
                    py_results.append(dict)?;
                }
                Ok(py_results)
            })
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn search_similar(
        &self,
        py: Python<'_>,
        vector: Vec<f32>,
        limit: usize,
    ) -> PyResult<&PyAny> {
        let core = &self.core;
        let fut = async move {
            let results = core.search_similar(vector, limit)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Search failed: {}", e)))?;

            Python::with_gil(|py| {
                let py_results = PyList::empty(py);
                for result in results {
                    let dict = PyDict::new(py);
                    dict.set_item("conversation_id", result.conversation_id)?;
                    dict.set_item("similarity", result.similarity)?;
                    dict.set_item("vector_distance", result.vector_distance)?;
                    py_results.append(dict)?;
                }
                Ok(py_results)
            })
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn get_conversation(
        &self,
        py: Python<'_>,
        conversation_id: String,
    ) -> PyResult<&PyAny> {
        let core = &self.core;
        let fut = async move {
            let conversation = core.get_conversation(&conversation_id)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get conversation: {}", e)))?;

            Python::with_gil(|py| {
                let dict = PyDict::new(py);
                dict.set_item("id", conversation.id)?;
                dict.set_item("messages", conversation.messages)?;
                dict.set_item("created_at", conversation.created_at)?;
                dict.set_item("updated_at", conversation.updated_at)?;
                if let Some(metadata) = conversation.metadata {
                    dict.set_item("metadata", metadata)?;
                }
                Ok(dict)
            })
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn delete_conversation(
        &self,
        py: Python<'_>,
        conversation_id: String,
    ) -> PyResult<&PyAny> {
        let core = &self.core;
        let fut = async move {
            core.delete_conversation(&conversation_id)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to delete conversation: {}", e)))
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn clear(&self, py: Python<'_>) -> PyResult<&PyAny> {
        let core = &self.core;
        let fut = async move {
            core.clear()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to clear: {}", e)))
        };

        pyo3_asyncio::tokio::future_into_py(py, fut)
    }

    fn get_config(&self, py: Python<'_>) -> PyResult<PyObject> {
        let config = self.core.get_config();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("security", &config.security)?;
            dict.set_item("index", &config.index)?;
            dict.set_item("format", &config.format)?;
            Ok(dict.into())
        })
    }

    fn update_config(&mut self, py: Python<'_>, config: &PyDict) -> PyResult<()> {
        let config = Python::with_gil(|py| {
            config.extract::<Config>()
                .map_err(|e| PyValueError::new_err(format!("Invalid config: {}", e)))
        })?;

        self.core.update_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to update config: {}", e)))
    }
}

/// Python module
#[pymodule]
fn promptveil_core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyPromptVeilCore>()?;
    Ok(())
} 