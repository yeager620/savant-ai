use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use pulldown_cmark::{Parser, html};
use gloo_timers::future::TimeoutFuture;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    content: String,
    is_user: bool,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaArgs {
    model: String,
    prompt: String,
    conversation_history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContextUsage {
    used_tokens: u32,
    max_tokens: u32,
    percentage: f32,
    prompt_tokens: u32,
    response_tokens: u32,
}


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn MinimalChat() -> impl IntoView {
    let (messages, set_messages) = signal(Vec::<ChatMessage>::new());
    let (input_text, set_input_text) = signal(String::new());
    let (streaming_content, set_streaming_content) = signal(String::new());
    let (is_streaming, set_is_streaming) = signal(false);
    let (context_usage, set_context_usage) = signal(ContextUsage { 
        used_tokens: 0, 
        max_tokens: 4096, 
        percentage: 0.0,
        prompt_tokens: 0,
        response_tokens: 0,
    });
    let (context_warning, set_context_warning) = signal(Option::<String>::None);
    

    let send_message = move |_: web_sys::MouseEvent| {
        let text = input_text.get().trim().to_string();
        if text.is_empty() || is_streaming.get() {
            return;
        }

        // Add user message
        let user_message = ChatMessage {
            content: text.clone(),
            is_user: true,
            timestamp: get_current_time(),
        };
        
        set_messages.update(|msgs| msgs.push(user_message));
        set_input_text.set(String::new());
        set_is_streaming.set(true);
        set_streaming_content.set(String::new());

        // Build conversation history
        let conversation_history = build_conversation_history(&messages.get());
        
        // Setup streaming event listener
        let set_streaming_content_clone = set_streaming_content.clone();
        let set_is_streaming_clone = set_is_streaming.clone();
        let set_messages_clone = set_messages.clone();
        let set_context_usage_clone = set_context_usage.clone();
        let set_context_warning_clone = set_context_warning.clone();
        
        spawn_local(async move {
            // Setup event listener for streaming
            setup_streaming_listener(
                set_streaming_content_clone,
                set_is_streaming_clone,
                set_messages_clone,
                set_context_usage_clone,
                set_context_warning_clone,
            ).await;
            
            // Start streaming request
            match send_to_ollama_streaming(text, conversation_history).await {
                Ok(_) => {
                    // Streaming started successfully - UI already set up
                }
                Err(err) => {
                    let error_message = ChatMessage {
                        content: format!("Error: {}", err),
                        is_user: false,
                        timestamp: get_current_time(),
                    };
                    set_messages.update(|msgs| msgs.push(error_message));
                    set_is_streaming.set(false);
                    set_streaming_content.set(String::new());
                }
            }
        });
    };

    let handle_keypress = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            // Trigger send by creating a synthetic mouse event
            let text = input_text.get().trim().to_string();
            if !text.is_empty() && !is_streaming.get() {
                // Add user message
                let user_message = ChatMessage {
                    content: text.clone(),
                    is_user: true,
                    timestamp: get_current_time(),
                };
                
                set_messages.update(|msgs| msgs.push(user_message));
                set_input_text.set(String::new());
                set_is_streaming.set(true);
                set_streaming_content.set(String::new());

                // Build conversation history
                let conversation_history = build_conversation_history(&messages.get());
                
                // Setup streaming event listener
                let set_streaming_content_clone = set_streaming_content.clone();
                let set_is_streaming_clone = set_is_streaming.clone();
                let set_messages_clone = set_messages.clone();
                let set_context_usage_clone = set_context_usage.clone();
                let set_context_warning_clone = set_context_warning.clone();
                
                spawn_local(async move {
                    // Setup event listener for streaming
                    setup_streaming_listener(
                        set_streaming_content_clone,
                        set_is_streaming_clone,
                        set_messages_clone,
                        set_context_usage_clone,
                        set_context_warning_clone,
                    ).await;
                    
                    // Start streaming request
                    match send_to_ollama_streaming(text, conversation_history).await {
                        Ok(_) => {
                            // Streaming started successfully - UI already set up
                        }
                        Err(err) => {
                            let error_message = ChatMessage {
                                content: format!("Error: {}", err),
                                is_user: false,
                                timestamp: get_current_time(),
                            };
                            set_messages.update(|msgs| msgs.push(error_message));
                            set_is_streaming.set(false);
                            set_streaming_content.set(String::new());
                        }
                    }
                });
            }
        }
    };

    view! {
        <div class="minimal-chat">
            <div class="chat-header">
                <h3>"Savant AI"</h3>
                <div class="header-right">
                    <div class="context-usage">
                        <span class="context-text">
                            {move || {
                                let usage = context_usage.get();
                                format!("{}% ({}/{})", 
                                    usage.percentage as u32,
                                    usage.used_tokens,
                                    usage.max_tokens
                                )
                            }}
                        </span>
                        <span class="context-breakdown">
                            {move || {
                                let usage = context_usage.get();
                                if usage.response_tokens > 0 {
                                    format!("P:{} R:{}", usage.prompt_tokens, usage.response_tokens)
                                } else {
                                    format!("P:{}", usage.prompt_tokens)
                                }
                            }}
                        </span>
                        <div class="context-bar">
                            <div 
                                class="context-fill"
                                style=move || format!("width: {}%", context_usage.get().percentage)
                            ></div>
                        </div>
                    </div>
                    <div class="status-indicator" class:streaming=is_streaming>
                        {move || if is_streaming.get() { "●" } else { "●" }}
                    </div>
                </div>
            </div>
            
            <Show when=move || context_warning.get().is_some()>
                <div class="context-warning">
                    <span class="warning-icon">"⚠"</span>
                    <span class="warning-text">
                        {move || context_warning.get().unwrap_or_default()}
                    </span>
                </div>
            </Show>
            
            <div class="chat-messages">
                <For
                    each=move || messages.get()
                    key=|msg| format!("{}-{}", msg.timestamp, msg.content.len())
                    children=move |message| {
                        view! {
                            <div class="message" class:user=message.is_user class:ai=!message.is_user>
                                <div class="message-content" inner_html=render_markdown(&message.content)>
                                </div>
                                <div class="message-time">
                                    {message.timestamp}
                                </div>
                            </div>
                        }
                    }
                />
                
                <Show when=move || is_streaming.get()>
                    <div class="message ai streaming">
                        <div class="message-content" inner_html=move || render_markdown(&streaming_content.get())>
                        </div>
                    </div>
                </Show>
                
            </div>
            
            <div class="chat-input">
                <textarea
                    placeholder="Ask a question..."
                    prop:value=move || input_text.get()
                    on:input=move |ev| {
                        set_input_text.set(event_target_value(&ev));
                    }
                    on:keypress=handle_keypress
                    disabled=move || is_streaming.get()
                ></textarea>
                <button
                    on:click=send_message
                    disabled=move || is_streaming.get() || input_text.get().trim().is_empty()
                >
                    "Send"
                </button>
            </div>
        </div>
    }
}

fn build_conversation_history(messages: &[ChatMessage]) -> String {
    messages.iter()
        .map(|msg| {
            let role = if msg.is_user { "User" } else { "AI" };
            format!("{}: {}", role, msg.content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn send_to_ollama_streaming(prompt: String, conversation_history: String) -> Result<(), String> {
    let args = OllamaArgs {
        model: "devstral".to_string(),
        prompt,
        conversation_history: Some(conversation_history),
    };
    
    let args_value = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    let result = invoke("query_ollama_streaming_simple", args_value).await;
    serde_wasm_bindgen::from_value::<()>(result)
        .map_err(|e| format!("Streaming error: {}", e))?;
    
    Ok(())
}

async fn setup_streaming_listener(
    set_streaming_content: WriteSignal<String>,
    set_is_streaming: WriteSignal<bool>,
    set_messages: WriteSignal<Vec<ChatMessage>>,
    set_context_usage: WriteSignal<ContextUsage>,
    set_context_warning: WriteSignal<Option<String>>,
) {
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
        async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
    }
    
    let handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(event) {
            if let Some(payload) = event_data.get("payload") {
                if let (Some(content), Some(done)) = (payload.get("content"), payload.get("done")) {
                    if let (Some(content_str), Some(is_done)) = (content.as_str(), done.as_bool()) {
                        if is_done {
                            // Final message - add to messages and stop streaming
                            let ai_message = ChatMessage {
                                content: content_str.to_string(),
                                is_user: false,
                                timestamp: get_current_time(),
                            };
                            set_messages.update(|msgs| msgs.push(ai_message));
                            set_is_streaming.set(false);
                            set_streaming_content.set(String::new());
                        } else {
                            // Update streaming content
                            set_streaming_content.set(content_str.to_string());
                        }
                    }
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    let _ = listen("ollama_stream", handler.as_ref().unchecked_ref()).await;
    handler.forget();
    
    // Setup context usage listener
    let context_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(event) {
            if let Some(payload) = event_data.get("payload") {
                if let Ok(usage) = serde_json::from_value::<ContextUsage>(payload.clone()) {
                    set_context_usage.set(usage);
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    let _ = listen("context_usage", context_handler.as_ref().unchecked_ref()).await;
    context_handler.forget();
    
    // Setup context truncation warning listener
    let warning_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(event) {
            if let Some(payload) = event_data.get("payload") {
                if let Some(message) = payload.get("message").and_then(|m| m.as_str()) {
                    set_context_warning.set(Some(message.to_string()));
                    // Auto-hide warning after 5 seconds
                    let warning_setter = set_context_warning.clone();
                    spawn_local(async move {
                        TimeoutFuture::new(5000).await;
                        warning_setter.set(None);
                    });
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    let _ = listen("context_truncated", warning_handler.as_ref().unchecked_ref()).await;
    warning_handler.forget();
}

fn render_markdown(content: &str) -> String {
    let parser = Parser::new(content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn get_current_time() -> String {
    let date = js_sys::Date::new_0();
    format!("{:02}:{:02}", 
        date.get_hours() as u32, 
        date.get_minutes() as u32
    )
}