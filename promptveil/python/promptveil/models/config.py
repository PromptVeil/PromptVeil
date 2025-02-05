from typing import Optional
from pathlib import Path
from pydantic import BaseModel, Field, validator

class SecurityConfig(BaseModel):
    """Security-related configuration."""
    key_rotation_days: int = Field(default=30, description="Days between key rotations")
    encryption_enabled: bool = Field(default=True, description="Whether encryption is enabled")
    hardware_acceleration: bool = Field(default=True, description="Whether to use hardware acceleration")

class IndexConfig(BaseModel):
    """Index-related configuration."""
    vector_dim: int = Field(default=768, description="Dimension of vectors")
    max_elements: int = Field(default=100_000, description="Maximum number of elements in the index")
    ef_construction: int = Field(default=200, description="HNSW ef_construction parameter")
    m: int = Field(default=16, description="HNSW M parameter")

class FormatConfig(BaseModel):
    """Format-related configuration."""
    compression_enabled: bool = Field(default=True, description="Whether compression is enabled")
    compression_level: int = Field(default=6, description="Compression level (1-9)")

class PromptVeilConfig(BaseModel):
    """Main configuration for PromptVeil."""
    base_path: Path = Field(..., description="Base path for all data")
    security: SecurityConfig = Field(default_factory=SecurityConfig, description="Security configuration")
    index: IndexConfig = Field(default_factory=IndexConfig, description="Index configuration")
    format: FormatConfig = Field(default_factory=FormatConfig, description="Format configuration")

    @validator("base_path")
    def validate_base_path(cls, v: Path) -> Path:
        """Ensure base path exists and is writable."""
        v.mkdir(parents=True, exist_ok=True)
        if not v.is_dir():
            raise ValueError(f"Base path {v} is not a directory")
        return v.absolute() 