using Test
using PromptVeilCore
using LinearAlgebra
using CUDA

@testset "PromptVeilCore Tests" begin
    @testset "Basic Compression" begin
        # Create test data with patterns
        base_pattern = UInt32[1000, 2000, 3000, 4000]
        tokens = vcat(fill(base_pattern, 10)...)  # Reduced repetitions for basic test
        
        # Test without patterns (basic compression)
        config = PromptVeilCore.CompressionConfig(false, false, false)
        result = PromptVeilCore.optimize_tokens(tokens, config)
        
        # Basic checks
        @test result.compressed_size <= result.original_size
        @test eltype(result.data) == UInt32
        @test length(result.data) > 0
        
        # Test with pattern detection
        config_patterns = PromptVeilCore.CompressionConfig(false, false, true)
        pattern_result = PromptVeilCore.optimize_tokens(tokens, config_patterns)
        
        # Pattern detection should at least not increase size
        @test pattern_result.compressed_size <= pattern_result.original_size
        @test pattern_result.compressed_size <= result.compressed_size
    end
    
    @testset "Pattern Detection" begin
        # Create sequence with patterns
        pattern1 = UInt32[1000, 2000]
        pattern2 = UInt32[3000, 4000]
        tokens = vcat(repeat(pattern1, 5), repeat(pattern2, 5))  # Same as TokenCompression tests
        
        config = PromptVeilCore.CompressionConfig(false, false, true)
        result = PromptVeilCore.optimize_tokens(tokens, config)
        
        # Basic pattern checks
        @test result.compressed_size < result.original_size
        
        # Test with random data (should compress less)
        random_tokens = rand(UInt32(1):UInt32(1000), length(tokens))
        random_result = PromptVeilCore.optimize_tokens(random_tokens, config)
        
        # Random data should not compress better than patterned data
        @test result.compressed_size <= random_result.compressed_size
    end
    
    @testset "Batch Compression" begin
        # Create test batch with patterns
        base_pattern = UInt32[1 2 3; 4 5 6; 7 8 9]
        pattern_batch = repeat(base_pattern, outer=(10, 1))  # Reduced size for basic test
        
        # Test without patterns
        config = PromptVeilCore.CompressionConfig(false, false, false)
        result = PromptVeilCore.compress_batch(pattern_batch, config)
        
        # Basic checks
        @test result.compressed_size <= result.original_size
        @test eltype(result.data) == UInt32
        
        # Test with pattern detection
        config_patterns = PromptVeilCore.CompressionConfig(false, false, true)
        pattern_result = PromptVeilCore.compress_batch(pattern_batch, config_patterns)
        
        # Pattern detection should at least not increase size
        @test pattern_result.compressed_size <= pattern_result.original_size
        @test pattern_result.compressed_size <= result.compressed_size
        
        # Test with random batch
        random_batch = rand(UInt32(1):UInt32(1000), size(pattern_batch)...)
        random_result = PromptVeilCore.compress_batch(random_batch, config_patterns)
        
        # Random data should not compress better than patterned data
        @test pattern_result.compressed_size <= random_result.compressed_size
    end
    
    @testset "SIMD Optimization" begin
        # Test with data size multiple of SIMD vector size
        tokens = UInt32[1000, 2000, 3000, 4000, 1000, 2000, 3000, 4000]
        
        # Test with and without SIMD
        config_no_simd = PromptVeilCore.CompressionConfig(false, false, true)
        config_simd = PromptVeilCore.CompressionConfig(false, true, true)
        
        result_no_simd = PromptVeilCore.optimize_tokens(tokens, config_no_simd)
        result_simd = PromptVeilCore.optimize_tokens(tokens, config_simd)
        
        # Results should be consistent
        @test result_no_simd.compressed_size == result_simd.compressed_size
    end
    
    @testset "Error Handling" begin
        config = PromptVeilCore.CompressionConfig(false, false, false)
        
        # Test empty inputs
        @test_throws ArgumentError PromptVeilCore.optimize_tokens(UInt32[], config)
        @test_throws ArgumentError PromptVeilCore.compress_batch(Matrix{UInt32}(undef, 0, 0), config)
        
        # Test invalid batch dimensions
        invalid_batch = rand(UInt32(1):UInt32(1000), 100, 1)  # Only one column
        @test_throws DimensionMismatch PromptVeilCore.compress_batch(invalid_batch, config)
        
        # Test decompression with invalid dimensions
        compressed = UInt32[1, 2, 3, 4]
        @test_throws DimensionMismatch PromptVeilCore.decompress_batch(compressed, 3, 3)
    end
end 