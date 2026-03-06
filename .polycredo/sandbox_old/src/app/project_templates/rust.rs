use std::fs;
use std::path::Path;

pub(crate) fn generate(name: &str, path: &Path) -> Result<(), String> {
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
"#,
        name
    );

    let main_rs = r#"fn main() {
    println!("Hello, world!");
}
"#;

    fs::write(path.join("Cargo.toml"), cargo_toml).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("src")).map_err(|e| e.to_string())?;
    fs::write(path.join("src/main.rs"), main_rs).map_err(|e| e.to_string())?;
    fs::write(path.join(".gitignore"), "/target\n").map_err(|e| e.to_string())?;

    Ok(())
}
