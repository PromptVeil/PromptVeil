"""
Tests for the .pveil binary format.
"""

import unittest
import io
import os
import tempfile
from pathlib import Path

from promptveil.format import (
    Header,
    PVeilWriter,
    PVeilReader,
    PVEIL_MAGIC,
    FormatError
)

class TestHeader(unittest.TestCase):
    def test_header_creation(self):
        """Test header creation and serialization."""
        header = Header()
        self.assertEqual(header.magic, PVEIL_MAGIC)
        self.assertEqual(header.version, 1)
        
        # Test serialization
        data = header.to_bytes()
        self.assertEqual(len(data), 28)  # 4 + 4 + 4 + 8 + 8
        
        # Test deserialization
        loaded = Header.from_bytes(data)
        self.assertEqual(loaded.magic, PVEIL_MAGIC)
        self.assertEqual(loaded.version, 1)
        self.assertEqual(loaded.flags, 0)
        self.assertEqual(loaded.partition_count, 0)
        self.assertEqual(loaded.schema_offset, 0)
        
    def test_invalid_header(self):
        """Test header validation."""
        # Test invalid magic
        with self.assertRaises(FormatError):
            Header.from_bytes(b"INVALID" + b"\x00" * 22)
            
        # Test invalid size
        with self.assertRaises(FormatError):
            Header.from_bytes(b"PVEIL")

class TestPVeilWriter(unittest.TestCase):
    def test_write_header(self):
        """Test writing file header."""
        buffer = io.BytesIO()
        writer = PVeilWriter(buffer)
        
        # Check header was written
        buffer.seek(0)
        data = buffer.read()
        self.assertEqual(data[:4], PVEIL_MAGIC)
        self.assertEqual(len(data), 28)
        
    def test_partition_counting(self):
        """Test partition count tracking."""
        buffer = io.BytesIO()
        writer = PVeilWriter(buffer)
        
        writer.begin_partition()
        writer.begin_partition()
        
        self.assertEqual(writer.header.partition_count, 2)
        
        # Check header was updated
        writer.finalize()
        buffer.seek(0)
        reader = PVeilReader(buffer)
        self.assertEqual(reader.header.partition_count, 2)

class TestPVeilReader(unittest.TestCase):
    def setUp(self):
        """Create a test file."""
        self.buffer = io.BytesIO()
        writer = PVeilWriter(self.buffer)
        writer.finalize()
        self.buffer.seek(0)
        
    def test_read_header(self):
        """Test reading file header."""
        reader = PVeilReader(self.buffer)
        self.assertEqual(reader.header.magic, PVEIL_MAGIC)
        self.assertEqual(reader.header.version, 1)
        
    def test_invalid_file(self):
        """Test reading invalid file."""
        invalid = io.BytesIO(b"NOT_A_PVEIL_FILE")
        with self.assertRaises(FormatError):
            PVeilReader(invalid)

class TestFormatIntegration(unittest.TestCase):
    def test_write_and_read(self):
        """Test writing and reading a complete file."""
        # Create test data
        metadata = {
            'version': "0.1.0",
            'created_at': 1234567890.0,
            'conversation_count': 1
        }
        
        messages = [
            {
                'role': "user",
                'content': "Hello!",
                'timestamp': 1234567890.0,
                'metadata': None
            }
        ]
        
        # Write to temporary file
        with tempfile.NamedTemporaryFile(suffix='.pveil', delete=False) as tf:
            writer = PVeilWriter(tf)
            writer.write_metadata(metadata)
            writer.begin_partition()
            writer.write_conversation("test-id", messages, {})
            writer.finalize()
            path = Path(tf.name)
            
        try:
            # Read back
            with open(path, 'rb') as f:
                reader = PVeilReader(f)
                read_metadata = reader.read_metadata()
                read_conversations = reader.read_conversations()
                
            # Verify metadata
            self.assertEqual(read_metadata['version'], "0.1.0")
            self.assertEqual(read_metadata['conversation_count'], 1)
            
            # Verify conversations
            self.assertIn("test-id", read_conversations)
            conv = read_conversations["test-id"]
            self.assertEqual(len(conv['messages']), 1)
            self.assertEqual(conv['messages'][0]['content'], "Hello!")
            
        finally:
            os.unlink(path)

if __name__ == '__main__':
    unittest.main() 