"""Tests for conversation store functionality."""

import unittest
import os
import tempfile
from pathlib import Path
import time

from promptveil.store import ConversationStore
from promptveil.conversation import Conversation
from promptveil.exceptions import StoreError, SecurityError
from promptveil.utils import generate_key


class TestConversationStore(unittest.TestCase):
    """Test conversation store functionality."""
    
    def setUp(self):
        """Set up test environment."""
        self.temp_dir = tempfile.mkdtemp()
        self.test_file = Path(self.temp_dir) / "test.pveil"
        self.store = ConversationStore()
        
        # Add test conversations
        self.conv1 = Conversation()
        self.conv1.add_message("user", "Hello, how are you?")
        self.conv1.add_message("assistant", "I'm doing well, thank you!")
        self.conv1_id = self.store.add_conversation(self.conv1)
        
        self.conv2 = Conversation()
        self.conv2.add_message("user", "What is machine learning?")
        self.conv2.add_message("assistant", "Machine learning is a field of AI...")
        self.conv2.add_message("user", "Can you give me an example?")
        self.conv2_id = self.store.add_conversation(self.conv2)
    
    def tearDown(self):
        """Clean up test files."""
        if os.path.exists(self.test_file):
            os.unlink(self.test_file)
        os.rmdir(self.temp_dir)
    
    def test_add_get_conversation(self):
        """Test adding and retrieving conversations."""
        # Create conversation
        conv = Conversation()
        conv.add_message("user", "Hello!")
        conv.add_message("assistant", "Hi there!")
        
        # Add to store
        conv_id = self.store.add_conversation(conv)
        self.assertIn(conv_id, self.store)
        
        # Retrieve conversation
        retrieved = self.store.get_conversation(conv_id)
        self.assertEqual(len(retrieved.messages), 2)
        self.assertEqual(retrieved.messages[0].content, "Hello!")
        self.assertEqual(retrieved.messages[1].content, "Hi there!")
    
    def test_save_load_store(self):
        """Test saving and loading store with encryption."""
        # Save store
        self.store.save(str(self.test_file))
        
        # Create new store and load
        loaded_store = ConversationStore()
        loaded_store._key = self.store._key  # Share key for testing
        loaded_store._load(str(self.test_file))
        
        # Verify conversations
        self.assertEqual(len(loaded_store), 2)
        self.assertIn(self.conv1_id, loaded_store)
        self.assertIn(self.conv2_id, loaded_store)
        
        loaded_conv1 = loaded_store.get_conversation(self.conv1_id)
        self.assertEqual(len(loaded_conv1.messages), 2)
        self.assertEqual(loaded_conv1.messages[0].content, "Hello, how are you?")
        
        loaded_conv2 = loaded_store.get_conversation(self.conv2_id)
        self.assertEqual(len(loaded_conv2.messages), 3)
        self.assertEqual(loaded_conv2.messages[0].content, "What is machine learning?")
    
    def test_wrong_key(self):
        """Test loading store with wrong key."""
        # Save store
        self.store.save(str(self.test_file))
        
        # Try to load with wrong key
        wrong_store = ConversationStore()
        wrong_store._key = generate_key()  # Different key
        
        with self.assertRaises(StoreError):
            wrong_store._load(str(self.test_file))
    
    def test_metadata_persistence(self):
        """Test store metadata persistence with encryption."""
        # Save store
        self.store.save(str(self.test_file))
        
        # Load in new store
        loaded_store = ConversationStore()
        loaded_store._key = self.store._key
        loaded_store._load(str(self.test_file))
        
        # Verify metadata
        self.assertEqual(loaded_store.metadata.version, self.store.metadata.version)
        self.assertEqual(loaded_store.metadata.conversation_count, 2)
        self.assertAlmostEqual(
            loaded_store.metadata.created_at,
            self.store.metadata.created_at,
            places=2
        )
        self.assertAlmostEqual(
            loaded_store.metadata.last_modified,
            self.store.metadata.last_modified,
            places=2
        )
    
    def test_file_corruption(self):
        """Test handling of corrupted store files."""
        # Save store
        self.store.save(str(self.test_file))
        
        # Corrupt file
        with open(self.test_file, 'rb') as f:
            data = bytearray(f.read())
        data[20:30] = b'0' * 10  # Corrupt some bytes
        with open(self.test_file, 'wb') as f:
            f.write(data)
        
        # Try to load corrupted file
        loaded_store = ConversationStore()
        loaded_store._key = self.store._key
        
        with self.assertRaises(StoreError):
            loaded_store._load(str(self.test_file))
    
    def test_basic_search(self):
        """Test basic search functionality."""
        # Search for exact terms
        results = self.store.search("hello")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].conversation_id, self.conv1_id)
        self.assertEqual(results[0].message_index, 0)
        
        # Search for multiple terms
        results = self.store.search("machine learning")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].conversation_id, self.conv2_id)
        
        # Search with no matches
        results = self.store.search("nonexistent")
        self.assertEqual(len(results), 0)
    
    def test_search_role_filter(self):
        """Test search with role filtering."""
        # Search user messages only
        results = self.store.search("machine learning", role_filter="user")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].role, "user")
        
        # Search assistant messages only
        results = self.store.search("machine learning", role_filter="assistant")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].role, "assistant")
    
    def test_search_persistence(self):
        """Test search functionality after save/load."""
        # Save store
        self.store.save(str(self.test_file))
        
        # Load in new store
        loaded_store = ConversationStore()
        loaded_store._key = self.store._key
        loaded_store._load(str(self.test_file))
        
        # Verify search still works
        results = loaded_store.search("hello")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].conversation_id, self.conv1_id)
        
        results = loaded_store.search("machine learning")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].conversation_id, self.conv2_id)


if __name__ == '__main__':
    unittest.main() 