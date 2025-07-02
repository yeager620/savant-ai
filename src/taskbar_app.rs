use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::components::{MinimalChat, NaturalQueryInterface};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserWindow {
    pub id: String,
    pub title: String,
    pub app_name: String,
    pub is_active: bool,
    pub content: String,
    pub detected_prompts: Vec<DetectedPrompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPrompt {
    pub id: String,
    pub text: String,
    pub confidence: f32,
    pub priority: f32,
    pub window_id: String,
    pub context: String,
    pub position: PromptPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub is_connected: bool,
    pub active_window_id: Option<String>,
    pub windows: std::collections::HashMap<String, BrowserWindow>,
    pub top_prompts: Vec<DetectedPrompt>,
}

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Chat,
    Browser,
    Database,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[component]
pub fn TaskbarApp() -> impl IntoView {
    let (app_mode, set_app_mode) = signal(AppMode::Chat);
    let (browser_state, set_browser_state) = signal(BrowserState {
        is_connected: false,
        active_window_id: None,
        windows: std::collections::HashMap::new(),
        top_prompts: Vec::new(),
    });
    let (selected_index, set_selected_index) = signal(0usize);
    let (is_monitoring, set_is_monitoring) = signal(false);
    
    // Setup browser event listeners
    spawn_local(async move {
        setup_browser_event_listeners(set_browser_state).await;
    });
    
    // Handle keyboard navigation for browser mode
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if app_mode.get() != AppMode::Browser {
            return;
        }
        
        let prompts = browser_state.get().top_prompts;
        if prompts.is_empty() {
            return;
        }

        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                set_selected_index.update(|idx| {
                    *idx = (*idx + 1).min(prompts.len().saturating_sub(1));
                });
            }
            "ArrowUp" => {
                ev.prevent_default();
                set_selected_index.update(|idx| {
                    *idx = idx.saturating_sub(1);
                });
            }
            "Enter" => {
                ev.prevent_default();
                let current_idx = selected_index.get();
                if let Some(prompt) = prompts.get(current_idx) {
                    let prompt_id = prompt.id.clone();
                    spawn_local(async move {
                        let _ = select_prompt(prompt_id).await;
                    });
                }
            }
            "Escape" => {
                ev.prevent_default();
                set_app_mode.set(AppMode::Chat);
            }
            _ => {}
        }
    };
    
    // Add global keydown event listener
    spawn_local(async move {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(handle_keydown) as Box<dyn FnMut(_)>);
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    });
    
    // Listen for browser toggle events
    let toggle_mode = set_app_mode.clone();
    let get_mode = app_mode.clone();
    spawn_local(async move {
        let window = web_sys::window().unwrap();
        
        let toggle_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            let current_mode = get_mode.get_untracked();
            match current_mode {
                AppMode::Chat => toggle_mode.set(AppMode::Browser),
                AppMode::Browser => toggle_mode.set(AppMode::Chat),
            }
        }) as Box<dyn FnMut(_)>);
        
        window
            .add_event_listener_with_callback("toggle_browser_mode", toggle_handler.as_ref().unchecked_ref())
            .unwrap();
        toggle_handler.forget();
    });
    
    // Auto-start monitoring when switching to browser mode
    let _effect = Effect::new(move |_| {
        let mode = app_mode.get();
        if mode == AppMode::Browser && !is_monitoring.get() {
            spawn_local(async move {
                if let Ok(_) = start_browser_monitoring().await {
                    set_is_monitoring.set(true);
                }
            });
        } else if mode == AppMode::Chat && is_monitoring.get() {
            spawn_local(async move {
                let _ = stop_browser_monitoring().await;
                set_is_monitoring.set(false);
            });
        }
    });
    view! {
        <div class="taskbar-app">
            <Show when=move || app_mode.get() == AppMode::Chat>
                <MinimalChat 
                    on_browser_mode={
                        let set_app_mode = set_app_mode.clone();
                        Some(std::sync::Arc::new(move || set_app_mode.set(AppMode::Browser)))
                    }
                    on_database_mode={
                        let set_app_mode = set_app_mode.clone();
                        Some(std::sync::Arc::new(move || set_app_mode.set(AppMode::Database)))
                    }
                />
            </Show>
            
            <Show when=move || app_mode.get() == AppMode::Database>
                <div class="database-mode">
                    <div class="database-header">
                        <h3>"Database Query Interface"</h3>
                    </div>
                    <div class="database-content">
                        <NaturalQueryInterface />
                    </div>
                    <div class="database-controls">
                        <button 
                            class="back-btn"
                            on:click=move |_| set_app_mode.set(AppMode::Chat)
                        >
                            "← Back to Chat"
                        </button>
                    </div>
                </div>
            </Show>
            
            <Show when=move || app_mode.get() == AppMode::Browser>
                <div class="browser-mode">
                    <div class="browser-header">
                        <h3>"Browser Assistant"</h3>
                        <div class="browser-status">
                            {move || if browser_state.get().is_connected { "🟢" } else { "🔴" }}
                        </div>
                    </div>
                    
                    <Show when=move || !browser_state.get().top_prompts.is_empty()>
                        <div class="prompt-list">
                            <For
                                each=move || browser_state.get().top_prompts.clone().into_iter().enumerate()
                                key=|(i, prompt)| (i.clone(), prompt.id.clone())
                                children=move |(index, prompt)| {
                                    let is_selected = move || selected_index.get() == index;
                                    let prompt_clone = prompt.clone();
                                    
                                    view! {
                                        <div 
                                            class="prompt-item"
                                            class:selected=is_selected
                                            on:click=move |_| {
                                                set_selected_index.set(index);
                                                let prompt_id = prompt_clone.id.clone();
                                                spawn_local(async move {
                                                    let _ = select_prompt(prompt_id).await;
                                                });
                                            }
                                        >
                                            <div class="prompt-text">
                                                {prompt.text.clone()}
                                            </div>
                                            <div class="prompt-meta">
                                                <span class="confidence">
                                                    {format!("{}%", (prompt.confidence * 100.0) as i32)}
                                                </span>
                                                <span class="window-title">
                                                    {move || {
                                                        browser_state.get().windows
                                                            .get(&prompt.window_id)
                                                            .map(|window| format!("{}: {}", window.app_name, window.title))
                                                            .unwrap_or_else(|| "Unknown".to_string())
                                                    }}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </Show>
                    
                    <Show when=move || browser_state.get().is_connected && browser_state.get().top_prompts.is_empty()>
                        <div class="scanning-message">
                            <div class="spinner"></div>
                            "Scanning for prompts..."
                        </div>
                    </Show>
                    
                    <Show when=move || !browser_state.get().is_connected>
                        <div class="connection-message">
                            "Enable Accessibility permissions:\nSystem Preferences > Security & Privacy > Privacy > Accessibility\n\nAdd Savant AI to the list and check the box."
                        </div>
                    </Show>
                    
                    <div class="browser-controls">
                        <button 
                            class="back-btn"
                            on:click=move |_| set_app_mode.set(AppMode::Chat)
                        >
                            "← Back to Chat"
                        </button>
                    </div>
                </div>
            </Show>
            
            // Taskbar-specific CSS
            <style>
                "
                * {
                    margin: 0;
                    padding: 0;
                    box-sizing: border-box;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                }

                body {
                    background: transparent;
                    color: #ffffff;
                    font-size: 13px;
                    overflow: hidden;
                }

                .taskbar-app {
                    width: 100vw;
                    height: 100vh;
                    display: flex;
                    flex-direction: column;
                    background: rgba(16, 16, 16, 0.75);
                    backdrop-filter: blur(20px) saturate(180%);
                    border-right: 1px solid rgba(255, 255, 255, 0.1);
                }

                .minimal-chat {
                    display: flex;
                    flex-direction: column;
                    height: 100%;
                    padding: 12px;
                }

                .chat-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 8px 0;
                    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                    margin-bottom: 12px;
                    position: relative;
                }
                
                .browser-toggle {
                    position: absolute;
                    top: -2px;
                    right: 120px;
                    background: rgba(255, 255, 255, 0.1);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 4px;
                    padding: 2px 6px;
                    color: rgba(255, 255, 255, 0.8);
                    font-size: 10px;
                    cursor: pointer;
                    transition: all 0.2s ease;
                }
                
                .browser-toggle:hover {
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                }
                
                .header-right {
                    display: flex;
                    align-items: center;
                    gap: 12px;
                }
                
                .context-usage {
                    display: flex;
                    flex-direction: column;
                    align-items: flex-end;
                    gap: 2px;
                }
                
                .context-text {
                    font-size: 10px;
                    color: rgba(255, 255, 255, 0.7);
                    font-weight: 500;
                }
                
                .context-breakdown {
                    font-size: 8px;
                    color: rgba(255, 255, 255, 0.5);
                    font-weight: 400;
                    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
                }
                
                .context-bar {
                    width: 60px;
                    height: 3px;
                    background: rgba(255, 255, 255, 0.2);
                    border-radius: 2px;
                    overflow: hidden;
                }
                
                .context-fill {
                    height: 100%;
                    background: linear-gradient(90deg, #10b981 0%, #f59e0b 70%, #ef4444 100%);
                    border-radius: 2px;
                    transition: width 0.3s ease;
                }
                
                .context-warning {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    padding: 8px 12px;
                    margin-bottom: 8px;
                    background: rgba(245, 158, 11, 0.1);
                    border: 1px solid rgba(245, 158, 11, 0.3);
                    border-radius: 6px;
                    animation: slide-in 0.3s ease-out;
                }
                
                .warning-icon {
                    color: #f59e0b;
                    font-size: 14px;
                    font-weight: bold;
                }
                
                .warning-text {
                    color: rgba(255, 255, 255, 0.9);
                    font-size: 11px;
                    line-height: 1.3;
                }
                
                @keyframes slide-in {
                    from {
                        opacity: 0;
                        transform: translateY(-10px);
                    }
                    to {
                        opacity: 1;
                        transform: translateY(0);
                    }
                }

                .chat-header h3 {
                    font-size: 14px;
                    font-weight: 600;
                    color: #ffffff;
                }

                .status-indicator {
                    font-size: 8px;
                    color: #10b981;
                    animation: pulse 2s infinite;
                }

                .status-indicator.streaming {
                    color: #00ff41;
                    animation: streaming-pulse 0.8s infinite;
                }

                @keyframes pulse {
                    0%, 100% { opacity: 1; }
                    50% { opacity: 0.5; }
                }

                @keyframes streaming-pulse {
                    0%, 100% { opacity: 0.6; }
                    50% { opacity: 1; }
                }

                .chat-messages {
                    flex: 1;
                    overflow-y: auto;
                    padding: 4px 0;
                    margin-bottom: 12px;
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                }

                .chat-messages::-webkit-scrollbar {
                    width: 4px;
                }

                .chat-messages::-webkit-scrollbar-track {
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 2px;
                }

                .chat-messages::-webkit-scrollbar-thumb {
                    background: rgba(255, 255, 255, 0.2);
                    border-radius: 2px;
                }

                .message {
                    padding: 8px 10px;
                    border-radius: 8px;
                    max-width: 100%;
                    word-wrap: break-word;
                    position: relative;
                }

                .message.user {
                    background: rgba(59, 130, 246, 0.15);
                    border: 1px solid rgba(59, 130, 246, 0.3);
                    align-self: flex-end;
                    margin-left: 20px;
                }

                .message.ai {
                    background: rgba(16, 185, 129, 0.15);
                    border: 1px solid rgba(16, 185, 129, 0.3);
                    align-self: flex-start;
                    margin-right: 20px;
                }

                .message.typing {
                    background: rgba(107, 114, 128, 0.15);
                    border: 1px solid rgba(107, 114, 128, 0.3);
                }
                
                .message.streaming {
                    background: rgba(16, 185, 129, 0.15);
                    border: 1px solid rgba(16, 185, 129, 0.3);
                    align-self: flex-start;
                    margin-right: 20px;
                }

                .message-content {
                    font-size: 12px;
                    line-height: 1.4;
                    color: #ffffff;
                }
                
                .message-content p {
                    margin: 0 0 8px 0;
                }
                
                .message-content p:last-child {
                    margin-bottom: 0;
                }
                
                .message-content code {
                    background: rgba(255, 255, 255, 0.1);
                    padding: 2px 4px;
                    border-radius: 3px;
                    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
                    font-size: 11px;
                }
                
                .message-content pre {
                    background: rgba(255, 255, 255, 0.05);
                    padding: 8px;
                    border-radius: 4px;
                    overflow-x: auto;
                    margin: 4px 0;
                }
                
                .message-content pre code {
                    background: none;
                    padding: 0;
                }

                .message-time {
                    font-size: 10px;
                    color: rgba(255, 255, 255, 0.5);
                    margin-top: 4px;
                    text-align: right;
                }

                .typing-indicator {
                    display: flex;
                    gap: 2px;
                    align-items: center;
                }

                .typing-indicator span {
                    width: 4px;
                    height: 4px;
                    background: rgba(255, 255, 255, 0.6);
                    border-radius: 50%;
                    animation: typing 1.4s infinite ease-in-out;
                }

                .typing-indicator span:nth-child(1) { animation-delay: 0s; }
                .typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
                .typing-indicator span:nth-child(3) { animation-delay: 0.4s; }

                @keyframes typing {
                    0%, 60%, 100% {
                        transform: translateY(0);
                        opacity: 0.4;
                    }
                    30% {
                        transform: translateY(-10px);
                        opacity: 1;
                    }
                }

                .chat-input {
                    display: flex;
                    gap: 8px;
                    align-items: flex-end;
                }

                .chat-input textarea {
                    flex: 1;
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 6px;
                    padding: 8px 10px;
                    color: #ffffff;
                    font-size: 12px;
                    resize: none;
                    min-height: 36px;
                    max-height: 100px;
                    font-family: inherit;
                }

                .chat-input textarea::placeholder {
                    color: rgba(255, 255, 255, 0.4);
                }

                .chat-input textarea:focus {
                    outline: none;
                    border-color: rgba(59, 130, 246, 0.6);
                    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
                }

                .chat-input textarea:disabled {
                    opacity: 0.5;
                    cursor: not-allowed;
                }

                .chat-input button {
                    background: rgba(59, 130, 246, 0.8);
                    border: none;
                    border-radius: 6px;
                    padding: 8px 12px;
                    color: #ffffff;
                    font-size: 11px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.2s ease;
                    min-width: 50px;
                    height: 36px;
                }

                .chat-input button:hover:not(:disabled) {
                    background: rgba(59, 130, 246, 1);
                    transform: translateY(-1px);
                }

                .chat-input button:disabled {
                    opacity: 0.5;
                    cursor: not-allowed;
                    transform: none;
                }

                .chat-input button:active {
                    transform: translateY(0);
                }
                
                /* Database Mode Styles */
                .database-mode {
                    display: flex;
                    flex-direction: column;
                    height: 100%;
                    padding: 12px;
                }
                
                .database-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 8px 0;
                    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                    margin-bottom: 12px;
                }
                
                .database-header h3 {
                    font-size: 14px;
                    font-weight: 600;
                    color: #ffffff;
                    margin: 0;
                }
                
                .database-content {
                    flex: 1;
                    overflow: hidden;
                }
                
                .database-controls {
                    margin-top: auto;
                    padding-top: 12px;
                }
                
                .database-toggle {
                    background: rgba(79, 70, 229, 0.8) !important;
                }
                
                .database-toggle:hover {
                    background: rgba(79, 70, 229, 1) !important;
                }
                
                /* Browser Mode Styles */
                .browser-mode {
                    display: flex;
                    flex-direction: column;
                    height: 100%;
                    padding: 12px;
                }
                
                .browser-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 8px 0;
                    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                    margin-bottom: 12px;
                }
                
                .browser-header h3 {
                    font-size: 14px;
                    font-weight: 600;
                    color: #ffffff;
                    margin: 0;
                }
                
                .browser-status {
                    font-size: 12px;
                }
                
                .prompt-list {
                    flex: 1;
                    overflow-y: auto;
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                    margin-bottom: 12px;
                }
                
                .prompt-item {
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 6px;
                    padding: 8px;
                    cursor: pointer;
                    transition: all 0.2s ease;
                }
                
                .prompt-item:hover {
                    background: rgba(255, 255, 255, 0.1);
                    border-color: rgba(0, 255, 65, 0.3);
                }
                
                .prompt-item.selected {
                    background: rgba(0, 255, 65, 0.1);
                    border-color: #00ff41;
                    box-shadow: 0 1px 4px rgba(0, 255, 65, 0.2);
                }
                
                .prompt-text {
                    font-size: 11px;
                    font-weight: 500;
                    margin-bottom: 4px;
                    line-height: 1.3;
                    color: white;
                }
                
                .prompt-meta {
                    display: flex;
                    justify-content: space-between;
                    font-size: 9px;
                    opacity: 0.7;
                }
                
                .confidence {
                    color: #4CAF50;
                    font-weight: 500;
                }
                
                .window-title {
                    color: #2196F3;
                    max-width: 120px;
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                
                .scanning-message {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 8px;
                    padding: 20px;
                    text-align: center;
                    font-size: 11px;
                    opacity: 0.7;
                }
                
                .connection-message {
                    padding: 12px;
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 6px;
                    font-size: 10px;
                    line-height: 1.3;
                    white-space: pre-line;
                    margin-bottom: 12px;
                    opacity: 0.8;
                }
                
                .spinner {
                    width: 12px;
                    height: 12px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-top: 1px solid #00ff41;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                }
                
                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }
                
                .browser-controls {
                    margin-top: auto;
                }
                
                .back-btn {
                    width: 100%;
                    background: rgba(255, 255, 255, 0.1);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 6px;
                    padding: 8px;
                    color: white;
                    font-size: 11px;
                    cursor: pointer;
                    transition: all 0.2s ease;
                }
                
                .back-btn:hover {
                    background: rgba(255, 255, 255, 0.15);
                }
                "
            </style>
        </div>
    }
}

async fn setup_browser_event_listeners(set_browser_state: WriteSignal<BrowserState>) {
    // Listen for browser state updates
    let state_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(event) {
            if let Some(payload) = event_data.get("payload") {
                if let Ok(browser_state) = serde_json::from_value::<BrowserState>(payload.clone()) {
                    console::log_1(&format!("Browser state updated: {} windows, {} prompts", 
                        browser_state.windows.len(), 
                        browser_state.top_prompts.len()).into());
                    set_browser_state.set(browser_state);
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    let _ = listen("browser_state_updated", state_handler.as_ref().unchecked_ref()).await;
    state_handler.forget();
}

async fn start_browser_monitoring() -> Result<(), String> {
    let result = invoke("start_browser_monitoring", serde_wasm_bindgen::to_value(&()).unwrap()).await;
    serde_wasm_bindgen::from_value::<()>(result)
        .map_err(|e| format!("Failed to start browser monitoring: {}", e))
}

async fn stop_browser_monitoring() -> Result<(), String> {
    let result = invoke("stop_browser_monitoring", serde_wasm_bindgen::to_value(&()).unwrap()).await;
    serde_wasm_bindgen::from_value::<()>(result)
        .map_err(|e| format!("Failed to stop browser monitoring: {}", e))
}

async fn select_prompt(prompt_id: String) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "prompt_id": prompt_id
    })).unwrap();
    
    let result = invoke("select_prompt", args).await;
    serde_wasm_bindgen::from_value::<()>(result)
        .map_err(|e| format!("Failed to select prompt: {}", e))
}