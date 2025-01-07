"""
Utility functions for PromptVeil.
"""

import os
from typing import Optional

def generate_key() -> bytes:
    """
    Generate a new encryption key.
    
    Returns:
        32 bytes of cryptographically secure random data
    """
    return os.urandom(32)

def get_version() -> str:
    """
    Get the current version of PromptVeil.
    
    Returns:
        Version string
    """
    from . import __version__
    return __version__

def validate_file(path: str) -> bool:
    """
    Validate a .pveil file.
    
    Args:
        path: Path to the file to validate
        
    Returns:
        True if the file is valid, False otherwise
    """
    if not os.path.exists(path):
        return False
        
    if not path.endswith('.pveil'):
        return False
        
    # TODO: Implement actual validation using core functionality
    return True

def get_file_info(path: str) -> Optional[dict]:
    """
    Get information about a .pveil file.
    
    Args:
        path: Path to the file
        
    Returns:
        Dictionary containing file information or None if file is invalid
    """
    if not validate_file(path):
        return None
        
    # TODO: Implement actual file info extraction using core functionality
    return {
        'version': get_version(),
        'compression': 'unknown',
        'encrypted': True,
        'size': os.path.getsize(path)
    } 