use crate::app::registry::plugins::host::request_plugin_approval;
use crate::app::registry::plugins::security::HostState;
use extism::{CurrentPlugin, Val};
use std::fs;
use std::path::Path;

pub fn host_read_file(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;

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

    let lines: Vec<&str> = full_content.lines().collect();
    let total_lines = lines.len();
    let line_end = input["line_end"].as_u64().map(|v| v as usize);
    let content = if line_start > 1 || line_end.is_some() {
        let start = if line_start > 1 { (line_start - 1).min(total_lines) } else { 0 };
        let end = line_end.map(|e| e.min(total_lines)).unwrap_or(total_lines);
        lines[start..end].join("\n")
    } else {
        full_content
    };

    let max_chars = input["max_chars_limit"]
        .as_u64()
        .map(|v| v as usize)
        .unwrap_or(10000);

    let content = if content.len() > max_chars {
        let mut new_len = max_chars;
        while new_len > 0 && !content.is_char_boundary(new_len) {
            new_len -= 1;
        }
        let mut truncated = content[..new_len].to_string();
        truncated.push_str(&format!(
            "\n\n[FILE TRUNCATED: Showing {} chars from line {}. Total lines in file: {}. Use 'line_start' to read the next segment!]",
            max_chars, line_start, total_lines
        ));
        truncated
    } else {
        content.to_string()
    };

    let h = plugin.memory_alloc(content.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(content.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_write_file(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;

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

    let is_trace_log = path_str == ".gemini_trace.log"
        || path_str == ".ollama_trace.log"
        || path_str == "ollama_trace.log"
        || path_str == ".ai_trace.log";

    if !is_trace_log && !state.is_allowed(rel_path) {
        eprintln!("SECURITY VIOLATION in plugin: {}", path_str);
        return Ok(());
    }

    let needs_approval = !is_trace_log && !path_str.ends_with(".log");

    if needs_approval {
        match request_plugin_approval(
            &state,
            "write_file",
            &format!("Zapsat do souboru: {}", path_str),
            content,
        ) {
            Ok(true) => {}
            Ok(false) | Err(_) => return Ok(()),
        }
    }

    let full_path = state.sandbox_root.join(rel_path);

    if let Some(parent) = full_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(e) = fs::write(&full_path, content) {
        eprintln!("Failed to write file from plugin: {}", e);
    }

    if let Some(ctx) = &state.egui_ctx {
        ctx.request_repaint();
    }

    Ok(())
}

pub fn host_replace_file(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;

    let input_str: String = plugin.memory_get_val(&inputs[0])?;
    let input: serde_json::Value =
        serde_json::from_str(&input_str).map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;

    let path_str = input["path"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing path"))?;
    let old_string = input["old_string"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing old_string"))?;
    let new_string = input["new_string"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing new_string"))?;
    let rel_path = Path::new(&path_str);

    if !state.is_allowed(rel_path) {
        eprintln!("SECURITY VIOLATION in plugin: {}", path_str);
        return Ok(());
    }

    let full_path = state.sandbox_root.join(rel_path);
    let current_content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Replace failed: could not read {}: {}", path_str, e);
            return Ok(());
        }
    };

    // 1. Try exact match, then fuzzy match (whitespace tolerant)
    let match_range = if let Some(pos) = current_content.find(old_string) {
        Some(pos..pos + old_string.len())
    } else {
        find_fuzzy_match(&current_content, old_string)
    };

    let Some(range) = match_range else {
        eprintln!(
            "Replace failed: 'old_string' not found (even with fuzzy matching) in {}",
            path_str
        );
        return Ok(());
    };

    let matched_text = &current_content[range.clone()];

    let mut diff_display = format!(
        "### {}
```diff
",
        path_str
    );
    let start_line = current_content[..range.start].lines().count() + 1;

    let lines_before: Vec<&str> = current_content[..range.start]
        .lines()
        .rev()
        .take(3)
        .collect();
    let lines_after: Vec<&str> = current_content[range.end..].lines().take(3).collect();

    for (i, line) in lines_before.into_iter().rev().enumerate() {
        let num = start_line.saturating_sub(3).saturating_add(i);
        if num < start_line {
            diff_display.push_str(&format!(
                "{:4}   {}
",
                num, line
            ));
        }
    }

    let diff = similar::TextDiff::from_lines(matched_text, new_string);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };

        let line_num = match change.tag() {
            similar::ChangeTag::Delete | similar::ChangeTag::Equal => {
                format!("{:4}", start_line + change.old_index().unwrap_or(0))
            }
            similar::ChangeTag::Insert => {
                format!("{:4}", start_line + change.new_index().unwrap_or(0))
            }
        };

        diff_display.push_str(&format!("{} {} {}", line_num, sign, change));
        if !change.value().ends_with('\n') {
            diff_display.push('\n');
        }
    }

    let final_start = start_line + matched_text.lines().count();
    for (i, line) in lines_after.into_iter().enumerate() {
        diff_display.push_str(&format!("{:4}   {}\n", final_start + i, line));
    }

    diff_display.push_str("```");
    match request_plugin_approval(
        &state,
        "replace",
        &format!("Upravit kód v: {}", path_str),
        &diff_display,
    ) {
        Ok(true) => {}
        Ok(false) | Err(_) => return Ok(()),
    }

    let mut new_content = current_content[..range.start].to_string();
    new_content.push_str(new_string);
    new_content.push_str(&current_content[range.end..]);

    if let Err(e) = fs::write(&full_path, new_content) {
        eprintln!("Failed to write replacement to {}: {}", path_str, e);
    }

    if let Some(ctx) = &state.egui_ctx {
        ctx.request_repaint();
    }

    Ok(())
}

pub fn host_store_scratch(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;

    let input_str: String = plugin.memory_get_val(&inputs[0])?;
    let input: serde_json::Value =
        serde_json::from_str(&input_str).map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;

    let key = input["key"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing key"))?;
    let value = input["value"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing value"))?;

    let ctx = state
        .context
        .lock()
        .map_err(|_| anyhow::anyhow!("Context lock poisoned"))?;
    let mut scratch = ctx
        .scratch
        .lock()
        .map_err(|_| anyhow::anyhow!("Scratch lock poisoned"))?;

    scratch.insert(key.to_string(), value.to_string());

    Ok(())
}

pub fn host_retrieve_scratch(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;

    let key: String = plugin.memory_get_val(&inputs[0])?;

    let ctx = state
        .context
        .lock()
        .map_err(|_| anyhow::anyhow!("Context lock poisoned"))?;
    let scratch = ctx
        .scratch
        .lock()
        .map_err(|_| anyhow::anyhow!("Scratch lock poisoned"))?;

    let value = scratch.get(&key).cloned().unwrap_or_default();

    let h = plugin.memory_alloc(value.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(value.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_store_fact(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;

    let input_str: String = plugin.memory_get_val(&inputs[0])?;
    let input: serde_json::Value =
        serde_json::from_str(&input_str).map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;

    let key = input["key"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing key"))?;
    let value = input["value"]
        .as_str()
        .ok_or(anyhow::anyhow!("Missing value"))?;

    let ctx = state
        .context
        .lock()
        .map_err(|_| anyhow::anyhow!("Context lock poisoned"))?;
    let mut memory = ctx
        .agent_memory
        .lock()
        .map_err(|_| anyhow::anyhow!("Memory lock poisoned"))?;

    memory.facts.insert(key.to_string(), value.to_string());
    if let Err(e) = memory.save() {
        eprintln!("Failed to save agent memory to disk: {}", e);
    } else {
        // Log to agent monologue so the user sees progress
        if let Some(sender) = &state.action_sender {
            let _ = sender.send(crate::app::types::AppAction::PluginMonologue(
                state.plugin_id.clone(),
                format!("💾 Fact stored in long-term memory: {} = {}", key, value),
            ));
        }
    }

    Ok(())
}

pub fn host_retrieve_fact(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;

    let key: String = plugin.memory_get_val(&inputs[0])?;

    let ctx = state
        .context
        .lock()
        .map_err(|_| anyhow::anyhow!("Context lock poisoned"))?;
    let memory = ctx
        .agent_memory
        .lock()
        .map_err(|_| anyhow::anyhow!("Memory lock poisoned"))?;

    let value = memory.facts.get(&key).cloned().unwrap_or_default();

    let h = plugin.memory_alloc(value.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(value.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_list_files(
    plugin: &mut CurrentPlugin,
    _inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    // Reset search chain count on actual work
    *state.search_chain_count.lock().expect("lock") = 0;
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

pub fn host_list_facts(
    plugin: &mut CurrentPlugin,
    _inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let ctx = state
        .context
        .lock()
        .map_err(|_| anyhow::anyhow!("Context lock poisoned"))?;
    let memory = ctx
        .agent_memory
        .lock()
        .map_err(|_| anyhow::anyhow!("Memory lock poisoned"))?;

    let keys: Vec<String> = memory.facts.keys().cloned().collect();
    let result = serde_json::to_string(&serde_json::json!({ "keys": keys })).unwrap_or_default();

    let h = plugin.memory_alloc(result.len() as u64)?;
    plugin.memory_bytes_mut(h)?.copy_from_slice(result.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_delete_fact(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let key: String = plugin.memory_get_val(&inputs[0])?;

    let ctx = state
        .context
        .lock()
        .map_err(|_| anyhow::anyhow!("Context lock poisoned"))?;
    let mut memory = ctx
        .agent_memory
        .lock()
        .map_err(|_| anyhow::anyhow!("Memory lock poisoned"))?;

    memory.facts.remove(&key);
    if let Err(e) = memory.save() {
        eprintln!("Failed to save agent memory after delete: {}", e);
    }

    Ok(())
}

/// Normalizes a string for fuzzy comparison (removes all non-essential whitespace and empty lines)
fn normalize_for_fuzzy(s: &str) -> String {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Attempts to find a block in content that matches old_string sémantically.
/// This version is even more robust, ignoring empty lines in the search.
fn find_fuzzy_match(content: &str, old_string: &str) -> Option<std::ops::Range<usize>> {
    let normalized_old = normalize_for_fuzzy(old_string);
    if normalized_old.is_empty() {
        return None;
    }

    let old_lines: Vec<&str> = normalized_old.lines().collect();

    // We search through the content lines, but we must skip empty lines
    // while keeping track of original byte positions.
    let content_lines: Vec<(usize, &str)> = content
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| (i, l.trim()))
        .collect();

    if content_lines.len() < old_lines.len() {
        return None;
    }

    for i in 0..=content_lines.len() - old_lines.len() {
        let mut match_found = true;
        for j in 0..old_lines.len() {
            if content_lines[i + j].1 != old_lines[j] {
                match_found = false;
                break;
            }
        }

        if match_found {
            // Found a match! Map back to byte offsets.
            let mut line_byte_offsets = Vec::new();
            let mut offset = 0;
            for line in content.lines() {
                line_byte_offsets.push(offset);
                offset += line.len() + 1;
            }

            let start_line_idx = content_lines[i].0;
            let end_line_idx = content_lines[i + old_lines.len() - 1].0;

            if start_line_idx < line_byte_offsets.len() {
                let start_byte = line_byte_offsets[start_line_idx];
                let end_byte = if end_line_idx + 1 < line_byte_offsets.len() {
                    line_byte_offsets[end_line_idx + 1]
                } else {
                    content.len()
                };
                return Some(start_byte..end_byte);
            }
        }
    }

    None
}
