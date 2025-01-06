"""Tests for security functionality."""

import unittest
import os
import tempfile
from pathlib import Path

from promptveil.core.security import encrypt, decrypt
from promptveil.utils import generate_key
from promptveil.exceptions import SecurityError
from promptveil.conversation import Conversation


class TestEncryption(unittest.TestCase):
    """Test encryption and decryption functionality."""
    
    def setUp(self):
        """Set up test environment."""
        self.key = generate_key()
        self.test_data = b"Hello, World!"
    
    def test_encryption_decryption(self):
        """Test basic encryption and decryption."""
        # Encrypt data
        encrypted = encrypt(self.test_data, self.key)
        self.assertNotEqual(encrypted, self.test_data)
        
        # Decrypt data
        decrypted = decrypt(encrypted, self.key)
        self.assertEqual(decrypted, self.test_data)
    
    def test_string_input(self):
        """Test encryption with string input."""
        test_str = "Hello, World!"
        encrypted = encrypt(test_str, self.key)
        decrypted = decrypt(encrypted, self.key)
        self.assertEqual(decrypted.decode('utf-8'), test_str)
    
    def test_invalid_key(self):
        """Test encryption with invalid key."""
        invalid_key = b"too short"
        with self.assertRaises(SecurityError):
            encrypt(self.test_data, invalid_key)
    
    def test_wrong_key(self):
        """Test decryption with wrong key."""
        encrypted = encrypt(self.test_data, self.key)
        wrong_key = generate_key()  # Different key
        with self.assertRaises(SecurityError):
            decrypt(encrypted, wrong_key)


class TestConversationEncryption(unittest.TestCase):
    """Test conversation encryption functionality."""
    
    def setUp(self):
        """Set up test environment."""
        self.temp_dir = tempfile.mkdtemp()
        self.test_file = Path(self.temp_dir) / "test.pveil"
    
    def tearDown(self):
        """Clean up test files."""
        if os.path.exists(self.test_file):
            os.unlink(self.test_file)
        os.rmdir(self.temp_dir)
    
    def test_conversation_save_load(self):
        """Test saving and loading encrypted conversations."""
        # Create and save conversation
        conv = Conversation()
        conv.add_message("user", "Hello!")
        conv.add_message("assistant", "Hi there!")
        conv.save(str(self.test_file))
        
        # Load conversation
        loaded = Conversation.load(str(self.test_file), conv._key)
        
        # Verify contents
        self.assertEqual(len(loaded.messages), 2)
        self.assertEqual(loaded.messages[0].role, "user")
        self.assertEqual(loaded.messages[0].content, "Hello!")
        self.assertEqual(loaded.messages[1].role, "assistant")
        self.assertEqual(loaded.messages[1].content, "Hi there!")
    
    def test_conversation_wrong_key(self):
        """Test loading conversation with wrong key."""
        # Save conversation
        conv = Conversation()
        conv.add_message("user", "Hello!")
        conv.save(str(self.test_file))
        
        # Try to load with wrong key
        wrong_key = generate_key()
        with self.assertRaises(SecurityError):
            Conversation.load(str(self.test_file), wrong_key)
    
    def test_conversation_no_key(self):
        """Test loading conversation without key."""
        # Save conversation
        conv = Conversation()
        conv.add_message("user", "Hello!")
        conv.save(str(self.test_file))
        
        # Try to load without key
        with self.assertRaises(SecurityError):
            Conversation.load(str(self.test_file))


if __name__ == '__main__':
    unittest.main() 