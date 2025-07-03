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
