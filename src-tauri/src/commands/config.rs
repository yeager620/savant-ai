use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AiConfig,
    pub stealth: StealthConfig,
    pub hotkeys: HotkeyConfig,
    pub scanning: ScanningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub ollama_url: String,
    pub openai_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfig {
    pub enabled: bool,
    pub transparency_level: f32,
    pub hide_from_screenshots: bool,
    pub system_tray_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub toggle_overlay: String,
    pub manual_scan: String,
    pub show_dashboard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanningConfig {
    pub enabled: bool,
    pub interval_ms: u64,
    pub confidence_threshold: f32,
    pub question_patterns: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AiConfig {
                provider: "ollama".to_string(),
                ollama_url: "http://localhost:11434".to_string(),
                openai_api_key: None,
                deepseek_api_key: None,
                anthropic_api_key: None,
                model: "llama3.2".to_string(),
                temperature: 0.7,
                max_tokens: 1000,
            },
            stealth: StealthConfig {
                enabled: true,
                transparency_level: 0.9,
                hide_from_screenshots: true,
                system_tray_only: false,
            },
            hotkeys: HotkeyConfig {
                toggle_overlay: "cmd+shift+a".to_string(),
                manual_scan: "cmd+shift+s".to_string(),
                show_dashboard: "cmd+shift+d".to_string(),
            },
            scanning: ScanningConfig {
                enabled: true,
                interval_ms: 2000,
                confidence_threshold: 0.8,
                question_patterns: vec![
                    r"\?$".to_string(),
                    r"^(what|how|why|when|where|who)".to_string(),
                    r"^(help|explain|show|tell)".to_string(),
                ],
            },
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("savant-ai");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    
    Ok(config_dir.join("config.toml"))
}

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    let config_path = get_config_path().map_err(|e| e.to_string())?;
    
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let config: AppConfig = toml::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        save_config_internal(&config).map_err(|e| e.to_string())?;
        Ok(config)
    }
}

#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    save_config_internal(&config).map_err(|e| e.to_string())
}

fn save_config_internal(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(&config_path, toml_string)?;
    Ok(())
}

#[tauri::command]
pub async fn reset_config() -> Result<AppConfig, String> {
    let config = AppConfig::default();
    save_config_internal(&config).map_err(|e| e.to_string())?;
    Ok(config)
}