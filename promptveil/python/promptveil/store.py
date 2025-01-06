"""
Core conversation store handling for PromptVeil.
"""

from typing import List, Dict, Optional, Union
from dataclasses import dataclass
import os
import uuid
import time
import json
from pathlib import Path

from .conversation import Conversation
from .exceptions import StoreError, SecurityError
from .format import save_store, load_store
from .utils import generate_key
from .core.security import encrypt, decrypt
from .indexing import ConversationIndex, SearchResult


@dataclass
class StoreMetadata:
    """Metadata for the conversation store."""
    version: str = "0.1.0"
    created_at: float = 0.0
    last_modified: float = 0.0
    conversation_count: int = 0


class ConversationStore:
    """
    Manages multiple conversations with basic storage and retrieval.
    
    Example:
        >>> store = ConversationStore()
        >>> conv = Conversation()
        >>> conv.add_message("user", "Hello!")
        >>> conv_id = store.add_conversation(conv)
        >>> retrieved = store.get_conversation(conv_id)
        >>> results = store.search("Hello")
    """
    
    def __init__(self, path: Optional[str] = None):
        """
        Initialize a new conversation store.
        
        Args:
            path: Optional path to load an existing store
        """
        self.conversations: Dict[str, Conversation] = {}
        self.metadata = StoreMetadata(created_at=time.time(), last_modified=time.time())
        self._key = generate_key()  # Store-level encryption key
        self._index = ConversationIndex()  # Search index
        
        if path:
            self._load(path)
    
    def add_conversation(self, conversation: Conversation) -> str:
        """
        Add a conversation to the store.
        
        Args:
            conversation: The conversation to add
            
        Returns:
            str: The ID of the added conversation
        """
        conv_id = str(uuid.uuid4())
        self.conversations[conv_id] = conversation
        self.metadata.conversation_count += 1
        self.metadata.last_modified = time.time()
        
        # Update index
        self._index.add_conversation(conv_id, conversation)
        
        return conv_id
    
    def get_conversation(self, conv_id: str) -> Conversation:
        """
        Retrieve a conversation by ID.
        
        Args:
            conv_id: The ID of the conversation to retrieve
            
        Returns:
            The requested conversation
            
        Raises:
            KeyError: If the conversation is not found
        """
        if conv_id not in self.conversations:
            raise KeyError(f"Conversation not found: {conv_id}")
        return self.conversations[conv_id]
    
    def search(
        self,
        query: str,
        limit: int = 10,
        role_filter: Optional[str] = None
    ) -> List[SearchResult]:
        """
        Search for conversations matching the query.
        
        Args:
            query: Search query
            limit: Maximum number of results
            role_filter: Optional filter by message role
            
        Returns:
            List of search results, sorted by relevance
        """
        return self._index.search(query, limit, role_filter)
    
    def save(self, path: str) -> None:
        """
        Save the store to a .pveil file.
        
        Args:
            path: Path to save the file
            
        Raises:
            StoreError: If there's an error saving the store
            SecurityError: If encryption fails
        """
        if not path.endswith('.pveil'):
            path += '.pveil'
            
        try:
            # Prepare store data
            store_data = {
                'metadata': {
                    'version': self.metadata.version,
                    'created_at': self.metadata.created_at,
                    'last_modified': self.metadata.last_modified,
                    'conversation_count': self.metadata.conversation_count
                },
                'conversations': {},
                'index': {
                    'term_index': {
                        term: {
                            conv_id: list(msg_indices)
                            for conv_id, msg_indices in conv_dict.items()
                        }
                        for term, conv_dict in self._index.term_index.items()
                    },
                    'metadata': self._index.conversation_metadata,
                    'last_update': self._index.last_update
                }
            }
            
            # Add conversations
            for conv_id, conv in self.conversations.items():
                store_data['conversations'][conv_id] = {
                    'messages': [
                        {
                            'role': msg.role,
                            'content': msg.content,
                            'timestamp': msg.timestamp,
                            'metadata': msg.metadata
                        }
                        for msg in conv.messages
                    ],
                    'metadata': conv.metadata
                }
            
            # Convert to bytes and encrypt
            json_data = json.dumps(store_data).encode('utf-8')
            encrypted_data = encrypt(json_data, self._key)
            
            # Write to file
            with open(path, 'wb') as f:
                f.write(encrypted_data)
                
        except Exception as e:
            raise StoreError(f"Error saving store: {str(e)}")
    
    def _load(self, path: str) -> None:
        """
        Load a store from a .pveil file.
        
        Args:
            path: Path to the .pveil file
            
        Raises:
            FileNotFoundError: If the file doesn't exist
            StoreError: If there's an error loading the store
            SecurityError: If decryption fails
        """
        if not os.path.exists(path):
            raise FileNotFoundError(f"File not found: {path}")
            
        try:
            # Read and decrypt data
            with open(path, 'rb') as f:
                encrypted_data = f.read()
                
            decrypted_data = decrypt(encrypted_data, self._key)
            store_data = json.loads(decrypted_data.decode('utf-8'))
            
            # Load metadata
            meta = store_data['metadata']
            self.metadata = StoreMetadata(
                version=meta['version'],
                created_at=meta['created_at'],
                last_modified=meta['last_modified'],
                conversation_count=meta['conversation_count']
            )
            
            # Load conversations
            self.conversations = {}
            for conv_id, conv_data in store_data['conversations'].items():
                conv = Conversation()
                conv.metadata = conv_data['metadata']
                
                for msg_data in conv_data['messages']:
                    conv.add_message(
                        role=msg_data['role'],
                        content=msg_data['content'],
                        metadata=msg_data.get('metadata')
                    )
                    
                self.conversations[conv_id] = conv
                
            # Load index
            index_data = store_data.get('index', {})
            if index_data:
                for term, conv_dict in index_data['term_index'].items():
                    for conv_id, msg_indices in conv_dict.items():
                        self._index.term_index[term][conv_id] = set(msg_indices)
                self._index.conversation_metadata = index_data['metadata']
                self._index.last_update = index_data['last_update']
            else:
                # Rebuild index if not present
                for conv_id, conv in self.conversations.items():
                    self._index.add_conversation(conv_id, conv)
                
        except Exception as e:
            raise StoreError(f"Error loading store: {str(e)}")
    
    def __len__(self) -> int:
        return len(self.conversations)
    
    def __contains__(self, conv_id: str) -> bool:
        return conv_id in self.conversations 