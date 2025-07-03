use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub model: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

impl Default for LLMRequest {
    fn default() -> Self {
        Self {
            model: "devstral:latest".to_string(),
            prompt: String::new(),
            system_prompt: None,
            max_tokens: Some(2048),
            temperature: Some(0.7),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub model: String,
    pub content: String,
    pub tokens_used: Option<usize>,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait LLMProviderTrait: Send + Sync {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse>;
    async fn complete_streaming(
        &self,
        request: LLMRequest,
        callback: Box<dyn Fn(String) + Send>,
    ) -> Result<LLMResponse>;
    async fn list_models(&self) -> Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub enum LLMProvider {
    Ollama(OllamaProvider),
    Mock(MockLLMProvider),
}

impl LLMProvider {
    pub fn new_ollama(base_url: String, default_model: Option<String>) -> Self {
        LLMProvider::Ollama(OllamaProvider {
            base_url,
            default_model,
        })
    }
}

#[async_trait]
impl LLMProviderTrait for LLMProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        match self {
            LLMProvider::Ollama(provider) => provider.complete(request).await,
            LLMProvider::Mock(provider) => provider.complete(request).await,
        }
    }

    async fn complete_streaming(
        &self,
        request: LLMRequest,
        callback: Box<dyn Fn(String) + Send>,
    ) -> Result<LLMResponse> {
        match self {
            LLMProvider::Ollama(provider) => provider.complete_streaming(request, callback).await,
            LLMProvider::Mock(provider) => provider.complete_streaming(request, callback).await,
        }
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        match self {
            LLMProvider::Ollama(provider) => provider.list_models().await,
            LLMProvider::Mock(provider) => provider.list_models().await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    pub base_url: String,
    pub default_model: Option<String>,
}

#[async_trait]
impl LLMProviderTrait for OllamaProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        // Simple implementation - in real code this would call Ollama API
        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", self.base_url);
        
        let model = self.default_model.as_ref().unwrap_or(&request.model);
        let body = serde_json::json!({
            "model": model,
            "prompt": request.prompt,
            "system": request.system_prompt,
            "options": {
                "temperature": request.temperature.unwrap_or(0.7),
                "num_predict": request.max_tokens.unwrap_or(2048),
            },
            "stream": false,
        });
        
        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        Ok(LLMResponse {
            model: model.to_string(),
            content: response["response"].as_str().unwrap_or("").to_string(),
            tokens_used: response["eval_count"].as_u64().map(|n| n as usize),
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn complete_streaming(
        &self,
        request: LLMRequest,
        _callback: Box<dyn Fn(String) + Send>,
    ) -> Result<LLMResponse> {
        // Simplified - just use regular complete for now
        self.complete(request).await
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", self.base_url);
        
        let response = client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let models = response["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(models)
    }
}

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