use anyhow::Result;
use config::{Config, ConfigError, File, FileFormat};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
pub struct AppConfig {
    pub llm: LlmConfig,
    pub stealth_mode: bool,
    pub auto_scan: bool,
    pub scan_interval: u32,
    pub hotkey_enabled: bool,
    pub transparency: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            stealth_mode: true,
            auto_scan: false,
            scan_interval: 5,
            hotkey_enabled: true,
            transparency: 0.9,
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    let app_config_dir = config_dir.join("savant-ai");
    
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)?;
    }
    
    Ok(app_config_dir.join("config.toml"))
}

pub fn initialize_config() -> Result<()> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        let default_config = AppConfig::default();
        save_config_to_file(&default_config)?;
    }
    
    Ok(())
}

pub fn load_config_from_file() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    
    let config = Config::builder()
        .add_source(File::new(config_path.to_str().unwrap(), FileFormat::Toml))
        .build()?;
    
    let app_config = config.try_deserialize::<AppConfig>()?;
    Ok(app_config)
}

pub fn save_config_to_file(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(config_path, toml_string)?;
    Ok(())
}

#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    load_config_from_file().map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    save_config_to_file(&config).map_err(|e| format!("Failed to save config: {}", e))
}