"""
Integration tests for PromptVeil security system.
Tests the interaction between Rust and Python layers.
"""

import unittest
from promptveil_core import encrypt, decrypt, generate_key

class TestSecurityIntegration(unittest.TestCase):
    """Test full security pipeline."""
    
    def setUp(self):
        """Set up test environment."""
        self.key = generate_key()
        self.data = b"Sensitive data"
    
    def test_encryption_decryption(self):
        """Test complete encryption and decryption pipeline."""
        encrypted = encrypt(self.data, self.key)
        decrypted = decrypt(encrypted, self.key)
        self.assertEqual(self.data, decrypted)
    
    def test_key_generation(self):
        """Test key generation."""
        key = generate_key()
        self.assertEqual(len(key), 32)

if __name__ == '__main__':
    unittest.main() 