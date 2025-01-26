using Pkg

# Ensure we're in the right environment
Pkg.activate(@__DIR__)

# Ensure all dependencies are installed
Pkg.instantiate()

# Add registries if needed
if !isfile(joinpath(DEPOT_PATH[1], "registries", "General", "Registry.toml"))
    Pkg.Registry.add("General")
end

# Add core dependencies first
Pkg.add([
    Pkg.PackageSpec(name="JlrsCore", version="0.5.0"),
    Pkg.PackageSpec(name="TokenCompression", version="0.1.0"),
    Pkg.PackageSpec(name="CUDA", version="5")
])

# Add standard dependencies
Pkg.add([
    "PackageCompiler",
    "SIMD",
    "LinearAlgebra",
    "Statistics",
    "Test"
])

# Force update to get latest versions
Pkg.update()

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

# Test that everything is working
@info "Testing if module can be loaded..."
try
    # Since we're already in the package directory, just use the module directly
    include(joinpath(@__DIR__, "src", "PromptVeilCore.jl"))
    using .PromptVeilCore
    @info "PromptVeilCore loaded successfully"
catch e
    @error "Failed to load PromptVeilCore" exception=e
end 