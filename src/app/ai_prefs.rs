use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiExpertiseRole {
    Junior,
    #[default]
    Senior,
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
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiReasoningDepth {
    Fast,
    #[default]
    Balanced,
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
}

#[derive(Clone, Debug, Default)]
pub struct AiPanelState {
    pub font_scale: u32,
}
