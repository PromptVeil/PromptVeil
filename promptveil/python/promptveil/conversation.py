"""
Core conversation handling for PromptVeil.
"""

from typing import List, Dict, Optional, Union
from dataclasses import dataclass
import json
import os
import time

from .utils import generate_key
from .core.security import encrypt, decrypt
from .exceptions import SecurityError


@dataclass
class Message:
    """A single message in a conversation."""
    role: str
    content: str
    timestamp: float
    metadata: Optional[Dict] = None


class Conversation:
    """
    Manages LLM conversations with compression and encryption.
    
    Example:
        >>> conv = Conversation()
        >>> conv.add_message("user", "Hello, how are you?")
        >>> conv.add_message("assistant", "I'm doing well, thank you!")
        >>> conv.save("chat.pveil")
        >>> 
        >>> loaded = Conversation.load("chat.pveil")
        >>> print(loaded.messages)
    """
    
    def __init__(self):
        self.messages: List[Message] = []
        self.metadata: Dict = {}
        self._key = None
    
    def add_message(self, role: str, content: str, metadata: Optional[Dict] = None) -> None:
        """Add a message to the conversation."""
        message = Message(role=role, content=content, timestamp=time.time(), metadata=metadata)
        self.messages.append(message)
    
    def save(self, path: str, compression_level: int = 9) -> None:
        """
        Save the conversation to a .pveil file.
        
        Args:
            path: Path to save the file
            compression_level: Compression level (1-9)
            
        Raises:
            SecurityError: If encryption fails
        """
        if not path.endswith('.pveil'):
            path += '.pveil'
            
        if self._key is None:
            self._key = generate_key()
            
        # Convert to format structure
        data = {
            'messages': [
                {
                    'role': msg.role,
                    'content': msg.content,
                    'timestamp': msg.timestamp,
                    'metadata': msg.metadata
                }
                for msg in self.messages
            ],
            'metadata': self.metadata
        }
        
        # Convert to bytes and encrypt
        json_data = json.dumps(data).encode('utf-8')
        encrypted_data = encrypt(json_data, self._key)
        
        # Write encrypted data
        with open(path, 'wb') as f:
            f.write(encrypted_data)
    
    @classmethod
    def load(cls, path: str, key: Optional[bytes] = None) -> 'Conversation':
        """
        Load a conversation from a .pveil file.
        
        Args:
            path: Path to the .pveil file
            key: Optional decryption key. If not provided, will try to use the key
                 from the original conversation.
            
        Returns:
            Loaded conversation
            
        Raises:
            FileNotFoundError: If file doesn't exist
            SecurityError: If decryption fails
        """
        if not os.path.exists(path):
            raise FileNotFoundError(f"File not found: {path}")
            
        # Read and decrypt data
        with open(path, 'rb') as f:
            encrypted_data = f.read()
            
        if key is None:
            # This would be the case when loading a conversation that was just saved
            key = cls._key
            
        if key is None:
            raise SecurityError("No decryption key provided")
            
        try:
            decrypted_data = decrypt(encrypted_data, key)
            data = json.loads(decrypted_data.decode('utf-8'))
        except Exception as e:
            raise SecurityError(f"Failed to decrypt conversation: {str(e)}")
            
        # Create conversation object
        conv = cls()
        conv.metadata = data.get('metadata', {})
        conv._key = key
        
        for msg_data in data['messages']:
            msg = Message(
                role=msg_data['role'],
                content=msg_data['content'],
                timestamp=msg_data['timestamp'],
                metadata=msg_data.get('metadata')
            )
            conv.messages.append(msg)
            
        return conv
    
    def __len__(self) -> int:
        return len(self.messages)
    
    def __getitem__(self, idx: Union[int, slice]) -> Union[Message, List[Message]]:
        return self.messages[idx] 