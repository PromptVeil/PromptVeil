use std::env;
use std::path::PathBuf;
use std::os::unix::fs as unix_fs;
use std::fs;
use std::process::Command;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../compression/src/PromptVeilCore.jl");
    
    // Get the workspace directory (root of the project)
    let current_dir = env::current_dir()
        .expect("Failed to get current directory");
    let workspace_dir = current_dir.ancestors()
        .find(|p| p.join("promptveil").join("core").exists())
        .expect("Failed to find workspace root directory");
    
    // Get the Julia library path from environment
    let julia_dir = env::var("PROMPTVEIL_CORE_DIR")
        .expect("PROMPTVEIL_CORE_DIR must be set");
    
    // Get Julia installation directory
    let julia_install_dir = env::var("JULIA_DIR")
        .expect("JULIA_DIR must be set");
    let julia_lib_dir = PathBuf::from("/opt/julia-1.11.2/lib");
    let julia_internal_lib_dir = PathBuf::from("/opt/julia-1.11.2/lib/julia");
    
    // Determine platform-specific library names
    let (lib_name, lib_ext) = if cfg!(target_os = "windows") {
        ("PromptVeilCore.dll", "PromptVeilCore.lib")
    } else if cfg!(target_os = "macos") {
        ("PromptVeilCore.dylib", "PromptVeilCore.dylib")
    } else {
        ("PromptVeilCore.so", "PromptVeilCore.so")
    };

    let julia_lib_path = PathBuf::from(&julia_dir).join(lib_name);

    // Print debug information
    println!("promptveil-core@0.1.0: Looking for PromptVeilCore in: {}", julia_dir);
    println!("promptveil-core@0.1.0: Found library at: {}", julia_lib_path.display());
    println!("promptveil-core@0.1.0: Julia lib directory: {}", julia_lib_dir.display());
    println!("promptveil-core@0.1.0: Current directory: {}", current_dir.display());
    println!("promptveil-core@0.1.0: Workspace directory: {}", workspace_dir.display());

    // Tell cargo to look for shared libraries in the specified directories
    println!("cargo:rustc-link-search=native={}", julia_dir);
    println!("cargo:rustc-link-search=native={}", julia_lib_dir.display());
    
    // Link against the Julia library
    if cfg!(target_os = "windows") {
        // Windows uses the .lib file for linking
        println!("cargo:rustc-cdylib-link-arg=/DEFAULTLIB:{}", lib_ext);

        // Copy the Julia library to the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
        let target_lib = PathBuf::from(&out_dir).join(lib_name);
    
        std::fs::copy(&julia_lib_path, &target_lib)
            .expect("Failed to copy Julia library");

        // Windows-specific linking
    println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:{}", lib_name);
        println!("cargo:rustc-cdylib-link-arg=/INCLUDE:__rust_julia_init");
    } else {
        // Try both possible locations for the library
        let compression_lib = workspace_dir.join("promptveil/core/compression").join(lib_name);
        let build_lib = workspace_dir.join("build/julia_build").join(lib_name);
        
        // Print debug information about library locations
        println!("promptveil-core@0.1.0: Trying to copy from compression dir: {}", compression_lib.display());
        println!("promptveil-core@0.1.0: Trying to copy from build dir: {}", build_lib.display());
        
        // Get the source library path
        let source_lib = if compression_lib.exists() {
            println!("promptveil-core@0.1.0: Found library in compression dir");
            compression_lib
        } else if build_lib.exists() {
            println!("promptveil-core@0.1.0: Found library in build dir");
            build_lib
        } else {
            panic!("Could not find PromptVeilCore library in any expected location");
        };

        // Copy to both the output directory and the Python package directory
        let out_dir = env::var("OUT_DIR").unwrap();
        let target_lib = PathBuf::from(&out_dir).join(lib_name);
        
        // Copy to output directory for linking
        std::fs::copy(&source_lib, &target_lib)
            .expect("Failed to copy Julia library to output directory");
            
        // Copy to Python package directory
        let python_lib_dir = workspace_dir.join("build/venv/lib/python3.10/site-packages/promptveil_core");
        if !python_lib_dir.exists() {
            std::fs::create_dir_all(&python_lib_dir)
                .expect("Failed to create Python package directory");
        }
        
        // Copy PromptVeilCore
        let python_target = python_lib_dir.join(lib_name);
        std::fs::copy(&source_lib, &python_target)
            .expect("Failed to copy Julia library to Python package directory");

        // Copy Julia runtime libraries
        let julia_libs = [
            "libjulia.so",
            "libjulia.so.1.11",
            "libjulia.so.1.11.2"
        ];

        // Copy main Julia libraries
        for julia_lib in julia_libs.iter() {
            let source = julia_lib_dir.join(julia_lib);
            let target = python_lib_dir.join(julia_lib);
            
            println!("promptveil-core@0.1.0: Copying Julia runtime library: {} -> {}", 
                source.display(), target.display());
                
            if source.is_symlink() {
                let link_target = source.read_link()
                    .expect(&format!("Failed to read symlink: {}", julia_lib));
                
                // Remove existing symlink if it exists
                if target.exists() {
                    std::fs::remove_file(&target)
                        .expect(&format!("Failed to remove existing symlink: {}", julia_lib));
                }
                
                unix_fs::symlink(link_target, &target)
                    .expect(&format!("Failed to create symlink: {}", julia_lib));
            } else {
                // Remove existing file if it exists
                if target.exists() {
                    std::fs::remove_file(&target)
                        .expect(&format!("Failed to remove existing file: {}", julia_lib));
                }
                
                std::fs::copy(&source, &target)
                    .expect(&format!("Failed to copy Julia runtime library: {}", julia_lib));
            }
        }

        // Copy Julia auxiliary libraries directory
        let julia_aux_dir = julia_lib_dir.join("julia");
        let target_julia_dir = python_lib_dir.join("julia");
        
        println!("promptveil-core@0.1.0: Copying Julia auxiliary libraries from: {} -> {}", 
            julia_aux_dir.display(), target_julia_dir.display());
            
        // Remove existing directory if it exists
        if target_julia_dir.exists() {
            std::fs::remove_dir_all(&target_julia_dir)
                .expect("Failed to remove existing Julia auxiliary directory");
        }
        
        // Create target directory
        std::fs::create_dir_all(&target_julia_dir)
            .expect("Failed to create Julia auxiliary directory");
            
        // Copy all files from julia directory
        for entry in std::fs::read_dir(&julia_aux_dir)
            .expect("Failed to read Julia auxiliary directory") {
            let entry = entry.expect("Failed to read directory entry");
            let source = entry.path();
            let target = target_julia_dir.join(entry.file_name());
            
            if source.is_file() {
                println!("promptveil-core@0.1.0: Copying Julia auxiliary library: {} -> {}", 
                    source.display(), target.display());
                    
                std::fs::copy(&source, &target)
                    .expect(&format!("Failed to copy Julia auxiliary library: {}", source.display()));
            }
        }

        // Linking configuration
        println!("cargo:rustc-link-lib=dylib=PromptVeilCore");
        
        // Add output directory to rpath
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
    }

    // Get the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let layouts_path = Path::new(&out_dir).join("layouts.rs");
    
    // Run Julia to generate layouts
    let julia_script = r#"
        using JlrsCore.Reflect
        include("../compression/src/PromptVeilCore.jl")
        
        # Generate layouts for our types
        layouts = reflect([PromptVeilCore.CompressionConfig, PromptVeilCore.CompressedResult])
        
        # Write to the output file
        open(ARGS[1], "w") do f
            write(f, layouts)
        end
    "#;
    
    // Create temporary script file
    std::fs::write("build_script.jl", julia_script).unwrap();
    
    // Run Julia script
    let status = Command::new("julia")
        .arg("build_script.jl")
        .arg(&layouts_path)
        .status()
        .expect("Failed to run Julia script");
        
    if !status.success() {
        panic!("Failed to generate Rust layouts from Julia types");
    }
    
    // Clean up
    std::fs::remove_file("build_script.jl").unwrap();
    
    // Copy generated layouts to src directory
    std::fs::copy(layouts_path, "src/layouts.rs").unwrap();
    
    println!("cargo:rerun-if-changed=src/layouts.rs");
} 