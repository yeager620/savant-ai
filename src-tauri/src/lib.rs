mod commands;

use commands::*;
use reqwest::Client;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, menu::{MenuBuilder, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent, MouseButton}};
use tokio::sync::Mutex;
use tokio::time::timeout;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Configuration commands
            get_config,
            save_config,
            reset_config,
            // Browser monitoring commands
            start_browser_monitoring,
            stop_browser_monitoring,
            get_browser_state,
            select_prompt,
            set_active_window,
            // LLM commands
            query_llm,
            get_available_models,
            test_api_connection,
            query_ollama_simple,
            query_ollama_streaming_simple,
            query_ollama_chat_streaming,
            calculate_context_usage_command,
            // Chat history commands
            save_chat_history,
            load_chat_history,
            clear_chat_history,
            // System commands
            enable_stealth_mode,
            disable_stealth_mode,
            take_screenshot,
            get_screen_info,
            set_window_always_on_top,
            set_window_transparency,
            hide_window_from_capture,
            show_window_in_capture,
            // Hotkey commands
            register_global_hotkeys,
            unregister_global_hotkeys,
            get_overlay_state,
            set_overlay_state,
            test_hotkey,
            // System audio commands
            check_system_audio_status,
            setup_system_audio,
            start_audio_daemon,
            stop_audio_daemon,
            list_captures,
            search_captures,
            get_daemon_logs,
            // LLM database commands
            natural_language_query,
            start_mcp_server,
            get_mcp_server_status,
            test_database_connection,
            get_database_stats,
            search_conversations,
            analyze_conversation,
            list_speakers_with_stats
        ])
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match init_database(&app_handle).await {
                    Ok(_) => println!("Database initialized successfully"),
                    Err(e) => eprintln!("Failed to initialize database: {}", e),
                }
            });
            // Create system tray
            let show = MenuItem::with_id(app, "show", "Show Savant AI", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show, &hide, &quit])
                .build()?;
            
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        if let Some(app) = tray.app_handle().get_webview_window("main") {
                            if app.is_visible().unwrap_or(false) {
                                let _ = app.hide();
                            } else {
                                let _ = app.show();
                                let _ = app.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;
            
            // Initialize global hotkeys on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = register_global_hotkeys(app_handle).await {
                    eprintln!("Failed to register global hotkeys: {}", error);
                }
            });
            
            // Check and start Ollama if needed
            tauri::async_runtime::spawn(async move {
                ensure_ollama_running().await;
            });
            
            // Hide from dock and make main window invisible to screen capture
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::{NSApp, NSApplicationActivationPolicy};
                use objc::*;
                
                unsafe {
                    let app: cocoa::base::id = NSApp();
                    let _: () = msg_send![app, setActivationPolicy: NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory];
                }
            }
            
            if let Some(main_window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                {
                    use objc::runtime::Object;
                    use objc::*;
                    
                    if let Ok(ns_window) = main_window.ns_window() {
                        unsafe {
                            let ns_window_ptr = ns_window as *mut Object;
                            let _: () = msg_send![ns_window_ptr, setSharingType: 0]; // NSWindowSharingNone = 0
                        }
                    }
                }
                
                #[cfg(target_os = "windows")]
                {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    
                    if let Ok(hwnd) = main_window.hwnd() {
                        unsafe {
                            let hwnd = HWND(hwnd.0);
                            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                            SetWindowLongW(hwnd, GWL_EXSTYLE, 
                                ex_style | WS_EX_TOOLWINDOW.0 as i32);
                        }
                    }
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn ensure_ollama_running() {
    println!("Checking if Ollama is running...");
    
    // First check if Ollama is already running
    if is_ollama_running().await {
        println!("Ollama is already running");
        return;
    }
    
    println!("Ollama is not running, attempting to start it...");
    
    // Try to start Ollama
    #[cfg(target_os = "macos")]
    {
        // Try different ways to start Ollama on macOS
        let start_commands = [
            // Direct binary call
            "ollama serve",
            // Using launchctl if it's installed as a service
            "launchctl kickstart -k gui/$(id -u)/com.ollama.ollama",
            // Using brew services
            "brew services start ollama",
        ];
        
        for cmd in &start_commands {
            println!("Trying to start Ollama with: {}", cmd);
            let mut command = Command::new("sh");
            command.arg("-c").arg(cmd);
            
            match command.spawn() {
                Ok(mut child) => {
                    // Give it a moment to start
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    
                    // Check if Ollama is now running
                    if is_ollama_running().await {
                        println!("Successfully started Ollama with: {}", cmd);
                        return;
                    }
                    
                    // Kill the process if it didn't work
                    let _ = child.kill();
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", cmd, e);
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Try to start Ollama on Windows
        let start_commands = [
            "ollama serve",
            "C:\\Users\\%USERNAME%\\AppData\\Local\\Programs\\Ollama\\ollama.exe serve",
        ];
        
        for cmd in &start_commands {
            println!("Trying to start Ollama with: {}", cmd);
            let mut command = Command::new("cmd");
            command.arg("/C").arg(cmd);
            
            match command.spawn() {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    if is_ollama_running().await {
                        println!("Successfully started Ollama with: {}", cmd);
                        return;
                    }
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", cmd, e);
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try to start Ollama on Linux
        let start_commands = [
            "ollama serve",
            "systemctl --user start ollama",
            "/usr/local/bin/ollama serve",
        ];
        
        for cmd in &start_commands {
            println!("Trying to start Ollama with: {}", cmd);
            let mut command = Command::new("sh");
            command.arg("-c").arg(cmd);
            
            match command.spawn() {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    if is_ollama_running().await {
                        println!("Successfully started Ollama with: {}", cmd);
                        return;
                    }
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", cmd, e);
                }
            }
        }
    }
    
    println!("Could not start Ollama automatically. Please start it manually with 'ollama serve'");
}

async fn is_ollama_running() -> bool {
    let client = Client::new();
    
    match timeout(Duration::from_secs(5), client.get("http://localhost:11434/api/tags").send()).await {
        Ok(Ok(response)) => response.status().is_success(),
        _ => false,
    }
}

/// Initialize the database and set up shared state
async fn init_database(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the transcript database
    let database = Arc::new(savant_db::TranscriptDatabase::new(None).await?);
    
    // Create MCP server state (initially empty)
    let mcp_server_state: MCPServerState = Arc::new(Mutex::new(None));
    
    // Store the database in Tauri's state management
    app.manage(database);
    app.manage(mcp_server_state);
    
    println!("Database and MCP state initialized successfully");
    Ok(())
}
