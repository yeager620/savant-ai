//! Configuration management utilities

use std::path::PathBuf;
use anyhow::Result;
use crate::types::AppConfig;

/// Get the default configuration directory for Savant AI
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("savant-ai");
    
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}

/// Get the path to the main configuration file
pub fn get_config_file() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.json"))
}

/// Get the path to the chat history file
pub fn get_chat_history_file() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("chat_history.json"))
}

/// Load configuration from file or return default
pub fn load_config() -> Result<AppConfig> {
    let config_file = get_config_file()?;
    
    if config_file.exists() {
        let content = std::fs::read_to_string(config_file)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

/// Save configuration to file
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_file = get_config_file()?;
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(config_file, content)?;
    Ok(())
}

impl Default for AppConfig {
    fn default() -> Self {
        use crate::types::*;
        
        Self {
            llm: LlmConfig {
                default_provider: LlmProvider::Ollama {
                    url: "http://localhost:11434".to_string()
                },
                default_model: "llama3.2".to_string(),
                default_options: LlmOptions::default(),
            },
            browser: BrowserConfig {
                enabled: true,
                scan_interval_ms: 2000,
                supported_browsers: vec![
                    "Google Chrome".to_string(),
                    "Chromium".to_string(),
                    "Microsoft Edge".to_string(),
                    "Arc".to_string(),
                ],
                question_detection_threshold: 0.7,
            },
            stealth: StealthConfig {
                hide_from_screenshots: true,
                hide_from_taskbar: true,
                always_on_top: true,
                transparency: 0.9,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                position: WindowPosition {
                    x: 100,
                    y: 100,
                    width: 400,
                    height: 600,
                },
                auto_hide: false,
            },
        }
    }
}