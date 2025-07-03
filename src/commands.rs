use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionListenerCallback {
    pub problem: savant_video::DetectedCodingProblem,
    pub solution: savant_video::GeneratedSolution,
}

pub async fn listen_for_solutions<F>(mut callback: F) -> Result<(), String>
where
    F: FnMut(savant_video::DetectedCodingProblem, savant_video::GeneratedSolution) + 'static,
{
    // Listen for solution events
    let solution_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(event) {
            if let Some(payload) = event_data.get("payload") {
                if let Ok(solution) = serde_json::from_value::<savant_video::GeneratedSolution>(payload.clone()) {
                    // We need to get the problem from somewhere - for now, create a placeholder
                    // In real implementation, the event would include both problem and solution
                    let problem = savant_video::DetectedCodingProblem {
                        id: solution.problem_id.clone(),
                        problem_type: savant_video::CodingProblemType::AlgorithmChallenge,
                        title: "Detected Problem".to_string(),
                        description: "A coding problem was detected on your screen".to_string(),
                        code_context: savant_video::CodeContext {
                            visible_code: String::new(),
                            focused_function: None,
                            imports: vec![],
                            class_context: None,
                            line_numbers: None,
                            cursor_position: None,
                            selected_text: None,
                        },
                        error_details: None,
                        platform: None,
                        language: savant_video::ProgrammingLanguage::Unknown,
                        starter_code: None,
                        test_cases: vec![],
                        constraints: vec![],
                        confidence: solution.confidence_score,
                        detected_at: solution.generated_at,
                        screen_region: savant_video::ScreenRegion {
                            x: 0,
                            y: 0,
                            width: 1920,
                            height: 1080,
                        },
                    };
                    
                    callback(problem, solution);
                }
            }
        }
    }) as Box<dyn FnMut(_)>);

    let _ = listen("solution-generated", solution_handler.as_ref().unchecked_ref()).await;
    solution_handler.forget();

    // Also listen for problem detection events
    let problem_handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: JsValue| {
        // Handle problem detection events
    }) as Box<dyn FnMut(_)>);

    let _ = listen("coding-problem-detected", problem_handler.as_ref().unchecked_ref()).await;
    problem_handler.forget();

    Ok(())
}

pub async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "text": text
    })).unwrap();
    
    let result = invoke("copy_to_clipboard", args).await;
    serde_wasm_bindgen::from_value::<String>(result)
        .map(|_| ())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}

pub async fn apply_solution(solution_code: &str) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "solution_code": solution_code
    })).unwrap();
    
    let result = invoke("apply_solution", args).await;
    serde_wasm_bindgen::from_value::<String>(result)
        .map(|_| ())
        .map_err(|e| format!("Failed to apply solution: {}", e))
}

pub async fn regenerate_solution() -> Result<(), String> {
    // For now, this is a placeholder
    Err("Solution regeneration not yet implemented".to_string())
}

pub async fn init_solution_processor(enable_auto_solutions: bool) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "enable_auto_solutions": enable_auto_solutions
    })).unwrap();
    
    let result = invoke("init_solution_processor", args).await;
    serde_wasm_bindgen::from_value::<String>(result)
        .map(|_| ())
        .map_err(|e| format!("Failed to initialize solution processor: {}", e))
}