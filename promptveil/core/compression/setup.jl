using Pkg

# Ensure we're in the right environment
Pkg.activate(@__DIR__)

# Add registries if needed
if !isfile(joinpath(DEPOT_PATH[1], "registries", "General", "Registry.toml"))
    Pkg.Registry.add("General")
end

# Force update to get latest versions
Pkg.update()

# Ensure CUDA is at version 5
Pkg.add(name="CUDA", version="5")

# Add other dependencies
Pkg.add([
    "PackageCompiler",
    "SIMD",
    "LinearAlgebra",
    "Statistics",
    "TokenCompression"
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