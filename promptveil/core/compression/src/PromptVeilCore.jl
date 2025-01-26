module PromptVeilCore

using SIMD
using TokenCompression
using JlrsCore.Wrap
using JlrsCore.Reflect
using CUDA
using LinearAlgebra

# Structs that will be reflected to Rust
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

# Generate Rust layouts
const LAYOUTS_PATH = joinpath(@__DIR__, "..", "..", "security", "src", "layouts.rs")
layouts = reflect([CompressionConfig, CompressedResult])

# Rename fields to follow Rust naming conventions
renamefields!(layouts, CompressionConfig, [
    :use_gpu => "use_gpu",
    :use_simd => "use_simd",
    :use_patterns => "use_patterns"
])

renamefields!(layouts, CompressedResult, [
    :data => "data",
    :original_size => "original_size",
    :compressed_size => "compressed_size"
])

# Write layouts to file
open(LAYOUTS_PATH, "w") do f
    write(f, layouts)
end

# Export our functions
export optimize_tokens, compress_batch, decompress_tokens, decompress_batch
export CompressionConfig, CompressedResult

# Helper function to apply SIMD operations safely
function apply_simd(data::Vector{UInt32})::Vector{UInt32}
    len = length(data)
    padded_len = (len + 3) ÷ 4 * 4
    padded = vcat(data, fill(UInt32(0), padded_len - len))
    
    result = similar(padded)
    for i in 1:4:padded_len
        v = vload(Vec{4,UInt32}, padded, i)
        vstore(v, result, i)
    end
    return result[1:len]
end

# Core functions
function optimize_tokens(tokens::Vector{UInt32}, config::CompressionConfig)::CompressedResult
    # Validate input
    if isempty(tokens)
        throw(ArgumentError("Input tokens cannot be empty"))
    end

    # Apply SIMD if enabled (before compression)
    input_tokens = if config.use_simd
        apply_simd(tokens)
    else
        tokens
    end

    # Train model and compress based on configuration
    model = if config.use_patterns
        try
            TokenCompression.train_bpe(input_tokens)
        catch e
            @warn "Failed to train BPE model, falling back to basic compression" exception=e
            nothing
        end
    else
        nothing
    end

    # Apply compression with GPU fallback
    compressed = try
        if config.use_gpu && TokenCompression.has_gpu()
            if model !== nothing
                TokenCompression.optimize_tokens(input_tokens, model)
            else
                TokenCompression.optimize_tokens(input_tokens)
            end
        else
            throw(ErrorException("GPU not available or not requested"))
        end
    catch e
        @warn "GPU compression failed, falling back to CPU" exception=e
        if model !== nothing
            TokenCompression.optimize_tokens(input_tokens, model)
        else
            TokenCompression.optimize_tokens(input_tokens)
        end
    end

    return CompressedResult(
        compressed,
        length(tokens),
        length(compressed)
    )
end

function decompress_tokens(compressed::Vector{UInt32})::Vector{UInt32}
    if isempty(compressed)
        throw(ArgumentError("Input tokens cannot be empty"))
    end
    
    try
        TokenCompression.decompress_tokens(compressed)
    catch e
        @warn "Decompression failed" exception=e
        rethrow(e)
    end
end

function compress_batch(tokens::Matrix{UInt32}, config::CompressionConfig)::CompressedResult
    # Validate dimensions
    if size(tokens, 1) == 0 || size(tokens, 2) == 0
        throw(ArgumentError("Input matrix cannot be empty"))
    end
    if size(tokens, 2) < 2
        throw(DimensionMismatch("Input matrix must have at least 2 columns for compression"))
    end

    total_tokens = size(tokens, 1) * size(tokens, 2)
    
    # Apply SIMD if enabled (before compression)
    input_matrix = if config.use_simd
        reshape(apply_simd(vec(tokens)), size(tokens))
    else
        tokens
    end
    
    # Apply compression with GPU fallback
    compressed = try
        if config.use_gpu && TokenCompression.has_gpu()
            TokenCompression.compress_batch(input_matrix)
        else
            throw(ErrorException("GPU not available or not requested"))
        end
    catch e
        @warn "GPU batch compression failed, falling back to CPU" exception=e
        TokenCompression.compress_batch(input_matrix)
    end

    return CompressedResult(
        vec(compressed),  # Convert matrix to vector for consistent return type
        total_tokens,
        length(compressed)
    )
end

function decompress_batch(compressed::Vector{UInt32}, rows::Int, cols::Int)::Matrix{UInt32}
    if isempty(compressed)
        throw(ArgumentError("Input tokens cannot be empty"))
    end
    if rows <= 0 || cols <= 0
        throw(ArgumentError("Invalid dimensions: rows and cols must be positive"))
    end
    if length(compressed) != rows * cols
        throw(DimensionMismatch("Compressed data length does not match specified dimensions"))
    end
    
    try
        reshape(TokenCompression.decompress_tokens(compressed), rows, cols)
    catch e
        @warn "Batch decompression failed" exception=e
        rethrow(e)
    end
end

end # module 