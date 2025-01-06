"""
Indexing and search functionality for PromptVeil.
"""

from typing import List, Dict, Optional, Set
import json
import time
from dataclasses import dataclass
from collections import defaultdict
import re

from .conversation import Conversation, Message


@dataclass
class SearchResult:
    """A search result with relevance information."""
    conversation_id: str
    message_index: int
    score: float
    snippet: str
    timestamp: float
    role: str


class ConversationIndex:
    """
    Maintains searchable indexes for conversations.
    
    Example:
        >>> index = ConversationIndex()
        >>> index.add_conversation("conv1", conversation)
        >>> results = index.search("Hello world")
    """
    
    def __init__(self):
        """Initialize index structures."""
        self.term_index: Dict[str, Dict[str, Set[int]]] = defaultdict(lambda: defaultdict(set))
        self.conversation_metadata: Dict[str, Dict] = {}
        self.last_update = time.time()
    
    def add_conversation(self, conv_id: str, conversation: Conversation) -> None:
        """
        Add or update a conversation in the index.
        
        Args:
            conv_id: Unique conversation ID
            conversation: Conversation to index
        """
        # Remove existing entries for this conversation
        self.remove_conversation(conv_id)
        
        # Index each message
        for i, message in enumerate(conversation.messages):
            terms = self._tokenize(message.content)
            for term in terms:
                self.term_index[term][conv_id].add(i)
        
        # Store metadata
        self.conversation_metadata[conv_id] = {
            'timestamp': time.time(),
            'message_count': len(conversation.messages),
            'roles': [msg.role for msg in conversation.messages],
            'messages': {
                str(i): {
                    'content': msg.content,
                    'role': msg.role,
                    'timestamp': msg.timestamp
                }
                for i, msg in enumerate(conversation.messages)
            }
        }
        
        self.last_update = time.time()
    
    def remove_conversation(self, conv_id: str) -> None:
        """
        Remove a conversation from the index.
        
        Args:
            conv_id: ID of conversation to remove
        """
        # Remove from term index
        for term_dict in self.term_index.values():
            if conv_id in term_dict:
                del term_dict[conv_id]
        
        # Remove metadata
        if conv_id in self.conversation_metadata:
            del self.conversation_metadata[conv_id]
            
        self.last_update = time.time()
    
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
        query_terms = self._tokenize(query)
        scores: Dict[tuple[str, int], float] = defaultdict(float)
        
        # Calculate scores
        for term in query_terms:
            if term in self.term_index:
                # Check if term is a phrase
                is_phrase = ' ' in term
                phrase_boost = 2.0 if is_phrase else 1.0
                
                for conv_id, message_indices in self.term_index[term].items():
                    for msg_idx in message_indices:
                        # Skip if role doesn't match filter
                        if role_filter and self.conversation_metadata[conv_id]['roles'][msg_idx] != role_filter:
                            continue
                        
                        # Calculate term frequency in message
                        msg_content = self.conversation_metadata[conv_id]['messages'][str(msg_idx)]['content'].lower()
                        term_freq = msg_content.count(term)
                        
                        # Calculate document frequency
                        doc_freq = len(self.term_index[term])
                        
                        # TF-IDF score with phrase boost
                        score = (term_freq * phrase_boost) / (doc_freq + 1)
                        
                        # Boost score for title/first message
                        if msg_idx == 0:
                            score *= 1.5
                        
                        # Boost score for recent messages
                        msg_time = self.conversation_metadata[conv_id]['messages'][str(msg_idx)]['timestamp']
                        time_diff = time.time() - msg_time
                        recency_boost = 1.0 + (1.0 / (time_diff / 86400 + 1))  # 86400 seconds in a day
                        score *= recency_boost
                        
                        scores[(conv_id, msg_idx)] += score
        
        # Check for term proximity in messages
        for conv_id, msg_indices in set((k[0], v) for k, v in scores.items()):
            for msg_idx in msg_indices:
                msg_content = self.conversation_metadata[conv_id]['messages'][str(msg_idx)]['content'].lower()
                words = msg_content.split()
                
                # Find positions of query terms
                term_positions = defaultdict(list)
                for i, word in enumerate(words):
                    if word in query_terms:
                        term_positions[word].append(i)
                
                # Calculate minimum distance between terms
                if len(term_positions) > 1:
                    positions = [pos for pos_list in term_positions.values() for pos in pos_list]
                    min_dist = min(abs(p1 - p2) for p1 in positions for p2 in positions if p1 != p2)
                    proximity_boost = 1.0 + (1.0 / (min_dist + 1))
                    scores[(conv_id, msg_idx)] *= proximity_boost
        
        # Sort results
        results = []
        for (conv_id, msg_idx), score in sorted(scores.items(), key=lambda x: x[1], reverse=True):
            if len(results) >= limit:
                break
                
            metadata = self.conversation_metadata[conv_id]
            results.append(SearchResult(
                conversation_id=conv_id,
                message_index=msg_idx,
                score=score,
                snippet=self._get_snippet(conv_id, msg_idx),
                timestamp=metadata['messages'][str(msg_idx)]['timestamp'],
                role=metadata['roles'][msg_idx]
            ))
        
        return results
    
    def _tokenize(self, text: str) -> Set[str]:
        """
        Convert text to searchable terms.
        
        Args:
            text: Text to tokenize
            
        Returns:
            Set of terms
        """
        if not text:
            return set()
            
        # Convert to lowercase
        text = text.lower()
        
        # Replace special characters with spaces
        text = re.sub(r'[^\w\s]', ' ', text)
        
        # Split on whitespace
        words = text.split()
        
        # Remove very short words and common stop words
        stop_words = {'a', 'an', 'and', 'are', 'as', 'at', 'be', 'by', 'for',
                     'from', 'has', 'he', 'in', 'is', 'it', 'its', 'of', 'on',
                     'that', 'the', 'to', 'was', 'were', 'will', 'with'}
        
        terms = {word for word in words 
                if len(word) > 2 and word not in stop_words}
        
        # Add word pairs for phrase matching
        if len(words) > 1:
            pairs = {f"{words[i]} {words[i+1]}"
                    for i in range(len(words)-1)}
            terms.update(pairs)
        
        return terms
    
    def _get_snippet(self, conv_id: str, msg_idx: int) -> str:
        """
        Get a preview snippet of the message.
        
        Args:
            conv_id: Conversation ID
            msg_idx: Message index
            
        Returns:
            Snippet of the message content
        """
        if conv_id not in self.conversation_metadata:
            return "..."
            
        metadata = self.conversation_metadata[conv_id]
        if msg_idx >= metadata['message_count']:
            return "..."
            
        # Get message content
        message = metadata.get('messages', {}).get(str(msg_idx), {}).get('content', '')
        if not message:
            return "..."
            
        # Create snippet
        max_length = 100
        if len(message) <= max_length:
            return message
            
        # Try to break at word boundary
        end = max_length
        while end > max_length - 20 and end > 0:
            if message[end].isspace():
                break
            end -= 1
            
        if end == 0:
            end = max_length
            
        return message[:end].strip() + "..."
    
    def save(self, path: str) -> None:
        """
        Save index to file.
        
        Args:
            path: Path to save to
        """
        data = {
            'term_index': {
                term: {
                    conv_id: list(msg_indices)
                    for conv_id, msg_indices in conv_dict.items()
                }
                for term, conv_dict in self.term_index.items()
            },
            'metadata': self.conversation_metadata,
            'last_update': self.last_update
        }
        
        with open(path, 'w') as f:
            json.dump(data, f)
    
    @classmethod
    def load(cls, path: str) -> 'ConversationIndex':
        """
        Load index from file.
        
        Args:
            path: Path to load from
            
        Returns:
            Loaded index
        """
        index = cls()
        
        with open(path) as f:
            data = json.load(f)
            
        # Reconstruct term index
        for term, conv_dict in data['term_index'].items():
            for conv_id, msg_indices in conv_dict.items():
                index.term_index[term][conv_id] = set(msg_indices)
        
        index.conversation_metadata = data['metadata']
        index.last_update = data['last_update']
        
        return index 