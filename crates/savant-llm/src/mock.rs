use anyhow::Result;
use std::collections::HashMap;
use async_trait::async_trait;

use crate::{LLMProvider as LLMProviderTrait, LLMRequest, LLMResponse};

#[derive(Debug, Clone)]
pub struct MockLLMProvider {
    responses: HashMap<String, String>,
    failures: HashMap<String, bool>,
}

impl MockLLMProvider {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            failures: HashMap::new(),
        }
    }

    pub fn set_response(&mut self, keyword: &str, response: &str) {
        self.responses.insert(keyword.to_lowercase(), response.to_string());
    }

    pub fn fail_for_model(&mut self, model: &str) {
        self.failures.insert(model.to_string(), true);
    }

    fn find_response(&self, prompt: &str, model: &str) -> Result<String> {
        // Check if this model should fail
        if self.failures.get(model).copied().unwrap_or(false) {
            return Err(anyhow::anyhow!("Mock failure for model: {}", model));
        }

        // Look for keywords in prompt
        let prompt_lower = prompt.to_lowercase();

        // First try exact model match
        if let Some(response) = self.responses.get(model) {
            return Ok(response.clone());
        }

        // Then try keyword matching
        for (keyword, response) in &self.responses {
            if prompt_lower.contains(keyword) {
                return Ok(response.clone());
            }
        }

        // Default response
        Ok("```solution\n// Mock solution\npass\n```".to_string())
    }
}

#[async_trait]
impl LLMProviderTrait for MockLLMProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        let content = self.find_response(&request.prompt, &request.model)?;

        Ok(LLMResponse {
            model: request.model,
            content,
            tokens_used: Some(100),
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn complete_streaming(
        &self,
        request: LLMRequest,
        callback: Box<dyn Fn(String) + Send>,
    ) -> Result<LLMResponse> {
        let content = self.find_response(&request.prompt, &request.model)?;

        // Simulate streaming
        for chunk in content.split(' ') {
            callback(format!("{} ", chunk));
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Ok(LLMResponse {
            model: request.model,
            content,
            tokens_used: Some(100),
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec!["mock-model".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_creation() {
        let provider = MockLLMProvider::new();
        assert!(provider.responses.is_empty());
        assert!(provider.failures.is_empty());
    }

    #[test]
    fn test_set_response() {
        let mut provider = MockLLMProvider::new();
        provider.set_response("hello", "Hello, world!");
        
        assert_eq!(provider.responses.get("hello"), Some(&"Hello, world!".to_string()));
    }

    #[test]
    fn test_fail_for_model() {
        let mut provider = MockLLMProvider::new();
        provider.fail_for_model("failing-model");
        
        assert_eq!(provider.failures.get("failing-model"), Some(&true));
    }

    #[test]
    fn test_find_response_default() {
        let provider = MockLLMProvider::new();
        let result = provider.find_response("any prompt", "any-model").unwrap();
        
        assert_eq!(result, "```solution\n// Mock solution\npass\n```");
    }

    #[test]
    fn test_find_response_model_failure() {
        let mut provider = MockLLMProvider::new();
        provider.fail_for_model("failing-model");
        
        let result = provider.find_response("any prompt", "failing-model");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock failure for model: failing-model"));
    }

    #[test]
    fn test_find_response_exact_model_match() {
        let mut provider = MockLLMProvider::new();
        provider.set_response("gpt-4", "GPT-4 specific response");
        
        let result = provider.find_response("any prompt", "gpt-4").unwrap();
        assert_eq!(result, "GPT-4 specific response");
    }

    #[test]
    fn test_find_response_keyword_matching() {
        let mut provider = MockLLMProvider::new();
        provider.set_response("python", "Python code solution");
        provider.set_response("javascript", "JavaScript solution");
        
        let result1 = provider.find_response("Write some Python code", "any-model").unwrap();
        assert_eq!(result1, "Python code solution");
        
        let result2 = provider.find_response("JavaScript function needed", "any-model").unwrap();
        assert_eq!(result2, "JavaScript solution");
    }

    #[test]
    fn test_find_response_case_insensitive() {
        let mut provider = MockLLMProvider::new();
        provider.set_response("ALGORITHM", "Algorithm response");
        
        let result = provider.find_response("algorithm implementation", "test-model").unwrap();
        assert_eq!(result, "Algorithm response");
    }

    #[tokio::test]
    async fn test_complete() {
        let mut provider = MockLLMProvider::new();
        provider.set_response("test", "Test response content");
        
        let request = LLMRequest {
            prompt: "test prompt".to_string(),
            model: "test-model".to_string(),
            provider: None,
            temperature: Some(0.5),
            max_tokens: Some(100),
        };
        
        let response = provider.complete(request).await.unwrap();
        
        assert_eq!(response.model, "test-model");
        assert_eq!(response.content, "Test response content");
        assert_eq!(response.tokens_used, Some(100));
        assert_eq!(response.finish_reason, Some("stop".to_string()));
    }

    #[tokio::test]
    async fn test_complete_with_failure() {
        let mut provider = MockLLMProvider::new();
        provider.fail_for_model("failing-model");
        
        let request = LLMRequest {
            prompt: "any prompt".to_string(),
            model: "failing-model".to_string(),
            provider: None,
            temperature: None,
            max_tokens: None,
        };
        
        let result = provider.complete(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_streaming() {
        let mut provider = MockLLMProvider::new();
        provider.set_response("stream", "Hello world streaming");
        
        let request = LLMRequest {
            prompt: "stream test".to_string(),
            model: "stream-model".to_string(),
            provider: None,
            temperature: None,
            max_tokens: None,
        };
        
        let callback = Box::new(|_chunk: String| {
            // Note: In real test we'd use Arc<Mutex<Vec<String>>> to capture chunks
            // For now just verify callback is called by testing the final response
        });
        
        let response = provider.complete_streaming(request, callback).await.unwrap();
        
        assert_eq!(response.model, "stream-model");
        assert_eq!(response.content, "Hello world streaming");
        assert_eq!(response.tokens_used, Some(100));
        assert_eq!(response.finish_reason, Some("stop".to_string()));
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = MockLLMProvider::new();
        let models = provider.list_models().await.unwrap();
        
        assert_eq!(models, vec!["mock-model".to_string()]);
    }

    #[test]
    fn test_multiple_keywords() {
        let mut provider = MockLLMProvider::new();
        provider.set_response("python", "Python solution");
        provider.set_response("algorithm", "Algorithm solution");
        
        // First match should win
        let result = provider.find_response("Python algorithm", "test-model").unwrap();
        // The order depends on HashMap iteration, but one of them should match
        assert!(result == "Python solution" || result == "Algorithm solution");
    }
}
