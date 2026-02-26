use crate::app::registry::plugins::security::HostState;
use extism::{CurrentPlugin, Val};

pub fn host_search_project(
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
    let ctx = state.context.lock().expect("lock");

    let result_json = if let Some(root) = &ctx.root_path {
        // PROFESSIONAL UPGRADE: Use ripgrep (rg) for extreme speed and better defaults
        let output = std::process::Command::new("rg")
            .arg("--column")
            .arg("--line-number")
            .arg("--no-heading")
            .arg("--color=never")
            .arg("--smart-case")
            .arg("-C")
            .arg("2")
            .arg("--max-columns=512")
            .arg("--max-columns-preview")
            .arg(&query)
            .current_dir(root)
            .output();

        match output {
            Ok(out) if out.status.success() || !out.stdout.is_empty() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut results = Vec::new();
                for line in text.lines().take(60) {
                    // rg format: path:line:col:content
                    let parts: Vec<&str> = line.splitn(4, ':').collect();
                    if parts.len() >= 4 {
                        results.push(serde_json::json!({
                            "file": parts[0].trim(),
                            "line": parts[1].parse::<usize>().unwrap_or(0),
                            "column": parts[2].parse::<usize>().unwrap_or(0),
                            "content": parts[3].trim()
                        }));
                    } else {
                        results.push(serde_json::json!({ "raw": line }));
                    }
                }
                serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
            }
            _ => {
                // Fallback to grep if rg is not found or fails
                let grep_output = std::process::Command::new("grep")
                    .arg("-r")
                    .arg("-n")
                    .arg("-C")
                    .arg("2")
                    .arg("--exclude-dir=target")
                    .arg("--exclude-dir=.git")
                    .arg(&query)
                    .current_dir(root)
                    .output();

                match grep_output {
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
            }
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
    let ctx = state.context.lock().expect("lock");

    let result_json = if let Some(index_arc) = &ctx.semantic_index {
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
