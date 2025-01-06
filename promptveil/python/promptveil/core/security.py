"""Security and encryption functionality."""

from typing import Union, BinaryIO
from pathlib import Path
import os

from ..exceptions import SecurityError


def encrypt(data: Union[bytes, str], key: Union[str, bytes]) -> bytes:
    """
    Encrypt data using AES-GCM.
    
    Args:
        data: Data to encrypt
        key: Encryption key (32 bytes)
        
    Returns:
        Encrypted data
        
    Raises:
        SecurityError: If encryption fails
    """
    try:
        # Convert string data to bytes if needed
        if isinstance(data, str):
            data = data.encode('utf-8')
            
        # Convert string key to bytes if needed
        if isinstance(key, str):
            key = key.encode('utf-8')
            
        # Validate key length
        if len(key) != 32:
            raise SecurityError("Key must be 32 bytes")
            
        # Call Rust encryption
        from .._promptveil_core import encrypt
        return encrypt(data, key)
        
    except Exception as e:
        raise SecurityError(f"Encryption failed: {str(e)}")


def decrypt(data: Union[bytes, BinaryIO], key: Union[str, bytes]) -> bytes:
    """
    Decrypt data using AES-GCM.
    
    Args:
        data: Encrypted data
        key: Decryption key (32 bytes)
        
    Returns:
        Decrypted data
        
    Raises:
        SecurityError: If decryption fails
    """
    try:
        # Handle file-like objects
        if hasattr(data, 'read'):
            data = data.read()
            
        # Convert string key to bytes if needed
        if isinstance(key, str):
            key = key.encode('utf-8')
            
        # Validate key length
        if len(key) != 32:
            raise SecurityError("Key must be 32 bytes")
            
        # Call Rust decryption
        from .._promptveil_core import decrypt
        return decrypt(data, key)
        
    except Exception as e:
        raise SecurityError(f"Decryption failed: {str(e)}") 