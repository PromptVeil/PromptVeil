# Check required files first
const REQUIRED_FILES = @static if Sys.iswindows()
    [
        "PromptVeilCore.dll",
        "PromptVeilCore.lib",
        "PromptVeilCore.exp",
        "PromptVeilCore.def"
    ]
else
    ["PromptVeilCore.$(Sys.isapple() ? "dylib" : "so")"]
end

function generate_import_files()
    println("Generating Windows import files...")
    
    # Find Visual Studio
    program_files_x86 = get(ENV, "ProgramFiles(x86)", get(ENV, "PROGRAMFILES(X86)", nothing))
    if program_files_x86 === nothing
        error("Could not find Program Files (x86) directory")
    end
    
    # Find vswhere
    vswhere = joinpath(program_files_x86, "Microsoft Visual Studio", "Installer", "vswhere.exe")
    if !isfile(vswhere)
        error("vswhere.exe not found. Please install Visual Studio with C++ support.")
    end
    
    # Get VS path
    vs_path = strip(read(`"$vswhere" -latest -products "*" -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`, String))
    if isempty(vs_path)
        error("Visual Studio with C++ tools not found")
    end
    
    # Find lib.exe
    vc_tools = joinpath(vs_path, "VC", "Tools", "MSVC")
    latest_ver = sort(readdir(vc_tools))[end]
    lib_exe = joinpath(vc_tools, latest_ver, "bin", "Hostx64", "x64", "lib.exe")
    
    if !isfile(lib_exe)
        error("lib.exe not found at: $lib_exe")
    end
    
    # Generate .def file
    println("Generating .def file...")
    open("PromptVeilCore.def", "w") do f
        println(f, "LIBRARY PromptVeilCore")
        println(f, "EXPORTS")
        # Core Julia functions
        println(f, "    jl_init")
        println(f, "    jl_eval_string")
        println(f, "    jl_call")
        println(f, "    jl_box_float64")
        println(f, "    jl_unbox_float64")
        println(f, "    jl_symbol")
        println(f, "    jl_get_global")
        println(f, "    jl_get_function")
        # Our functions with configuration
        println(f, "    julia_optimize_tokens_config")
        println(f, "    julia_compress_batch_config")
        println(f, "    julia_decompress_batch")
        # Utility functions
        println(f, "    has_cuda_gpu")
        println(f, "    init_cuda")
        println(f, "    cleanup_cuda")
    end
    
    # Generate .lib file
    println("Generating import library...")
    run(`"$lib_exe" /def:PromptVeilCore.def /out:PromptVeilCore.lib /machine:x64`)
    
    # Verify generated files
    for file in ["PromptVeilCore.lib", "PromptVeilCore.exp", "PromptVeilCore.def"]
        if !isfile(file)
            error("Failed to generate: $file")
        end
        println("Successfully generated: $file")
    end
end

# Check what files are missing
const MISSING_FILES = filter(!isfile, REQUIRED_FILES)

# If we're here, we need a full build
println("Running setup...")
include("setup.jl")

println("Loading PackageCompiler...")
using PackageCompiler

if !isfile("PromptVeilCore.dll")
    println("Compiling system image...")
    create_sysimage(
        ["PromptVeilCore", "TokenCompression", "SIMD", "CUDA"],
        sysimage_path="PromptVeilCore.dll",
        precompile_execution_file="test/runtests.jl",
        cpu_target="native"
    )
    
    # After DLL is created, generate import files on Windows
    if Sys.iswindows()
        try
            generate_import_files()
        catch e
            println("Error generating import files: $e")
            error("Failed to generate Windows import files")
        end
    end
else
    # If DLL exists but other files are missing on Windows
    if Sys.iswindows() && !isempty(MISSING_FILES)
        try
            generate_import_files()
        catch e
            println("Error generating import files: $e")
            error("Failed to generate Windows import files")
        end
    end
end 