use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Window};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfig {
    pub hide_from_screenshots: bool,
    pub hide_from_taskbar: bool,
    pub system_tray_only: bool,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            hide_from_screenshots: true,
            hide_from_taskbar: true,
            system_tray_only: true,
        }
    }
}

#[tauri::command]
pub async fn enable_stealth_mode(_window: Window) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // macOS stealth implementation will be added later
        println!("macOS stealth mode enabled");
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows stealth implementation will be added later
        println!("Windows stealth mode enabled");
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux stealth implementation would go here
        println!("Linux stealth mode not implemented yet");
    }
    
    Ok(())
}

#[tauri::command]
pub async fn disable_stealth_mode(_window: Window) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // macOS stealth implementation will be added later
        println!("macOS stealth mode disabled");
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows stealth implementation will be added later
        println!("Windows stealth mode disabled");
    }
    
    #[cfg(target_os = "linux")]
    {
        println!("Linux stealth mode not implemented yet");
    }
    
    Ok(())
}

#[tauri::command]
pub async fn take_screenshot() -> Result<Vec<u8>, String> {
    // Stub implementation - screenshot functionality has been removed
    // This function is kept for compatibility but returns an error
    Err("Screenshot functionality has been removed from this version".to_string())
}

#[tauri::command]
pub async fn get_screen_info() -> Result<HashMap<String, serde_json::Value>, String> {
    // Stub implementation - screen info functionality has been simplified
    let mut screen_info = HashMap::new();
    
    // Return basic mock screen info for compatibility
    screen_info.insert(
        "screen_0".to_string(),
        serde_json::json!({
            "id": 0,
            "x": 0,
            "y": 0,
            "width": 1920,
            "height": 1080,
            "scale_factor": 1.0,
            "is_primary": true,
        }),
    );
    
    Ok(screen_info)
}

#[tauri::command]
pub async fn set_window_always_on_top(window: Window, always_on_top: bool) -> Result<(), String> {
    window
        .set_always_on_top(always_on_top)
        .map_err(|e| format!("Failed to set always on top: {}", e))
}

#[tauri::command]
pub async fn set_window_transparency(_window: Window, transparency: f64) -> Result<(), String> {
    // TODO: Implement platform-specific window transparency
    println!("Window transparency set to: {}", transparency);
    Ok(())
}

#[tauri::command]
pub async fn hide_window_from_capture(window: Window) -> Result<(), String> {
    enable_stealth_mode(window).await
}

#[tauri::command]
pub async fn show_window_in_capture(window: Window) -> Result<(), String> {
    disable_stealth_mode(window).await
}

// Store overlay window state
static OVERLAY_WINDOW_CREATED: Mutex<bool> = Mutex::new(false);


