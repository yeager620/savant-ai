use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Emitter};
use log::{debug, info, error};
use tokio::sync::Mutex;
use std::sync::Arc;

use savant_video::{
    IntegratedProcessor, ProcessorConfig, ProcessingEvent,
    DetectedCodingProblem, GeneratedSolution,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionEvent {
    pub problem: DetectedCodingProblem,
    pub solution: GeneratedSolution,
}

#[derive(Debug)]
pub struct SolutionState {
    pub processor: Option<Arc<Mutex<IntegratedProcessor>>>,
    pub event_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<ProcessingEvent>>,
}

impl Default for SolutionState {
    fn default() -> Self {
        Self {
            processor: None,
            event_receiver: None,
        }
    }
}

#[tauri::command]
pub async fn init_solution_processor(
    app: AppHandle,
    enable_auto_solutions: bool,
) -> Result<String, String> {
    info!("Initializing solution processor");

    let state = app.state::<Arc<Mutex<SolutionState>>>();
    let mut state_guard = state.lock().await;

    // Create processor config
    let config = ProcessorConfig {
        enable_ocr: true,
        enable_vision: true,
        enable_real_time_analysis: true,
        enable_problem_detection: true,
        enable_auto_solutions,
        min_change_threshold: 0.05,
        processing_timeout_ms: 5000,
    };

    // Get LLM provider from app state
    // Create a mock LLM provider for now
    let mock_provider = savant_video::llm_provider::MockLLMProvider::new();
    let llm_provider = savant_video::llm_provider::LLMProvider::Mock(mock_provider);

    // Get database pool
    let db_path = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("savant-ai")
        .join("visual_data.db");

    let db_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite:{}", db_path.display()))
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Create processor
    match IntegratedProcessor::new(config, llm_provider, db_pool).await {
        Ok((processor, mut event_rx)) => {
            let processor = Arc::new(Mutex::new(processor));
            state_guard.processor = Some(processor.clone());

            // Spawn task to handle events
            let app_handle = app.clone();
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    match event {
                        ProcessingEvent::CodingProblemDetected { problem } => {
                            info!("Coding problem detected: {:?}", problem.problem_type);
                            let _ = app_handle.emit("coding-problem-detected", &problem);
                        }
                        ProcessingEvent::SolutionGenerated { solution, problem_id } => {
                            info!("Solution generated for problem: {}", problem_id);
                            let _ = app_handle.emit("solution-generated", &solution);
                        }
                        ProcessingEvent::TaskDetected { task } => {
                            debug!("Task detected: {:?}", task.task_type);
                            let _ = app_handle.emit("task-detected", &task);
                        }
                        ProcessingEvent::QuestionDetected { question } => {
                            debug!("Question detected: {:?}", question.question_type);
                            let _ = app_handle.emit("question-detected", &question);
                        }
                        _ => {}
                    }
                }
            });

            Ok("Solution processor initialized successfully".to_string())
        }
        Err(e) => {
            error!("Failed to initialize processor: {}", e);
            Err(format!("Failed to initialize processor: {}", e))
        }
    }
}

#[tauri::command]
pub async fn process_screen_capture(
    app: AppHandle,
    frame_path: String,
) -> Result<ProcessingResult, String> {
    let state = app.state::<Arc<Mutex<SolutionState>>>();
    let state_guard = state.lock().await;

    if let Some(processor) = &state_guard.processor {
        let mut processor_guard = processor.lock().await;

        // Create frame metadata
        let frame = savant_video::VideoFrame {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            file_path: frame_path.into(),
            resolution: (1920, 1080), // TODO: Get actual resolution
            file_size: 0, // TODO: Get actual file size
            image_hash: String::new(), // TODO: Calculate hash
            metadata: savant_video::FrameMetadata {
                session_id: "current_session".to_string(),
                display_id: None,
                active_application: None,
                window_title: None,
                change_detected: true,
                ocr_text: None,
                enhanced_analysis: None,
                detected_applications: vec![],
                activity_classification: None,
                visual_context: None,
            },
        };

        match processor_guard.process_frame(&frame).await {
            Ok(result) => Ok(ProcessingResult::from(result)),
            Err(e) => {
                error!("Failed to process frame: {}", e);
                Err(format!("Failed to process frame: {}", e))
            }
        }
    } else {
        Err("Solution processor not initialized".to_string())
    }
}

#[tauri::command]
pub async fn regenerate_solution(
    app: AppHandle,
    problem_id: String,
) -> Result<GeneratedSolution, String> {
    info!("Regenerating solution for problem: {}", problem_id);

    // This would regenerate the solution for a specific problem
    // For now, return an error as we need to store problems to regenerate
    Err("Solution regeneration not yet implemented".to_string())
}

#[tauri::command]
pub async fn apply_solution(
    solution_code: String,
) -> Result<String, String> {
    info!("Applying solution code");

    // This would apply the solution to the active editor
    // For now, just copy to clipboard
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSPasteboard, NSPasteboardTypeString};
        use cocoa::foundation::{NSString, NSAutoreleasePool};
        use objc::runtime::Object;

        unsafe {
            let pool = NSAutoreleasePool::new(cocoa::base::nil);
            let pasteboard: *mut Object = NSPasteboard::generalPasteboard(cocoa::base::nil);
            NSPasteboard::clearContents(pasteboard);

            let ns_string = NSString::alloc(cocoa::base::nil)
                .init_str(&solution_code);
            let success = NSPasteboard::setString_forType(
                pasteboard,
                ns_string,
                NSPasteboardTypeString,
            );

            let _ = pool;

            if success {
                Ok("Solution copied to clipboard".to_string())
            } else {
                Err("Failed to copy solution to clipboard".to_string())
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Clipboard functionality not implemented for this platform".to_string())
    }
}

#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<String, String> {
    apply_solution(text).await
}

#[tauri::command]
pub async fn listen_for_solutions(
    app: AppHandle,
) -> Result<(), String> {
    // This is called from the frontend to set up event listeners
    // The actual events are emitted by the processor
    Ok(())
}

// Simplified ProcessingResult for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub frame_id: String,
    pub processing_time_ms: u64,
    pub changes_detected: bool,
    pub detected_problems: Vec<DetectedCodingProblem>,
    pub generated_solutions: Vec<GeneratedSolution>,
}

impl From<savant_video::ProcessingResult> for ProcessingResult {
    fn from(result: savant_video::ProcessingResult) -> Self {
        Self {
            frame_id: result.frame_id,
            processing_time_ms: result.processing_time_ms,
            changes_detected: result.changes_detected,
            detected_problems: result.detected_problems,
            generated_solutions: result.generated_solutions,
        }
    }
}
