use std::env;
use std::path::PathBuf;
use std::os::unix::fs as unix_fs;
use std::fs;
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
    let julia_lib_dir = PathBuf::from("/opt/julia-1.11.2/lib");
    
    // Determine platform-specific library names
    let (lib_name, _) = if cfg!(target_os = "windows") {
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
    
    // Check if layouts file exists
    let layouts_path = current_dir.join("src").join("layouts.rs");
    if !layouts_path.exists() {
        panic!("layouts.rs not found. Make sure to build Julia core first to generate the layouts file.");
    }

    // Copy libraries and set up linking
    if !cfg!(target_os = "windows") {
        let compression_lib = PathBuf::from(&julia_dir).join(lib_name);
        let build_lib = workspace_dir.join("build").join("julia_build").join(lib_name);
        
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
                
                unix_fs::symlink(&link_target, &target)
                    .expect(&format!("Failed to create symlink: {}", julia_lib));
            } else {
                std::fs::copy(&source, &target)
                    .expect(&format!("Failed to copy Julia library: {}", julia_lib));
            }
        }

        // Copy auxiliary libraries
        let julia_aux_dir = julia_lib_dir.join("julia");
        let python_aux_dir = python_lib_dir.join("julia");
        
        if !python_aux_dir.exists() {
            std::fs::create_dir_all(&python_aux_dir)
                .expect("Failed to create Julia auxiliary directory");
        }
        
        println!("promptveil-core@0.1.0: Copying Julia auxiliary libraries from: {} -> {}", 
            julia_aux_dir.display(), python_aux_dir.display());
            
        for entry in std::fs::read_dir(julia_aux_dir).expect("Failed to read Julia auxiliary directory") {
            let entry = entry.expect("Failed to read directory entry");
            let source = entry.path();
            let target = python_aux_dir.join(entry.file_name());
            
            println!("promptveil-core@0.1.0: Copying Julia auxiliary library: {} -> {}", 
                source.display(), target.display());
                
            if source.is_symlink() {
                let link_target = source.read_link()
                    .expect(&format!("Failed to read symlink: {}", source.display()));
                
                // Remove existing symlink if it exists
                if target.exists() {
                    std::fs::remove_file(&target)
                        .expect(&format!("Failed to remove existing symlink: {}", target.display()));
                }
                
                unix_fs::symlink(&link_target, &target)
                    .expect(&format!("Failed to create symlink: {}", target.display()));
            } else {
                std::fs::copy(&source, &target)
                    .expect(&format!("Failed to copy Julia auxiliary library: {}", source.display()));
            }
        }
        
        // Linking configuration
        println!("cargo:rustc-link-lib=dylib=PromptVeilCore");
        
        // Add output directory to rpath
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
    }
} 