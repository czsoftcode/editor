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
                    },
                    "line_end": {
                        "type": "integer",
                        "description": "Optional: Last line to read (inclusive). Use together with 'line_start' to read a precise segment without loading the entire file."
                    }
                },
                "required": ["path"]
            }),
        },
        AiToolDeclaration {
            name: "write_file".to_string(),
            description: "Creates a NEW file or generates documentation. FORBIDDEN for editing existing source code: You MUST use 'replace' for modifications. Use this only for new files, logs, or final reports.".to_string(),
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
            name: "exec".to_string(),
            description: "Executes a shell command within the project directory and returns output.".to_string(),
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
        AiToolDeclaration {
            name: "store_scratch".to_string(),
            description: "Stores a key-value note in the agent's temporary RAM scratchpad. Use this during a long task to remember intermediate findings (e.g. what a file does, what you've already explored). This scratchpad is cleared at the start of every new query — it is NOT persisted. For permanent memory, use 'store_fact'.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "A short identifier for the note (e.g. 'auth_flow', 'files_read')."
                    },
                    "value": {
                        "type": "string",
                        "description": "The content to remember for the duration of this task."
                    }
                },
                "required": ["key", "value"]
            }),
        },
        AiToolDeclaration {
            name: "retrieve_scratch".to_string(),
            description: "Retrieves a note from the temporary RAM scratchpad by key. Use this when context is getting full and you need to recall something you noted earlier in this task. Returns an empty string if the key does not exist.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key of the note to retrieve."
                    }
                },
                "required": ["key"]
            }),
        },
        AiToolDeclaration {
            name: "store_fact".to_string(),
            description: "Stores a key-value fact in the agent's long-term memory. This fact survives session restarts and is shared across projects. Use it to remember user preferences, architectural decisions, or important project state.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Unique identifier for the fact."
                    },
                    "value": {
                        "type": "string",
                        "description": "The information to remember."
                    }
                },
                "required": ["key", "value"]
            }),
        },
        AiToolDeclaration {
            name: "retrieve_fact".to_string(),
            description: "Retrieves a previously stored fact from long-term memory by its key. Returns an empty string if not found.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key of the fact to retrieve."
                    }
                },
                "required": ["key"]
            }),
        },
        AiToolDeclaration {
            name: "list_facts".to_string(),
            description: "Returns all keys currently stored in long-term memory. Use this at the start of a new task to check what you already know about the project before asking the user or searching files.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        AiToolDeclaration {
            name: "delete_fact".to_string(),
            description: "Removes an outdated or incorrect fact from long-term memory. Use when you discover that a stored fact is no longer valid (e.g. an API changed, a file was renamed).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key of the fact to delete."
                    }
                },
                "required": ["key"]
            }),
        },
        AiToolDeclaration {
            name: "ask_user".to_string(),
            description: "Ask the user a clarifying question when the task is ambiguous or requires a decision. Call this BEFORE making a guess that could lead to wrong code. The user's answer will be injected as the next user message. Use 'options' to offer concrete choices and reduce back-and-forth.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The clarifying question to ask the user."
                    },
                    "options": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional list of suggested answers for the user to choose from."
                    }
                },
                "required": ["question"]
            }),
        },
        AiToolDeclaration {
            name: "announce_completion".to_string(),
            description: "Signal that the task is fully complete. ALWAYS call this as your FINAL action — never end a task without it. This tells the UI to stop the loading indicator and display your summary to the user.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "summary": {
                        "type": "string",
                        "description": "Brief human-readable description of what was accomplished."
                    },
                    "files_modified": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional list of relative paths of files that were changed."
                    },
                    "follow_up": {
                        "type": "string",
                        "description": "Optional: suggested next steps or warnings for the user (e.g. 'run cargo check to verify')."
                    }
                },
                "required": ["summary"]
            }),
        },
    ]
}
