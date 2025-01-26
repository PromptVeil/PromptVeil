using Pkg

# Ensure we're in the right environment
Pkg.activate(@__DIR__)

# Develop local TokenCompression package
token_compression_path = joinpath(@__DIR__, "TokenCompression.jl")
if isdir(token_compression_path)
    Pkg.develop(path=token_compression_path)
else
    error("TokenCompression.jl not found at $token_compression_path")
end

# Add registries if needed
if !isfile(joinpath(DEPOT_PATH[1], "registries", "General", "Registry.toml"))
    Pkg.Registry.add("General")
end

# Force update to get latest versions
Pkg.update()

# Ensure CUDA is at version 5
Pkg.add(name="CUDA", version="5")

# Add standard dependencies
Pkg.add([
    "PackageCompiler",
    "SIMD",
    "LinearAlgebra",
    "Statistics",
    "JlrsCore"
])

# Build CUDA if available
try
    using CUDA
    CUDA.set_runtime_version!()
    @info "CUDA runtime configured successfully"
catch e
    @warn "CUDA configuration failed, but continuing without GPU support" exception=e
end

# Precompile everything
Pkg.precompile() 