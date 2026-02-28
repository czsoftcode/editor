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
    println!("cargo:rerun-if-changed=packaging/icons/icon.ico");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let icon_path = "packaging/icons/icon.ico";
        let rc_path = std::path::Path::new(&out_dir).join("icon.rc");
        let res_path = std::path::Path::new(&out_dir).join("icon.res");

        // 1. Vytvoříme resource script (.rc)
        // Cesta k ikoně musí být relativní k místu spuštění nebo absolutní.
        // Použijeme absolutní cestu pro jistotu.
        let abs_icon_path = std::fs::canonicalize(icon_path).unwrap();
        let rc_content = format!(
            "1 ICON \"{}\"",
            abs_icon_path.display().to_string().replace("\\", "/")
        );
        std::fs::write(&rc_path, rc_content).unwrap();

        // 2. Pokusíme se zkompilovat .res soubor pomocí llvm-rc
        let status = std::process::Command::new("llvm-rc")
            .arg("/fo")
            .arg(&res_path)
            .arg(&rc_path)
            .status();

        if let Ok(s) = status {
            if s.success() {
                // 3. Řekneme Cargu, aby slinkoval vygenerovaný .res soubor
                println!("cargo:rustc-link-arg={}", res_path.display());
            } else {
                panic!("llvm-rc failed to compile resources. Check if the icon.ico is valid.");
            }
        } else {
            // Pokud llvm-rc není v systému, zkusíme fallback na winres (např. pro nativní build na Windows)
            let mut res = winres::WindowsResource::new();
            res.set_icon(icon_path);
            if let Err(e) = res.compile() {
                eprintln!("Warning: Could not compile Windows resources: {}", e);
            }
        }
    }
}
