"""Tests for indexing and search functionality."""

import unittest
import os
import tempfile
from pathlib import Path
import time

from promptveil.indexing import ConversationIndex, SearchResult
from promptveil.conversation import Conversation


class TestConversationIndex(unittest.TestCase):
    """Test conversation indexing functionality."""
    
    def setUp(self):
        """Set up test environment."""
        self.index = ConversationIndex()
        
        # Create test conversations
        self.conv1 = Conversation()
        self.conv1.add_message("user", "Hello, how are you?")
        self.conv1.add_message("assistant", "I'm doing well, thank you!")
        
        self.conv2 = Conversation()
        self.conv2.add_message("user", "What is machine learning?")
        self.conv2.add_message("assistant", "Machine learning is a field of AI...")
        self.conv2.add_message("user", "Can you give me an example?")
        
        # Add to index
        self.index.add_conversation("conv1", self.conv1)
        self.index.add_conversation("conv2", self.conv2)
    
    def test_basic_search(self):
        """Test basic search functionality."""
        # Search for exact terms
        results = self.index.search("hello")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].conversation_id, "conv1")
        self.assertEqual(results[0].message_index, 0)
        
        # Search for multiple terms
        results = self.index.search("machine learning")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].conversation_id, "conv2")
        
        # Search with no matches
        results = self.index.search("nonexistent")
        self.assertEqual(len(results), 0)
    
    def test_role_filter(self):
        """Test searching with role filters."""
        # Search user messages only
        results = self.index.search("machine learning", role_filter="user")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].role, "user")
        
        # Search assistant messages only
        results = self.index.search("machine learning", role_filter="assistant")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].role, "assistant")
        
        # Search with non-matching role
        results = self.index.search("hello", role_filter="assistant")
        self.assertEqual(len(results), 0)
    
    def test_result_limit(self):
        """Test result limit functionality."""
        # Add more conversations with similar content
        for i in range(5):
            conv = Conversation()
            conv.add_message("user", f"Hello {i}")
            self.index.add_conversation(f"conv_hello_{i}", conv)
        
        # Search with different limits
        results = self.index.search("hello", limit=3)
        self.assertEqual(len(results), 3)
        
        results = self.index.search("hello", limit=10)
        self.assertEqual(len(results), 6)  # Original + 5 new
    
    def test_conversation_removal(self):
        """Test removing conversations from index."""
        # Remove conversation
        self.index.remove_conversation("conv1")
        
        # Verify it's no longer searchable
        results = self.index.search("hello")
        self.assertEqual(len(results), 0)
        
        # Other conversations should still be searchable
        results = self.index.search("machine")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].conversation_id, "conv2")
    
    def test_index_persistence(self):
        """Test saving and loading index."""
        # Save index
        with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as tf:
            self.index.save(tf.name)
            
            # Load in new index
            loaded_index = ConversationIndex.load(tf.name)
            
            # Verify search still works
            results = loaded_index.search("hello")
            self.assertEqual(len(results), 1)
            self.assertEqual(results[0].conversation_id, "conv1")
            
            # Clean up
            os.unlink(tf.name)
    
    def test_metadata_update(self):
        """Test metadata updates."""
        # Add conversation
        conv = Conversation()
        conv.add_message("user", "Test message")
        self.index.add_conversation("conv_test", conv)
        
        # Verify metadata
        metadata = self.index.conversation_metadata["conv_test"]
        self.assertEqual(metadata['message_count'], 1)
        self.assertEqual(metadata['roles'], ["user"])
        self.assertGreater(metadata['timestamp'], 0)
    
    def test_search_ranking(self):
        """Test search result ranking."""
        # Add conversations with varying relevance
        conv_high = Conversation()
        conv_high.add_message("user", "machine learning is amazing")
        conv_high.add_message("assistant", "Yes, machine learning is transforming technology")
        
        conv_medium = Conversation()
        conv_medium.add_message("user", "Tell me about machine vision")
        
        conv_low = Conversation()
        conv_low.add_message("user", "What is learning?")
        
        self.index.add_conversation("high", conv_high)
        self.index.add_conversation("medium", conv_medium)
        self.index.add_conversation("low", conv_low)
        
        # Search and check ranking
        results = self.index.search("machine learning")
        self.assertGreater(len(results), 2)
        
        # Higher relevance should come first
        scores = [r.score for r in results]
        self.assertEqual(sorted(scores, reverse=True), scores)
    
    def test_snippets(self):
        """Test snippet generation."""
        # Test short message (no truncation needed)
        results = self.index.search("hello")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].snippet, "Hello, how are you?")
        
        # Test long message
        conv = Conversation()
        long_msg = "This is a very long message that should be truncated. " * 10
        conv.add_message("user", long_msg)
        self.index.add_conversation("long_conv", conv)
        
        results = self.index.search("long message")
        self.assertEqual(len(results), 1)
        self.assertLess(len(results[0].snippet), len(long_msg))
        self.assertTrue(results[0].snippet.endswith("..."))
        
        # Test word boundary truncation
        conv = Conversation()
        msg = "This message should be truncated at a word boundary and not in the middle of a word"
        conv.add_message("user", msg)
        self.index.add_conversation("boundary_conv", conv)
        
        results = self.index.search("boundary")
        self.assertEqual(len(results), 1)
        snippet = results[0].snippet
        self.assertTrue(snippet.endswith("..."))
        last_word = snippet[:-3].split()[-1]
        self.assertTrue(any(word.startswith(last_word) for word in msg.split()))
    
    def test_tokenization(self):
        """Test text tokenization."""
        # Test basic tokenization
        tokens = self.index._tokenize("Hello World!")
        self.assertEqual(tokens, {"hello world", "hello", "world"})
        
        # Test stop word removal
        tokens = self.index._tokenize("The quick brown fox jumps over the lazy dog")
        self.assertNotIn("the", tokens)
        self.assertIn("quick", tokens)
        self.assertIn("brown", tokens)
        self.assertIn("quick brown", tokens)
        self.assertIn("brown fox", tokens)
        
        # Test special character handling
        tokens = self.index._tokenize("Hello, World! How's it going?")
        self.assertEqual(tokens, {
            "hello world",
            "hello",
            "world",
            "going"
        })
        
        # Test short word removal
        tokens = self.index._tokenize("A to do list")
        self.assertEqual(tokens, {"list"})
        
        # Test case insensitivity
        tokens = self.index._tokenize("HELLO world")
        self.assertEqual(tokens, {"hello world", "hello", "world"})
        
        # Test empty input
        tokens = self.index._tokenize("")
        self.assertEqual(tokens, set())
        
        # Test phrase search
        conv = Conversation()
        conv.add_message("user", "I love machine learning")
        self.index.add_conversation("phrase_conv", conv)
        
        # Should find exact phrase
        results = self.index.search("machine learning")
        self.assertEqual(len(results), 2)  # Including conv2 from setUp
        
        # Should also find individual words
        results = self.index.search("machine")
        self.assertEqual(len(results), 2)
        
        results = self.index.search("learning")
        self.assertEqual(len(results), 2)
    
    def test_search_scoring(self):
        """Test search result scoring and ranking."""
        # Test phrase boost
        conv1 = Conversation()
        conv1.add_message("user", "machine learning is amazing")
        
        conv2 = Conversation()
        conv2.add_message("user", "machine and learning are separate")
        
        self.index.add_conversation("phrase_conv", conv1)
        self.index.add_conversation("separate_conv", conv2)
        
        results = self.index.search("machine learning")
        self.assertGreater(len(results), 1)
        # Exact phrase should rank higher
        phrase_score = next(r.score for r in results if r.conversation_id == "phrase_conv")
        separate_score = next(r.score for r in results if r.conversation_id == "separate_conv")
        self.assertGreater(phrase_score, separate_score)
    
    def test_term_proximity(self):
        """Test term proximity scoring."""
        conv1 = Conversation()
        conv1.add_message("user", "machine learning techniques")
        
        conv2 = Conversation()
        conv2.add_message("user", "machine code and learning concepts")
        
        self.index.add_conversation("close_conv", conv1)
        self.index.add_conversation("far_conv", conv2)
        
        results = self.index.search("machine learning")
        self.assertGreater(len(results), 1)
        # Terms closer together should rank higher
        close_score = next(r.score for r in results if r.conversation_id == "close_conv")
        far_score = next(r.score for r in results if r.conversation_id == "far_conv")
        self.assertGreater(close_score, far_score)
    
    def test_recency_boost(self):
        """Test recency boost in scoring."""
        # Add old conversation
        conv_old = Conversation()
        conv_old.add_message("user", "machine learning is great")
        self.index.add_conversation("old_conv", conv_old)
        
        # Wait a bit
        time.sleep(1)
        
        # Add new conversation
        conv_new = Conversation()
        conv_new.add_message("user", "machine learning is great")
        self.index.add_conversation("new_conv", conv_new)
        
        results = self.index.search("machine learning")
        self.assertGreater(len(results), 1)
        # More recent conversation should rank higher
        new_score = next(r.score for r in results if r.conversation_id == "new_conv")
        old_score = next(r.score for r in results if r.conversation_id == "old_conv")
        self.assertGreater(new_score, old_score)
    
    def test_first_message_boost(self):
        """Test first message boost in scoring."""
        # Conversation with term in first message
        conv1 = Conversation()
        conv1.add_message("user", "machine learning discussion")
        conv1.add_message("assistant", "other content")
        
        # Conversation with term in second message
        conv2 = Conversation()
        conv2.add_message("user", "other content")
        conv2.add_message("assistant", "machine learning discussion")
        
        self.index.add_conversation("first_conv", conv1)
        self.index.add_conversation("second_conv", conv2)
        
        results = self.index.search("machine learning")
        self.assertGreater(len(results), 1)
        # Term in first message should rank higher
        first_score = next(r.score for r in results if r.conversation_id == "first_conv")
        second_score = next(r.score for r in results if r.conversation_id == "second_conv")
        self.assertGreater(first_score, second_score)


if __name__ == '__main__':
    unittest.main() 