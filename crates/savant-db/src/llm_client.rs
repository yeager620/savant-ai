//! LLM client implementations for query processing
//! 
//! Provides trait-based LLM integration for natural language query understanding

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use crate::natural_query::LLMClientWrapper;

/// LLM client trait for query processing
#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}

/// Configuration for LLM clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub timeout_seconds: u64,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub api_key: Option<String>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
            timeout_seconds: 30,
            max_tokens: Some(1000),
            temperature: Some(0.1),
            api_key: None,
        }
    }
}

/// Local Ollama client implementation
#[derive(Debug, Clone)]
pub struct OllamaClient {
    config: LLMConfig,
    client: reqwest::Client,
}

/// OpenAI-compatible client implementation
#[derive(Debug, Clone)]
pub struct OpenAIClient {
    config: LLMConfig,
    client: reqwest::Client,
}

/// Mock client for testing
#[derive(Debug, Clone)]
pub struct MockLLMClient {
    responses: HashMap<String, String>,
}

impl OllamaClient {
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
            
        Self { config, client }
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.config.endpoint);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[async_trait]
impl LLMClient for OllamaClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.config.endpoint);
        
        let payload = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature.unwrap_or(0.1),
                "num_predict": self.config.max_tokens.unwrap_or(1000),
            }
        });
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Ollama: {}", e))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama request failed with status {}: {}", status, text));
        }
        
        let result: Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;
            
        result.get("response")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Invalid response format from Ollama"))
    }
    
    fn name(&self) -> &str {
        "ollama"
    }
    
    fn is_available(&self) -> bool {
        // Non-blocking availability check - in production this would cache the result
        true
    }
}

impl OpenAIClient {
    pub fn new(config: LLMConfig) -> Result<Self> {
        if config.api_key.is_none() {
            return Err(anyhow!("OpenAI API key is required"));
        }
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;
            
        Ok(Self { config, client })
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/v1/chat/completions", self.config.endpoint);
        
        let payload = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": self.config.max_tokens.unwrap_or(1000),
            "temperature": self.config.temperature.unwrap_or(0.1),
        });
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to OpenAI: {}", e))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI request failed with status {}: {}", status, text));
        }
        
        let result: Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse OpenAI response: {}", e))?;
            
        result.get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Invalid response format from OpenAI"))
    }
    
    fn name(&self) -> &str {
        "openai"
    }
    
    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }
}

impl MockLLMClient {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        
        // Default mock responses for testing
        responses.insert(
            "intent_classification".to_string(),
            r#"{"intent": "find_conversations", "sql_query": "SELECT * FROM conversations WHERE speaker = ? LIMIT 50", "parameters": {"0": "john"}, "confidence": 0.9}"#.to_string()
        );
        
        responses.insert(
            "speaker_analysis".to_string(),
            r#"{"intent": "analyze_speaker", "sql_query": "SELECT speaker, COUNT(*) as count FROM segments WHERE speaker = ? GROUP BY speaker", "parameters": {"0": "alice"}, "confidence": 0.85}"#.to_string()
        );
        
        Self { responses }
    }
    
    pub fn add_response(&mut self, key: &str, response: &str) {
        self.responses.insert(key.to_string(), response.to_string());
    }
}

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // Simple pattern matching for mock responses
        if prompt.contains("find_conversations") || prompt.contains("conversations") {
            return Ok(self.responses.get("intent_classification").unwrap().clone());
        }
        
        if prompt.contains("analyze_speaker") || prompt.contains("speaker") {
            return Ok(self.responses.get("speaker_analysis").unwrap().clone());
        }
        
        // Default structured response
        Ok(r#"{"intent": "search_content", "sql_query": "SELECT * FROM segments WHERE text LIKE ? LIMIT 20", "parameters": {"0": "%test%"}, "confidence": 0.7}"#.to_string())
    }
    
    fn name(&self) -> &str {
        "mock"
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

/// Factory for creating LLM clients
pub struct LLMClientFactory;

impl LLMClientFactory {
    pub fn create_client(config: &LLMConfig) -> Result<LLMClientWrapper> {
        match config.provider.to_lowercase().as_str() {
            "ollama" => Ok(LLMClientWrapper::Ollama(OllamaClient::new(config.clone()))),
            "openai" => Ok(LLMClientWrapper::OpenAI(OpenAIClient::new(config.clone())?)),
            "mock" => Ok(LLMClientWrapper::Mock(MockLLMClient::new())),
            _ => Err(anyhow!("Unsupported LLM provider: {}", config.provider))
        }
    }
    
    pub async fn create_best_available(configs: Vec<LLMConfig>) -> Result<LLMClientWrapper> {
        for config in configs {
            if let Ok(client) = Self::create_client(&config) {
                if client.is_available() {
                    // For Ollama, do an actual health check
                    if config.provider == "ollama" {
                        if let LLMClientWrapper::Ollama(ref ollama_client) = client {
                            if let Ok(available) = ollama_client.health_check().await {
                                if available {
                                    return Ok(client);
                                }
                            }
                        }
                    } else {
                        return Ok(client);
                    }
                }
            }
        }
        
        // Fallback to mock client
        log::warn!("No LLM providers available, using mock client");
        Ok(LLMClientWrapper::Mock(MockLLMClient::new()))
    }
}

/// Query prompt templates for different intents
pub struct QueryPromptTemplates;

impl QueryPromptTemplates {
    pub fn structured_query_prompt(query: &str, context: Option<&str>) -> String {
        let context_section = if let Some(ctx) = context {
            format!("Previous conversation context:\n{}\n\n", ctx)
        } else {
            String::new()
        };
        
        format!(r#"{}Convert this natural language query to a structured database query for a conversation transcript database.

Database Schema:
- conversations(id, title, start_time, end_time, context)
- segments(id, conversation_id, speaker, text, processed_text, timestamp, confidence, start_time, end_time)

Query: "{}"

Return ONLY a JSON object with:
- intent: one of [find_conversations, analyze_speaker, search_content, get_statistics, list_speakers]
- sql_query: parameterized SQL query with ? placeholders
- parameters: object with string keys "0", "1", etc. and parameter values
- confidence: float between 0.0-1.0

JSON:"#, context_section, query)
    }
    
    pub fn suggestion_prompt(query: &str, similar_queries: &[String]) -> String {
        let suggestions = if similar_queries.is_empty() {
            "No similar queries found.".to_string()
        } else {
            format!("Similar successful queries:\n{}", similar_queries.join("\n- "))
        };
        
        format!(r#"Suggest better ways to phrase this database query based on successful patterns.

Query: "{}"

{}

Provide 2-3 alternative phrasings that might work better. Return as JSON array of strings:

["suggestion 1", "suggestion 2", "suggestion 3"]"#, query, suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_llm_client() {
        let client = MockLLMClient::new();
        
        let response = client.complete("find conversations with john").await.unwrap();
        assert!(response.contains("find_conversations"));
        assert!(response.contains("sql_query"));
    }
    
    #[test]
    fn test_llm_config_default() {
        let config = LLMConfig::default();
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.model, "llama3.2");
    }
    
    #[test]
    fn test_client_factory() {
        let config = LLMConfig::default();
        let client = LLMClientFactory::create_client(&config).unwrap();
        assert_eq!(client.name(), "ollama");
    }
    
    #[test]
    fn test_prompt_templates() {
        let prompt = QueryPromptTemplates::structured_query_prompt("find john", None);
        assert!(prompt.contains("conversations"));
        assert!(prompt.contains("segments"));
        assert!(prompt.contains("JSON"));
    }
}