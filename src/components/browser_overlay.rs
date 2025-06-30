use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTab {
    pub id: String,
    pub title: String,
    pub url: String,
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
    pub tab_id: String,
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
    pub active_tab_id: Option<String>,
    pub tabs: std::collections::HashMap<String, BrowserTab>,
    pub top_prompts: Vec<DetectedPrompt>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[component]
pub fn BrowserOverlay() -> impl IntoView {
    let (browser_state, set_browser_state) = signal(BrowserState {
        is_connected: false,
        active_tab_id: None,
        tabs: std::collections::HashMap::new(),
        top_prompts: Vec::new(),
    });
    
    let (selected_index, set_selected_index) = signal(0usize);
    let (is_monitoring, set_is_monitoring) = signal(false);
    let (status_message, set_status_message) = signal(String::from("Browser monitoring not started"));

    // Setup event listeners for browser state updates
    spawn_local(async move {
        setup_browser_event_listeners(set_browser_state, set_status_message).await;
    });

    // Handle keyboard navigation
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
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
                        select_prompt(prompt_id).await;
                    });
                }
            }
            "Escape" => {
                ev.prevent_default();
                // Hide overlay or stop monitoring
                spawn_local(async move {
                    stop_browser_monitoring().await;
                });
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

    let start_monitoring = move |_| {
        spawn_local(async move {
            match start_browser_monitoring().await {
                Ok(_) => {
                    set_is_monitoring.set(true);
                    set_status_message.set("Browser monitoring started".to_string());
                }
                Err(e) => {
                    set_status_message.set(format!("Failed to start monitoring: {}", e));
                }
            }
        });
    };

    let stop_monitoring = move |_| {
        spawn_local(async move {
            match stop_browser_monitoring().await {
                Ok(_) => {
                    set_is_monitoring.set(false);
                    set_status_message.set("Browser monitoring stopped".to_string());
                }
                Err(e) => {
                    set_status_message.set(format!("Failed to stop monitoring: {}", e));
                }
            }
        });
    };

    view! {
        <div class="browser-overlay">
            <div class="browser-controls">
                <div class="control-header">
                    <h3>"Browser Assistant"</h3>
                    <div class="connection-status" class:connected=move || browser_state.get().is_connected>
                        {move || if browser_state.get().is_connected { "🟢 Connected" } else { "🔴 Disconnected" }}
                    </div>
                </div>
                
                <div class="control-buttons">
                    <button 
                        on:click=start_monitoring
                        disabled=move || is_monitoring.get()
                        class="start-btn"
                    >
                        "Start Monitoring"
                    </button>
                    <button 
                        on:click=stop_monitoring
                        disabled=move || !is_monitoring.get()
                        class="stop-btn"
                    >
                        "Stop Monitoring"
                    </button>
                </div>
                
                <div class="status-message">
                    {move || status_message.get()}
                </div>
            </div>

            <Show when=move || !browser_state.get().top_prompts.is_empty()>
                <div class="prompt-selection-overlay">
                    <div class="prompt-header">
                        <h4>"Detected Prompts"</h4>
                        <div class="navigation-hint">
                            "↑↓ Navigate • Enter Select • Esc Cancel"
                        </div>
                    </div>
                    
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
                                                select_prompt(prompt_id).await;
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
                                            <span class="tab-title">
                                                {move || {
                                                    browser_state.get().tabs
                                                        .get(&prompt.tab_id)
                                                        .map(|tab| tab.title.clone())
                                                        .unwrap_or_else(|| "Unknown Tab".to_string())
                                                }}
                                            </span>
                                        </div>
                                        <div class="prompt-context">
                                            {prompt.context.clone()}
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </Show>

            <Show when=move || browser_state.get().is_connected && browser_state.get().top_prompts.is_empty()>
                <div class="no-prompts-message">
                    <div class="scanning-indicator">
                        <div class="spinner"></div>
                        "Scanning browser tabs for prompts..."
                    </div>
                    <div class="tabs-info">
                        {move || {
                            let tab_count = browser_state.get().tabs.len();
                            format!("{} tabs being monitored", tab_count)
                        }}
                    </div>
                </div>
            </Show>
        </div>
    }
}

async fn setup_browser_event_listeners(
    set_browser_state: WriteSignal<BrowserState>,
    set_status_message: WriteSignal<String>,
) {
    // Listen for browser state updates
    let state_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(event) {
            if let Some(payload) = event_data.get("payload") {
                if let Ok(browser_state) = serde_json::from_value::<BrowserState>(payload.clone()) {
                    console::log_1(&format!("Browser state updated: {} tabs, {} prompts", 
                        browser_state.tabs.len(), 
                        browser_state.top_prompts.len()).into());
                    set_browser_state.set(browser_state);
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    let _ = listen("browser_state_updated", state_handler.as_ref().unchecked_ref()).await;
    state_handler.forget();

    // Listen for prompt selection events
    let prompt_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(event) {
            if let Some(payload) = event_data.get("payload") {
                if let Ok(prompt) = serde_json::from_value::<DetectedPrompt>(payload.clone()) {
                    console::log_1(&format!("Prompt selected: {}", prompt.text).into());
                    set_status_message.set(format!("Selected: {}", prompt.text));
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    let _ = listen("prompt_selected", prompt_handler.as_ref().unchecked_ref()).await;
    prompt_handler.forget();
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