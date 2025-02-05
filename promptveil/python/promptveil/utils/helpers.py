"""Utility functions for PromptVeil."""

import uuid
from typing import Any, Dict, Optional
from datetime import datetime
import json

def generate_id() -> str:
    """Generate a unique ID.

    Returns:
        Unique identifier string
    """
    return str(uuid.uuid4())

def format_timestamp(dt: Optional[datetime] = None) -> float:
    """Format a timestamp for storage.

    Args:
        dt: Datetime to format, or current time if None

    Returns:
        Unix timestamp as float
    """
    if dt is None:
        dt = datetime.now()
    return dt.timestamp()

def parse_timestamp(ts: float) -> datetime:
    """Parse a stored timestamp.

    Args:
        ts: Unix timestamp

    Returns:
        Datetime object
    """
    return datetime.fromtimestamp(ts)

def sanitize_metadata(metadata: Optional[Dict[str, Any]]) -> Dict[str, Any]:
    """Sanitize metadata for storage.

    Args:
        metadata: Raw metadata dictionary

    Returns:
        Sanitized metadata dictionary
    """
    if metadata is None:
        return {}
    
    # Convert to JSON and back to ensure serializable
    return json.loads(json.dumps(metadata))

def merge_metadata(base: Dict[str, Any], update: Optional[Dict[str, Any]]) -> Dict[str, Any]:
    """Merge two metadata dictionaries.

    Args:
        base: Base metadata
        update: Updates to apply

    Returns:
        Merged metadata dictionary
    """
    if update is None:
        return base.copy()
    
    result = base.copy()
    result.update(update)
    return result 