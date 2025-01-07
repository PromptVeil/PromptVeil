use std::env;
use std::path::PathBuf;
use std::os::unix::fs;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Get the workspace directory (where the build is being run from)
    let workspace_dir = env::current_dir()
        .expect("Failed to get current directory");
    
    // Get the Julia library path from environment
    let julia_dir = env::var("PROMPTVEIL_CORE_DIR")
        .expect("PROMPTVEIL_CORE_DIR must be set");
    
    // Get Julia installation directory
    let julia_install_dir = env::var("JULIA_DIR")
        .expect("JULIA_DIR must be set");
    let julia_lib_dir = PathBuf::from("/opt/julia-1.11.2/lib");
    
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
        let python_target = python_lib_dir.join(lib_name);
        std::fs::copy(&source_lib, &python_target)
            .expect("Failed to copy Julia library to Python package directory");

        // Linking configuration
        println!("cargo:rustc-link-lib=dylib=PromptVeilCore");
        
        // Add output directory to rpath
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
    }
}