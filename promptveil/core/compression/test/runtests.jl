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

@testset "PromptVeilCore Tests" begin
    @testset "Token Optimization" begin
        # Test both GPU and CPU paths
        configs = [
            CompressionConfig(true, true, true),   # GPU + SIMD
            CompressionConfig(false, true, true),  # CPU + SIMD
            CompressionConfig(true, false, true),  # GPU only
            CompressionConfig(false, false, true)  # CPU only
        ]
        
        for config in configs
            desc = config.use_gpu ? "GPU" : "CPU"
            desc *= config.use_simd ? "+SIMD" : ""
            
            @testset "$desc path" begin
                # Test with small sequence
                small_tokens = UInt32[1, 2, 3, 1, 2, 3, 4, 5]
                small_result = optimize_tokens(small_tokens, config)
                @test small_result.compressed_size <= small_result.original_size
                @test eltype(small_result.data) == UInt32
                @test length(small_result.data) > 0

                # Test with medium sequence
                medium_tokens = rand(UInt32, 5_000)
                medium_result = optimize_tokens(medium_tokens, config)
                @test medium_result.compressed_size <= medium_result.original_size
                @test eltype(medium_result.data) == UInt32
                @test length(medium_result.data) > 0

                # Test with large sequence (GPU threshold)
                large_tokens = rand(UInt32, 20_000)
                large_result = optimize_tokens(large_tokens, config)
                @test large_result.compressed_size <= large_result.original_size
                @test eltype(large_result.data) == UInt32
                @test length(large_result.data) > 0

                # Test compression effectiveness with patterns
                repeated_tokens = repeat(UInt32[1, 2, 3, 4], 5000)
                pattern_result = optimize_tokens(repeated_tokens, config)
                @test pattern_result.compressed_size < pattern_result.original_size
                @test eltype(pattern_result.data) == UInt32
                @test length(pattern_result.data) > 0
            end
        end
    end

    @testset "Batch Compression" begin
        configs = [
            CompressionConfig(true, true, true),   # GPU + SIMD
            CompressionConfig(false, true, true),  # CPU + SIMD
            CompressionConfig(true, false, true),  # GPU only
            CompressionConfig(false, false, true)  # CPU only
        ]
        
        for config in configs
            desc = config.use_gpu ? "GPU" : "CPU"
            desc *= config.use_simd ? "+SIMD" : ""
            
            @testset "$desc batch compression" begin
                # Test with small batch
                small_batch = UInt32[1 2 3; 4 5 6; 7 8 9]
                small_result = compress_batch(small_batch, config)
                @test length(small_result.data) > 0
                @test small_result.compressed_size <= small_result.original_size
                @test eltype(small_result.data) == UInt32

                # Test with large batch (GPU threshold)
                large_batch = rand(UInt32, 1000, 100)
                large_result = compress_batch(large_batch, config)
                @test length(large_result.data) > 0
                @test large_result.compressed_size <= large_result.original_size
                @test eltype(large_result.data) == UInt32

                # Test compression effectiveness with patterns
                pattern_batch = repeat(UInt32[1 2 3 4; 5 6 7 8], 500)
                pattern_result = compress_batch(pattern_batch, config)
                @test length(pattern_result.data) > 0
                @test pattern_result.compressed_size < pattern_result.original_size
                @test eltype(pattern_result.data) == UInt32
            end
        end
    end

    @testset "Error Handling" begin
        config = CompressionConfig(true, true, true)
        
        # Test empty input
        @test_throws ArgumentError optimize_tokens(UInt32[], config)
        @test_throws ArgumentError compress_batch(Matrix{UInt32}(undef, 0, 0), config)

        # Test invalid dimensions
        invalid_batch = reshape(UInt32[1,2,3], (3,1))
        @test_throws DimensionMismatch compress_batch(invalid_batch, config)
        
        # Test GPU fallback
        if !CUDA.functional()
            @test_logs (:warn, r"GPU.*failed.*CPU") match_mode=:any begin
                large_tokens = rand(UInt32, 20_000)
                result = optimize_tokens(large_tokens, CompressionConfig(true, false, true))
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