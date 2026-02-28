use extism::Plugin;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "type")]
    pub plugin_type: Option<String>,
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

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct AgentMemory {
    pub facts: std::collections::HashMap<String, String>,
}

impl AgentMemory {
    pub fn memory_path() -> PathBuf {
        crate::ipc::plugins_dir()
            .parent()
            .unwrap()
            .join("agent_memory.json")
    }

    pub fn load() -> Self {
        let path = Self::memory_path();
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(path)
            && let Ok(mem) = serde_json::from_str::<AgentMemory>(&content)
        {
            return mem;
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::memory_path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
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
    pub agent_memory: Arc<Mutex<AgentMemory>>,
    /// Dočasný scratchpad — vymazán při každém novém dotazu, nikdy neperzistován.
    pub scratch: Arc<Mutex<std::collections::HashMap<String, String>>>,
    /// Expertise role of the current session — used for runtime safety enforcement.
    pub expertise_role: crate::app::ai::types::AiExpertiseRole,
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
            agent_memory: Arc::new(Mutex::new(AgentMemory::default())),
            scratch: Arc::new(Mutex::new(std::collections::HashMap::new())),
            expertise_role: crate::app::ai::types::AiExpertiseRole::Senior,
        }
    }
}

/// Information about a loaded WASM plugin.
pub struct LoadedPlugin {
    pub id: String,
    pub path: PathBuf,
    pub status: PluginStatus,
    pub metadata: Option<PluginMetadata>,
    pub wasm_bytes: Vec<u8>,
}
