use extism::{CurrentPlugin, Function, Manifest, Plugin, UserData, Val, ValType, Wasm};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
}

pub enum PluginStatus {
    /// Plugin is fully loaded and ready to use.
    Active {
        inner: Box<Plugin>,
        config_hash: u64,
    },
    /// Plugin is waiting for user to approve permissions.
    PendingAuthorization {
        metadata: PluginMetadata,
        wasm_bytes: Vec<u8>,
    },
    /// User explicitly denied permissions.
    #[allow(dead_code)]
    Denied,
    /// Failed to load (e.g. invalid WASM or TOML).
    #[allow(dead_code)]
    Error(String),
}

/// State passed to host functions
#[derive(Clone, Default)]
pub struct HostContext {
    pub active_file_path: Option<String>,
    pub active_file_content: Option<String>,
}

#[derive(Clone)]
struct HostState {
    sandbox_root: PathBuf,
    blacklist: Arc<Mutex<Vec<String>>>,
    context: Arc<Mutex<HostContext>>,
}

impl HostState {
    fn is_allowed(&self, rel_path: &Path) -> bool {
        let path_str = rel_path.to_string_lossy();

        if rel_path.is_absolute() || path_str.contains("..") {
            return false;
        }

        let blacklist = self.blacklist.lock().expect("lock");
        for pattern in blacklist.iter() {
            if glob_match(pattern, &path_str) {
                return false;
            }
        }

        true
    }
}

fn glob_match(pattern: &str, path: &str) -> bool {
    let regex_pattern = pattern
        .replace('.', "\\.")
        .replace('*', ".*")
        .replace('?', ".");
    if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
        re.is_match(path)
    } else {
        path.contains(pattern)
    }
}

/// Information about a loaded WASM plugin.
pub struct LoadedPlugin {
    pub id: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub status: PluginStatus,
    pub wasm_bytes: Vec<u8>,
}

/// Manages loading and communication with WASM plugins.
pub struct PluginManager {
    pub plugins: Arc<Mutex<Vec<LoadedPlugin>>>,
    pub sandbox_root: PathBuf,
    pub blacklist: Arc<Mutex<Vec<String>>>,
    pub current_context: Arc<Mutex<HostContext>>,
}

impl PluginManager {
    pub fn new(sandbox_root: PathBuf) -> Self {
        Self {
            plugins: Arc::new(Mutex::new(Vec::new())),
            sandbox_root,
            blacklist: Arc::new(Mutex::new(Vec::new())),
            current_context: Arc::new(Mutex::new(HostContext::default())),
        }
    }

    pub fn set_context(&self, context: HostContext) {
        let mut ctx = self.current_context.lock().expect("lock");
        *ctx = context;
    }

    pub fn set_blacklist(&self, mut blacklist: Vec<String>) {
        let gitignore_path = self
            .sandbox_root
            .parent()
            .and_then(|p| p.parent().map(|p| p.join(".gitignore")));
        if let Some(path) = gitignore_path
            && let Ok(content) = fs::read_to_string(path)
        {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    blacklist.push(line.to_string());
                }
            }
        }
        let mut b = self.blacklist.lock().expect("lock");
        *b = blacklist;
    }

    pub fn load_from_dir<P: AsRef<Path>>(&self, dir_path: P) -> anyhow::Result<()> {
        let dir = dir_path.as_ref();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm")
                && let Err(e) = self.load_plugin(path)
            {
                eprintln!("Error loading plugin: {}", e);
            }
        }

        Ok(())
    }

    pub fn load_plugin(&self, path: PathBuf) -> anyhow::Result<()> {
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let wasm_bytes = fs::read(&path)?;
        let metadata_path = path.with_extension("toml");

        let mut plugins = self.plugins.lock().expect("lock");

        if !metadata_path.exists() {
            plugins.push(LoadedPlugin {
                id,
                path,
                status: PluginStatus::Error("Missing .toml manifest".to_string()),
                wasm_bytes,
            });
            return Ok(());
        }

        let metadata_content = fs::read_to_string(&metadata_path)?;
        let metadata: PluginMetadata = toml::from_str(&metadata_content)?;

        if !metadata.allowed_hosts.is_empty() {
            plugins.push(LoadedPlugin {
                id,
                path,
                status: PluginStatus::PendingAuthorization {
                    metadata,
                    wasm_bytes: wasm_bytes.clone(),
                },
                wasm_bytes,
            });
        } else {
            let plugin =
                self.create_instance(&wasm_bytes, &metadata, &std::collections::HashMap::new())?;
            plugins.push(LoadedPlugin {
                id,
                path,
                status: PluginStatus::Active {
                    inner: Box::new(plugin),
                    config_hash: 0,
                },
                wasm_bytes,
            });
        }

        Ok(())
    }

    pub fn authorize(
        &self,
        plugin_id: &str,
        plugin_config: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let (metadata, wasm_bytes) = {
            let plugins = self.plugins.lock().expect("lock");
            let p = plugins
                .iter()
                .find(|p| p.id == plugin_id)
                .ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;
            match &p.status {
                PluginStatus::PendingAuthorization {
                    metadata,
                    wasm_bytes,
                } => (metadata.clone(), wasm_bytes.clone()),
                _ => anyhow::bail!("Plugin is not in pending state"),
            }
        };

        let plugin = self.create_instance(&wasm_bytes, &metadata, plugin_config)?;
        let config_hash = self.calculate_config_hash(plugin_config);

        let mut plugins = self.plugins.lock().expect("lock");
        if let Some(p) = plugins.iter_mut().find(|p| p.id == plugin_id) {
            p.status = PluginStatus::Active {
                inner: Box::new(plugin),
                config_hash,
            };
        }

        Ok(())
    }

    fn calculate_config_hash(&self, config: &std::collections::HashMap<String, String>) -> u64 {
        use std::collections::BTreeMap;
        use std::hash::{Hash, Hasher};
        let sorted: BTreeMap<_, _> = config.iter().collect();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for (k, v) in sorted {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }
        hasher.finish()
    }

    fn create_instance(
        &self,
        wasm_bytes: &[u8],
        metadata: &PluginMetadata,
        plugin_config: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<Plugin> {
        let mut manifest = Manifest::new([Wasm::data(wasm_bytes.to_vec())]);
        for host in &metadata.allowed_hosts {
            manifest = manifest.with_allowed_host(host);
        }
        manifest = manifest.with_config(plugin_config.iter());

        let host_state = HostState {
            sandbox_root: self.sandbox_root.clone(),
            blacklist: Arc::clone(&self.blacklist),
            context: Arc::clone(&self.current_context),
        };

        let functions = vec![
            Function::new(
                "read_project_file",
                [ValType::I64],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_read_file,
            ),
            Function::new(
                "list_project_files",
                [],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_list_files,
            ),
            Function::new(
                "get_active_file_path",
                [],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_get_active_path,
            ),
            Function::new(
                "get_active_file_content",
                [],
                [ValType::I64],
                UserData::new(host_state),
                host_get_active_content,
            ),
        ];

        Plugin::new(&manifest, functions, true).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn call(
        &self,
        plugin_id: &str,
        func_name: &str,
        input: &str,
        current_config: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let (wasm_bytes, metadata, needs_reinit) = {
            let plugins = self.plugins.lock().expect("lock");
            let p = plugins
                .iter()
                .find(|p| p.id == plugin_id)
                .ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;
            match &p.status {
                PluginStatus::Active { config_hash, .. } => {
                    let new_hash = self.calculate_config_hash(current_config);
                    if *config_hash != new_hash {
                        let meta_path = p.path.with_extension("toml");
                        let meta = if meta_path.exists() {
                            let content = fs::read_to_string(meta_path)?;
                            toml::from_str(&content)?
                        } else {
                            PluginMetadata::default()
                        };
                        (p.wasm_bytes.clone(), meta, true)
                    } else {
                        (Vec::new(), PluginMetadata::default(), false)
                    }
                }
                _ => anyhow::bail!("Plugin {} is not active", plugin_id),
            }
        };

        if needs_reinit {
            let plugin = self.create_instance(&wasm_bytes, &metadata, current_config)?;
            let config_hash = self.calculate_config_hash(current_config);
            let mut plugins = self.plugins.lock().expect("lock");
            if let Some(p) = plugins.iter_mut().find(|p| p.id == plugin_id) {
                p.status = PluginStatus::Active {
                    inner: Box::new(plugin),
                    config_hash,
                };
            }
        }

        let mut plugins = self.plugins.lock().expect("lock");
        if let Some(p) = plugins.iter_mut().find(|p| p.id == plugin_id) {
            if let PluginStatus::Active { inner, .. } = &mut p.status {
                let output = inner.call::<&str, &str>(func_name, input)?;
                Ok(output.to_string())
            } else {
                unreachable!()
            }
        } else {
            anyhow::bail!("Plugin not found during call execution")
        }
    }

    pub fn get_pending_authorizations(&self) -> Vec<(String, PluginMetadata)> {
        let plugins = self.plugins.lock().expect("lock");
        plugins
            .iter()
            .filter_map(|p| {
                if let PluginStatus::PendingAuthorization { metadata, .. } = &p.status {
                    Some((p.id.clone(), metadata.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_loaded_ids(&self) -> Vec<String> {
        let plugins = self.plugins.lock().expect("lock");
        plugins.iter().map(|p| p.id.clone()).collect()
    }
}

// ---------------------------------------------------------------------------
// Host Functions Implementation
// ---------------------------------------------------------------------------

fn host_read_file(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state = user_data.get()?;
    let state_lock = state
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // V Extismu 1.13 se String získá takto bez přímého MemoryHandle
    let path_str: String = plugin.memory_get_val(&inputs[0])?;
    let rel_path = Path::new(&path_str);

    if !state_lock.is_allowed(rel_path) {
        let msg = format!("Security violation: Access to '{}' is blocked", path_str);
        let h = plugin.memory_alloc(msg.len() as u64)?;
        plugin.memory_bytes_mut(h)?.copy_from_slice(msg.as_bytes());
        outputs[0] = Val::I64(h.offset() as i64);
        return Ok(());
    }

    let full_path = state_lock.sandbox_root.join(rel_path);
    let content = fs::read_to_string(full_path)
        .unwrap_or_else(|_| "File not found or unreadable".to_string());

    let h = plugin.memory_alloc(content.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(content.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn host_list_files(
    plugin: &mut CurrentPlugin,
    _inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state = user_data.get()?;
    let state_lock = state
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(&state_lock.sandbox_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && let Ok(rel) = entry.path().strip_prefix(&state_lock.sandbox_root)
            && state_lock.is_allowed(rel)
        {
            files.push(rel.to_string_lossy().into_owned());
        }
    }

    let result = files.join("\n");
    let h = plugin.memory_alloc(result.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(result.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn host_get_active_path(
    plugin: &mut CurrentPlugin,
    _inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state = user_data.get()?;
    let state_lock = state
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
    let ctx = state_lock.context.lock().expect("lock");

    let path = ctx.active_file_path.as_deref().unwrap_or("");
    let h = plugin.memory_alloc(path.len() as u64)?;
    plugin.memory_bytes_mut(h)?.copy_from_slice(path.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn host_get_active_content(
    plugin: &mut CurrentPlugin,
    _inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state = user_data.get()?;
    let state_lock = state
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
    let ctx = state_lock.context.lock().expect("lock");

    let content = ctx.active_file_content.as_deref().unwrap_or("");
    let h = plugin.memory_alloc(content.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(content.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_plugin() {
        let manager = PluginManager::new(PathBuf::from("/tmp"));
        let plugins_dir = crate::ipc::plugins_dir();
        if let Err(e) = manager.load_from_dir(&plugins_dir) {
            println!("No plugins found or failed to load: {}", e);
            return;
        }

        if manager.get_loaded_ids().contains(&"hello".to_string()) {
            match manager.call(
                "hello",
                "hello",
                "Gemini",
                &std::collections::HashMap::new(),
            ) {
                Ok(res) => println!("Plugin response: {}", res),
                Err(e) => panic!("Plugin call failed: {}", e),
            }
        } else {
            println!("hello.wasm not found in {:?}", plugins_dir);
        }
    }
}
