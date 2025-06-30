mod commands;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Configuration commands
            get_config,
            save_config,
            reset_config,
            // OCR commands
            detect_questions,
            process_screenshot,
            start_overlay_scanning,
            stop_overlay_scanning,
            scan_for_questions,
            // LLM commands
            query_llm,
            get_available_models,
            test_api_connection,
            stream_response_for_question,
            query_question,
            // System commands
            enable_stealth_mode,
            disable_stealth_mode,
            take_screenshot,
            get_screen_info,
            set_window_always_on_top,
            set_window_transparency,
            hide_window_from_capture,
            show_window_in_capture,
            create_invisible_overlay,
            show_overlay_window,
            hide_overlay_window,
            toggle_overlay_window,
            // Hotkey commands
            register_global_hotkeys,
            unregister_global_hotkeys,
            get_overlay_state,
            set_overlay_state,
            test_hotkey
        ])
        .setup(|app| {
            // Initialize global hotkeys on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = register_global_hotkeys(app_handle).await {
                    eprintln!("Failed to register global hotkeys: {}", error);
                }
            });
            
            // Create invisible overlay window on startup
            let app_handle_overlay = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = create_invisible_overlay(app_handle_overlay).await {
                    eprintln!("Failed to create invisible overlay: {}", error);
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
