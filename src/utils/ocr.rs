use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use gloo_utils::format::JsValueSerdeExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub questions: Vec<Question>,
    pub processed_at: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn detect_questions_from_screen() -> Result<OcrResult, String> {
    let args = JsValue::NULL;
    let result = invoke("detect_questions", args).await;
    
    result
        .into_serde::<OcrResult>()
        .map_err(|e| format!("Failed to parse OCR result: {}", e))
}

pub async fn process_screenshot(screenshot_data: Vec<u8>) -> Result<OcrResult, String> {
    let args = serde_wasm_bindgen::to_value(&screenshot_data)
        .map_err(|e| format!("Failed to serialize screenshot data: {}", e))?;
    
    let result = invoke("process_screenshot", args).await;
    
    result
        .into_serde::<OcrResult>()
        .map_err(|e| format!("Failed to parse OCR result: {}", e))
}

pub fn is_question_text(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    let question_indicators = [
        "?", "what", "how", "why", "when", "where", "who", "which", "can you", "could you",
        "would you", "should", "do you", "did you", "will you", "help", "explain", "define",
        "describe", "tell me", "show me"
    ];
    
    question_indicators.iter().any(|&indicator| text_lower.contains(indicator))
}

pub fn extract_context_around_question(full_text: &str, question: &str) -> String {
    if let Some(pos) = full_text.find(question) {
        let start = pos.saturating_sub(100);
        let end = (pos + question.len() + 100).min(full_text.len());
        full_text[start..end].to_string()
    } else {
        question.to_string()
    }
}