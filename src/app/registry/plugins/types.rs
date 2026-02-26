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

/// Information about a loaded WASM plugin.
pub struct LoadedPlugin {
    pub id: String,
    pub path: PathBuf,
    pub status: PluginStatus,
    pub metadata: Option<PluginMetadata>,
    pub wasm_bytes: Vec<u8>,
}
