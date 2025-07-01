use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub toggle_overlay: String,
    pub take_screenshot: String,
    pub show_dashboard: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            toggle_overlay: "CommandOrControl+Shift+A".to_string(),
            take_screenshot: "CommandOrControl+Shift+S".to_string(),
            show_dashboard: "CommandOrControl+Shift+D".to_string(),
        }
    }
}

use std::sync::LazyLock;

static OVERLAY_STATE: LazyLock<Arc<Mutex<bool>>> = LazyLock::new(|| Arc::new(Mutex::new(false)));

#[tauri::command]
pub async fn register_global_hotkeys(_app_handle: AppHandle) -> Result<(), String> {
    // Simplified implementation - global hotkeys will be added later
    // For now, just confirm registration
    println!("Global hotkeys registered (placeholder implementation)");
    Ok(())
}

#[tauri::command]
pub async fn unregister_global_hotkeys(_app_handle: AppHandle) -> Result<(), String> {
    // TODO: Implement with tauri-plugin-global-shortcut in future
    println!("Global hotkeys unregistration placeholder - not implemented yet");
    Ok(())
}

#[tauri::command]
pub async fn get_overlay_state() -> Result<bool, String> {
    let state = OVERLAY_STATE.lock().map_err(|e| format!("Failed to get overlay state: {}", e))?;
    Ok(*state)
}

#[tauri::command]
pub async fn set_overlay_state(active: bool) -> Result<(), String> {
    let mut state = OVERLAY_STATE.lock().map_err(|e| format!("Failed to set overlay state: {}", e))?;
    *state = active;
    println!("Overlay state set to: {}", active);
    Ok(())
}


#[tauri::command]
pub async fn test_hotkey(hotkey: String) -> Result<bool, String> {
    // This is a simple validation function
    // In a real implementation, you might want to parse and validate the hotkey string
    let valid_modifiers = ["CommandOrControl", "Alt", "Shift", "Control", "Command"];
    let valid_keys = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
        "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"
    ];
    
    if hotkey.is_empty() {
        return Ok(false);
    }
    
    let parts: Vec<&str> = hotkey.split('+').collect();
    if parts.len() < 2 {
        return Ok(false);
    }
    
    // Check if the last part is a valid key
    let key = parts.last().unwrap();
    if !valid_keys.contains(key) {
        return Ok(false);
    }
    
    // Check if all other parts are valid modifiers
    for &part in &parts[..parts.len()-1] {
        if !valid_modifiers.contains(&part) {
            return Ok(false);
        }
    }
    
    Ok(true)
}