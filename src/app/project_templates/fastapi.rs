use std::fs;
use std::path::Path;

pub(crate) fn generate(name: &str, path: &Path) -> Result<(), String> {
    let main_py = format!(
        r#"from fastapi import FastAPI

app = FastAPI(title="{0}")

@app.get("/")
async def root():
    return {{"message": "Hello from FastAPI in PolyCredo Editor"}}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
"#,
        name
    );

    let requirements = "fastapi[all]>=0.135.1
uvicorn>=0.30.0
";

    fs::write(path.join("main.py"), main_py).map_err(|e| e.to_string())?;
    fs::write(path.join("requirements.txt"), requirements).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("app")).map_err(|e| e.to_string())?;
    fs::write(path.join("app/__init__.py"), "").map_err(|e| e.to_string())?;
    fs::write(
        path.join(".gitignore"),
        "__pycache__/
.venv/
.env
",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
