using Test
using PromptVeilCore
using LinearAlgebra
using TokenCompression

@testset "PromptVeilCore Tests" begin
    @testset "Token Optimization" begin
        # Test with small sequence
        small_tokens = UInt32[1, 2, 3, 1, 2, 3, 4, 5]
        small_compressed = optimize_tokens_simd(small_tokens)
        @test length(small_compressed) <= length(small_tokens)
        @test eltype(small_compressed) == UInt32

        # Test with medium sequence
        medium_tokens = rand(UInt32, 5_000)
        medium_compressed = optimize_tokens_simd(medium_tokens)
        @test length(medium_compressed) <= length(medium_tokens)
        @test eltype(medium_compressed) == UInt32

        # Test with large sequence
        large_tokens = rand(UInt32, 20_000)
        large_compressed = optimize_tokens_simd(large_tokens)
        @test length(large_compressed) <= length(large_tokens)
        @test eltype(large_compressed) == UInt32

        # Test compression effectiveness with patterns
        repeated_tokens = repeat(UInt32[1, 2, 3, 4], 5000)  # 20k tokens with clear patterns
        compressed = optimize_tokens_simd(repeated_tokens)
        @test length(compressed) < length(repeated_tokens)
        @test eltype(compressed) == UInt32
    end

    @testset "Batch Compression" begin
        # Test with small batch
        small_batch = UInt32[1 2 3; 4 5 6; 7 8 9]
        small_compressed = compress_batch_gpu(small_batch)
        @test size(small_compressed, 1) == size(small_batch, 1)
        @test eltype(small_compressed) == UInt32

        # Test with large batch
        large_batch = rand(UInt32, 1000, 100)  # 100k tokens total
        large_compressed = compress_batch_gpu(large_batch)
        @test size(large_compressed, 1) == size(large_batch, 1)
        @test eltype(large_compressed) == UInt32

        # Test compression effectiveness with patterns
        pattern_batch = repeat(UInt32[1 2 3 4; 5 6 7 8], 500)  # 1000x4 matrix
        pattern_compressed = compress_batch_gpu(pattern_batch)
        @test size(pattern_compressed, 1) == size(pattern_batch, 1)
        @test eltype(pattern_compressed) == UInt32
    end

    @testset "Error Handling" begin
        # Test empty input
        @test_throws ArgumentError optimize_tokens_simd(UInt32[])
        @test_throws ArgumentError compress_batch_gpu(Matrix{UInt32}(undef, 0, 0))

        # Test invalid dimensions
        invalid_batch = reshape(UInt32[1,2,3], (3,1))
        @test_throws DimensionMismatch compress_batch_gpu(invalid_batch)
    end
end 