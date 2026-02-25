use eframe::egui;
use serde::{Deserialize, Serialize};

/// Expertise level of the AI agent, defining its persona and code quality standards.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiExpertiseRole {
    /// Focuses on simple tasks, might need more guidance, uses basic patterns.
    Junior,
    /// Experienced developer, follows conventions, thinks about technical debt.
    #[default]
    Senior,
    /// Architect level. Deep understanding of the system, optimization, and security.
    Master,
}

impl AiExpertiseRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiExpertiseRole::Junior => "Junior",
            AiExpertiseRole::Senior => "Senior",
            AiExpertiseRole::Master => "Master",
        }
    }

    pub fn get_persona_mandate(&self) -> &'static str {
        match self {
            AiExpertiseRole::Junior => "ROLE: JUNIOR DEVELOPER. You are eager to help but cautious. Follow instructions literally. Use simple, readable code. If unsure, ask for clarification. Do not over-engineer.",
            AiExpertiseRole::Senior => "ROLE: SENIOR DEVELOPER. You are an experienced engineer. Maintain high standards, follow project conventions, and ensure code is maintainable. Proactively suggest improvements for readability and performance.",
            AiExpertiseRole::Master => "ROLE: MASTER ARCHITECT. You have a deep understanding of software systems. Prioritize security, scalability, and extreme optimization. Think about long-term architectural impacts and edge cases. Your code must be impeccable.",
        }
    }
}

/// Reasoning depth defining how much analysis the agent should perform.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiReasoningDepth {
    /// Quick responses, low token usage, minimal research.
    Fast,
    /// Standard balance, reads relevant files.
    #[default]
    Balanced,
    /// Deep analysis, multiple research steps, exhaustive validation.
    Deep,
}

impl AiReasoningDepth {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiReasoningDepth::Fast => "Fast",
            AiReasoningDepth::Balanced => "Balanced",
            AiReasoningDepth::Deep => "Deep",
        }
    }

    pub fn get_reasoning_mandate(&self) -> &'static str {
        match self {
            AiReasoningDepth::Fast => "REASONING: FAST. Provide direct answers. Minimize file reading. Focus on the immediate prompt and currently open files.",
            AiReasoningDepth::Balanced => "REASONING: BALANCED. Perform necessary research. Check 2-3 related files if needed to ensure consistency. Think before you act.",
            AiReasoningDepth::Deep => "REASONING: DEEP. This is a complex task. You MUST perform exhaustive codebase research using semantic search and file reading. Map dependencies. Verify your logic through multi-step 'monologue' steps (> step). Do not rush.",
        }
    }
}

/// Data structure representing the current project context for AI agents.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AiContextPayload {
    pub open_files: Vec<AiFileContext>,
    pub build_errors: Vec<AiBuildErrorContext>,
    pub active_file: Option<AiFileContext>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiFileContext {
    pub path: String,
    pub content: Option<String>,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiBuildErrorContext {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub is_warning: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiToolDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// StandardAI provides shared logic for AI CLI-like interfaces.
pub struct StandardAI;

impl StandardAI {
    /// Returns the centralized system mandates for an agent based on its configuration.
    pub fn get_system_mandates(role: AiExpertiseRole, depth: AiReasoningDepth) -> String {
        format!(
            "{}\n{}\n\nStrictly adhere to these levels of expertise and reasoning depth.",
            role.get_persona_mandate(),
            depth.get_reasoning_mandate()
        )
    }

    /// Returns a list of standard tools available to all AI agents.
    pub fn get_standard_tools() -> Vec<AiToolDeclaration> {
        vec![
            AiToolDeclaration {
                name: "list_project_files".to_string(),
                description: "BACKUP TOOL: Use ONLY if 'semantic_search' failed to find what you need. Provides a raw list of files for manual directory mapping.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            AiToolDeclaration {
                name: "read_project_file".to_string(),
                description: "Reads the content of a specific file from the project. Use this if you need to analyze code. Use 'line_start' to read large files in segments or to jump to a specific location found by semantic_search.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to the file."
                        },
                        "line_start": {
                            "type": "integer",
                            "description": "Optional: Line number to start reading from (default is 1). Use this to read the next segment if a file was truncated."
                        }
                    },
                    "required": ["path"]
                }),
            },
            AiToolDeclaration {
                name: "write_file".to_string(),
                description: "MANDATORY FOR REPORTS: Creates or overwrites a file in the project sandbox. Use this for ALL reports, documentation, or code proposals. NEVER claim you cannot write files.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to the file."
                        },
                        "content": {
                            "type": "string",
                            "description": "The full text content to write."
                        }
                    },
                    "required": ["path", "content"]
                }),
            },
            AiToolDeclaration {
                name: "search_project".to_string(),
                description: "Performs a full-text search across all files. Returns snippets of matching lines.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Text to search for."
                        }
                    },
                    "required": ["query"]
                }),
            },
            AiToolDeclaration {
                name: "semantic_search".to_string(),
                description: "Searches the project by meaning (semantic search). Use this for conceptual questions like 'how is auth handled' or 'find code related to terminal rendering'.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query or concept to search for."
                        }
                    },
                    "required": ["query"]
                }),
            },
            AiToolDeclaration {
                name: "exec_in_sandbox".to_string(),
                description: "Executes a shell command within the project sandbox and returns output.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to run (e.g. 'cargo check')."
                        }
                    },
                    "required": ["command"]
                }),
            },
        ]
    }

    /// Returns the centralized ASCII logo with version and model info.
    pub fn get_logo(version: &str, model: &str, role: AiExpertiseRole, depth: AiReasoningDepth) -> String {
        format!(
            r#"    ____        __       ______              __
   / __ \____  / /_  __ / ____/_______  ____/ /___
  / /_/ / __ \/ / / / // /   / ___/ _ \/ __  / __ \
 / ____/ /_/ / / /_/ // /___/ /  /  __/ /_/ / /_/ /
/_/    \____/_/\__, / \____/_/   \___/\__,_/\____/
              /____/                              CLI

 Version: {}
 Model:   {}
 Rank:    {} ({})"#,
            version, model, role.as_str(), depth.as_str()
        )
    }

    /// Generates a unified context payload from the current workspace state.
    pub fn generate_context(
        ws: &crate::app::ui::workspace::state::WorkspaceState,
    ) -> AiContextPayload {
        let mut payload = AiContextPayload::default();

        // 1. Gather Open Files
        for (i, tab) in ws.editor.tabs.iter().enumerate() {
            let rel_path = tab
                .path
                .strip_prefix(&ws.root_path)
                .unwrap_or(&tab.path)
                .to_string_lossy()
                .into_owned();

            let is_active = Some(i) == ws.editor.active_tab;
            let file_ctx = AiFileContext {
                path: rel_path.clone(),
                content: if is_active {
                    Some(tab.content.clone())
                } else {
                    None
                },
                is_active,
            };

            payload.open_files.push(file_ctx.clone());
            if is_active {
                payload.active_file = Some(file_ctx);
            }
        }

        // 2. Gather Build Errors
        for err in &ws.build_errors {
            let rel_path = err
                .file
                .strip_prefix(&ws.root_path)
                .unwrap_or(&err.file)
                .to_string_lossy()
                .into_owned();

            payload.build_errors.push(AiBuildErrorContext {
                file: rel_path,
                line: err.line,
                message: err.message.clone(),
                is_warning: err.is_warning,
            });
        }

        payload
    }

    /// Renders a multiline text edit with CLI-like behavior:
    /// - Enter: returns true (should send)
    /// - Shift+Enter, Ctrl+Enter, Ctrl+J: inserts a newline
    /// - Arrow Up/Down: cycles through history
    /// - Returns (should_send, egui_response)
    pub fn ui_input(
        ui: &mut egui::Ui,
        text: &mut String,
        font_size: f32,
        hint: &str,
        history: &[String],
        history_index: &mut Option<usize>,
    ) -> (bool, egui::Response) {
        let mut send = false;

        // Determine if we should intercept keys
        let (enter_pressed, shift, ctrl, j_pressed, up_pressed, down_pressed) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Enter),
                i.modifiers.shift,
                i.modifiers.ctrl,
                i.key_pressed(egui::Key::J),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
            )
        });

        // 1. Enter without modifiers = SEND
        if enter_pressed && !shift && !ctrl {
            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            if !text.trim().is_empty() {
                send = true;
                *history_index = None;
            }
        }

        // 2. Ctrl+J = NEWLINE
        if ctrl && j_pressed {
            ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::J));
            text.push('\n');
        }

        // 3. History Navigation
        if !history.is_empty() {
            if up_pressed {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
                let new_idx = match *history_index {
                    None => Some(history.len().saturating_sub(1)),
                    Some(idx) => Some(idx.saturating_sub(1)),
                };
                if let Some(idx) = new_idx {
                    *text = history[idx].clone();
                    *history_index = Some(idx);
                }
            } else if down_pressed {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
                if let Some(idx) = *history_index {
                    if idx + 1 < history.len() {
                        let next_idx = idx + 1;
                        *text = history[next_idx].clone();
                        *history_index = Some(next_idx);
                    } else {
                        *text = String::new();
                        *history_index = None;
                    }
                }
            }
        }

        // Render the text edit
        let response = ui.add(
            egui::TextEdit::multiline(text)
                .hint_text(hint)
                .desired_width(f32::INFINITY)
                .font(egui::FontId::monospace(font_size))
                .desired_rows(4),
        );

        (send, response)
    }

    /// Renders AI conversation history in a terminal-like style.
    pub fn ui_response(
        ui: &mut egui::Ui,
        conversation: &[(String, String)],
        font_size: f32,
        _cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        let terminal_bg = egui::Color32::from_rgb(20, 20, 25);
        let terminal_text = egui::Color32::from_rgb(175, 175, 175);
        let question_text = egui::Color32::from_rgb(70, 110, 160);
        let poly_color = egui::Color32::from_rgb(70, 110, 160);
        let credo_color = egui::Color32::from_rgb(70, 160, 110);
        let path_purple = egui::Color32::from_rgb(120, 80, 170);

        egui::Frame::new()
            .fill(terminal_bg)
            .inner_margin(8.0)
            .corner_radius(4.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("terminal_response_scroll")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let path_re = regex::Regex::new(r"(?P<link>\[[^\]]+\]\([^\)]+\))|`(?P<code_inner>[^`]+)`|(?P<path>\b(?:src|locales|docs|app|ui|workspace|packaging|privacy|vendor|target)/[a-zA-Z0-9_\-./]+\.[a-z0-9]+\b|\b[a-zA-Z0-9_\-./]+\.(?:rs|toml|md|ftl|sh|json)\b)").ok();

                        for (q, a) in conversation {
                            // Question
                            if !q.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(">>>")
                                            .color(question_text)
                                            .monospace()
                                            .size(font_size),
                                    );
                                    let mut q_mut = q.clone();
                                    ui.add(
                                        egui::TextEdit::multiline(&mut q_mut)
                                            .font(egui::FontId::monospace(font_size))
                                            .text_color(question_text)
                                            .code_editor()
                                            .lock_focus(false)
                                            .interactive(true)
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                                ui.add_space(4.0);
                            }

                            // Answer
                            if !a.is_empty() {
                                if a.contains("____        __") && a.contains("CLI") {
                                    // Special rendering for the ASCII logo
                                    let mut logo_line_idx = 0;
                                    for line in a.lines() {
                                        if line.contains("____") || line.contains(" / ") || line.contains("/_/") || line.contains("/____/") {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                let split_point = match logo_line_idx {
                                                    0 => 25, 1 => 24, 2 => 23, 3 => 22, 4 => 21, 5 => 20, _ => 22,
                                                };
                                                logo_line_idx += 1;
                                                let actual_split = split_point.min(line.len());
                                                let poly = &line[..actual_split];
                                                let credo_full = &line[actual_split..];

                                                ui.label(egui::RichText::new(poly).color(poly_color).monospace().size(font_size));
                                                if line.contains("CLI") {
                                                    let parts: Vec<&str> = credo_full.splitn(2, "CLI").collect();
                                                    ui.label(egui::RichText::new(parts[0]).color(credo_color).monospace().size(font_size));
                                                    ui.label(egui::RichText::new("CLI").color(egui::Color32::from_rgb(110, 90, 0)).monospace().size(font_size));
                                                    if parts.len() > 1 {
                                                        ui.label(egui::RichText::new(parts[1]).color(credo_color).monospace().size(font_size));
                                                    }
                                                } else {
                                                    ui.label(egui::RichText::new(credo_full).color(credo_color).monospace().size(font_size));
                                                }
                                            });
                                        } else if line.trim().starts_with("Version:") || line.trim().starts_with("Model:") || line.trim().starts_with("Rank:") {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                let parts: Vec<&str> = line.splitn(2, ':').collect();
                                                if parts.len() == 2 {
                                                    ui.label(egui::RichText::new(parts[0]).color(egui::Color32::from_gray(90)).monospace().size(font_size));
                                                    ui.label(egui::RichText::new(":").color(egui::Color32::from_gray(50)).monospace().size(font_size));
                                                    ui.label(egui::RichText::new(parts[1]).color(terminal_text).monospace().size(font_size));
                                                } else {
                                                    ui.label(egui::RichText::new(line).color(terminal_text).monospace().size(font_size));
                                                }
                                            });
                                        } else {
                                            ui.label(egui::RichText::new(line).color(terminal_text).monospace().size(font_size));
                                        }
                                    }
                                } else {
                                    // Normal Markdown Rendering with "Monologue" support
                                    ui.scope(|ui| {
                                        let md_font_size = font_size * 1.2;
                                        let style = ui.style_mut();
                                        style.visuals.widgets.noninteractive.fg_stroke.color = terminal_text;
                                        style.visuals.widgets.inactive.fg_stroke.color = terminal_text;
                                        style.visuals.widgets.active.fg_stroke.color = path_purple;
                                        style.visuals.hyperlink_color = path_purple;
                                        style.visuals.code_bg_color = egui::Color32::TRANSPARENT;

                                        let text_styles = &mut style.text_styles;
                                        text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(md_font_size));
                                        text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(md_font_size));
                                        text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(md_font_size));
                                        text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(md_font_size * 1.25));

                                        style.spacing.item_spacing.y = 8.0;

                                        let mut current_block = String::new();
                                        let mut is_monologue_mode = false;

                                        let flush_block = |ui: &mut egui::Ui, block: &mut String, mono: bool, cache: &mut egui_commonmark::CommonMarkCache| {
                                            if block.is_empty() { return; }
                                            let mut text = block.clone();
                                            if let Some(re) = &path_re {
                                                text = re.replace_all(&text, |caps: &regex::Captures| {
                                                    if caps.name("link").is_some() { caps[0].to_string() }
                                                    else if let Some(c) = caps.name("code_inner") { format!("[{}](code)", c.as_str()) }
                                                    else { format!("[{}](path)", &caps[0]) }
                                                }).to_string();
                                            }

                                            if mono {
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 0.0;
                                                    let (rect, _) = ui.allocate_at_least(egui::vec2(2.0, 0.0), egui::Sense::hover());
                                                    ui.painter().rect_filled(rect, 0.0, terminal_text);
                                                    egui::Frame::new()
                                                        .fill(egui::Color32::from_gray(35))
                                                        .inner_margin(egui::Margin::symmetric(12, 8))
                                                        .corner_radius(egui::CornerRadius { nw: 0, ne: 4, sw: 0, se: 4 })
                                                        .show(ui, |ui| {
                                                            egui_commonmark::CommonMarkViewer::new().show(ui, cache, &text);
                                                        });
                                                });
                                            } else {
                                                egui_commonmark::CommonMarkViewer::new().show(ui, cache, &text);
                                            }
                                            block.clear();
                                        };

                                        for line in a.lines() {
                                            let trimmed = line.trim();
                                            let is_mono_line = trimmed.starts_with('>') || trimmed.starts_with("Step");

                                            if is_mono_line != is_monologue_mode && !current_block.is_empty() {
                                                flush_block(ui, &mut current_block, is_monologue_mode, _cache);
                                            }
                                            
                                            is_monologue_mode = is_mono_line;
                                            if is_mono_line {
                                                let clean = line.replace('>', "").trim().to_string();
                                                if clean.starts_with("Step") {
                                                    current_block.push_str(&format!("_{}_\n", clean));
                                                } else {
                                                    current_block.push_str(&format!("{}\n", clean));
                                                }
                                            } else {
                                                current_block.push_str(line);
                                                current_block.push('\n');
                                            }
                                        }
                                        flush_block(ui, &mut current_block, is_monologue_mode, _cache);
                                    });
                                }
                            }

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("📋 Copy Thread").clicked() {
                                        let full_text = if q.is_empty() { a.clone() } else { format!(">>> {}\n\n{}", q, a) };
                                        ui.ctx().copy_text(full_text);
                                    }
                                });
                            });
                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);
                        }
                    });
            });
    }
}
