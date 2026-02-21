use std::fs;
use std::path::Path;

fn main() {
    let build_number_path = Path::new(".build_number");
    let profile = std::env::var("PROFILE").unwrap_or_default();

    let mut build_number: u32 = fs::read_to_string(build_number_path)
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0);

    if profile == "release" {
        build_number += 1;
        fs::write(build_number_path, format!("{}\n", build_number)).ok();
    }

    let base_version = env!("CARGO_PKG_VERSION");
    println!("cargo:rustc-env=BUILD_VERSION={}", base_version);
    println!("cargo:rustc-env=BUILD_NUMBER={}", build_number);

    println!("cargo:rerun-if-changed=.build_number");
    println!("cargo:rerun-if-changed=build.rs");
}
