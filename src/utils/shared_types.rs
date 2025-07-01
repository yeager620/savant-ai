use serde::{Deserialize, Serialize};
use leptos::prelude::ReadSignal;

// Shared types between frontend and backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub questions: Vec<Question>,
    pub processed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub config: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub answer: String,
    pub provider: String,
    pub model: String,
    pub processing_time_ms: u64,
    pub tokens_used: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub use_local: bool,
    pub ollama_model: String,
    pub ollama_url: String,
    pub api_provider: String,
    pub api_key: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            use_local: true,
            ollama_model: "llama3.2".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            api_provider: "ollama".to_string(),
            api_key: None,
            temperature: 0.7,
            max_tokens: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct DetectedQuestion {
    pub id: String,
    pub question: String,
    pub response: ReadSignal<String>,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverlayState {
    Hidden,
    Visible,
    Scanning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    pub question_id: String,
    pub content: String,
    pub is_complete: bool,
}
