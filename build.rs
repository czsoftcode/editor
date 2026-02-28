use std::fs;
use std::path::Path;
use std::thread;

fn main() {
    // Dynamické nastavení prostředků pro Cargo
    let cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let build_jobs = (cores * 2 / 3).max(1);
    let env_threads = (cores / 2).max(1);

    let config_dir = Path::new(".cargo");
    let config_path = config_dir.join("config.toml");

    if !config_dir.exists() {
        let _ = fs::create_dir_all(config_dir);
    }

    let config_content = format!(
        "[build]\njobs = {}\n\n[env]\nRAYON_NUM_THREADS = \"{}\"\nTOKIO_WORKER_THREADS = \"{}\"\n",
        build_jobs, env_threads, env_threads
    );

    // Zapíšeme pouze pokud se obsah změnil (aby se zbytečně neaktualizovaly časy souborů)
    let current_content = fs::read_to_string(&config_path).unwrap_or_default();
    if current_content != config_content {
        let _ = fs::write(&config_path, config_content).ok();
    }

    // Stávající logika číslování verzí
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
