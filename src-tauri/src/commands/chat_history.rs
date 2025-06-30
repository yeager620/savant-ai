use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub is_user: bool,
    pub timestamp: String,
}

fn get_chat_history_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("savant-ai");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    
    Ok(config_dir.join("chat_history.json"))
}

#[tauri::command]
pub async fn save_chat_history(messages: Vec<ChatMessage>) -> Result<(), String> {
    let history_path = get_chat_history_path().map_err(|e| e.to_string())?;
    
    let json = serde_json::to_string_pretty(&messages)
        .map_err(|e| format!("Failed to serialize chat history: {}", e))?;
    
    fs::write(&history_path, json)
        .map_err(|e| format!("Failed to write chat history: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn load_chat_history() -> Result<Vec<ChatMessage>, String> {
    let history_path = get_chat_history_path().map_err(|e| e.to_string())?;
    
    if !history_path.exists() {
        return Ok(Vec::new());
    }
    
    let contents = fs::read_to_string(&history_path)
        .map_err(|e| format!("Failed to read chat history: {}", e))?;
    
    serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse chat history: {}", e))
}

#[tauri::command]
pub async fn clear_chat_history() -> Result<(), String> {
    let history_path = get_chat_history_path().map_err(|e| e.to_string())?;
    
    if history_path.exists() {
        fs::remove_file(&history_path)
            .map_err(|e| format!("Failed to delete chat history: {}", e))?;
    }
    
    Ok(())
}