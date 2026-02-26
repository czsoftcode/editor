pub mod fs;
pub mod search;
pub mod sys;

pub use fs::*;
pub use search::*;
pub use sys::*;

use crate::app::registry::plugins::security::HostState;

pub fn request_plugin_approval(
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
