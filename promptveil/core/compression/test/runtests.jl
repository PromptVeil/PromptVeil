using Test
using PromptVeilCore
using LinearAlgebra
using CUDA

# Define test configurations
const TEST_CONFIGS = [
    PromptVeilCore.CompressionConfig(true, true, true),    # GPU + SIMD + Patterns
    PromptVeilCore.CompressionConfig(true, true, false),   # GPU + SIMD
    PromptVeilCore.CompressionConfig(false, true, true),   # CPU + SIMD + Patterns
    PromptVeilCore.CompressionConfig(false, false, false)  # CPU only
]

@testset "PromptVeilCore Tests" begin
    @testset "Token Optimization" begin
        for config in TEST_CONFIGS
            desc = "$(config.use_gpu ? "GPU" : "CPU")$(config.use_simd ? "+SIMD" : "")$(config.use_patterns ? "+Patterns" : "")"
            
            @testset "$desc path" begin
                # Create test data with actual patterns that BPE can detect
                base_pattern = UInt32[100, 200, 300, 400]
                repeated_tokens = vcat(fill(base_pattern, 100)...)  # Flatten the array
                
                # Test pattern compression using PromptVeilCore
                result = PromptVeilCore.optimize_tokens(repeated_tokens, config)
                
                # Basic checks
                @test result.compressed_size <= result.original_size
                @test eltype(result.data) == UInt32
                @test length(result.data) > 0
                
                # Test with random data
                random_tokens = rand(UInt32(1):UInt32(1000), 1000)
                random_result = PromptVeilCore.optimize_tokens(random_tokens, config)
                
                # Basic checks for random data
                @test random_result.compressed_size <= random_result.original_size
                @test eltype(random_result.data) == UInt32
                @test length(random_result.data) > 0
                
                # Pattern-specific tests
                if config.use_patterns
                    # Repeated patterns should compress significantly better
                    @test result.compressed_size < 0.5 * result.original_size
                    # Random data should compress less than patterned data
                    @test result.compressed_size < random_result.compressed_size
                end
            end
        end
    end

    @testset "Batch Compression" begin
        for config in TEST_CONFIGS
            desc = "$(config.use_gpu ? "GPU" : "CPU")$(config.use_simd ? "+SIMD" : "")$(config.use_patterns ? "+Patterns" : "")"
            
            @testset "$desc batch compression" begin
                # Create test batch with actual patterns
                base_pattern = UInt32[1 2 3; 4 5 6; 7 8 9]
                pattern_batch = repeat(base_pattern, outer=(100, 1))
                
                # Test pattern compression using PromptVeilCore
                result = PromptVeilCore.compress_batch(pattern_batch, config)
                
                # Basic checks
                @test result.compressed_size <= result.original_size
                @test eltype(result.data) == UInt32
                @test length(result.data) > 0
                
                # Test with random batch
                random_batch = rand(UInt32(1):UInt32(1000), 100, 9)
                random_result = PromptVeilCore.compress_batch(random_batch, config)
                
                # Basic checks for random data
                @test random_result.compressed_size <= random_result.original_size
                @test eltype(random_result.data) == UInt32
                @test length(random_result.data) > 0
                
                # Pattern-specific tests
                if config.use_patterns
                    # Repeated patterns should compress significantly better
                    @test result.compressed_size < 0.5 * result.original_size
                    # Random data should compress less than patterned data
                    @test result.compressed_size < random_result.compressed_size
                end
            end
        end
    end

    @testset "Error Handling" begin
        config = PromptVeilCore.CompressionConfig(false, false, false)  # Use CPU for consistent behavior
        
        # Test empty inputs
        @test_throws ArgumentError PromptVeilCore.optimize_tokens(UInt32[], config)
        @test_throws ArgumentError PromptVeilCore.compress_batch(Matrix{UInt32}(undef, 0, 0), config)
        
        # Test invalid batch dimensions
        invalid_batch = rand(UInt32(1):UInt32(1000), 100, 1)  # Only one column
        @test_throws DimensionMismatch PromptVeilCore.compress_batch(invalid_batch, config)
        
        # Test GPU fallback
        if CUDA.functional()
            gpu_config = PromptVeilCore.CompressionConfig(true, false, false)
            tokens = rand(UInt32(1):UInt32(1000), 1000)
            result = PromptVeilCore.optimize_tokens(tokens, gpu_config)
            @test length(result.data) > 0
            @test result.compressed_size <= result.original_size
        end
    end
end 