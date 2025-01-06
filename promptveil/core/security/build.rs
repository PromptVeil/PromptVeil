use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Get the Julia DLL path from environment
    let julia_dir = env::var("PROMPTVEIL_CORE_DIR")
        .expect("PROMPTVEIL_CORE_DIR must be set");
    
    let julia_dll = PathBuf::from(&julia_dir).join("PromptVeilCore.dll");
    let julia_lib = PathBuf::from(&julia_dir).join("PromptVeilCore.lib");

    // Print debug information
    println!("promptveil-core@0.1.0: Looking for PromptVeilCore in: {}", julia_dir);
    println!("promptveil-core@0.1.0: Found DLL at: {}", julia_dll.display());
    println!("promptveil-core@0.1.0: Found LIB at: {}", julia_lib.display());

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=native={}", julia_dir);
    
    // Link against the Julia library
    println!("cargo:rustc-link-lib=dylib=PromptVeilCore");

    // Copy the Julia DLL to the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dll = PathBuf::from(&out_dir).join("PromptVeilCore.dll");
    
    // Copy the DLL to the output directory
    std::fs::copy(&julia_dll, &target_dll)
        .expect("Failed to copy Julia DLL");

    // Link against delayimp.lib for delay-loading support
    println!("cargo:rustc-link-lib=delayimp");

    // Configure delay-loading for the Julia DLL
    println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:PromptVeilCore.dll");
    println!("cargo:rustc-link-arg=/INCLUDE:__rust_julia_init");
} 