use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use savant_core::LlmRequest as CoreLlmRequest;
use savant_core::LlmProvider as CoreLlmProvider;

// Re-export the mock module
pub mod mock;

/// LLM request with all parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub model: String,
    pub provider: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl From<CoreLlmRequest> for LLMRequest {
    fn from(req: CoreLlmRequest) -> Self {
        Self {
            prompt: req.prompt,
            model: req.model,
            provider: None, // This would need to be extracted from req.provider
            temperature: Some(req.options.temperature),
            max_tokens: Some(req.options.max_tokens),
        }
    }
}

/// LLM response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub model: String,
    pub content: String,
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<String>,
}

/// LLM provider trait
#[async_trait]
pub trait LLMProvider {
    /// Complete a prompt with the LLM
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse>;
    
    /// Complete a prompt with streaming response
    async fn complete_streaming(
        &self,
        request: LLMRequest,
        callback: Box<dyn Fn(String) + Send>,
    ) -> Result<LLMResponse>;
    
    /// List available models
    async fn list_models(&self) -> Result<Vec<String>>;
}