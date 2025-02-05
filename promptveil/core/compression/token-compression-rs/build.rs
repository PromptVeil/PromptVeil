use std::env;
use std::path::PathBuf;
use std::fs;

fn main() {
    // Get the path to TokenCompression.jl from package metadata
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let token_compression_path = PathBuf::from(&manifest_dir)
        .join("../TokenCompression.jl")
        .canonicalize()
        .expect("Failed to find TokenCompression.jl");

    // Get Julia's user depot path
    let julia_depot = if let Some(home) = dirs::home_dir() {
        home.join(".julia")
    } else {
        panic!("Could not determine home directory");
    };

    // Create dev directory if it doesn't exist
    let dev_dir = julia_depot.join("dev");
    fs::create_dir_all(&dev_dir).unwrap();

    // Create or update the dev package
    let target_path = dev_dir.join("TokenCompression");
    
    if target_path.exists() {
        fs::remove_dir_all(&target_path).unwrap();
    }

    // Instead of symlink, we'll create a Julia Project.toml that points to our local package
    fs::create_dir_all(&target_path).unwrap();
    
    let project_toml = format!(
        r#"[deps]
        TokenCompression = "c6f261d0-14a8-4f75-8fa4-1d88e9d6eb58"

        [package]
        name = "TokenCompression"
        path = "{}"
        "#,
        token_compression_path.display().to_string().replace('\\', "/")
    );

    fs::write(target_path.join("Project.toml"), project_toml).unwrap();

    // Tell cargo to rerun this script if TokenCompression.jl files change
    println!("cargo:rerun-if-changed={}", token_compression_path.display());
} 