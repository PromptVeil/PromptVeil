"""
PromptVeil Python Package

This package provides Python bindings for the PromptVeil core library implemented in Rust.
It offers a high-level interface for secure conversation management and retrieval.
"""

from typing import List, Optional, Dict, Any, Union
from pathlib import Path
import asyncio
import json

from .models.conversation import Conversation, Message, SearchResult, SimilarityMatch
from .models.config import PromptVeilConfig, SecurityConfig, IndexConfig, FormatConfig
from promptveil_core import PromptVeilCore

__version__ = "0.2.0"
__all__ = ["PromptVeilCore"]

class PromptVeil:
    """Main interface for PromptVeil functionality."""

    def __init__(self, base_path: Union[str, Path], **config_kwargs):
        """Initialize PromptVeil.

        Args:
            base_path: Base path for all data
            **config_kwargs: Additional configuration options
        """
        self.config = PromptVeilConfig(base_path=Path(base_path), **config_kwargs)
        self._core = PromptVeilCore(str(self.config.base_path))
        self._loop = asyncio.get_event_loop()

    async def add_conversation(self, conversation: Conversation) -> None:
        """Add a conversation to the store.

        Args:
            conversation: Conversation to add
        """
        # Convert conversation to format expected by Rust
        content = "\n".join(f"{msg.role}: {msg.content}" for msg in conversation.messages)
        vector = self._compute_vector(content)  # TODO: Implement vector computation
        
        await self._core.add_conversation(
            id=conversation.id,
            content=content,
            vector=vector,
            metadata=conversation.metadata
        )

    async def search_text(
        self,
        query: str,
        limit: int = 10,
        min_score: float = 0.0
    ) -> List[SearchResult]:
        """Search conversations by text.

        Args:
            query: Search query
            limit: Maximum number of results
            min_score: Minimum score threshold

        Returns:
            List of search results
        """
        return await self._core.search_text(query, limit)

    async def search_similar(
        self,
        text: str,
        limit: int = 10,
        min_similarity: float = 0.0
    ) -> List[SimilarityMatch]:
        """Search conversations by similarity.

        Args:
            text: Text to find similar conversations to
            limit: Maximum number of results
            min_similarity: Minimum similarity threshold

        Returns:
            List of similarity matches
        """
        vector = self._compute_vector(text)  # TODO: Implement vector computation
        return await self._core.search_similar(vector, limit)

    async def get_conversation(self, conversation_id: str) -> Conversation:
        """Retrieve a conversation by ID.

        Args:
            conversation_id: ID of the conversation to retrieve

        Returns:
            Retrieved conversation
        """
        data = await self._core.get_conversation(conversation_id)
        return Conversation.model_validate(data)

    async def delete_conversation(self, conversation_id: str) -> None:
        """Delete a conversation.

        Args:
            conversation_id: ID of the conversation to delete
        """
        await self._core.delete_conversation(conversation_id)

    async def clear(self) -> None:
        """Clear all conversations."""
        await self._core.clear()

    def _compute_vector(self, text: str) -> List[float]:
        """Compute vector representation of text.

        Args:
            text: Text to vectorize

        Returns:
            Vector representation
        """
        # TODO: Implement proper vector computation
        return [0.0] * self.config.index.vector_dim

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        pass

    async def __aenter__(self):
        """Async context manager entry."""
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit.""" 