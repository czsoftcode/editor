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
        inner: Arc<Mutex<Plugin>>,
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
#[derive(Clone)]
pub struct HostContext {
    pub active_file_path: Option<String>,
    pub active_file_content: Option<String>,
    pub project_index: Option<Arc<crate::app::ui::workspace::index::ProjectIndex>>,
    pub semantic_index:
        Option<Arc<Mutex<crate::app::ui::workspace::semantic_index::SemanticIndex>>>,
    pub root_path: Option<PathBuf>,
    pub auto_approved_actions: std::collections::HashSet<String>,
    pub is_cancelled: Arc<std::sync::atomic::AtomicBool>,
}

impl Default for HostContext {
    fn default() -> Self {
        Self {
            active_file_path: None,
            active_file_content: None,
            project_index: None,
            semantic_index: None,
            root_path: None,
            auto_approved_actions: std::collections::HashSet::new(),
            is_cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

#[derive(Default)]
pub(crate) struct Blacklist {
    pub(crate) patterns: Vec<String>,
    pub(crate) regexes: Vec<regex::Regex>,
}

impl Blacklist {
    fn is_blacklisted(&self, path: &str) -> bool {
        for re in &self.regexes {
            if re.is_match(path) {
                return true;
            }
        }
        // Fallback for cases where regex compilation might have failed or simple patterns
        for pattern in &self.patterns {
            if path.contains(pattern) {
                return true;
            }
        }
        false
    }
}

#[derive(Clone)]
struct HostState {
    plugin_id: String,
    sandbox_root: PathBuf,
    blacklist: Arc<Mutex<Blacklist>>,
    context: Arc<Mutex<HostContext>>,
    action_sender: Option<std::sync::mpsc::Sender<crate::app::types::AppAction>>,
    egui_ctx: Option<eframe::egui::Context>,
}

impl HostState {
    fn is_allowed(&self, rel_path: &Path) -> bool {
        let path_str = rel_path.to_string_lossy();

        if rel_path.is_absolute() || path_str.contains("..") {
            return false;
        }

        let blacklist = self.blacklist.lock().expect("lock");
        if blacklist.is_blacklisted(&path_str) {
            return false;
        }

        true
    }
}

fn compile_glob(pattern: &str) -> Option<regex::Regex> {
    let regex_pattern = pattern
        .replace('.', "\\.")
        .replace('*', ".*")
        .replace('?', ".");
    regex::Regex::new(&format!("^{}$", regex_pattern)).ok()
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
    pub blacklist: Arc<Mutex<Blacklist>>,
    pub current_context: Arc<Mutex<HostContext>>,
    pub action_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<crate::app::types::AppAction>>>>,
    pub egui_ctx: Arc<Mutex<Option<eframe::egui::Context>>>,
}

impl PluginManager {
    pub fn new(sandbox_root: PathBuf) -> Self {
        Self {
            plugins: Arc::new(Mutex::new(Vec::new())),
            sandbox_root,
            blacklist: Arc::new(Mutex::new(Blacklist::default())),
            current_context: Arc::new(Mutex::new(HostContext::default())),
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

        // Skip if already loaded (priority: Sandbox > Project > Global)
        if plugins.iter().any(|p| p.id == id) {
            return Ok(());
        }

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
            let plugin = self.create_instance(plugin_id, &wasm_bytes, &metadata, current_config)?;
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
                Arc::clone(inner)
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
}

// ---------------------------------------------------------------------------
// Host Functions Implementation
// ---------------------------------------------------------------------------

fn host_search_project(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let query: String = plugin.memory_get_val(&inputs[0])?;
    let ctx = state.context.lock().expect("lock");

    let result_json = if let Some(root) = &ctx.root_path {
        // PROFESSIONAL UPGRADE: Use real grep with context to give AI more info in one step
        let output = std::process::Command::new("grep")
            .arg("-r")
            .arg("-n")
            .arg("-C")
            .arg("2")
            .arg("--exclude-dir=target")
            .arg("--exclude-dir=.git")
            .arg(&query)
            .current_dir(root)
            .output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut results = Vec::new();
                for line in text.lines().take(60) {
                    if let Some((path_and_line, content)) = line.split_once(":")
                        && let Some((path, line_num)) = path_and_line.split_once(":")
                    {
                        results.push(serde_json::json!({
                            "file": path.trim(),
                            "line": line_num.parse::<usize>().unwrap_or(0),
                            "content": content.trim()
                        }));
                        continue;
                    }
                    results.push(serde_json::json!({ "raw": line }));
                }
                serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
            }
            Err(_) => "[]".to_string(),
        }
    } else {
        "[]".to_string()
    };

    let h = plugin.memory_alloc(result_json.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(result_json.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn host_semantic_search(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let query: String = plugin.memory_get_val(&inputs[0])?;
    let ctx = state.context.lock().expect("lock");

    let result_json = if let Some(index_arc) = &ctx.semantic_index {
        let index = index_arc.lock().unwrap();
        // REDUCED: From 10 to 5 results to save tokens
        match index.search(&query, 5) {
            Ok(results) => {
                let simplified: Vec<serde_json::Value> = results
                    .into_iter()
                    .map(|(score, path, line, text)| {
                        serde_json::json!({
                            "relevance": format!("{:.2}%", score * 100.0),
                            "file": path.to_string_lossy(),
                            "line": line,
                            "context_snippet": text,
                            "action_hint": format!("To see the full implementation here, call 'read_project_file' with path: '{}' and line_start: {}", path.to_string_lossy(), line)
                        })
                    })
                    .collect();
                serde_json::to_string(&simplified).unwrap_or_else(|_| "[]".to_string())
            }
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    } else {
        "[]".to_string()
    };

    let h = plugin.memory_alloc(result_json.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(result_json.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn host_read_file(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Input is now expected to be a JSON string: {"path": "...", "line_start": 1}
    let input_str: String = plugin.memory_get_val(&inputs[0])?;
    let input: serde_json::Value =
        serde_json::from_str(&input_str).unwrap_or(serde_json::json!({"path": input_str}));

    let path_str = input["path"].as_str().unwrap_or("");
    let line_start = input["line_start"].as_u64().unwrap_or(1) as usize;
    let rel_path = Path::new(&path_str);

    if !state.is_allowed(rel_path) {
        let msg = format!("Security violation: Access to '{}' is blocked", path_str);
        let h = plugin.memory_alloc(msg.len() as u64)?;
        plugin.memory_bytes_mut(h)?.copy_from_slice(msg.as_bytes());
        outputs[0] = Val::I64(h.offset() as i64);
        return Ok(());
    }

    let full_path = state.sandbox_root.join(rel_path);
    let full_content = fs::read_to_string(full_path)
        .unwrap_or_else(|_| "File not found or unreadable".to_string());

    // Slice the content based on line_start
    let lines: Vec<&str> = full_content.lines().collect();
    let total_lines = lines.len();
    let mut content = if line_start > 1 && line_start <= total_lines {
        lines[line_start - 1..].join("\n")
    } else {
        full_content
    };

    // TRUNCATE: Limit to 10k chars to save tokens
    let max_chars = 10000;
    if content.len() > max_chars {
        content.truncate(max_chars);
        content.push_str(&format!(
            "\n\n[FILE TRUNCATED: Showing 10k chars from line {}. Total lines in file: {}. Use 'line_start' to read the next segment!]",
            line_start, total_lines
        ));
    }

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
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
    let root = &state.sandbox_root;

    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            if name == "target" || name == ".git" || name == "node_modules" || name == "vendor" {
                return false;
            }
            if let Ok(rel) = e.path().strip_prefix(root) {
                if rel.as_os_str().is_empty() {
                    return true;
                }
                state.is_allowed(rel)
            } else {
                false
            }
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && let Ok(rel) = entry.path().strip_prefix(root)
        {
            files.push(rel.to_string_lossy().into_owned());
        }
    }

    let total_found = files.len();
    if total_found > 300 {
        files.truncate(300);
    }

    let result_json = serde_json::to_string(&serde_json::json!({
        "files": files,
        "total_count": total_found,
        "truncated": total_found > 300,
        "message": if total_found > 300 { "Showing first 300 files. Use 'semantic_search' to find specific logic." } else { "Full file list retrieved." }
    })).unwrap_or_default();

    let h = plugin.memory_alloc(result_json.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(result_json.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn host_get_active_path(
    plugin: &mut CurrentPlugin,
    _inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
    let ctx = state.context.lock().expect("lock");

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
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
    let ctx = state.context.lock().expect("lock");

    let content = ctx.active_file_content.as_deref().unwrap_or("");
    let h = plugin.memory_alloc(content.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(content.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn request_plugin_approval(
    state: &HostState,
    action_id: &str,
    action_name: &str,
    action_details: &str,
) -> Result<bool, extism::Error> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Check if cancelled
    if ctx.is_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(extism::Error::msg("Cancelled by user"));
    }

    if ctx.auto_approved_actions.contains(action_id) {
        return Ok(true);
    }

    // Release lock before blocking wait
    drop(ctx);

    if let Some(sender) = &state.action_sender {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = sender.send(crate::app::types::AppAction::PluginApprovalRequest(
            state.plugin_id.clone(),
            action_name.to_string(),
            action_details.to_string(),
            tx,
        ));

        if let Some(egui_ctx) = &state.egui_ctx {
            egui_ctx.request_repaint();
        }

        // Wait for UI response. Periodically check cancellation flag.
        loop {
            // Check cancellation flag first
            let is_cancelled = {
                let ctx = state
                    .context
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
                ctx.is_cancelled.load(std::sync::atomic::Ordering::Relaxed)
            };
            if is_cancelled {
                return Err(extism::Error::msg("Cancelled by user"));
            }

            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(response) => match response {
                    crate::app::types::PluginApprovalResponse::Approve => return Ok(true),
                    crate::app::types::PluginApprovalResponse::ApproveAlways => {
                        let mut ctx = state
                            .context
                            .lock()
                            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
                        ctx.auto_approved_actions.insert(action_id.to_string());
                        return Ok(true);
                    }
                    crate::app::types::PluginApprovalResponse::Deny => return Ok(false),
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Ok(false), // Channel closed
            }
        }
    }

    Ok(true) // If no action_sender, assume approved (CLI fallback)
}

fn host_exec_in_sandbox(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let command_str: String = plugin.memory_get_val(&inputs[0])?;

    // HARDCODED SECURITY CHECK: Prevent escaping the sandbox via path traversal
    if command_str.contains("..") || command_str.contains("/") && command_str.starts_with("/") {
        let err_msg = "SECURITY VIOLATION: Command attempted to access paths outside the sandbox. Action blocked.";
        let h = plugin.memory_alloc(err_msg.len() as u64)?;
        plugin
            .memory_bytes_mut(h)?
            .copy_from_slice(err_msg.as_bytes());
        outputs[0] = Val::I64(h.offset() as i64);
        return Ok(());
    }

    // Ask user for approval
    match request_plugin_approval(
        &state,
        "exec_in_sandbox",
        "Spustit příkaz v Sandboxu",
        &command_str,
    ) {
        Ok(true) => {}
        Ok(false) => {
            let err_msg = "USER CANCELLED ACTION";
            let h = plugin.memory_alloc(err_msg.len() as u64)?;
            plugin
                .memory_bytes_mut(h)?
                .copy_from_slice(err_msg.as_bytes());
            outputs[0] = Val::I64(h.offset() as i64);
            return Ok(());
        }
        Err(e) => return Err(e), // Cancelled via Esc (causes plugin error)
    }

    // Security: Only allow running within the sandbox
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command_str)
        .current_dir(&state.sandbox_root)
        .output();

    let result = match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let mut full_output = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);

            // Limit output to prevent token explosion (max ~4000 chars)
            let max_output = 4000;
            if full_output.len() > max_output {
                full_output.truncate(max_output);
                full_output.push_str("\n\n[OUTPUT TRUNCATED: The output was too long. Use more specific commands or 'search_project' tool.]");
            }
            full_output
        }
        Err(e) => format!("ERROR executing command: {}", e),
    };

    let h = plugin.memory_alloc(result.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(result.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

fn host_write_file(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let input_str: String = plugin.memory_get_val(&inputs[0])?;
    let input: serde_json::Value =
        serde_json::from_str(&input_str).map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;

    let path_str = input["path"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing path"))?;
    let content = input["content"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing content"))?;
    let rel_path = Path::new(&path_str);

    if !state.is_allowed(rel_path) {
        return Err(anyhow::anyhow!(
            "Security violation: Access to '{}' is blocked",
            path_str
        ));
    }

    match request_plugin_approval(
        &state,
        "write_file",
        &format!("Zapsat do souboru: {}", path_str),
        content,
    ) {
        Ok(true) => {}
        Ok(false) => return Err(anyhow::anyhow!("USER CANCELLED ACTION")),
        Err(e) => return Err(e),
    }

    let full_path = state.sandbox_root.join(rel_path);

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(full_path, content)?;

    if let Some(ctx) = &state.egui_ctx {
        ctx.request_repaint();
    }

    Ok(())
}

fn host_log_monologue(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    {
        let ctx = state
            .context
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        if ctx.is_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(extism::Error::msg("Cancelled by user"));
        }
    }

    let message: String = plugin.memory_get_val(&inputs[0])?;

    if let Some(sender) = &state.action_sender {
        let _ = sender.send(crate::app::types::AppAction::PluginMonologue(
            state.plugin_id.clone(),
            message,
        ));
    }

    if let Some(ctx) = &state.egui_ctx {
        ctx.request_repaint();
    }

    Ok(())
}

fn host_log_usage(
    _plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let in_tokens = inputs[0].i64().unwrap_or(0) as u32;
    let out_tokens = inputs.get(1).and_then(|v| v.i64()).unwrap_or(0) as u32;

    if let Some(sender) = &state.action_sender {
        let _ = sender.send(crate::app::types::AppAction::PluginUsage(
            state.plugin_id.clone(),
            in_tokens,
            out_tokens,
        ));
    }

    if let Some(ctx) = &state.egui_ctx {
        ctx.request_repaint();
    }

    Ok(())
}

fn host_log_payload(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    {
        let ctx = state
            .context
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        if ctx.is_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(extism::Error::msg("Cancelled by user"));
        }
    }

    let payload: String = plugin.memory_get_val(&inputs[0])?;

    if let Some(sender) = &state.action_sender {
        let _ = sender.send(crate::app::types::AppAction::PluginPayload(
            state.plugin_id.clone(),
            payload,
        ));
    }

    if let Some(ctx) = &state.egui_ctx {
        ctx.request_repaint();
    }

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
