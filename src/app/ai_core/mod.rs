pub mod types;

pub mod state {
    pub use crate::app::cli::state::*;
}

pub mod executor {
    pub use crate::app::cli::executor::*;
}

pub use crate::app::cli::{AiManager, OllamaStatus, spawn_ollama_check};
pub use state::AiState;
pub use types::*;
