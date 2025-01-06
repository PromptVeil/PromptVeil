"""
Binary format handling for PromptVeil (.pveil).
"""

import struct
from typing import BinaryIO, Dict, Any
from dataclasses import dataclass
import ctypes
from pathlib import Path
import json
import zlib

from .exceptions import FormatError

# Magic bytes that identify a .pveil file
PVEIL_MAGIC = b"PVEIL"

@dataclass
class Header:
    """File header structure."""
    magic: bytes = PVEIL_MAGIC
    version: int = 1
    flags: int = 0
    partition_count: int = 0
    schema_offset: int = 0

    @classmethod
    def from_bytes(cls, data: bytes) -> 'Header':
        """Create header from bytes."""
        if len(data) < 24:  # 4 + 4 + 4 + 8 + 8
            raise FormatError("Invalid header size")
            
        magic = data[:4]
        if magic != PVEIL_MAGIC:
            raise FormatError("Invalid magic bytes")
            
        version, flags = struct.unpack("<II", data[4:12])
        partition_count, schema_offset = struct.unpack("<QQ", data[12:28])
        
        return cls(
            magic=magic,
            version=version,
            flags=flags,
            partition_count=partition_count,
            schema_offset=schema_offset
        )
    
    def to_bytes(self) -> bytes:
        """Convert header to bytes."""
        return (
            self.magic +
            struct.pack("<II", self.version, self.flags) +
            struct.pack("<QQ", self.partition_count, self.schema_offset)
        )

class PVeilWriter:
    """Writes data in .pveil format."""
    
    def __init__(self, file: BinaryIO):
        self.file = file
        self.header = Header()
        self._write_header()
        self._current_offset = 28  # After header
    
    def _write_header(self) -> None:
        """Write the file header."""
        self.file.write(self.header.to_bytes())
    
    def _write_block(self, block_type: int, data: bytes) -> None:
        """Write a data block with size prefix."""
        # Block structure:
        # - Type (4 bytes)
        # - Size (8 bytes)
        # - Data (variable)
        # - CRC32 (4 bytes)
        
        size = len(data)
        crc = zlib.crc32(data)
        
        self.file.write(struct.pack("<IQ", block_type, size))
        self.file.write(data)
        self.file.write(struct.pack("<I", crc))
        
        self._current_offset += 16 + size  # 4 + 8 + size + 4
    
    def write_metadata(self, metadata: Dict[str, Any]) -> None:
        """Write store metadata."""
        # Convert metadata to bytes
        meta_bytes = json.dumps(metadata).encode('utf-8')
        
        # Write as block type 1
        self._write_block(1, meta_bytes)
        
        # Update schema offset in header
        self.header.schema_offset = self._current_offset
    
    def begin_partition(self) -> None:
        """Begin a new partition."""
        self.header.partition_count += 1
        
        # Write partition header (block type 2)
        partition_header = {
            'index': self.header.partition_count,
            'offset': self._current_offset
        }
        header_bytes = json.dumps(partition_header).encode('utf-8')
        self._write_block(2, header_bytes)
    
    def write_conversation(self, conv_id: str, messages: list, metadata: Dict[str, Any]) -> None:
        """Write a conversation to the current partition."""
        # Prepare conversation data
        conv_data = {
            'id': conv_id,
            'messages': messages,
            'metadata': metadata
        }
        conv_bytes = json.dumps(conv_data).encode('utf-8')
        
        # Write as block type 3
        self._write_block(3, conv_bytes)
    
    def finalize(self) -> None:
        """Finalize the file and update header."""
        # Write end marker (block type 0)
        self._write_block(0, b"")
        
        # Seek back to start and write updated header
        self.file.seek(0)
        self._write_header()

class PVeilReader:
    """Reads data from .pveil format."""
    
    def __init__(self, file: BinaryIO):
        self.file = file
        self.header = self._read_header()
        self._current_offset = 28  # After header
    
    def _read_header(self) -> Header:
        """Read and validate the file header."""
        header_data = self.file.read(28)  # Fixed header size
        return Header.from_bytes(header_data)
    
    def _read_block(self) -> tuple[int, bytes]:
        """Read a data block and verify CRC."""
        # Read block header
        block_header = self.file.read(12)  # 4 + 8
        if not block_header:
            return (0, b"")  # End of file
            
        block_type, size = struct.unpack("<IQ", block_header)
        
        # Read data
        data = self.file.read(size)
        if len(data) != size:
            raise FormatError("Incomplete block data")
            
        # Read and verify CRC
        crc_bytes = self.file.read(4)
        if not crc_bytes:
            raise FormatError("Missing block CRC")
            
        expected_crc = struct.unpack("<I", crc_bytes)[0]
        actual_crc = zlib.crc32(data)
        
        if actual_crc != expected_crc:
            raise FormatError("Block CRC mismatch")
            
        self._current_offset += 16 + size  # 4 + 8 + size + 4
        return (block_type, data)
    
    def read_metadata(self) -> Dict[str, Any]:
        """Read store metadata."""
        block_type, data = self._read_block()
        if block_type != 1:
            raise FormatError("Expected metadata block")
            
        return json.loads(data.decode('utf-8'))
    
    def read_conversations(self) -> Dict[str, Dict]:
        """Read all conversations."""
        conversations = {}
        
        while True:
            block_type, data = self._read_block()
            
            if block_type == 0:  # End marker
                break
                
            if block_type == 2:  # Partition header
                continue  # Skip for now
                
            if block_type == 3:  # Conversation
                conv_data = json.loads(data.decode('utf-8'))
                conversations[conv_data['id']] = {
                    'messages': conv_data['messages'],
                    'metadata': conv_data['metadata']
                }
                
        return conversations

def save_store(store: Any, path: Path) -> None:
    """
    Save a store to a .pveil file.
    
    Args:
        store: The store to save
        path: Path to save to
    """
    with open(path, 'wb') as f:
        writer = PVeilWriter(f)
        
        # Write store metadata
        writer.write_metadata({
            'version': store.metadata.version,
            'created_at': store.metadata.created_at,
            'last_modified': store.metadata.last_modified,
            'conversation_count': store.metadata.conversation_count
        })
        
        # Write conversations
        writer.begin_partition()
        for conv_id, conv in store.conversations.items():
            messages = [
                {
                    'role': msg.role,
                    'content': msg.content,
                    'timestamp': msg.timestamp,
                    'metadata': msg.metadata
                }
                for msg in conv.messages
            ]
            writer.write_conversation(conv_id, messages, conv.metadata)
            
        writer.finalize()

def load_store(path: Path) -> Dict[str, Any]:
    """
    Load a store from a .pveil file.
    
    Args:
        path: Path to load from
        
    Returns:
        Dict containing store data
    """
    with open(path, 'rb') as f:
        reader = PVeilReader(f)
        
        return {
            'metadata': reader.read_metadata(),
            'conversations': reader.read_conversations()
        } 