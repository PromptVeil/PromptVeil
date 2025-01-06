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
    function find_vs_installation()
        vswhere = joinpath(ENV["ProgramFiles (x86)"], "Microsoft Visual Studio", "Installer", "vswhere.exe")
        if !isfile(vswhere)
            error("vswhere.exe not found. Please install Visual Studio with C++ support.")
        end
        
        # Use vswhere to find latest VS installation
        vs_path = read(`$vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`, String)
        if isempty(vs_path)
            error("Visual Studio with C++ tools not found")
        end
        return strip(vs_path)
    end

    function find_lib_exe(vs_path)
        # Find latest MSVC tools version
        vc_tools = joinpath(vs_path, "VC", "Tools", "MSVC")
        if !isdir(vc_tools)
            error("MSVC tools directory not found")
        end
        
        versions = readdir(vc_tools)
        if isempty(versions)
            error("No MSVC versions found")
        end
        
        latest_ver = sort(versions)[end]
        lib_exe = joinpath(vc_tools, latest_ver, "bin", "Hostx64", "x64", "lib.exe")
        
        if !isfile(lib_exe)
            error("lib.exe not found at: $lib_exe")
        end
        
        return lib_exe
    end

    # List of functions to export
    const EXPORTED_FUNCTIONS = [
        # Core Julia functions needed by Rust
        "jl_init",
        "jl_eval_string",
        "jl_call",
        "jl_box_float64",
        "jl_unbox_float64",
        "jl_symbol",
        "jl_get_global",
        "jl_get_function",
        # Our compression functions
        "compress_tokens",
        "decompress_tokens",
        "init_compression",
        "cleanup_compression",
        # CUDA-related functions
        "has_cuda_gpu",
        "init_cuda",
        "cleanup_cuda"
    ]

    println("Setting up Windows import library generation...")
    
    try
        vs_path = find_vs_installation()
        lib_exe = find_lib_exe(vs_path)
        
        # Generate .def file with all exports
        println("Generating .def file...")
        def_file = "PromptVeilCore.def"
        open(def_file, "w") do f
            println(f, "LIBRARY PromptVeilCore")
            println(f, "EXPORTS")
            for func in EXPORTED_FUNCTIONS
                println(f, "    $func")
            end
        end
        
        # Generate .lib file
        println("Generating import library...")
        lib_cmd = `"$lib_exe" /def:$def_file /out:PromptVeilCore.lib /machine:x64`
        println("Running: $lib_cmd")
        run(lib_cmd)
        
        # Verify files were created
        for file in ["PromptVeilCore.lib", "PromptVeilCore.exp", def_file]
            if isfile(file)
                println("Successfully generated: $file")
            else
                error("Failed to generate: $file")
            end
        end
    catch e
        println("Error during import library generation:")
        println(e)
        exit(1)
    end
end

println("Build completed successfully!") 