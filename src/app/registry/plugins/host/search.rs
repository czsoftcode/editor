use crate::app::registry::plugins::security::HostState;
use crate::app::ui::workspace::semantic_index::SemanticIndex;
use extism::{CurrentPlugin, Val};
use std::sync::{Arc, Mutex};

pub fn host_search_project(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let _state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let _query: String = plugin.memory_get_val(&inputs[0])?;

    // TEMPORARY: Return empty list to fix compilation and allow restart.
    // Agents should use 'exec_in_sandbox' with 'rg' for now.
    let result_json = "[]";

    let h = plugin.memory_alloc(result_json.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(result_json.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_semantic_search(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let query: String = plugin.memory_get_val(&inputs[0])?;

    // Check for cancellation before expensive operation
    let semantic_index: Option<Arc<Mutex<SemanticIndex>>> = {
        let ctx = state.context.lock().expect("lock");
        if ctx.is_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(extism::Error::msg("Cancelled by user"));
        }
        ctx.semantic_index.as_ref().map(Arc::clone)
    };

    let result_json = if let Some(index_arc) = semantic_index {
        let index = index_arc.lock().unwrap();
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
