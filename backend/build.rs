use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/build");

    // Get the manifest directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let frontend_dir = Path::new(&manifest_dir).join("../frontend");
    let build_dir = frontend_dir.join("build");

    // Check if frontend build exists, if not try to build it
    if !build_dir.exists() {
        println!("cargo:warning=Frontend build directory not found, attempting to build...");
        
        // Try to build the frontend
        let output = Command::new("pnpm")
            .arg("build")
            .current_dir(&frontend_dir)
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    println!("cargo:warning=Failed to build frontend: {}", 
                             String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to run pnpm build: {}", e);
            }
        }
    }

    // Tell cargo where to find the frontend assets
    println!("cargo:rustc-env=FRONTEND_BUILD_DIR={}", build_dir.display());
}