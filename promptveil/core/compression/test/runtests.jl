using Test
using PromptVeilCore
using LinearAlgebra
using TokenCompression
using JlrsCore.Wrap
using CUDA

# Define structs that will be reflected to Rust
struct CompressionConfig
    use_gpu::Bool
    use_simd::Bool
    use_patterns::Bool
end

struct CompressedResult
    data::Vector{UInt32}
    original_size::Int64
    compressed_size::Int64
end

# Define test configurations
const TEST_CONFIGS = [
    CompressionConfig(true, true, true),    # GPU + SIMD + Patterns
    CompressionConfig(true, true, false),   # GPU + SIMD
    CompressionConfig(false, true, true),   # CPU + SIMD + Patterns
    CompressionConfig(false, false, false)  # CPU only
]

@testset "PromptVeilCore Tests" begin
    @testset "Token Optimization" begin
        for config in TEST_CONFIGS
            desc = "$(config.use_gpu ? "GPU" : "CPU")$(config.use_simd ? "+SIMD" : "")$(config.use_patterns ? "+Patterns" : "")"
            
            @testset "$desc path" begin
                # Create test data with actual patterns that BPE can detect
                base_pattern = UInt32[100, 200, 300, 400]
                repeated_tokens = vcat(fill(base_pattern, 100)...)  # Flatten the array
                
                # Test pattern compression
                pattern_result = optimize_tokens(repeated_tokens, config)
                
                # Basic checks
                @test pattern_result.compressed_size <= pattern_result.original_size
                @test eltype(pattern_result.data) == UInt32
                @test length(pattern_result.data) > 0
                
                # Test with random data (should compress less)
                random_tokens = rand(UInt32(1):UInt32(1000), 1000)
                random_result = optimize_tokens(random_tokens, config)
                
                # Basic checks for random data
                @test random_result.compressed_size <= random_result.original_size
                @test eltype(random_result.data) == UInt32
                @test length(random_result.data) > 0
                
                # Pattern-specific tests
                if config.use_patterns
                    # Repeated patterns should compress significantly better
                    @test pattern_result.compressed_size < 0.5 * pattern_result.original_size
                    # Random data should compress less than patterned data
                    @test pattern_result.compressed_size < random_result.compressed_size
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
                
                # Test pattern compression
                pattern_result = compress_batch(pattern_batch, config)
                
                # Basic checks
                @test pattern_result.compressed_size <= pattern_result.original_size
                @test eltype(pattern_result.data) == UInt32
                @test length(pattern_result.data) > 0
                
                # Test with random batch
                random_batch = rand(UInt32(1):UInt32(1000), 100, 9)
                random_result = compress_batch(random_batch, config)
                
                # Basic checks for random data
                @test random_result.compressed_size <= random_result.original_size
                @test eltype(random_result.data) == UInt32
                @test length(random_result.data) > 0
                
                # Pattern-specific tests
                if config.use_patterns
                    # Repeated patterns should compress better than random data
                    @test pattern_result.compressed_size < random_result.compressed_size
                end
            end
        end
    end

    @testset "Error Handling" begin
        config = CompressionConfig(false, false, false)
        
        # Test empty inputs
        @test_throws ArgumentError optimize_tokens(UInt32[], config)
        @test_throws ArgumentError compress_batch(Matrix{UInt32}(undef, 0, 0), config)

        # Test invalid dimensions
        invalid_batch = reshape(UInt32[1,2,3], (3,1))
        @test_throws DimensionMismatch compress_batch(invalid_batch, config)
        
        # Test GPU fallback
        if !CUDA.functional()
            gpu_config = CompressionConfig(true, false, false)
            @test_logs (:warn, r"GPU.*failed.*CPU") match_mode=:any begin
                tokens = rand(UInt32, 1000)
                result = optimize_tokens(tokens, gpu_config)
                @test length(result.data) > 0
            end
        end
    end
end

# Generate Rust layouts
using JlrsCore.Reflect

layouts = reflect([CompressionConfig, CompressedResult])
open(joinpath(@__DIR__, "..", "src", "layouts.rs"), "w") do f
    write(f, layouts)
end 