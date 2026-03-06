use crate::app::ai::AiManager;
use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use std::sync::{Arc, Mutex};

pub fn send_query_to_agent(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    if ws.ai.chat.prompt.trim().is_empty() {
        return;
    }

    let prompt = ws.ai.chat.prompt.clone();
    let context = AiManager::generate_context(ws, shared);
    let tools = crate::app::ai::get_standard_tools();

    let input = serde_json::json!({
        "prompt": prompt,
        "history": ws.ai.chat.conversation,
        "context": context,
        "tools": tools
    });

    ws.ai.chat.conversation.push((prompt.clone(), String::new()));
    ws.ai.chat.prompt.clear();
    ws.ai.chat.loading = true;
    ws.ai.chat.monologue.clear();
    ws.ai.cancellation_token = Arc::new(std::sync::atomic::AtomicBool::new(false));

    if ws.ai.chat.history.last() != Some(&prompt) {
        ws.ai.chat.history.push(prompt);
    }
    ws.ai.chat.history_index = None;

    let sh_arc = Arc::clone(shared);
    let provider = ws.ai.settings.selected_provider.clone();
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
            .get(&provider)
            .map(|s| s.config.clone())
            .unwrap_or_default();
        (
            Arc::clone(&sh.registry.plugins),
            config,
            ws.ai.settings.expertise,
            ws.ai.settings.reasoning_depth,
            ws.ai.chat.system_prompt.clone(),
            ws.ai.settings.language.clone(),
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

    let agent_memory = if let Ok(ctx) = plugin_manager.current_context.try_lock() {
        ctx.agent_memory.clone()
    } else {
        Arc::new(Mutex::new(
            crate::app::registry::plugins::types::AgentMemory::default(),
        ))
    };

    let scratch = Arc::new(Mutex::new(std::collections::HashMap::new()));

    plugin_manager.set_context(crate::app::registry::plugins::HostContext {
        active_file_path: active_path,
        active_file_content: active_content,
        project_index: Some(Arc::clone(&ws.project_index)),
        semantic_index: Some(Arc::clone(&ws.semantic_index)),
        root_path: Some(ws.root_path.clone()),
        auto_approved_actions: std::collections::HashSet::new(),
        is_cancelled: Arc::clone(&ws.ai.cancellation_token),
        agent_memory,
        scratch,
        expertise_role: ws.ai.settings.expertise,
    });

    std::thread::spawn(move || {
        let lang_name = crate::i18n::lang_display_name(&lang);
        let intelligence = AiManager::get_system_mandates(expertise, depth, lang_name);
        let mut final_config = config;

        let effective_lang = final_config.get("LANGUAGE").cloned().unwrap_or(lang);
        final_config.insert(
            "SYSTEM_PROMPT".to_string(),
            format!("{}\n        \n        {}", intelligence, sys_prompt),
        );
        final_config.insert("LANGUAGE".to_string(), effective_lang);

        let input_str = serde_json::to_string(&input).unwrap_or_default();
        let func_name = format!("ask_{}", provider);
        let result = plugin_manager.call(&provider, &func_name, &input_str, &final_config);

        let mut sh = sh_arc.lock().expect("lock");
        sh.actions
            .push(crate::app::types::AppAction::PluginResponse(
                provider,
                result.map_err(|e| e.to_string()),
            ));
    });
}
