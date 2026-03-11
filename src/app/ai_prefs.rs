use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiExpertiseRole {
    Junior,
    #[default]
    Senior,
    Master,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiReasoningDepth {
    Fast,
    #[default]
    Balanced,
    Deep,
}

#[derive(Clone, Debug, Default)]
pub struct AiPanelState {
    pub font_scale: u32,
}
