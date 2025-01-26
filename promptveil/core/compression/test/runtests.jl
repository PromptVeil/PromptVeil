using Test
using PromptVeilCore
using LinearAlgebra
using CUDA

@testset "PromptVeilCore Tests" begin
    @testset "Basic Compression" begin
        # Test data with known patterns (repeated sequence)
        base_pattern = UInt32[1000, 2000, 3000, 4000]
        tokens = vcat(fill(base_pattern, 50)...)  # Increased repetitions for better pattern detection
        
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
        
        # Pattern detection should give better compression for repeated data
        @test pattern_result.compressed_size < result.compressed_size
        @test pattern_result.compressed_size < 0.8 * pattern_result.original_size  # At least 20% compression
    end
    
    @testset "Pattern Detection" begin
        # Create sequence with known patterns
        pattern1 = UInt32[1000, 2000]
        pattern2 = UInt32[3000, 4000]
        tokens = vcat(repeat(pattern1, 50), repeat(pattern2, 50))  # Increased repetitions
        
        config = PromptVeilCore.CompressionConfig(false, false, true)
        result = PromptVeilCore.optimize_tokens(tokens, config)
        
        # Patterns should be detected and compressed significantly
        @test result.compressed_size < 0.6 * result.original_size  # At least 40% compression
        
        # Test with random data (should compress less)
        random_tokens = rand(UInt32(1):UInt32(1000), length(tokens))
        random_result = PromptVeilCore.optimize_tokens(random_tokens, config)
        
        # Random data should compress less than patterned data
        @test result.compressed_size < random_result.compressed_size
        @test random_result.compressed_size > 0.8 * random_result.original_size  # Random data compresses less
    end
    
    @testset "Batch Compression" begin
        # Create test batch with patterns
        base_pattern = UInt32[1 2 3; 4 5 6; 7 8 9]
        pattern_batch = repeat(base_pattern, outer=(50, 1))  # Increased repetitions
        
        # Test without patterns
        config = PromptVeilCore.CompressionConfig(false, false, false)
        result = PromptVeilCore.compress_batch(pattern_batch, config)
        
        # Basic checks
        @test result.compressed_size <= result.original_size
        @test eltype(result.data) == UInt32
        
        # Test with pattern detection
        config_patterns = PromptVeilCore.CompressionConfig(false, false, true)
        pattern_result = PromptVeilCore.compress_batch(pattern_batch, config_patterns)
        
        # Pattern detection should give better compression
        @test pattern_result.compressed_size < result.compressed_size
        @test pattern_result.compressed_size < 0.7 * pattern_result.original_size  # At least 30% compression
        
        # Test with random batch
        random_batch = rand(UInt32(1):UInt32(1000), size(pattern_batch)...)
        random_result = PromptVeilCore.compress_batch(random_batch, config_patterns)
        
        # Patterns should compress better than random data
        @test pattern_result.compressed_size < random_result.compressed_size
        @test random_result.compressed_size > 0.8 * random_result.original_size  # Random data compresses less
    end
    
    @testset "SIMD Optimization" begin
        tokens = UInt32[1000, 2000, 3000, 4000, 1000, 2000, 3000, 4000]
        
        # Test with and without SIMD
        config_no_simd = PromptVeilCore.CompressionConfig(false, false, true)
        config_simd = PromptVeilCore.CompressionConfig(false, true, true)
        
        result_no_simd = PromptVeilCore.optimize_tokens(tokens, config_no_simd)
        result_simd = PromptVeilCore.optimize_tokens(tokens, config_simd)
        
        # Results should be consistent regardless of SIMD
        @test result_no_simd.compressed_size == result_simd.compressed_size
        @test result_no_simd.data == result_simd.data
    end
    
    @testset "GPU Support" begin
        if CUDA.functional()
            tokens = rand(UInt32(1):UInt32(1000), 1000)
            
            # Compare GPU vs CPU results
            config_cpu = PromptVeilCore.CompressionConfig(false, false, true)
            config_gpu = PromptVeilCore.CompressionConfig(true, false, true)
            
            result_cpu = PromptVeilCore.optimize_tokens(tokens, config_cpu)
            result_gpu = PromptVeilCore.optimize_tokens(tokens, config_gpu)
            
            # Results should be consistent regardless of device
            @test result_cpu.compressed_size == result_gpu.compressed_size
            @test result_cpu.data == result_gpu.data
        end
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