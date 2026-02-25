use super::types::AiToolDeclaration;

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
            description: "Creates a NEW file or overwrites for REPORTS: Use this for new documentation, reports, or when creating a file from scratch. For editing existing code, use 'replace'.".to_string(),
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
            name: "replace".to_string(),
            description: "Surgical code edit: Replaces an exact block of text in an existing file. Use this for ALL code modifications. Provide enough context in 'old_string' to uniquely identify the block.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path to the file."
                    },
                    "old_string": {
                        "type": "string",
                        "description": "The exact block of text to be replaced. Must include surrounding lines for precision."
                    },
                    "new_string": {
                        "type": "string",
                        "description": "The new block of text to replace 'old_string' with."
                    }
                },
                "required": ["path", "old_string", "new_string"]
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
