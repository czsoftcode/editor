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
    let mut content = if line_start > 1 && line_start <= total_lines {
        lines[line_start - 1..].join(
            "
",
        )
    } else {
        full_content
    };

    let max_chars = 10000;
    if content.len() > max_chars {
        content.truncate(max_chars);
        content.push_str(&format!(
            "

[FILE TRUNCATED: Showing 10k chars from line {}. Total lines in file: {}. Use 'line_start' to read the next segment!]",
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
        eprintln!("SECURITY VIOLATION in plugin: {}", path_str);
        return Ok(());
    }

    let needs_approval = path_str != ".gemini_trace.log" && !path_str.ends_with(".log");

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

    if let Err(e) = fs::write(full_path, content) {
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

    if !current_content.contains(old_string) {
        eprintln!("Replace failed: 'old_string' not found in {}", path_str);
        return Ok(());
    }

    let mut diff_display = format!(
        "### {}
```diff
",
        path_str
    );
    let byte_pos = current_content.find(old_string).unwrap_or(0);
    let start_line = current_content[..byte_pos].lines().count() + 1;

    let lines_before: Vec<&str> = current_content[..byte_pos].lines().rev().take(3).collect();
    let lines_after: Vec<&str> = current_content[byte_pos + old_string.len()..]
        .lines()
        .take(3)
        .collect();

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

    let diff = similar::TextDiff::from_lines(old_string, new_string);
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

    let final_start = start_line + old_string.lines().count();
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

    let new_content = current_content.replace(old_string, new_string);
    if let Err(e) = fs::write(full_path, new_content) {
        eprintln!("Failed to write replacement to {}: {}", path_str, e);
    }

    if let Some(ctx) = &state.egui_ctx {
        ctx.request_repaint();
    }

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
