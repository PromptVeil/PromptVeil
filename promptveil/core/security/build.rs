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
        // On Linux/macOS, create symbolic links with the names expected by the linker
        let lib_link_name = PathBuf::from(&julia_dir).join("libPromptVeilCore.so");
        if !lib_link_name.exists() {
            // Create symbolic link libPromptVeilCore.so -> PromptVeilCore.so
            fs::symlink(&julia_lib_path, &lib_link_name)
                .expect("Failed to create symbolic link");
        }

        // Get the output directory where our Rust library will be installed
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
        let out_dir = PathBuf::from(out_dir);
        
        // Create symbolic link for Julia library in the output directory
        let julia_lib_target = PathBuf::from(&julia_lib_dir).join("libjulia.so.1.11");
        let julia_lib_link = out_dir.join("libjulia.so.1.11");
        
        if !julia_lib_link.exists() && julia_lib_target.exists() {
            println!("promptveil-core@0.1.0: Creating Julia library symlink at: {}", julia_lib_link.display());
            fs::symlink(&julia_lib_target, &julia_lib_link)
                .expect("Failed to create Julia symbolic link in output directory");
        }
        
        // Linking configuration
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,{}", julia_dir);
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,{}", julia_lib_dir.display());
        println!("cargo:rustc-link-lib=dylib=PromptVeilCore");
        println!("cargo:rustc-link-lib=dylib=julia");
        
        // Add current directory to rpath
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN/../../..");
    }
}