use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use savant_core::LlmRequest as CoreLlmRequest;

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

#[cfg(test)]
mod tests {
    use super::*;
    use savant_core::{LlmOptions, LlmProvider, LlmRequest as CoreLlmRequest};

    #[test]
    fn test_llm_request_creation() {
        let request = LLMRequest {
            prompt: "Test prompt".to_string(),
            model: "gpt-4".to_string(),
            provider: Some("openai".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
        };

        assert_eq!(request.prompt, "Test prompt");
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.provider, Some("openai".to_string()));
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(1000));
    }

    #[test]
    fn test_llm_request_serialization() {
        let request = LLMRequest {
            prompt: "Test prompt".to_string(),
            model: "gpt-4".to_string(),
            provider: Some("openai".to_string()),
            temperature: Some(0.8),
            max_tokens: Some(500),
        };

        // Test serialization
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("Test prompt"));
        assert!(serialized.contains("gpt-4"));
        assert!(serialized.contains("openai"));

        // Test deserialization
        let deserialized: LLMRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(request.prompt, deserialized.prompt);
        assert_eq!(request.model, deserialized.model);
        assert_eq!(request.provider, deserialized.provider);
        assert_eq!(request.temperature, deserialized.temperature);
        assert_eq!(request.max_tokens, deserialized.max_tokens);
    }

    #[test]
    fn test_llm_response_creation() {
        let response = LLMResponse {
            model: "gpt-4".to_string(),
            content: "Generated response".to_string(),
            tokens_used: Some(150),
            finish_reason: Some("stop".to_string()),
        };

        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.content, "Generated response");
        assert_eq!(response.tokens_used, Some(150));
        assert_eq!(response.finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_llm_response_serialization() {
        let response = LLMResponse {
            model: "claude-3".to_string(),
            content: "AI response content".to_string(),
            tokens_used: Some(75),
            finish_reason: Some("length".to_string()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("claude-3"));
        assert!(serialized.contains("AI response content"));

        // Test deserialization
        let deserialized: LLMResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(response.model, deserialized.model);
        assert_eq!(response.content, deserialized.content);
        assert_eq!(response.tokens_used, deserialized.tokens_used);
        assert_eq!(response.finish_reason, deserialized.finish_reason);
    }

    #[test]
    fn test_core_llm_request_conversion() {
        let core_request = CoreLlmRequest {
            prompt: "Convert me".to_string(),
            model: "test-model".to_string(),
            provider: LlmProvider::Ollama { url: "http://localhost:11434".to_string() },
            options: LlmOptions {
                temperature: 0.9,
                max_tokens: 2000,
                stream: false,
            },
            context: None,
        };

        let llm_request: LLMRequest = core_request.into();
        
        assert_eq!(llm_request.prompt, "Convert me");
        assert_eq!(llm_request.model, "test-model");
        assert_eq!(llm_request.provider, None); // Currently not extracted from enum
        assert_eq!(llm_request.temperature, Some(0.9));
        assert_eq!(llm_request.max_tokens, Some(2000));
    }

    #[test]
    fn test_llm_request_with_none_values() {
        let request = LLMRequest {
            prompt: "Minimal request".to_string(),
            model: "base-model".to_string(),
            provider: None,
            temperature: None,
            max_tokens: None,
        };

        assert_eq!(request.prompt, "Minimal request");
        assert_eq!(request.model, "base-model");
        assert!(request.provider.is_none());
        assert!(request.temperature.is_none());
        assert!(request.max_tokens.is_none());
    }

    #[test]
    fn test_llm_response_with_none_values() {
        let response = LLMResponse {
            model: "simple-model".to_string(),
            content: "Simple response".to_string(),
            tokens_used: None,
            finish_reason: None,
        };

        assert_eq!(response.model, "simple-model");
        assert_eq!(response.content, "Simple response");
        assert!(response.tokens_used.is_none());
        assert!(response.finish_reason.is_none());
    }
}
