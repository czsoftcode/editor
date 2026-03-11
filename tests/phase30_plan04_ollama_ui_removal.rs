use std::fs;
use std::path::Path;

const UI_FILES: &[&str] = &[
    "src/app/ui/terminal/right/ai_bar.rs",
    "src/app/ui/terminal/ai_chat/render.rs",
    "src/app/ui/terminal/ai_chat/mod.rs",
];

const FORBIDDEN: &[&str] = &[
    "OllamaConnectionStatus",
    "selected_model",
    "model_info",
    "ollama",
];

#[test]
fn terminal_ui_is_assistant_only_without_ollama_controls() {
    for rel in UI_FILES {
        let path = Path::new(rel);
        let content = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for token in FORBIDDEN {
            assert!(
                !content.contains(token),
                "forbidden token '{token}' found in {}",
                path.display()
            );
        }
    }
}
