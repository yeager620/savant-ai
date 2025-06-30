use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use gloo_utils::format::JsValueSerdeExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub use_local: bool,
    pub ollama_model: String,
    pub api_provider: String,
    pub api_key: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            use_local: true,
            ollama_model: "codellama".to_string(),
            api_provider: "ollama".to_string(),
            api_key: None,
            temperature: 0.7,
            max_tokens: 500,
        }
    }
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn query_llm(request: LlmRequest) -> Result<LlmResponse, String> {
    let args = serde_wasm_bindgen::to_value(&request)
        .map_err(|e| format!("Failed to serialize LLM request: {}", e))?;
    
    let result = invoke("query_llm", args).await;
    
    result
        .into_serde::<LlmResponse>()
        .map_err(|e| format!("Failed to parse LLM response: {}", e))
}

pub async fn get_available_models() -> Result<Vec<String>, String> {
    let args = JsValue::NULL;
    let result = invoke("get_available_models", args).await;
    
    result
        .into_serde::<Vec<String>>()
        .map_err(|e| format!("Failed to parse available models: {}", e))
}

pub async fn test_api_connection(config: &LlmConfig) -> Result<bool, String> {
    let args = serde_wasm_bindgen::to_value(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    let result = invoke("test_api_connection", args).await;
    
    result
        .as_bool()
        .ok_or_else(|| "Invalid response from API test".to_string())
}

pub fn format_question_prompt(question: &str, context: Option<&str>) -> String {
    let base_prompt = format!(
        "You are a helpful AI assistant. Please provide a concise, accurate answer to this question: {}\n\n",
        question
    );
    
    if let Some(ctx) = context {
        format!("{}Context: {}\n\nAnswer:", base_prompt, ctx)
    } else {
        format!("{}Answer:", base_prompt)
    }
}

pub fn is_valid_api_key(api_key: &str, provider: &str) -> bool {
    match provider {
        "openai" => api_key.starts_with("sk-") && api_key.len() > 20,
        "deepseek" => api_key.len() > 10,
        "anthropic" => api_key.starts_with("sk-ant-") && api_key.len() > 20,
        _ => api_key.len() > 5,
    }
}