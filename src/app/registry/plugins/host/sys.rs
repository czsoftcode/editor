use crate::app::registry::plugins::host::request_plugin_approval;
use crate::app::registry::plugins::security::HostState;
use extism::{CurrentPlugin, Val};

pub fn host_get_active_path(
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
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let path = ctx.active_file_path.clone().unwrap_or_default();
    let h = plugin.memory_alloc(path.len() as u64)?;
    plugin.memory_bytes_mut(h)?.copy_from_slice(path.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_get_active_content(
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
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let content = ctx.active_file_content.clone().unwrap_or_default();
    let h = plugin.memory_alloc(content.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(content.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_exec_in_sandbox(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
) -> Result<(), extism::Error> {
    let state_lock = user_data.get()?;
    let state = state_lock
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

    let command_str: String = plugin.memory_get_val(&inputs[0])?;

    // --- JUNIOR ROLE ENFORCEMENT ---
    // Block destructive commands as specified in the Junior persona mandate.
    {
        let ctx = state
            .context
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        if matches!(
            ctx.expertise_role,
            crate::app::ai::types::AiExpertiseRole::Junior
        ) {
            let lower = command_str.to_lowercase();
            let destructive_patterns = [
                "rm ",
                "rm\t",
                " rm ",
                "rmdir",
                "git reset",
                "git push",
                "git commit",
                "cargo clean",
                "truncate",
                " dd ",
            ];
            for pattern in &destructive_patterns {
                if lower.contains(pattern) {
                    let err_msg = format!(
                        "SECURITY BLOCK (Junior role): The command '{}' is destructive and is not permitted for Junior agents. Use 'ask_user' to request human approval before attempting destructive operations.",
                        command_str
                    );
                    let h = plugin.memory_alloc(err_msg.len() as u64)?;
                    plugin
                        .memory_bytes_mut(h)?
                        .copy_from_slice(err_msg.as_bytes());
                    outputs[0] = Val::I64(h.offset() as i64);
                    return Ok(());
                }
            }
        }
    }

    // --- HOST OBSERVATIONS ---
    let cmd_lower = command_str.to_lowercase();
    let is_discovery = cmd_lower.contains("rg ")
        || cmd_lower.contains("grep ")
        || cmd_lower.contains("curl ")
        || cmd_lower.contains("wget ")
        || cmd_lower.contains("find ")
        || cmd_lower.contains("ls ")
        || cmd_lower.contains("cat ")
        || cmd_lower.contains("head ");

    let mut advice = String::new();

    // Monitor Discovery Chain (Soft Nudge)
    {
        let mut chain = state.search_chain_count.lock().expect("lock");
        if is_discovery {
            *chain += 1;
            if *chain > 8 {
                advice.push_str("\nHOST OBSERVATION: You have performed multiple discovery actions. Remember your 'MISSION STATUS' and 'REFLECTION' protocols. Are you moving towards the goal?");
            }
        }
    }

    // Basic Path Security
    if command_str.contains("..") {
        let err_msg =
            "SECURITY VIOLATION: Path traversal (..) is blocked. Use project-relative paths.";
        let h = plugin.memory_alloc(err_msg.len() as u64)?;
        plugin
            .memory_bytes_mut(h)?
            .copy_from_slice(err_msg.as_bytes());
        outputs[0] = Val::I64(h.offset() as i64);
        return Ok(());
    }

    let is_absolute = command_str.starts_with("/");
    let is_allowed_tmp = command_str.starts_with("/tmp/") || command_str.starts_with("/var/tmp/");
    if is_absolute && !is_allowed_tmp {
        let err_msg = "SECURITY VIOLATION: Absolute system paths are blocked. Use project-relative paths or '/tmp/'. You do NOT have a persistent home directory.";
        let h = plugin.memory_alloc(err_msg.len() as u64)?;
        plugin
            .memory_bytes_mut(h)?
            .copy_from_slice(err_msg.as_bytes());
        outputs[0] = Val::I64(h.offset() as i64);
        return Ok(());
    }

    // Store in history
    state
        .command_history
        .lock()
        .expect("lock")
        .push(command_str.clone());

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
        Err(e) => return Err(e),
    }

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

            // --- INTELLIGENT ERROR COACHING ---
            if !out.status.success() {
                if full_output.contains("none of the selected features exist")
                    || full_output.contains("feature not found")
                {
                    advice.push_str("\nMETHODOLOGICAL HINT: The compiler cannot find a feature you requested. This usually means the crate version in Cargo.toml is too old. DO NOT search the web yet. Instead, check the latest available version of the crate on crates.io (or via 'cargo search') and update Cargo.toml.");
                } else if full_output.contains("no method named")
                    || full_output.contains("cannot find type")
                {
                    advice.push_str("\nMETHODOLOGICAL HINT: API mismatch detected. You might be using documentation for a different version than what is in Cargo.toml. Compare your local code with the crate version defined in Cargo.toml.");
                }
            }

            let max_output = 4000;
            if full_output.len() > max_output {
                full_output.truncate(max_output);
                full_output.push_str(
                    "\n\n[OUTPUT TRUNCATED: The output was too long. Use more specific commands or 'search_project' tool.]",
                );
            }

            // Append host guidance if generated
            if !advice.is_empty() {
                full_output.push_str("\n\n--- HOST GUIDANCE ---\n");
                full_output.push_str(&advice);
            }

            full_output
        }
        Err(e) => format!("ERROR executing command: {}\n{}", e, advice),
    };

    let h = plugin.memory_alloc(result.len() as u64)?;
    plugin
        .memory_bytes_mut(h)?
        .copy_from_slice(result.as_bytes());
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_ask_user(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
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

    let input_str: String = plugin.memory_get_val(&inputs[0])?;
    let input: serde_json::Value =
        serde_json::from_str(&input_str).unwrap_or(serde_json::json!({"question": input_str}));

    let question = input["question"].as_str().unwrap_or("?").to_string();
    let options: Vec<String> = input["options"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    if let Some(sender) = &state.action_sender {
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let _ = sender.send(crate::app::types::AppAction::PluginAskUser(
            state.plugin_id.clone(),
            question,
            options,
            tx,
        ));

        if let Some(egui_ctx) = &state.egui_ctx {
            egui_ctx.request_repaint();
        }

        loop {
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
                Ok(answer) => {
                    let result = answer.clone();
                    let h = plugin.memory_alloc(result.len() as u64)?;
                    plugin
                        .memory_bytes_mut(h)?
                        .copy_from_slice(result.as_bytes());
                    outputs[0] = Val::I64(h.offset() as i64);
                    return Ok(());
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => break,
            }
        }
    }

    // Fallback if no action_sender (CLI mode): return empty string
    let h = plugin.memory_alloc(0)?;
    outputs[0] = Val::I64(h.offset() as i64);
    Ok(())
}

pub fn host_announce_completion(
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
        serde_json::from_str(&input_str).unwrap_or(serde_json::json!({"summary": input_str}));

    let summary = input["summary"]
        .as_str()
        .unwrap_or("Task completed.")
        .to_string();

    if let Some(sender) = &state.action_sender {
        let _ = sender.send(crate::app::types::AppAction::PluginCompleted(
            state.plugin_id.clone(),
            summary,
        ));
    }

    if let Some(egui_ctx) = &state.egui_ctx {
        egui_ctx.request_repaint();
    }

    Ok(())
}

pub fn host_log_monologue(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
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

pub fn host_log_usage(
    _plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
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

pub fn host_log_payload(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    user_data: extism::UserData<HostState>,
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
