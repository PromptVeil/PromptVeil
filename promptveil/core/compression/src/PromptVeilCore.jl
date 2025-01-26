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

# Write layouts to file
open(LAYOUTS_PATH, "w") do f
    write(f, layouts)
end

# Export our functions
export optimize_tokens, compress_batch
export CompressionConfig, CompressedResult

# Core functions
function optimize_tokens(tokens::Vector{UInt32}, config::CompressionConfig)::CompressedResult
    # Validate input
    if isempty(tokens)
        throw(ArgumentError("Input tokens cannot be empty"))
    end

    # Apply compression based on config
    compressed = if config.use_gpu && length(tokens) >= 10_000 && TokenCompression.has_gpu()
        # Use GPU for large sequences when available
        TokenCompression.optimize_tokens(tokens)
    else
        # Use CPU for small sequences or when GPU is not available
        TokenCompression.parallel_countmap_cpu(tokens) |> TokenCompression.train_bpe |> model -> 
            TokenCompression.optimize_tokens(tokens, model)
    end
    
    # Apply SIMD if enabled
    if config.use_simd
        len = length(compressed)
        padded_len = (len + 3) ÷ 4 * 4
        padded = vcat(compressed, fill(UInt32(0), padded_len - len))
        
        result = similar(padded)
        for i in 1:4:padded_len
            v = vload(Vec{4,UInt32}, padded, i)
            vstore(v, result, i)
        end
        compressed = result[1:len]
    end

    return CompressedResult(
        compressed,
        length(tokens),
        length(compressed)
    )
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
    
    # Apply compression based on config
    compressed = if config.use_gpu && total_tokens >= 10_000 && TokenCompression.has_gpu()
        TokenCompression.compress_batch(tokens)
    else
        TokenCompression.compress_batch_cpu(tokens)
    end

    return CompressedResult(
        vec(compressed),
        total_tokens,
        length(compressed)
    )
end

end # module 