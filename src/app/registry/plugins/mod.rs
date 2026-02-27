pub mod host;
pub mod security;
pub mod types;

use extism::{Function, Manifest, Plugin, UserData, ValType, Wasm};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use self::host::*;
pub use self::security::Blacklist;
use self::security::{HostState, compile_glob};
pub use self::types::{HostContext, LoadedPlugin, PluginMetadata, PluginStatus};

/// Manages loading and communication with WASM plugins.
pub struct PluginManager {
    pub plugins: Arc<Mutex<Vec<LoadedPlugin>>>,
    pub sandbox_root: PathBuf,
    pub blacklist: Arc<Mutex<Blacklist>>,
    pub current_context: Arc<Mutex<HostContext>>,
    pub action_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<crate::app::types::AppAction>>>>,
    pub egui_ctx: Arc<Mutex<Option<eframe::egui::Context>>>,
}

impl PluginManager {
    pub fn new(sandbox_root: PathBuf) -> Self {
        let initial_context = HostContext {
            agent_memory: Arc::new(Mutex::new(types::AgentMemory::load())),
            ..Default::default()
        };
        Self {
            plugins: Arc::new(Mutex::new(Vec::new())),
            sandbox_root,
            blacklist: Arc::new(Mutex::new(Blacklist::default())),
            current_context: Arc::new(Mutex::new(initial_context)),
            action_sender: Arc::new(Mutex::new(None)),
            egui_ctx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_context(&self, context: HostContext) {
        let mut ctx = self.current_context.lock().expect("lock");
        *ctx = context;
    }

    pub fn set_blacklist(&self, mut blacklist_strings: Vec<String>) {
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
                    blacklist_strings.push(line.to_string());
                }
            }
        }
        let mut b = self.blacklist.lock().expect("lock");
        b.patterns = blacklist_strings;
        b.regexes = b.patterns.iter().filter_map(|p| compile_glob(p)).collect();
    }

    pub fn load_from_dir<P: AsRef<Path>>(&self, dir_path: P) -> anyhow::Result<()> {
        let dir = dir_path.as_ref();
        if !dir.exists() {
            let _ = fs::create_dir_all(dir);
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

        // Skip if already loaded (priority: Sandbox > Project > Global)
        if plugins.iter().any(|p| p.id == id) {
            return Ok(());
        }

        if !metadata_path.exists() {
            plugins.push(LoadedPlugin {
                id,
                path,
                status: PluginStatus::Error("Missing .toml manifest".to_string()),
                metadata: None,
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
                    metadata: metadata.clone(),
                    wasm_bytes: wasm_bytes.clone(),
                },
                metadata: Some(metadata),
                wasm_bytes,
            });
        } else {
            let plugin = self.create_instance(
                &id,
                &wasm_bytes,
                &metadata,
                &std::collections::HashMap::new(),
            )?;
            plugins.push(LoadedPlugin {
                id,
                path,
                status: PluginStatus::Active {
                    inner: Arc::new(Mutex::new(plugin)),
                    config_hash: 0,
                },
                metadata: Some(metadata),
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

        let plugin = self.create_instance(plugin_id, &wasm_bytes, &metadata, plugin_config)?;
        let config_hash = self.calculate_config_hash(plugin_config);

        let mut plugins = self.plugins.lock().expect("lock");
        if let Some(p) = plugins.iter_mut().find(|p| p.id == plugin_id) {
            p.status = PluginStatus::Active {
                inner: Arc::new(Mutex::new(plugin)),
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
        plugin_id: &str,
        wasm_bytes: &[u8],
        metadata: &PluginMetadata,
        plugin_config: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<Plugin> {
        let mut manifest = Manifest::new([Wasm::data(wasm_bytes.to_vec())]);
        for host in &metadata.allowed_hosts {
            manifest = manifest.with_allowed_host(host);
        }

        // Robustness: Automatically allow the host from API_URL if provided in config
        if let Some(url_str) = plugin_config.get("API_URL")
            && let Ok(url) = url::Url::parse(url_str)
            && let Some(host) = url.host_str()
        {
            manifest = manifest.with_allowed_host(host);
        }

        manifest = manifest.with_config(plugin_config.iter());

        let host_state = HostState {
            plugin_id: plugin_id.to_string(),
            sandbox_root: self.sandbox_root.clone(),
            blacklist: Arc::clone(&self.blacklist),
            context: Arc::clone(&self.current_context),
            action_sender: self.action_sender.lock().expect("lock").clone(),
            egui_ctx: self.egui_ctx.lock().expect("lock").clone(),
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
                "write_project_file",
                [ValType::I64],
                [],
                UserData::new(host_state.clone()),
                host_write_file,
            ),
            Function::new(
                "replace_project_file",
                [ValType::I64],
                [],
                UserData::new(host_state.clone()),
                host_replace_file,
            ),
            Function::new(
                "store_scratch",
                [ValType::I64],
                [],
                UserData::new(host_state.clone()),
                host_store_scratch,
            ),
            Function::new(
                "retrieve_scratch",
                [ValType::I64],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_retrieve_scratch,
            ),
            Function::new(
                "store_fact",
                [ValType::I64],
                [],
                UserData::new(host_state.clone()),
                host_store_fact,
            ),
            Function::new(
                "retrieve_fact",
                [ValType::I64],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_retrieve_fact,
            ),
            Function::new(
                "list_project_files",
                [],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_list_files,
            ),
            Function::new(
                "search_project",
                [ValType::I64],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_search_project,
            ),
            Function::new(
                "semantic_search",
                [ValType::I64],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_semantic_search,
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
                UserData::new(host_state.clone()),
                host_get_active_content,
            ),
            Function::new(
                "exec_in_sandbox",
                [ValType::I64],
                [ValType::I64],
                UserData::new(host_state.clone()),
                host_exec_in_sandbox,
            ),
            Function::new(
                "log_monologue",
                [ValType::I64],
                [],
                UserData::new(host_state.clone()),
                host_log_monologue,
            ),
            Function::new(
                "log_usage",
                [ValType::I64, ValType::I64],
                [],
                UserData::new(host_state.clone()),
                host_log_usage,
            ),
            Function::new(
                "log_payload",
                [ValType::I64],
                [],
                UserData::new(host_state),
                host_log_payload,
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
                    if config_hash != &new_hash {
                        (p.wasm_bytes.clone(), p.metadata.clone(), true)
                    } else {
                        (Vec::new(), None, false)
                    }
                }
                _ => anyhow::bail!("Plugin {} is not active", plugin_id),
            }
        };

        if needs_reinit && let Some(meta) = metadata {
            let plugin = self.create_instance(plugin_id, &wasm_bytes, &meta, current_config)?;
            let config_hash = self.calculate_config_hash(current_config);
            let mut plugins = self.plugins.lock().expect("lock");
            if let Some(p) = plugins.iter_mut().find(|p| p.id == plugin_id) {
                p.status = PluginStatus::Active {
                    inner: Arc::new(Mutex::new(plugin)),
                    config_hash,
                };
            }
        }

        let inner = {
            let mut plugins = self.plugins.lock().expect("lock");
            let p = plugins
                .iter_mut()
                .find(|p| p.id == plugin_id)
                .ok_or_else(|| anyhow::anyhow!("Plugin not found during call execution"))?;
            if let PluginStatus::Active { inner, .. } = &p.status {
                inner.clone()
            } else {
                anyhow::bail!("Plugin {} is not active", plugin_id)
            }
        };

        let mut plugin_lock = inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Plugin mutex poisoned"))?;
        let output = plugin_lock.call::<&str, &str>(func_name, input)?;
        Ok(output.to_string())
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

    pub fn get_ai_agents(&self) -> Vec<(String, PluginMetadata)> {
        let plugins = self.plugins.lock().expect("lock");
        plugins
            .iter()
            .filter_map(|p| {
                if let Some(meta) = &p.metadata
                    && meta.plugin_type.as_deref() == Some("ai_agent")
                {
                    return Some((p.id.clone(), meta.clone()));
                }
                None
            })
            .collect()
    }
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
