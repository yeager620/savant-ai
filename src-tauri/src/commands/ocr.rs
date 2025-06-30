use anyhow::{anyhow, Result};
use screenshots::Screen;
use serde::{Deserialize, Serialize};

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

#[tauri::command]
pub async fn detect_questions() -> Result<OcrResult, String> {
    let screenshot = take_screenshot_internal().await
        .map_err(|e| format!("Failed to take screenshot: {}", e))?;
    
    process_screenshot_internal(screenshot).await
}

#[tauri::command]
pub async fn process_screenshot(screenshot_data: Vec<u8>) -> Result<OcrResult, String> {
    process_screenshot_internal(screenshot_data).await
}

async fn take_screenshot_internal() -> Result<Vec<u8>> {
    let screens = Screen::all()?;
    if screens.is_empty() {
        return Err(anyhow!("No screens found"));
    }
    
    let screen = &screens[0];
    let image = screen.capture()?;
    
    let png_data = image.buffer().to_vec();
    
    Ok(png_data)
}

async fn process_screenshot_internal(screenshot_data: Vec<u8>) -> Result<OcrResult, String> {
    let img = image::load_from_memory(&screenshot_data)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    
    // OCR implementation placeholder - will add Tesseract later
    // For now, simulate finding some example questions
    let text_data = Some("What is the capital of France? How do I solve this equation?".to_string());
    
    let mut questions = Vec::new();
    
    // Process text to find questions
    if let Some(text) = text_data {
        let lines: Vec<&str> = text.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if is_question_text(line) {
                // For now, we'll use estimated positions
                // In a full implementation, you'd use tesseract.get_component_images() 
                // to get exact bounding boxes
                let estimated_y = (i as u32 * 30) % height; // Rough line height estimation
                let estimated_x = 50; // Left margin
                let estimated_width = line.len() as u32 * 8; // Rough character width
                let estimated_height = 25; // Rough line height
                
                questions.push(Question {
                    text: line.trim().to_string(),
                    x: estimated_x,
                    y: estimated_y,
                    width: estimated_width.min(width - estimated_x),
                    height: estimated_height,
                    confidence: 0.8, // Placeholder confidence
                });
            }
        }
    }
    
    Ok(OcrResult {
        questions,
        processed_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn is_question_text(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    let text_trimmed = text_lower.trim();
    
    // Check for obvious question markers
    if text_trimmed.ends_with('?') {
        return true;
    }
    
    // Check for question words at the beginning
    let question_starters = [
        "what", "how", "why", "when", "where", "who", "which", "whose",
        "can you", "could you", "would you", "should", "do you", "did you",
        "will you", "are you", "is it", "help", "explain", "define",
        "describe", "tell me", "show me"
    ];
    
    for starter in &question_starters {
        if text_trimmed.starts_with(starter) {
            return true;
        }
    }
    
    // Check for imperative statements that might be questions
    let imperative_indicators = [
        "help", "explain", "show", "tell", "describe", "define", "clarify"
    ];
    
    for indicator in &imperative_indicators {
        if text_trimmed.contains(indicator) && text_trimmed.len() > 10 {
            return true;
        }
    }
    
    false
}