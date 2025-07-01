//! # Savant LLM CLI Tool
//!
//! A standalone CLI tool for LLM inference that follows UNIX philosophy:
//! - Does one thing well: LLM inference
//! - Reads from stdin or CLI args
//! - Outputs JSON to stdout
//! - Can be piped to other tools

use clap::{Parser, Subcommand};
use serde_json;
use savant_core::*;
use std::io::{self, Read};

#[derive(Parser)]
#[command(
    name = "savant-llm",
    about = "Standalone LLM inference tool for Savant AI",
    long_about = "A UNIX-philosophy CLI tool for LLM inference. Reads prompts from stdin or args, outputs JSON responses."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Prompt text (alternative to stdin)
    #[arg(short, long)]
    prompt: Option<String>,

    /// Model name to use
    #[arg(short, long, default_value = "devstral")]
    model: String,

    /// Temperature for generation (0.0-1.0)
    #[arg(short, long, default_value = "0.7")]
    temperature: f32,

    /// Maximum tokens to generate
    #[arg(long, default_value = "4096")]
    max_tokens: u32,

    /// Enable streaming output
    #[arg(short, long)]
    stream: bool,

    /// Provider (ollama, openai, deepseek, anthropic)
    #[arg(short = 'P', long, default_value = "ollama")]
    provider: String,

    /// Provider URL (for Ollama) or API key (for cloud providers)
    #[arg(long)]
    auth: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Process a single prompt
    Query {
        /// Prompt text
        prompt: String,
    },
    /// Process batch of prompts from stdin (one per line)
    Batch,
    /// List available models
    Models,
    /// Test connection to provider
    Test,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Query { ref prompt }) => {
            query_llm(prompt, &cli).await?;
        }
        Some(Commands::Batch) => {
            batch_process(&cli).await?;
        }
        Some(Commands::Models) => {
            list_models(&cli).await?;
        }
        Some(Commands::Test) => {
            test_connection(&cli).await?;
        }
        None => {
            // No subcommand - check for prompt arg or stdin
            if let Some(ref prompt) = cli.prompt {
                query_llm(prompt, &cli).await?;
            } else {
                // Read from stdin
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                
                // Try to parse as JSON first, then treat as plain text
                if let Ok(request) = serde_json::from_str::<LlmRequest>(&input.trim()) {
                    process_llm_request(request).await?;
                } else {
                    query_llm(&input.trim(), &cli).await?;
                }
            }
        }
    }

    Ok(())
}

async fn query_llm(prompt: &str, cli: &Cli) -> anyhow::Result<()> {
    let provider = create_provider(&cli.provider, &cli.auth)?;
    
    let request = LlmRequest {
        prompt: prompt.to_string(),
        model: cli.model.clone(),
        provider,
        options: LlmOptions {
            temperature: cli.temperature,
            max_tokens: cli.max_tokens,
            stream: cli.stream,
        },
        context: None,
    };

    process_llm_request(request).await
}

async fn process_llm_request(request: LlmRequest) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();
    
    let response = match &request.provider {
        LlmProvider::Ollama { url } => {
            query_ollama(&request, url).await?
        }
        LlmProvider::OpenAI { api_key } => {
            query_openai(&request, api_key).await?
        }
        LlmProvider::DeepSeek { api_key } => {
            query_deepseek(&request, api_key).await?
        }
        LlmProvider::Anthropic { api_key } => {
            query_anthropic(&request, api_key).await?
        }
    };

    let processing_time = start_time.elapsed().as_millis() as u64;

    let final_response = LlmResponse {
        content: response,
        model: request.model,
        provider: provider_name(&request.provider),
        tokens_used: None, // TODO: Extract from provider response
        processing_time_ms: processing_time,
        finished: true,
    };

    // Output JSON to stdout
    println!("{}", serde_json::to_string(&final_response)?);
    Ok(())
}

async fn batch_process(cli: &Cli) -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    for line in input.lines() {
        if !line.trim().is_empty() {
            query_llm(line.trim(), cli).await?;
        }
    }

    Ok(())
}

async fn list_models(cli: &Cli) -> anyhow::Result<()> {
    let models = match cli.provider.as_str() {
        "ollama" => {
            let url = cli.auth.as_deref().unwrap_or("http://localhost:11434");
            get_ollama_models(url).await?
        }
        _ => {
            vec!["Model listing not implemented for this provider".to_string()]
        }
    };

    let output = serde_json::json!({
        "provider": cli.provider,
        "models": models
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn test_connection(cli: &Cli) -> anyhow::Result<()> {
    let result = match cli.provider.as_str() {
        "ollama" => {
            let url = cli.auth.as_deref().unwrap_or("http://localhost:11434");
            test_ollama_connection(url).await
        }
        _ => {
            eprintln!("Connection test not implemented for provider: {}", cli.provider);
            false
        }
    };

    let output = serde_json::json!({
        "provider": cli.provider,
        "connected": result,
        "timestamp": chrono::Utc::now()
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn create_provider(provider: &str, auth: &Option<String>) -> anyhow::Result<LlmProvider> {
    match provider {
        "ollama" => Ok(LlmProvider::Ollama {
            url: auth.as_deref().unwrap_or("http://localhost:11434").to_string()
        }),
        "openai" => {
            let api_key = auth.as_ref()
                .ok_or_else(|| anyhow::anyhow!("OpenAI requires API key via --auth"))?;
            Ok(LlmProvider::OpenAI { api_key: api_key.clone() })
        }
        "deepseek" => {
            let api_key = auth.as_ref()
                .ok_or_else(|| anyhow::anyhow!("DeepSeek requires API key via --auth"))?;
            Ok(LlmProvider::DeepSeek { api_key: api_key.clone() })
        }
        "anthropic" => {
            let api_key = auth.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Anthropic requires API key via --auth"))?;
            Ok(LlmProvider::Anthropic { api_key: api_key.clone() })
        }
        _ => Err(anyhow::anyhow!("Unsupported provider: {}", provider))
    }
}

fn provider_name(provider: &LlmProvider) -> String {
    match provider {
        LlmProvider::Ollama { .. } => "ollama".to_string(),
        LlmProvider::OpenAI { .. } => "openai".to_string(),
        LlmProvider::DeepSeek { .. } => "deepseek".to_string(),
        LlmProvider::Anthropic { .. } => "anthropic".to_string(),
    }
}

// Provider-specific implementations
async fn query_ollama(request: &LlmRequest, url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "model": request.model,
        "prompt": request.prompt,
        "stream": false,
        "options": {
            "temperature": request.options.temperature,
            "num_predict": request.options.max_tokens
        }
    });

    let response = client
        .post(&format!("{}/api/generate", url))
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Ollama API error: {}", response.status()));
    }

    let result: serde_json::Value = response.json().await?;
    let content = result["response"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from Ollama"))?;

    Ok(content.to_string())
}

async fn query_openai(_request: &LlmRequest, _api_key: &str) -> anyhow::Result<String> {
    // TODO: Implement OpenAI API call
    Err(anyhow::anyhow!("OpenAI provider not yet implemented"))
}

async fn query_deepseek(_request: &LlmRequest, _api_key: &str) -> anyhow::Result<String> {
    // TODO: Implement DeepSeek API call
    Err(anyhow::anyhow!("DeepSeek provider not yet implemented"))
}

async fn query_anthropic(_request: &LlmRequest, _api_key: &str) -> anyhow::Result<String> {
    // TODO: Implement Anthropic API call
    Err(anyhow::anyhow!("Anthropic provider not yet implemented"))
}

async fn get_ollama_models(url: &str) -> anyhow::Result<Vec<String>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/tags", url))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to get models from Ollama"));
    }

    let result: serde_json::Value = response.json().await?;
    let models = result["models"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid models response format"))?
        .iter()
        .filter_map(|m| m["name"].as_str())
        .map(|s| s.to_string())
        .collect();

    Ok(models)
}

async fn test_ollama_connection(url: &str) -> bool {
    let client = reqwest::Client::new();
    match client.get(&format!("{}/api/tags", url)).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}