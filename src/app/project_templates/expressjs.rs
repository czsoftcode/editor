use std::fs;
use std::path::Path;

pub(crate) fn generate(name: &str, path: &Path) -> Result<(), String> {
    let package_json = format!(
        r#"{{
  "name": "{0}",
  "version": "1.0.0",
  "dependencies": {{
    "express": "^5.0.0"
  }},
  "devDependencies": {{
    "@types/express": "^5.0.0",
    "@types/node": "^22.0.0",
    "typescript": "^5.4.0"
  }}
}}"#,
        name
    );

    let index_ts = r#"import express, { Request, Response } from 'express';
const app = express();
app.get('/', (req: Request, res: Response) => res.send('Hello from Express 5!'));
app.listen(3000);
"#;

    fs::write(path.join("package.json"), package_json).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("src")).map_err(|e| e.to_string())?;
    fs::write(path.join("src/index.ts"), index_ts).map_err(|e| e.to_string())?;
    fs::write(path.join(".gitignore"), "node_modules/
dist/
").map_err(|e| e.to_string())?;

    Ok(())
}
