use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tauri::{AppHandle, Emitter};
use crate::commands::ocr::DetectedQuestion;

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
            ollama_model: "codellama".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
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

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIUsage {
    total_tokens: u32,
}

#[tauri::command]
pub async fn query_llm(request: LlmRequest) -> Result<LlmResponse, String> {
    let start_time = Instant::now();
    
    let response = if request.config.use_local {
        query_ollama(&request).await
    } else {
        match request.config.api_provider.as_str() {
            "openai" => query_openai(&request).await,
            "deepseek" => query_deepseek(&request).await,
            "anthropic" => query_anthropic(&request).await,
            _ => Err(anyhow!("Unsupported API provider: {}", request.config.api_provider)),
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    match response {
        Ok((answer, tokens)) => Ok(LlmResponse {
            answer,
            provider: if request.config.use_local { "ollama".to_string() } else { request.config.api_provider },
            model: if request.config.use_local { request.config.ollama_model } else { "gpt-3.5-turbo".to_string() },
            processing_time_ms: processing_time,
            tokens_used: tokens,
        }),
        Err(e) => Err(format!("LLM query failed: {}", e)),
    }
}

async fn query_ollama(request: &LlmRequest) -> Result<(String, Option<u32>)> {
    let client = Client::new();
    let full_prompt = format_prompt(&request.prompt, request.context.as_deref());
    
    let ollama_request = OllamaRequest {
        model: request.config.ollama_model.clone(),
        prompt: full_prompt,
        stream: false,
        options: OllamaOptions {
            temperature: request.config.temperature,
            num_predict: request.config.max_tokens,
        },
    };
    
    let url = format!("{}/api/generate", request.config.ollama_url);
    let response = timeout(Duration::from_secs(30), 
        client.post(&url)
            .json(&ollama_request)
            .send()
    ).await
        .map_err(|_| anyhow!("Ollama request timed out"))?
        .map_err(|e| anyhow!("Ollama request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Ollama returned error: {}", response.status()));
    }
    
    let ollama_response: OllamaResponse = response.json().await
        .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;
    
    Ok((ollama_response.response, None))
}

async fn query_openai(request: &LlmRequest) -> Result<(String, Option<u32>)> {
    let client = Client::new();
    let api_key = request.config.api_key.as_ref()
        .ok_or_else(|| anyhow!("OpenAI API key not provided"))?;
    
    let openai_request = OpenAIRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are a helpful AI assistant. Provide concise, accurate answers.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: format_prompt(&request.prompt, request.context.as_deref()),
            },
        ],
        temperature: request.config.temperature,
        max_tokens: request.config.max_tokens,
    };
    
    let response = timeout(Duration::from_secs(30),
        client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
    ).await
        .map_err(|_| anyhow!("OpenAI request timed out"))?
        .map_err(|e| anyhow!("OpenAI request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("OpenAI returned error: {}", response.status()));
    }
    
    let openai_response: OpenAIResponse = response.json().await
        .map_err(|e| anyhow!("Failed to parse OpenAI response: {}", e))?;
    
    let answer = openai_response.choices
        .first()
        .ok_or_else(|| anyhow!("No choices in OpenAI response"))?
        .message
        .content
        .clone();
    
    let tokens = openai_response.usage.map(|u| u.total_tokens);
    
    Ok((answer, tokens))
}

async fn query_deepseek(request: &LlmRequest) -> Result<(String, Option<u32>)> {
    let client = Client::new();
    let api_key = request.config.api_key.as_ref()
        .ok_or_else(|| anyhow!("DeepSeek API key not provided"))?;
    
    let deepseek_request = OpenAIRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are a helpful AI assistant. Provide concise, accurate answers.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: format_prompt(&request.prompt, request.context.as_deref()),
            },
        ],
        temperature: request.config.temperature,
        max_tokens: request.config.max_tokens,
    };
    
    let response = timeout(Duration::from_secs(30),
        client.post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&deepseek_request)
            .send()
    ).await
        .map_err(|_| anyhow!("DeepSeek request timed out"))?
        .map_err(|e| anyhow!("DeepSeek request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("DeepSeek returned error: {}", response.status()));
    }
    
    let deepseek_response: OpenAIResponse = response.json().await
        .map_err(|e| anyhow!("Failed to parse DeepSeek response: {}", e))?;
    
    let answer = deepseek_response.choices
        .first()
        .ok_or_else(|| anyhow!("No choices in DeepSeek response"))?
        .message
        .content
        .clone();
    
    let tokens = deepseek_response.usage.map(|u| u.total_tokens);
    
    Ok((answer, tokens))
}

async fn query_anthropic(_request: &LlmRequest) -> Result<(String, Option<u32>)> {
    // Note: This is a simplified implementation
    // Full Anthropic API integration would require more specific handling
    Err(anyhow!("Anthropic API integration not yet implemented"))
}

#[tauri::command]
pub async fn get_available_models() -> Result<Vec<String>, String> {
    let client = Client::new();
    
    // Try to get Ollama models
    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(response) if response.status().is_success() => {
            #[derive(Deserialize)]
            struct OllamaModelsResponse {
                models: Vec<OllamaModel>,
            }
            
            #[derive(Deserialize)]
            struct OllamaModel {
                name: String,
            }
            
            if let Ok(models_response) = response.json::<OllamaModelsResponse>().await {
                let models = models_response.models.into_iter().map(|m| m.name).collect();
                return Ok(models);
            }
        }
        _ => {}
    }
    
    // Return default models if Ollama is not available
    Ok(vec![
        "codellama".to_string(),
        "llama2".to_string(),
        "mistral".to_string(),
    ])
}

#[tauri::command]
pub async fn test_api_connection(config: LlmConfig) -> Result<bool, String> {
    if config.use_local {
        let client = Client::new();
        let url = format!("{}/api/tags", config.ollama_url);
        
        match timeout(Duration::from_secs(5), client.get(&url).send()).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false),
        }
    } else {
        let test_request = LlmRequest {
            prompt: "Hello".to_string(),
            context: None,
            config,
        };
        
        match query_llm(test_request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    pub question_id: String,
    pub content: String,
    pub is_complete: bool,
}

#[tauri::command]
pub async fn stream_response_for_question(app: AppHandle, question: DetectedQuestion) -> Result<(), String> {
    let config = LlmConfig::default(); // TODO: Get from actual config
    
    // Start streaming response
    let question_id = question.id.clone();
    let prompt = question.text.clone();
    
    if config.use_local {
        stream_ollama_response(app, question_id, prompt, config).await
    } else {
        stream_api_response(app, question_id, prompt, config).await
    }
}

async fn stream_ollama_response(app: AppHandle, question_id: String, prompt: String, config: LlmConfig) -> Result<(), String> {
    let client = Client::new();
    let full_prompt = format_prompt(&prompt, None);
    
    let ollama_request = OllamaRequest {
        model: config.ollama_model.clone(),
        prompt: full_prompt,
        stream: true,
        options: OllamaOptions {
            temperature: config.temperature,
            num_predict: config.max_tokens,
        },
    };
    
    let url = format!("{}/api/generate", config.ollama_url);
    let mut response = client.post(&url)
        .json(&ollama_request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;
    
    let mut accumulated_response = String::new();
    let mut buffer = Vec::new();
    
    while let Some(chunk) = response.chunk().await.map_err(|e| format!("Chunk error: {}", e))? {
        buffer.extend_from_slice(&chunk);
        
        // Try to parse complete JSON objects from buffer
        if let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
            let line = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
            let line_str = String::from_utf8_lossy(&line[..line.len()-1]); // Remove newline
            
            if let Ok(ollama_response) = serde_json::from_str::<OllamaResponse>(&line_str) {
                accumulated_response.push_str(&ollama_response.response);
                
                // Emit streaming update
                let streaming_response = StreamingResponse {
                    question_id: question_id.clone(),
                    content: if ollama_response.done { 
                        format!("{}[COMPLETE]", accumulated_response) 
                    } else { 
                        accumulated_response.clone() 
                    },
                    is_complete: ollama_response.done,
                };
                
                let _ = app.emit("streaming_response", &streaming_response);
                
                if ollama_response.done {
                    break;
                }
            }
        }
    }
    
    Ok(())
}

async fn stream_api_response(app: AppHandle, question_id: String, prompt: String, config: LlmConfig) -> Result<(), String> {
    // For non-streaming APIs, we'll simulate streaming by sending chunks
    let request = LlmRequest {
        prompt,
        context: None,
        config: config.clone(),
    };
    
    let response = query_llm(request).await?;
    let words: Vec<&str> = response.answer.split_whitespace().collect();
    
    let mut accumulated_response = String::new();
    
    for (i, word) in words.iter().enumerate() {
        accumulated_response.push_str(word);
        accumulated_response.push(' ');
        
        let is_complete = i == words.len() - 1;
        
        let streaming_response = StreamingResponse {
            question_id: question_id.clone(),
            content: if is_complete { 
                format!("{}[COMPLETE]", accumulated_response.trim()) 
            } else { 
                accumulated_response.clone() 
            },
            is_complete,
        };
        
        let _ = app.emit("streaming_response", &streaming_response);
        
        if !is_complete {
            tokio::time::sleep(Duration::from_millis(100)).await; // Simulate typing speed
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn query_question(_question_text: String) -> Result<(), String> {
    // This is called from the frontend to trigger a question query
    // The actual response streaming is handled by stream_response_for_question
    Ok(())
}

fn format_prompt(prompt: &str, context: Option<&str>) -> String {
    let base_prompt = format!(
        "You are a helpful AI assistant. Please provide a concise, accurate answer to this question: {}\n\n",
        prompt
    );
    
    if let Some(ctx) = context {
        format!("{}Context: {}\n\nAnswer:", base_prompt, ctx)
    } else {
        format!("{}Answer:", base_prompt)
    }
}