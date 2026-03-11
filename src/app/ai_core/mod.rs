pub mod types;

pub mod ollama {
    pub use crate::app::cli::ollama::*;
}

pub mod provider {
    pub use crate::app::cli::provider::*;
}

pub mod state {
    pub use crate::app::cli::state::*;
}

pub mod executor {
    pub use crate::app::cli::executor::*;
}

pub mod tools {
    pub use crate::app::cli::tools::*;
}

pub use crate::app::cli::AiManager;
pub use crate::app::cli::ollama::{OllamaStatus, spawn_ollama_check};
pub use state::AiState;
pub use types::*;
