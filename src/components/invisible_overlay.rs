use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;
use gloo_timers::future::TimeoutFuture;
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::utils::shared_types::{DetectedQuestion, StreamingResponse};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    fn listen(event: &str, handler: &js_sys::Function) -> js_sys::Promise;
}

#[component]
pub fn InvisibleOverlay() -> impl IntoView {
    let (detected_questions, set_detected_questions) = signal(Vec::<DetectedQuestion>::new());
    let (streaming_responses, set_streaming_responses) = signal(HashMap::<String, String>::new());
    let (is_scanning, set_is_scanning) = signal(false);

    // Start continuous scanning when component mounts
    Effect::new(move |_| {
        if is_scanning.get() {
            spawn_local(async move {
                start_continuous_scanning(set_detected_questions).await;
            });
        }
    });

    // Listen for streaming responses
    Effect::new(move |_| {
        spawn_local(async move {
            listen_for_streaming_responses(set_streaming_responses).await;
        });
    });

    // Auto-start scanning
    Effect::new(move |_| {
        spawn_local(async move {
            let _ = JsFuture::from(invoke("start_overlay_scanning", JsValue::NULL)).await.unwrap();
            set_is_scanning.set(true);
        });
    });

    view! {
        <div class="invisible-overlay-container">
            <For
                each=move || detected_questions.get()
                key=|q| q.id.clone()
                children=move |q: DetectedQuestion| {
                    let response_signal = Memo::new(move |_| {
                        q.response.get()
                    });
                    view! {
                        <div class="question-card">
                            <p class="question-text">{q.question.clone()}</p>
                            <p class="response-text">{response_signal}</p>
                        </div>
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn NeonQuestionBox(
    question: DetectedQuestion,
    response: Signal<String>,
) -> impl IntoView {
    let (is_visible, set_is_visible) = signal(true);
    let (fade_out, set_fade_out) = signal(false);

    // Auto-fade after response completion
    Effect::new(move |_| {
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
        let result = JsFuture::from(invoke("scan_for_questions", args.clone())).await.unwrap();
        if let Ok(result) = result.into_serde::<Vec<DetectedQuestion>>() {
            if !result.is_empty() {
                set_detected_questions.set(result.clone());
                
                // Trigger AI responses for each question
                for question in result {
                    let question_text = question.question.clone();
                    spawn_local(async move {
                        let _ = JsFuture::from(invoke("query_question", 
                            serde_wasm_bindgen::to_value(&question_text).unwrap()
                        )).await.unwrap();
                    });
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

    let _ = JsFuture::from(listen("streaming_response", handler.as_ref().unchecked_ref())).await;
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