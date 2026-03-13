use std::fs;
use std::path::Path;

pub(crate) fn generate(name: &str, path: &Path) -> Result<(), String> {
    let package_json = format!(
        r#"{{
  "name": "{0}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  }},
  "dependencies": {{
    "next": "^16.1.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }},
  "devDependencies": {{
    "typescript": "^5.4.0",
    "@types/node": "^22.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "eslint": "^9.0.0",
    "eslint-config-next": "^16.1.0"
  }}
}}"#,
        name
    );

    let page_tsx = r#"export default function Home() {
  return (
    <main style={{ padding: '2rem' }}>
      <h1>{0}</h1>
      <p>Created with PolyCredo Editor and Next.js</p>
    </main>
  );
}
"#
    .replace("{0}", name);

    fs::write(path.join("package.json"), package_json).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("app")).map_err(|e| e.to_string())?;
    fs::write(path.join("app/page.tsx"), page_tsx).map_err(|e| e.to_string())?;
    fs::write(
        path.join(".gitignore"),
        "node_modules/
.next/
",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
