using Pkg

println("Activating project environment...")

# Ensure we are in the correct directory
const PROJECT_DIR = @__DIR__
cd(PROJECT_DIR)

# Activate or create project environment
if !isfile(joinpath(PROJECT_DIR, "Project.toml"))
    println("Creating new project...")
    Pkg.init()
end

Pkg.activate(PROJECT_DIR)

# Install dependencies
deps = Dict(
    "PackageCompiler" => "2.1",
    "SIMD" => "3.4",
    "CUDA" => "4.4"
)

println("\nInstalling dependencies...")
for (dep, ver) in deps
    println("Installing $dep v$ver...")
    try
        Pkg.add(name=dep, version=ver)
    catch e
        println("Warning while installing $dep: ", e)
        # Try installing without specific version
        try
            Pkg.add(dep)
        catch e2
            println("Error installing $dep: ", e2)
            exit(1)
        end
    end
end

# Check if TokenCompression.jl exists
token_compression_path = joinpath(PROJECT_DIR, "TokenCompression.jl")
if isdir(token_compression_path)
    println("\nInstalling TokenCompression.jl...")
    try
        Pkg.develop(path=token_compression_path)
    catch e
        println("Error installing TokenCompression.jl: ", e)
        exit(1)
    end
else
    println("\nError: TokenCompression.jl directory not found at $token_compression_path")
    exit(1)
end

println("\nPrecompiling packages...")
try
    Pkg.precompile()
catch e
    println("Error during precompilation: ", e)
    exit(1)
end

# Verify all packages were installed
println("\nVerifying installations...")
installed = Pkg.installed()
for (dep, _) in deps
    if !haskey(installed, dep)
        println("Error: Package $dep was not installed correctly")
        exit(1)
    end
end

if !haskey(installed, "TokenCompression")
    println("Error: TokenCompression.jl was not installed correctly")
    exit(1)
end

println("\nSetup completed successfully!")

# Ensure environment is activated before returning
Pkg.activate(PROJECT_DIR) 