"""
PromptVeil - A secure framework for managing LLM conversations.
"""

from .conversation import Conversation, Message
from .store import ConversationStore, StoreMetadata
from .exceptions import (
    PromptVeilError,
    SecurityError,
    CompressionError,
    FormatError,
    StoreError
)

__version__ = "0.1.0"

__all__ = [
    'Conversation',
    'ConversationStore',
    'Message',
    'StoreMetadata',
    'PromptVeilError',
    'SecurityError',
    'CompressionError',
    'FormatError',
    'StoreError'
] 