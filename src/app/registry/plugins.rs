use extism::Plugin;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a loaded WASM plugin.
pub struct LoadedPlugin {
    pub id: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub inner: Plugin,
}

/// Manages loading and communication with WASM plugins.
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Loads all .wasm files from the given directory.
    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir_path: P) -> anyhow::Result<()> {
        let dir = dir_path.as_ref();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                self.load_plugin(path)?;
            }
        }

        Ok(())
    }

    /// Loads a single .wasm file as a plugin.
    pub fn load_plugin(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let wasm = fs::read(&path)?;
        let manifest = extism::Manifest::new([extism::Wasm::data(wasm)]);

        // In a real scenario, we'd add host functions here (e.g., to call back into the editor)
        let plugin = Plugin::new(&manifest, [], true)?;

        println!("Plugin loaded: {}", id);

        self.plugins.push(LoadedPlugin {
            id,
            path,
            inner: plugin,
        });

        Ok(())
    }

    /// Calls a function in a plugin and returns the response.
    pub fn call(
        &mut self,
        plugin_id: &str,
        func_name: &str,
        input: &str,
    ) -> anyhow::Result<String> {
        if let Some(p) = self.plugins.iter_mut().find(|p| p.id == plugin_id) {
            let output = p.inner.call::<&str, &str>(func_name, input)?;
            Ok(output.to_string())
        } else {
            anyhow::bail!("Plugin not found: {}", plugin_id)
        }
    }

    #[allow(dead_code)]
    pub fn get_loaded_ids(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.id.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_plugin() {
        let mut manager = PluginManager::new();
        let plugins_dir = crate::ipc::plugins_dir();
        if let Err(e) = manager.load_from_dir(&plugins_dir) {
            println!("No plugins found or failed to load: {}", e);
            return;
        }

        if manager.get_loaded_ids().contains(&"hello".to_string()) {
            match manager.call("hello", "hello", "Gemini") {
                Ok(res) => println!("Plugin response: {}", res),
                Err(e) => panic!("Plugin call failed: {}", e),
            }
        } else {
            println!("hello.wasm not found in {:?}", plugins_dir);
        }
    }
}
