mod commands;

use commands::*;
use tauri::{Manager, menu::{MenuBuilder, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent, MouseButton}};
use tauri::menu::MenuEvent;

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
            
            // Hide from dock and make main window invisible to screen capture
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy, NSWindow};
                use objc::runtime::Object;
                use objc::*;
                
                unsafe {
                    let app: cocoa::base::id = NSApp();
                    let _: () = msg_send![app, setActivationPolicy: NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory];
                }
            }
            
            if let Some(main_window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                {
                    use cocoa::appkit::NSWindow;
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
