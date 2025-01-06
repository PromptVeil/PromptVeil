# First, run the setup
println("Running setup...")
include("setup.jl")

# Now we can safely import PackageCompiler
println("Loading PackageCompiler...")
using PackageCompiler

println("Starting system image compilation...")

# Determine the correct library extension for each operating system
const LIB_EXTENSION = @static if Sys.iswindows()
    "dll"
elseif Sys.isapple()
    "dylib"
else
    "so"
end

# Output library path
const OUTPUT_LIB = "PromptVeilCore.$(LIB_EXTENSION)"

try
    # Compile our package into a shared library
    create_sysimage(
        ["PromptVeilCore", "TokenCompression", "SIMD", "CUDA"],
        sysimage_path=OUTPUT_LIB,
        precompile_execution_file="test/runtests.jl",
        cpu_target="native"
    )
    println("Successfully compiled system image to: $OUTPUT_LIB")
catch e
    println("Error during compilation:")
    println(e)
    exit(1)
end

# Get Julia installation directory for Rust linking
julia_dir = dirname(dirname(Sys.BINDIR))
println("JULIA_DIR=$julia_dir")

# Verify if library was created
if !isfile(OUTPUT_LIB)
    println("Error: Failed to create library file: $OUTPUT_LIB")
    exit(1)
end

# On Windows, generate import files
@static if Sys.iswindows()
    # Find Visual Studio installation
    vs_paths = if Sys.iswindows()
        [
            joinpath(ENV["ProgramFiles"], "Microsoft Visual Studio", "2022", "Community"),
            joinpath(ENV["ProgramFiles"], "Microsoft Visual Studio", "2019", "Community"),
            joinpath(ENV["ProgramFiles (x86)"], "Microsoft Visual Studio", "2022", "Community"),
            joinpath(ENV["ProgramFiles (x86)"], "Microsoft Visual Studio", "2019", "Community")
        ]
    else
        String[]
    end

    vs_path = nothing
    for path in vs_paths
        if isdir(path)
            vs_path = path
            break
        end
    end

    if vs_path === nothing
        println("Error: Visual Studio not found")
        exit(1)
    end

    # Find lib.exe
    lib_exe = joinpath(vs_path, "VC", "Tools", "MSVC")
    if !isdir(lib_exe)
        println("Error: MSVC tools directory not found")
        exit(1)
    end

    # Get latest MSVC version
    msvc_versions = readdir(lib_exe)
    if isempty(msvc_versions)
        println("Error: No MSVC versions found")
        exit(1)
    end
    latest_msvc = sort(msvc_versions)[end]
    lib_exe = joinpath(lib_exe, latest_msvc, "bin", "Hostx64", "x64", "lib.exe")

    if !isfile(lib_exe)
        println("Error: lib.exe not found at: $lib_exe")
        exit(1)
    end

    # Generate .def file
    println("Generating .def file...")
    open("PromptVeilCore.def", "w") do f
        println(f, "LIBRARY PromptVeilCore")
        println(f, "EXPORTS")
        println(f, "    jl_init")
        println(f, "    jl_eval_string")
        println(f, "    jl_call")
        # Add other exported functions as needed
    end

    # Generate .lib file
    println("Generating import library...")
    run(`"$lib_exe" /def:PromptVeilCore.def /out:PromptVeilCore.lib /machine:x64`)
end

println("Build completed successfully!") 