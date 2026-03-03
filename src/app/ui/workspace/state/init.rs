use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::{ProjectSearch, WorkspaceState};
use crate::app::project_config::load_profiles;
use crate::app::types::{FocusedPanel, PersistentState};
use crate::app::ui::background::{fetch_git_branch, fetch_git_status};
use crate::app::ui::editor::Editor;
use crate::app::ui::file_tree::FileTree;
use crate::watcher::{FileWatcher, ProjectWatcher};

pub fn init_workspace(
    root_path: PathBuf,
    panel_state: &PersistentState,
    egui_ctx: egui::Context,
    settings: &crate::settings::Settings,
    shared: Arc<Mutex<crate::app::types::AppShared>>,
) -> WorkspaceState {
    let sandbox = crate::app::sandbox::Sandbox::new(&root_path);
    let mut file_tree = FileTree::new();
    let file_tree_in_sandbox = settings.project_read_only;
    let target_tree_root = if file_tree_in_sandbox {
        &sandbox.root
    } else {
        &root_path
    };
    file_tree.load(target_tree_root);

    let mut project_watcher = ProjectWatcher::new(&root_path);
    project_watcher.add_path(&sandbox.root);
    let sandbox_staged_files = sandbox.get_staged_files();

    let git_cancel = Arc::new(AtomicBool::new(false));
    let git_branch_rx = fetch_git_branch(&root_path, Arc::clone(&git_cancel));
    let git_status_rx = fetch_git_status(&root_path, Arc::clone(&git_cancel));

    let project_index = Arc::new(crate::app::ui::workspace::index::ProjectIndex::new(
        root_path.clone(),
    ));
    let semantic_index = Arc::new(Mutex::new(
        crate::app::ui::workspace::semantic_index::SemanticIndex::new(root_path.clone()),
    ));
    // Load existing index from cache
    if let Ok(si) = semantic_index.lock() {
        let _ = si.load();
    }
    project_index.full_rescan();

    // Start semantic index initialization only if empty OR explicitly requested (Audit S-5)
    let is_empty = if let Ok(si) = semantic_index.lock() {
        si.snippets.lock().unwrap().is_empty()
    } else {
        true
    };

    if is_empty {
        spawn_semantic_indexer(
            Arc::clone(&semantic_index),
            root_path.clone(),
            egui_ctx.clone(),
            settings.blacklist.clone(),
            shared,
        );
    }

    let i18n = crate::i18n::I18n::new(&settings.lang);
    let profiles = load_profiles(&root_path);

    let selected_provider = panel_state
        .ai_selected_provider
        .clone()
        .unwrap_or_else(|| "gemini".to_string());
    let ai_settings = settings.plugins.get(&selected_provider);

    WorkspaceState {
        file_tree,
        editor: Editor::new(),
        watcher: FileWatcher::new(),
        project_watcher,
        claude_tabs: Vec::new(),
        claude_active_tab: 0,
        next_claude_tab_id: 1,
        next_terminal_id: 2,
        build_terminal: None,
        focused_panel: FocusedPanel::Editor,
        root_path: root_path.clone(),
        show_left_panel: panel_state.show_left_panel,
        show_right_panel: panel_state.show_right_panel,
        show_build_terminal: panel_state.show_build_terminal,
        build_terminal_float: false,
        left_panel_split: 0.55,
        show_about: false,
        show_support: false,
        show_settings: false,
        show_plugins: false,
        show_ai_chat: false,
        show_semantic_indexing_modal: true,
        ai_show_settings: false,
        selected_plugin_id: None,
        selected_settings_category: None,
        ai_font_scale: panel_state.ai_font_scale,
        profiles,
        build_errors: Vec::new(),
        build_error_rx: None,
        selected_agent_id: settings
            .custom_agents
            .first()
            .map(|a| a.name.to_lowercase().replace(' ', "_"))
            .unwrap_or_default(),
        ai_selected_provider: selected_provider.clone(),
        claude_float: panel_state.claude_float,
        show_new_project: false,
        wizard: crate::app::ui::dialogs::WizardState::default(),
        toasts: Vec::new(),
        folder_pick_rx: None,
        command_palette: None,
        project_index,
        semantic_index,
        file_picker: None,
        project_search: ProjectSearch::default(),
        lsp_client: None,
        lsp_binary_missing: false,
        lsp_install_rx: None,
        git_branch: None,
        git_branch_rx: Some(git_branch_rx),
        git_status_rx: Some(git_status_rx),
        git_last_refresh: std::time::Instant::now(),
        lsp_last_retry: std::time::Instant::now(),
        settings_draft: None,
        plugins_draft: None,
        settings_folder_pick_rx: None,
        ai_tool_available: HashMap::new(),
        ai_tool_check_rx: None,
        ai_tool_last_check: std::time::Instant::now(),
        win_tool_available: HashMap::new(),
        win_tool_check_rx: Some(crate::app::ui::workspace::state::actions::spawn_win_tool_check()),
        win_tool_last_check: std::time::Instant::now(),
        external_change_conflict: None,
        dep_wizard: crate::app::ui::dialogs::DependencyWizard::new(),
        build_all_modal: crate::app::ui::workspace::build_all_modal::BuildAllModal::new(),
        sandbox_deletion_sync: None,
        terminal_close_requested: None,
        ai_viewport_open: false,
        promotion_success: None,
        show_sandbox_staged: false,
        plugin_error: None,
        ai_prompt: String::new(),
        ai_history: Vec::new(),
        ai_history_index: None,
        ai_monologue: Vec::new(),
        ai_conversation: vec![(
            String::new(),
            crate::app::ai::AiManager::get_logo(
                crate::config::CLI_VERSION,
                &ai_settings
                    .and_then(|s| s.config.get("MODEL").cloned())
                    .unwrap_or_else(|| {
                        if selected_provider == "ollama" {
                            "llama3.1".to_string()
                        } else {
                            "gemini-1.5-flash".to_string()
                        }
                    }),
                panel_state
                    .ai_expertise
                    .unwrap_or_else(|| ai_settings.map(|s| s.expertise).unwrap_or_default()),
                panel_state
                    .ai_reasoning_depth
                    .unwrap_or_else(|| ai_settings.map(|s| s.reasoning_depth).unwrap_or_default()),
            ),
        )],
        ai_system_prompt: panel_state.ai_system_prompt.clone().unwrap_or_else(|| {
            ai_settings
                .and_then(|s| s.config.get("SYSTEM_PROMPT").cloned())
                .unwrap_or_else(|| i18n.get("ai-chat-default-prompt"))
        }),
        ai_language: panel_state.ai_language.clone().unwrap_or_else(|| {
            ai_settings
                .and_then(|s| s.config.get("LANGUAGE").cloned())
                .unwrap_or_else(|| i18n.lang().to_string())
        }),
        ai_expertise: panel_state
            .ai_expertise
            .unwrap_or_else(|| ai_settings.map(|s| s.expertise).unwrap_or_default()),
        ai_reasoning_depth: panel_state
            .ai_reasoning_depth
            .unwrap_or_else(|| ai_settings.map(|s| s.reasoning_depth).unwrap_or_default()),
        ai_in_tokens: 0,
        ai_out_tokens: 0,
        ai_inspector_open: false,
        ai_focus_requested: true,
        ai_last_payload: String::new(),
        ai_response: None,
        ai_loading: false,
        markdown_cache: egui_commonmark::CommonMarkCache::default(),
        sync_confirmation: None,
        pending_agent_id: None,
        build_in_sandbox: settings.project_read_only,
        file_tree_in_sandbox,
        git_cancel,
        local_history: crate::app::local_history::LocalHistory::new(&root_path),
        sandbox,
        sandbox_staged_files,
        sandbox_staged_rx: None,
        sandbox_staged_dirty: false,
        sandbox_staged_last_dirty: std::time::Instant::now(),
        sandbox_staged_last_refresh: std::time::Instant::now(),
        background_io_rx: None,
        applied_settings_version: 0,
        pending_plugin_approval: None,
        pending_ask_user: None,
        ai_cancellation_token: Arc::new(AtomicBool::new(false)),
    }
}

fn spawn_semantic_indexer(
    si_arc: Arc<Mutex<crate::app::ui::workspace::semantic_index::SemanticIndex>>,
    root_path: PathBuf,
    ctx: egui::Context,
    blacklist: Vec<String>,
    shared: Arc<Mutex<crate::app::types::AppShared>>,
) {
    let thread_root = root_path.clone();
    let thread_sandbox_root = thread_root.join(".polycredo").join("sandbox");

    // Load ignore patterns from .gitignore
    let mut blacklist_strings = blacklist;
    let gitignore_path = thread_root.join(".gitignore");
    if let Ok(content) = std::fs::read_to_string(gitignore_path) {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                if line.contains(".polycredo") || line.contains("sandbox") {
                    continue;
                }
                blacklist_strings.push(line.to_string());
            }
        }
    }

    // Compile regexes
    let mut blacklist_regexes = Vec::new();
    for pattern in &blacklist_strings {
        let regex_pattern = pattern
            .replace('.', r"\.")
            .replace('*', ".*")
            .replace('?', ".");
        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            blacklist_regexes.push(re);
        }
    }

    std::thread::spawn(move || {
        println!("[SemanticIndex] Thread started. Virtual Root: Sandbox.");

        {
            let si = si_arc.lock().unwrap();
            si.is_indexing.store(true, Ordering::SeqCst);
            si.stop_requested.store(false, Ordering::SeqCst);
            if let Err(e) = si.load() {
                eprintln!("[SemanticIndex] Cache load failed: {}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut files = Vec::new();
        if thread_sandbox_root.exists() {
            for entry in walkdir::WalkDir::new(&thread_sandbox_root)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }

                if let Ok(rel) = entry.path().strip_prefix(&thread_sandbox_root) {
                    let path_str = rel.to_string_lossy();
                    let mut is_blacklisted = false;

                    for re in &blacklist_regexes {
                        if re.is_match(&path_str) {
                            is_blacklisted = true;
                            break;
                        }
                    }

                    if !is_blacklisted {
                        for part in rel.components() {
                            let part_str = part.as_os_str().to_string_lossy();
                            for re in &blacklist_regexes {
                                if re.is_match(&part_str) {
                                    is_blacklisted = true;
                                    break;
                                }
                            }
                            if is_blacklisted {
                                break;
                            }
                        }
                    }

                    if !is_blacklisted {
                        files.push(rel.to_path_buf());
                    }
                }
            }
        }

        {
            let si = si_arc.lock().unwrap();
            si.files_total.store(files.len(), Ordering::SeqCst);
            let mut snippets = si.snippets.lock().unwrap();
            snippets.retain(|s| files.contains(&s.path));
        }
        ctx.request_repaint();

        // CHECK IF MODEL IS ALREADY IN SHARED STATE (Bod 4)
        let (model, tokenizer) = {
            let s = shared.lock().unwrap();
            (s.bert_model.clone(), s.bert_tokenizer.clone())
        };

        let (model, tokenizer) = if let (Some(m), Some(t)) = (model, tokenizer) {
            (m, t)
        } else {
            // Initialize new model and store it in shared state
            let mut temp_si =
                crate::app::ui::workspace::semantic_index::SemanticIndex::new(thread_root.clone());
            if let Err(e) = temp_si.init() {
                let si = si_arc.lock().unwrap();
                *si.error.lock().unwrap() =
                    Some(format!("Failed to initialize semantic index: {}", e));
                si.is_indexing.store(false, Ordering::SeqCst);
                ctx.request_repaint();
                return;
            }
            let m = temp_si.model.unwrap();
            let t = temp_si.tokenizer.unwrap();

            // Store in shared state for other windows
            {
                let mut s = shared.lock().unwrap();
                s.bert_model = Some(m.clone());
                s.bert_tokenizer = Some(t.clone());
            }
            (m, t)
        };

        // Create a lookup map of existing file hashes for fast O(1) checking
        let existing_hashes: HashMap<PathBuf, String> = {
            let si = si_arc.lock().unwrap();
            let snippets = si.snippets.lock().unwrap();
            snippets
                .iter()
                .map(|s| (s.path.clone(), s.file_hash.clone()))
                .collect()
        };

        for (idx, rel_path) in files.iter().enumerate() {
            // CHECK FOR STOP REQUEST (Bod 6)
            {
                let si = si_arc.lock().unwrap();
                if si.stop_requested.load(Ordering::SeqCst) {
                    si.is_indexing.store(false, Ordering::SeqCst);
                    break;
                }
                si.files_processed.store(idx + 1, Ordering::SeqCst);
                *si.current_file.lock().unwrap() = rel_path.to_string_lossy().to_string();
            }
            ctx.request_repaint();

            let abs_path = thread_sandbox_root.join(rel_path);
            let mtime = std::fs::metadata(&abs_path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // Compute file hash for incremental indexing
            let file_hash =
                match crate::app::ui::workspace::semantic_index::compute_file_hash(&abs_path) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!(
                            "[SemanticIndex] Failed to compute hash for {:?}: {}",
                            abs_path, e
                        );
                        continue;
                    }
                };

            // Check if file needs re-indexing by comparing hash
            let needs_indexing = existing_hashes.get(rel_path) != Some(&file_hash);

            if needs_indexing && let Ok(content) = std::fs::read_to_string(&abs_path) {
                // Skip binary files (null bytes)
                if content.as_bytes().contains(&0) {
                    continue;
                }
                {
                    let si = si_arc.lock().unwrap();
                    let mut snippets = si.snippets.lock().unwrap();
                    snippets.retain(|s| &s.path != rel_path);
                }

                let lines: Vec<&str> = content.lines().collect();
                let chunk_size = 30;
                let overlap = 5;
                let mut start = 0;

                while start < lines.len() {
                    // Check stop again during chunking
                    if si_arc.lock().unwrap().stop_requested.load(Ordering::SeqCst) {
                        break;
                    }

                    let end = (start + chunk_size).min(lines.len());
                    let chunk_text = lines[start..end].join("\n");

                    if !chunk_text.trim().is_empty()
                        && let Ok(embedding) =
                            crate::app::ui::workspace::semantic_index::vectorize_text(
                                &chunk_text,
                                &model,
                                &tokenizer,
                                &candle_core::Device::Cpu,
                            )
                    {
                        let si = si_arc.lock().unwrap();
                        si.snippets.lock().unwrap().push(
                            crate::app::ui::workspace::semantic_index::SemanticSnippet {
                                path: rel_path.clone(),
                                line_start: start + 1,
                                content: chunk_text,
                                embedding,
                                mtime,
                                file_hash: file_hash.clone(),
                            },
                        );
                    }
                    if end == lines.len() {
                        break;
                    }
                    start += chunk_size - overlap;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }

        {
            let mut si = si_arc.lock().unwrap();
            si.model = Some(model);
            si.tokenizer = Some(tokenizer);
            si.is_indexing.store(false, Ordering::SeqCst);
            let _ = si.save();
        }
        ctx.request_repaint();
    });
}
