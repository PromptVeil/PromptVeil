"""Benchmarking utilities for PromptVeil compression."""

import time
from dataclasses import dataclass
from typing import Dict, List, Optional
from pathlib import Path
import json

@dataclass
class CompressionResult:
    original_size: int
    compressed_size: int
    compression_ratio: float
    compression_time: float
    decompression_time: float
    method: str
    julia_enabled: bool
    gpu_available: bool

    def to_dict(self) -> dict:
        return {
            "original_size": self.original_size,
            "compressed_size": self.compressed_size,
            "compression_ratio": self.compression_ratio,
            "compression_time": self.compression_time,
            "decompression_time": self.decompression_time,
            "method": self.method,
            "julia_enabled": self.julia_enabled,
            "gpu_available": self.gpu_available
        }

class CompressionBenchmark:
    def __init__(self, save_path: Optional[Path] = None):
        self.save_path = save_path or Path("benchmark_results.json")
        self.results: List[CompressionResult] = []
        self._load_previous_results()

    def _load_previous_results(self):
        if self.save_path.exists():
            with open(self.save_path) as f:
                data = json.load(f)
                self.results = [CompressionResult(**r) for r in data]

    def run_benchmark(self, data: bytes, method: str = "zstd+rust", julia_enabled: bool = False) -> CompressionResult:
        from promptveil.core import compress, decompress
        try:
            from julia import Tokenizer
            gpu_available = Tokenizer.CUDA.functional()
        except ImportError:
            gpu_available = False

        # Measure compression
        start_time = time.time()
        compressed = compress(data)
        compression_time = time.time() - start_time

        # Measure decompression
        start_time = time.time()
        decompressed = decompress(compressed)
        decompression_time = time.time() - start_time

        # Verify correctness
        assert data == decompressed, "Decompression failed to recover original data"

        result = CompressionResult(
            original_size=len(data),
            compressed_size=len(compressed),
            compression_ratio=len(compressed) / len(data),
            compression_time=compression_time,
            decompression_time=decompression_time,
            method=method,
            julia_enabled=julia_enabled,
            gpu_available=gpu_available
        )

        self.results.append(result)
        self._save_results()
        return result

    def _save_results(self):
        with open(self.save_path, 'w') as f:
            json.dump([r.to_dict() for r in self.results], f, indent=2)

    def print_summary(self):
        """Print a summary of all benchmark results."""
        print("\nPromptVeil Compression Benchmark Summary")
        print("=" * 50)
        
        # Group by method
        methods: Dict[str, List[CompressionResult]] = {}
        for result in self.results:
            key = f"{result.method}{'[+Julia]' if result.julia_enabled else ''}"
            if key not in methods:
                methods[key] = []
            methods[key].append(result)

        # Print summary for each method
        for method, results in methods.items():
            avg_ratio = sum(r.compression_ratio for r in results) / len(results)
            best_ratio = min(r.compression_ratio for r in results)
            avg_time = sum(r.compression_time for r in results) / len(results)
            
            print(f"\nMethod: {method}")
            print(f"Average Compression Ratio: {avg_ratio:.2%}")
            print(f"Best Compression Ratio: {best_ratio:.2%}")
            print(f"Average Compression Time: {avg_time:.3f}s")
            print("-" * 50) 