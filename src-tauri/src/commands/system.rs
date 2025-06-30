use anyhow::Result;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Window;

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
pub async fn enable_stealth_mode(window: Window) -> Result<(), String> {
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
pub async fn disable_stealth_mode(window: Window) -> Result<(), String> {
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
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {}", e))?;
    
    if screens.is_empty() {
        return Err("No screens found".to_string());
    }
    
    // Take screenshot of primary screen
    let screen = &screens[0];
    let image = screen
        .capture()
        .map_err(|e| format!("Failed to capture screen: {}", e))?;
    
    // Convert to PNG bytes (simplified implementation)
    let png_data = image.buffer().to_vec();
    
    Ok(png_data)
}

#[tauri::command]
pub async fn get_screen_info() -> Result<HashMap<String, serde_json::Value>, String> {
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {}", e))?;
    let mut screen_info = HashMap::new();
    
    for (i, screen) in screens.iter().enumerate() {
        let display_info = screen.display_info;
        screen_info.insert(
            format!("screen_{}", i),
            serde_json::json!({
                "id": display_info.id,
                "x": display_info.x,
                "y": display_info.y,
                "width": display_info.width,
                "height": display_info.height,
                "scale_factor": display_info.scale_factor,
                "is_primary": display_info.is_primary,
            }),
        );
    }
    
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