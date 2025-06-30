use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use gloo_timers::future::TimeoutFuture;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedQuestion {
    pub id: String,
    pub text: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    pub question_id: String,
    pub content: String,
    pub is_complete: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[component]
pub fn InvisibleOverlay() -> impl IntoView {
    let (detected_questions, set_detected_questions) = signal(Vec::<DetectedQuestion>::new());
    let (streaming_responses, set_streaming_responses) = signal(HashMap::<String, String>::new());
    let (is_scanning, set_is_scanning) = signal(false);

    // Start continuous scanning when component mounts
    create_effect(move |_| {
        if is_scanning.get() {
            spawn_local(async move {
                start_continuous_scanning(set_detected_questions).await;
            });
        }
    });

    // Listen for streaming responses
    create_effect(move |_| {
        spawn_local(async move {
            listen_for_streaming_responses(set_streaming_responses).await;
        });
    });

    // Auto-start scanning
    create_effect(move |_| {
        spawn_local(async move {
            let _ = invoke("start_overlay_scanning", JsValue::NULL).await;
            set_is_scanning.set(true);
        });
    });

    view! {
        <div class="invisible-overlay-container">
            <For
                each=detected_questions
                key=|q| q.id.clone()
                children=move |question| {
                    let question_id = question.id.clone();
                    let response_signal = create_memo(move |_| {
                        streaming_responses.get().get(&question_id).cloned().unwrap_or_default()
                    });
                    
                    view! {
                        <NeonQuestionBox 
                            question=question
                            response=response_signal.into()
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
fn NeonQuestionBox(
    question: DetectedQuestion,
    response: Signal<String>,
) -> impl IntoView {
    let (is_visible, set_is_visible) = signal(true);
    let (fade_out, set_fade_out) = signal(false);

    // Auto-fade after response completion
    create_effect(move |_| {
        let response_text = response.get();
        if !response_text.is_empty() && response_text.ends_with("[COMPLETE]") {
            set_timeout(move || {
                set_fade_out.set(true);
                set_timeout(move || set_is_visible.set(false), 1000);
            }, 3000);
        }
    });

    let box_style = format!(
        "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px;",
        question.bounding_box.y,
        question.bounding_box.x,
        question.bounding_box.width,
        question.bounding_box.height
    );

    let response_style = format!(
        "position: absolute; top: {}px; left: {}px; max-width: 400px;",
        question.bounding_box.y + question.bounding_box.height + 10.0,
        question.bounding_box.x
    );

    view! {
        <div class="neon-question-container" class:visible=is_visible class:fade-out=fade_out>
            <div class="neon-question-box" style=box_style></div>
            <div class="neon-response-text" style=response_style>
                {move || response.get().replace("[COMPLETE]", "")}
            </div>
        </div>
    }
}

async fn start_continuous_scanning(set_detected_questions: WriteSignal<Vec<DetectedQuestion>>) {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    
    loop {
        if let Ok(result) = invoke("scan_for_questions", args.clone()).await {
            if let Ok(questions) = serde_wasm_bindgen::from_value::<Vec<DetectedQuestion>>(result) {
                if !questions.is_empty() {
                    set_detected_questions.set(questions.clone());
                    
                    // Trigger AI responses for each question
                    for question in questions {
                        let question_text = question.text.clone();
                        spawn_local(async move {
                            let _ = invoke("query_question", 
                                serde_wasm_bindgen::to_value(&question_text).unwrap()
                            ).await;
                        });
                    }
                }
            }
        }
        
        TimeoutFuture::new(500).await;
    }
}

async fn listen_for_streaming_responses(set_streaming_responses: WriteSignal<HashMap<String, String>>) {
    let handler = Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(response) = serde_wasm_bindgen::from_value::<StreamingResponse>(event) {
            set_streaming_responses.update(|responses| {
                responses.insert(response.question_id, response.content);
            });
        }
    }) as Box<dyn Fn(JsValue)>);

    let _ = listen("streaming_response", handler.as_ref().unchecked_ref()).await;
    handler.forget();
}

fn set_timeout<F>(f: F, ms: i32) 
where 
    F: FnOnce() + 'static 
{
    let closure = Closure::once_into_js(f);
    web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            ms,
        )
        .unwrap();
}