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
    let ctx = state.context.lock().expect("lock");

    let path = ctx.active_file_path.as_deref().unwrap_or("");
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
    let ctx = state.context.lock().expect("lock");

    let content = ctx.active_file_content.as_deref().unwrap_or("");
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

    if command_str.contains("..") || command_str.contains("/") && command_str.starts_with("/") {
        let err_msg = "SECURITY VIOLATION: Command attempted to access paths outside the sandbox. Action blocked.";
        let h = plugin.memory_alloc(err_msg.len() as u64)?;
        plugin
            .memory_bytes_mut(h)?
            .copy_from_slice(err_msg.as_bytes());
        outputs[0] = Val::I64(h.offset() as i64);
        return Ok(());
    }

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
            let mut full_output = format!(
                "STDOUT:
{}
STDERR:
{}",
                stdout, stderr
            );

            let max_output = 4000;
            if full_output.len() > max_output {
                full_output.truncate(max_output);
                full_output.push_str(
                    "

[OUTPUT TRUNCATED: The output was too long. Use more specific commands or 'search_project' tool.]",
                );
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
