from typing import List, Optional, Dict, Any
from datetime import datetime
from pydantic import BaseModel, Field

class Message(BaseModel):
    """A single message in a conversation."""
    role: str = Field(..., description="Role of the message sender (e.g., 'user', 'assistant')")
    content: str = Field(..., description="Content of the message")
    timestamp: datetime = Field(default_factory=datetime.now, description="When the message was created")
    metadata: Optional[Dict[str, Any]] = Field(default=None, description="Additional message metadata")

class Conversation(BaseModel):
    """A conversation containing multiple messages."""
    id: str = Field(..., description="Unique identifier for the conversation")
    messages: List[Message] = Field(default_factory=list, description="List of messages in the conversation")
    metadata: Optional[Dict[str, Any]] = Field(default=None, description="Additional conversation metadata")
    created_at: datetime = Field(default_factory=datetime.now, description="When the conversation was created")
    updated_at: datetime = Field(default_factory=datetime.now, description="When the conversation was last updated")

    def add_message(self, role: str, content: str, metadata: Optional[Dict[str, Any]] = None) -> None:
        """Add a new message to the conversation."""
        message = Message(role=role, content=content, metadata=metadata)
        self.messages.append(message)
        self.updated_at = datetime.now()

class SearchResult(BaseModel):
    """A search result containing conversation and relevance information."""
    conversation_id: str = Field(..., description="ID of the matching conversation")
    score: float = Field(..., description="Relevance score of the match")
    snippet: str = Field(..., description="Matching text snippet")
    highlights: List[str] = Field(default_factory=list, description="Highlighted terms in the snippet")

class SimilarityMatch(BaseModel):
    """A similarity search result."""
    conversation_id: str = Field(..., description="ID of the similar conversation")
    similarity: float = Field(..., description="Similarity score (0-1)")
    vector_distance: float = Field(..., description="Vector space distance") 