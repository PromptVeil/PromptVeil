use std::env;
use std::path::PathBuf;
use std::os::unix::fs;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
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
        // On Linux/macOS, copy the library to the output directory (similar to Windows)
        let out_dir = env::var("OUT_DIR").unwrap();
        let target_lib = PathBuf::from(&out_dir).join(lib_name);
        
        // Try both possible locations for the library
        let compression_lib = PathBuf::from(&julia_dir).join(lib_name);
        let build_lib = PathBuf::from("build/julia_build").join(lib_name);
        
        // Print debug information about library locations
        println!("promptveil-core@0.1.0: Trying to copy from compression dir: {}", compression_lib.display());
        println!("promptveil-core@0.1.0: Trying to copy from build dir: {}", build_lib.display());
        
        if compression_lib.exists() {
            println!("promptveil-core@0.1.0: Found library in compression dir");
            std::fs::copy(&compression_lib, &target_lib)
                .expect("Failed to copy Julia library from compression dir");
        } else if build_lib.exists() {
            println!("promptveil-core@0.1.0: Found library in build dir");
            std::fs::copy(&build_lib, &target_lib)
                .expect("Failed to copy Julia library from build dir");
        } else {
            panic!("Could not find PromptVeilCore library in any expected location");
        }

        // Linking configuration
        println!("cargo:rustc-link-lib=dylib=PromptVeilCore");
        
        // Add output directory to rpath
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
    }
}