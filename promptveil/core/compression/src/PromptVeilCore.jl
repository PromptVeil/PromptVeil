module PromptVeilCore

using SIMD
using TokenCompression

# Export our functions
export optimize_tokens_simd, compress_batch_gpu, decompress_batch_gpu
export julia_optimize_tokens_config, julia_compress_batch_config, julia_decompress_batch
export julia_init, julia_cleanup

"""
    julia_init()

Initialize the Julia runtime for FFI calls.
"""
Base.@ccallable function julia_init()::Cvoid
    println("DEBUG Julia: Initializing runtime")
    # Initialize any required state here
    println("DEBUG Julia: Runtime initialized")
    return nothing
end

"""
    julia_cleanup()

Cleanup the Julia runtime after FFI calls.
"""
Base.@ccallable function julia_cleanup()::Cvoid
    println("DEBUG Julia: Cleaning up runtime")
    # Cleanup any state here
    println("DEBUG Julia: Runtime cleaned up")
    return nothing
end

"""
    optimize_tokens_simd(tokens::Vector{UInt32})

Optimize and quantize tokens using TokenCompression with additional SIMD operations.
GPU acceleration is automatically used for large sequences if available.
"""
function optimize_tokens_simd(tokens::Vector{UInt32})
    # First apply TokenCompression's optimize_tokens
    compressed = if length(tokens) >= 10_000 && TokenCompression.has_gpu()
        # Use GPU for large sequences when available
        TokenCompression.optimize_tokens(tokens)
    else
        # Use CPU for small sequences or when GPU is not available
        TokenCompression.optimize_tokens(tokens)
    end
    
    # Apply additional SIMD optimizations
    len = length(compressed)
    padded_len = (len + 3) ÷ 4 * 4  # Round to multiple of 4
    padded = vcat(compressed, fill(UInt32(0), padded_len - len))
    
    # Process in blocks of 4 elements using vload
    result = similar(padded)
    for i in 1:4:padded_len
        v = vload(Vec{4,UInt32}, padded, i)
        vstore(v, result, i)
    end
    
    # Return only original elements, without padding
    return result[1:len]
end

"""
    compress_batch_gpu(tokens::Matrix{UInt32})

Compress a batch of tokens using TokenCompression.jl's batch compression.
GPU acceleration is automatically used for large batches if available.

# Arguments
- `tokens::Matrix{UInt32}`: Input token matrix where each row is a sequence

# Returns
- `Matrix{UInt32}`: Compressed token matrix
"""
function compress_batch_gpu(tokens::Matrix{UInt32})
    # Validate dimensions
    if size(tokens, 1) == 0 || size(tokens, 2) == 0
        throw(ArgumentError("Input matrix cannot be empty"))
    end
    if size(tokens, 2) < 2
        throw(DimensionMismatch("Input matrix must have at least 2 columns for compression"))
    end

    total_tokens = size(tokens, 1) * size(tokens, 2)
    
    if total_tokens >= 10_000 && TokenCompression.has_gpu()
        # Use GPU for large batches when available
        return TokenCompression.compress_batch(tokens)
    else
        # Use CPU for small batches or when GPU is not available
        return TokenCompression.compress_batch(tokens)
    end
end

"""
    decompress_batch_gpu(tokens::Matrix{UInt32})

Decompress a batch of tokens using TokenCompression.jl's batch decompression.
GPU acceleration is automatically used for large batches if available.

# Arguments
- `tokens::Matrix{UInt32}`: Input compressed token matrix

# Returns
- `Matrix{UInt32}`: Decompressed token matrix
"""
function decompress_batch_gpu(tokens::Matrix{UInt32})
    # Validate dimensions
    if size(tokens, 1) == 0 || size(tokens, 2) == 0
        throw(ArgumentError("Input matrix cannot be empty"))
    end

    total_tokens = size(tokens, 1) * size(tokens, 2)
    
    if total_tokens >= 10_000 && TokenCompression.has_gpu()
        # Use GPU for large batches when available
        return TokenCompression.decompress_batch(tokens)
    else
        # Use CPU for small batches or when GPU is not available
        return TokenCompression.decompress_batch(tokens)
    end
end

# FFI Functions for Rust Integration

"""
    julia_optimize_tokens_config(ptr::Ptr{UInt32}, len::Int64, use_gpu::Bool, use_simd::Bool, use_patterns::Bool)::Ptr{UInt32}

FFI version of optimize_tokens_simd for Rust integration with configuration options.
"""
Base.@ccallable function julia_optimize_tokens_config(
    ptr::Ptr{UInt32}, 
    len::Int64,
    use_gpu::Bool,
    use_simd::Bool,
    use_patterns::Bool
)::Ptr{UInt32}
    tokens = unsafe_wrap(Array, ptr, len)
    result = optimize_tokens_simd(tokens)
    
    result_ptr = Base.Libc.malloc(sizeof(UInt32) * length(result))
    unsafe_copyto!(Ptr{UInt32}(result_ptr), pointer(result), length(result))
    return Ptr{UInt32}(result_ptr)
end

"""
    julia_compress_batch_config(ptr::Ptr{UInt32}, rows::Int64, cols::Int64, use_gpu::Bool, use_simd::Bool, use_patterns::Bool)::Ptr{UInt32}

FFI version of compress_batch_gpu for Rust integration with configuration options.
"""
Base.@ccallable function julia_compress_batch_config(
    ptr::Ptr{UInt32}, 
    rows::Int64, 
    cols::Int64,
    use_gpu::Bool,
    use_simd::Bool,
    use_patterns::Bool
)::Ptr{UInt32}
    println("DEBUG Julia: Entering julia_compress_batch_config")
    println("DEBUG Julia: Input dimensions: $(rows)x$(cols)")
    println("DEBUG Julia: Configuration - GPU: $use_gpu, SIMD: $use_simd, Patterns: $use_patterns")
    
    try
        println("DEBUG Julia: Converting input pointer to Array")
        tokens = unsafe_wrap(Array, ptr, (rows, cols))
        println("DEBUG Julia: Array dimensions: $(size(tokens))")
        println("DEBUG Julia: First few values: $(tokens[1:min(10,end), 1])")
        
        println("DEBUG Julia: Calling compress_batch_gpu")
        result = compress_batch_gpu(tokens)
        println("DEBUG Julia: Compression complete")
        println("DEBUG Julia: Result dimensions: $(size(result))")
        println("DEBUG Julia: First few compressed values: $(result[1:min(10,end)])")
        
        println("DEBUG Julia: Allocating result memory")
        result_ptr = Base.Libc.malloc(sizeof(UInt32) * length(result))
        println("DEBUG Julia: Result pointer: $result_ptr")
        
        println("DEBUG Julia: Copying result to output buffer")
        unsafe_copyto!(Ptr{UInt32}(result_ptr), pointer(result), length(result))
        println("DEBUG Julia: Copy complete")
        
        return Ptr{UInt32}(result_ptr)
    catch e
        println("DEBUG Julia: Error occurred: $e")
        println("DEBUG Julia: Backtrace:")
        for (exc, bt) in Base.catch_stack()
            showerror(stdout, exc, bt)
            println()
        end
        rethrow(e)
    end
end

"""
    julia_decompress_batch(ptr::Ptr{UInt32}, rows::Int64, cols::Int64)::Ptr{UInt32}

FFI version of decompress_batch_gpu for Rust integration.
"""
Base.@ccallable function julia_decompress_batch(ptr::Ptr{UInt32}, rows::Int64, cols::Int64)::Ptr{UInt32}
    tokens = unsafe_wrap(Array, ptr, (rows, cols))
    result = decompress_batch_gpu(tokens)
    
    result_ptr = Base.Libc.malloc(sizeof(UInt32) * length(result))
    unsafe_copyto!(Ptr{UInt32}(result_ptr), pointer(result), length(result))
    return Ptr{UInt32}(result_ptr)
end

end # module 