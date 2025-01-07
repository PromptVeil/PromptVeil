use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Get the Julia library path from environment
    let julia_dir = env::var("PROMPTVEIL_CORE_DIR")
        .expect("PROMPTVEIL_CORE_DIR must be set");
    
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

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=native={}", julia_dir);
    
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
        // On Linux/macOS, pass the full library path
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,{}", julia_dir);
        println!("cargo:rustc-cdylib-link-arg=-Wl,--push-state,-Bdynamic");
        println!("cargo:rustc-cdylib-link-arg=-Wl,{}/{}", julia_dir, lib_name);
        println!("cargo:rustc-cdylib-link-arg=-Wl,--pop-state");
    }
}