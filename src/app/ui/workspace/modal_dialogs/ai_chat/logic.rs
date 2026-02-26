use crate::app::ai::AiManager;
use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use std::sync::{Arc, Mutex};

pub fn send_query_to_agent(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    if ws.gemini_prompt.trim().is_empty() {
        return;
    }

    let prompt = ws.gemini_prompt.clone();
    let context = AiManager::generate_context(ws);
    let tools = crate::app::ai::get_standard_tools();

    let input = serde_json::json!({
        "prompt": prompt,
        "history": ws.gemini_conversation,
        "context": context,
        "tools": tools
    });

    ws.gemini_conversation.push((prompt.clone(), String::new()));
    ws.gemini_prompt.clear();
    ws.gemini_loading = true;
    ws.gemini_monologue.clear();
    ws.gemini_cancellation_token = Arc::new(std::sync::atomic::AtomicBool::new(false));

    if ws.gemini_history.last() != Some(&prompt) {
        ws.gemini_history.push(prompt);
    }
    ws.gemini_history_index = None;

    let sh_arc = Arc::clone(shared);
    let (plugin_manager, config, expertise, depth, sys_prompt, lang): (
        Arc<crate::app::registry::plugins::PluginManager>,
        _,
        _,
        _,
        _,
        _,
    ) = {
        let sh = sh_arc.lock().expect("lock");
        let config = sh
            .settings
            .plugins
            .get("gemini")
            .map(|s| s.config.clone())
            .unwrap_or_default();
        (
            Arc::clone(&sh.registry.plugins),
            config,
            ws.gemini_expertise,
            ws.gemini_reasoning_depth,
            ws.gemini_system_prompt.clone(),
            ws.gemini_language.clone(),
        )
    };

    let active_path = ws.editor.active_path().map(|p| {
        p.strip_prefix(&ws.root_path)
            .unwrap_or(p)
            .to_string_lossy()
            .into_owned()
    });
    let active_content = ws
        .editor
        .active_tab
        .and_then(|idx| ws.editor.tabs.get(idx))
        .map(|t| t.content.clone());

    plugin_manager.set_context(crate::app::registry::plugins::HostContext {
        active_file_path: active_path,
        active_file_content: active_content,
        project_index: Some(Arc::clone(&ws.project_index)),
        semantic_index: Some(Arc::clone(&ws.semantic_index)),
        root_path: Some(ws.root_path.clone()),
        auto_approved_actions: std::collections::HashSet::new(),
        is_cancelled: Arc::clone(&ws.gemini_cancellation_token),
    });

    std::thread::spawn(move || {
        let lang_name = crate::i18n::lang_display_name(&lang);
        let intelligence = AiManager::get_system_mandates(expertise, depth, lang_name);
        let mut final_config = config;
        final_config.insert(
            "SYSTEM_PROMPT".to_string(),
            format!(
                "{}

{}",
                intelligence, sys_prompt
            ),
        );
        final_config.insert("LANGUAGE".to_string(), lang_name.to_string());

        let input_str = serde_json::to_string(&input).unwrap_or_default();
        let result = plugin_manager.call("gemini", "ask_gemini", &input_str, &final_config);

        let mut sh = sh_arc.lock().expect("lock");
        sh.actions
            .push(crate::app::types::AppAction::PluginResponse(
                "gemini".to_string(),
                result.map_err(|e| e.to_string()),
            ));
    });
}
