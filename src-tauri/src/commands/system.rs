use anyhow::Result;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Window, WebviewWindowBuilder, WebviewUrl};

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

// Store overlay window state
static OVERLAY_WINDOW_CREATED: Mutex<bool> = Mutex::new(false);

#[tauri::command]
pub async fn create_invisible_overlay(app: AppHandle) -> Result<(), String> {
    let mut overlay_created = OVERLAY_WINDOW_CREATED.lock().unwrap();
    if *overlay_created {
        return Ok(());
    }

    let overlay_window = WebviewWindowBuilder::new(
        &app,
        "overlay",
        WebviewUrl::App("/overlay".into())
    )
    .title("Savant AI Overlay")
    .fullscreen(true)
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .visible(false)
    .build()
    .map_err(|e| format!("Failed to create overlay window: {}", e))?;

    // Platform-specific invisibility settings
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSWindow;
        use objc::runtime::Object;
        use objc::*;
        
        if let Ok(ns_window) = overlay_window.ns_window() {
            unsafe {
                let ns_window_ptr = ns_window as *mut Object;
                let _: () = msg_send![ns_window_ptr, setSharingType: 0]; // NSWindowSharingNone = 0
                let _: () = msg_send![ns_window_ptr, setIgnoresMouseEvents: true];
                let _: () = msg_send![ns_window_ptr, setLevel: 25]; // Float above all other windows
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::Foundation::HWND;
        
        if let Ok(hwnd) = overlay_window.hwnd() {
            unsafe {
                let hwnd = HWND(hwnd.0);
                // Make window click-through and invisible to capture
                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                SetWindowLongW(hwnd, GWL_EXSTYLE, 
                    ex_style | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32 | WS_EX_TOOLWINDOW.0 as i32);
            }
        }
    }

    // Start the overlay as visible but transparent
    overlay_window.show().map_err(|e| format!("Failed to show overlay: {}", e))?;
    
    *overlay_created = true;
    Ok(())
}

#[tauri::command]
pub async fn show_overlay_window(app: AppHandle) -> Result<(), String> {
    if let Some(overlay_window) = app.get_webview_window("overlay") {
        overlay_window.show().map_err(|e| format!("Failed to show overlay: {}", e))?;
    } else {
        create_invisible_overlay(app).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_overlay_window(app: AppHandle) -> Result<(), String> {
    if let Some(overlay_window) = app.get_webview_window("overlay") {
        overlay_window.hide().map_err(|e| format!("Failed to hide overlay: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_overlay_window(app: AppHandle) -> Result<bool, String> {
    if let Some(overlay_window) = app.get_webview_window("overlay") {
        let is_visible = overlay_window.is_visible().map_err(|e| format!("Failed to check visibility: {}", e))?;
        
        if is_visible {
            overlay_window.hide().map_err(|e| format!("Failed to hide overlay: {}", e))?;
            Ok(false)
        } else {
            overlay_window.show().map_err(|e| format!("Failed to show overlay: {}", e))?;
            Ok(true)
        }
    } else {
        create_invisible_overlay(app).await?;
        Ok(true)
    }
}