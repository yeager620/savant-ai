use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use std::process::Command;
use std::sync::{Arc, Mutex};

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
    pub windows: HashMap<String, BrowserWindow>,
    pub top_prompts: Vec<DetectedPrompt>,
}

static mut BROWSER_STATE: Option<Arc<Mutex<BrowserState>>> = None;
static mut MONITORING_HANDLE: Option<tokio::task::JoinHandle<()>> = None;

#[tauri::command]
pub async fn start_browser_monitoring(app: AppHandle) -> Result<(), String> {
    println!("Starting Accessibility API browser monitoring...");
    
    // Check if accessibility permissions are granted
    let has_permissions = check_accessibility_permissions().await;
    println!("Accessibility permissions check: {}", has_permissions);
    
    if !has_permissions {
        return Err("Accessibility permissions required. Please enable in System Preferences > Security & Privacy > Privacy > Accessibility".to_string());
    }
    
    // Initialize browser state
    let state = Arc::new(Mutex::new(BrowserState {
        is_connected: true,
        active_window_id: None,
        windows: HashMap::new(),
        top_prompts: Vec::new(),
    }));
    
    unsafe {
        BROWSER_STATE = Some(state.clone());
    }
    
    // Start monitoring task
    let app_for_task = app.clone();
    let monitoring_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        
        loop {
            interval.tick().await;
            println!("Monitoring tick - scanning for browser windows...");
            
            match scan_browser_windows().await {
                Ok(windows) => {
                    let mut browser_windows = HashMap::new();
                    let mut all_prompts = Vec::new();
                    let mut active_window_id = None;
                    
                    // Process windows and collect prompts outside of mutex
                    for window in windows {
                        let prompts = detect_prompts_in_content(&window.content, &window.id).await;
                        
                        let browser_window = BrowserWindow {
                            id: window.id.clone(),
                            title: window.title,
                            app_name: window.app_name,
                            is_active: window.is_active,
                            content: window.content,
                            detected_prompts: prompts.clone(),
                        };
                        
                        browser_windows.insert(window.id.clone(), browser_window);
                        
                        // Boost priority for active window
                        for mut prompt in prompts {
                            if window.is_active {
                                prompt.priority *= 2.0;
                                active_window_id = Some(window.id.clone());
                            }
                            all_prompts.push(prompt);
                        }
                    }
                    
                    // Sort and limit to top 5 prompts
                    all_prompts.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
                    all_prompts.truncate(5);
                    
                    // Update state under mutex
                    let current_state = {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.windows = browser_windows;
                        state_guard.top_prompts = all_prompts;
                        if let Some(id) = active_window_id {
                            state_guard.active_window_id = Some(id);
                        }
                        state_guard.clone()
                    };
                    
                    println!("Emitting browser state update with {} windows and {} prompts", 
                        current_state.windows.len(), current_state.top_prompts.len());
                    let _ = app_for_task.emit("browser_state_updated", &current_state);
                }
                Err(e) => {
                    eprintln!("Error scanning browser windows: {}", e);
                    
                    // Emit error state
                    let error_state = BrowserState {
                        is_connected: false,
                        active_window_id: None,
                        windows: HashMap::new(),
                        top_prompts: Vec::new(),
                    };
                    let _ = app_for_task.emit("browser_state_updated", &error_state);
                }
            }
        }
    });
    
    unsafe {
        MONITORING_HANDLE = Some(monitoring_task);
    }
    
    // Emit initial state showing we're connected
    let initial_state = BrowserState {
        is_connected: true,
        active_window_id: None,
        windows: HashMap::new(),
        top_prompts: Vec::new(),
    };
    println!("Emitting initial browser state (connected: true)");
    let _ = app.emit("browser_state_updated", &initial_state);
    
    Ok(())
}

#[tauri::command]
pub async fn stop_browser_monitoring() -> Result<(), String> {
    println!("Stopping browser monitoring...");
    
    unsafe {
        if let Some(handle) = MONITORING_HANDLE.take() {
            handle.abort();
        }
        
        if let Some(state) = &BROWSER_STATE {
            let mut state_guard = state.lock().unwrap();
            state_guard.is_connected = false;
            state_guard.windows.clear();
            state_guard.top_prompts.clear();
        }
        BROWSER_STATE = None;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_browser_state() -> Result<BrowserState, String> {
    unsafe {
        if let Some(state) = &BROWSER_STATE {
            let state_guard = state.lock().unwrap();
            Ok(state_guard.clone())
        } else {
            Err("Browser monitoring not started".to_string())
        }
    }
}

#[tauri::command]
pub async fn select_prompt(app: AppHandle, prompt_id: String) -> Result<(), String> {
    let prompt = unsafe {
        if let Some(state) = &BROWSER_STATE {
            let state_guard = state.lock().unwrap();
            state_guard.top_prompts.iter().find(|p| p.id == prompt_id).cloned()
        } else {
            None
        }
    };
    
    if let Some(prompt) = prompt {
        println!("User selected prompt: {}", prompt.text);
        let _ = app.emit("prompt_selected", &prompt);
        Ok(())
    } else {
        Err("Prompt not found".to_string())
    }
}

// Check if accessibility permissions are granted
async fn check_accessibility_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Simple test - try to get process names
        let script = r#"
        tell application "System Events"
            try
                get name of first process
                return "true"
            on error
                return "false"
            end try
        end tell
        "#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();
            
        match output {
            Ok(result) => {
                let output_str = String::from_utf8_lossy(&result.stdout);
                println!("Accessibility check result: '{}'", output_str.trim());
                output_str.trim() == "true"
            }
            Err(e) => {
                println!("Accessibility check failed: {}", e);
                false
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, assume we have permissions
        true
    }
}

// Scan for browser windows using AppleScript
async fn scan_browser_windows() -> Result<Vec<BrowserWindow>> {
    println!("Scanning for browser windows...");
    
    let script = r#"
    set windowList to {}
    set browserApps to {"Google Chrome", "Safari", "Firefox", "Microsoft Edge", "Arc", "Brave Browser", "Opera"}
    
    tell application "System Events"
        repeat with appName in browserApps
            try
                if exists (process appName) then
                    tell process appName
                        set windowCount to count of windows
                        if windowCount > 0 then
                            repeat with w from 1 to windowCount
                                try
                                    set windowTitle to name of window w
                                    set isActive to (frontmost of process appName) and (w = 1)
                                    
                                    -- Simple format: appName|windowTitle|isActive
                                    set windowInfo to appName & "|" & windowTitle & "|" & isActive
                                    set end of windowList to windowInfo
                                end try
                            end repeat
                        end if
                    end tell
                end if
            end try
        end repeat
    end tell
    
    return windowList as string
    "#;
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute AppleScript: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("AppleScript output: {}", output_str);
    
    let mut windows = Vec::new();
    
    // Parse AppleScript output - each line is "appName|windowTitle|isActive"
    for line in output_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            let app_name = parts[0].to_string();
            let title = parts[1].to_string();
            let is_active = parts[2] == "true";
            
            println!("Found window: {} - {} (active: {})", app_name, title, is_active);
            
            // Get window content using accessibility APIs
            let content = get_window_content(&app_name, &title).await.unwrap_or_default();
            
            windows.push(BrowserWindow {
                id: format!("{}_{}", app_name, uuid::Uuid::new_v4()),
                title: title.clone(),
                app_name: app_name.clone(),
                is_active,
                content,
                detected_prompts: Vec::new(),
            });
        }
    }
    
    println!("Found {} browser windows", windows.len());
    Ok(windows)
}

// Get window content using AppleScript accessibility
async fn get_window_content(app_name: &str, window_title: &str) -> Result<String> {
    // For now, return a simple mock content to test the UI
    // Real content extraction is complex and may require different approaches per browser
    let mock_content = format!(
        "Sample content from {} window '{}'. How do I implement a REST API? What is the best way to optimize database queries? Can you explain machine learning concepts?",
        app_name, window_title
    );
    
    println!("Getting content for {} - {}", app_name, window_title);
    Ok(mock_content)
}

async fn detect_prompts_in_content(content: &str, window_id: &str) -> Vec<DetectedPrompt> {
    let mut prompts = Vec::new();
    
    // Split content into sentences/paragraphs
    let sentences: Vec<&str> = content
        .split(&['.', '!', '?', '\n'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 10)
        .collect();
    
    for (i, sentence) in sentences.iter().enumerate() {
        let confidence = calculate_prompt_confidence(sentence);
        
        if confidence > 0.3 {
            // Get surrounding context
            let context_start = i.saturating_sub(1);
            let context_end = (i + 2).min(sentences.len());
            let context = sentences[context_start..context_end].join(" ");
            
            let prompt = DetectedPrompt {
                id: Uuid::new_v4().to_string(),
                text: sentence.to_string(),
                confidence,
                priority: confidence,
                window_id: window_id.to_string(),
                context,
                position: PromptPosition {
                    x: 0,
                    y: i as i32 * 20,
                    width: sentence.len() as i32 * 8,
                    height: 20,
                },
            };
            
            prompts.push(prompt);
        }
    }
    
    // Sort by confidence and limit
    prompts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    prompts.truncate(10);
    prompts
}

fn calculate_prompt_confidence(text: &str) -> f32 {
    let text_lower = text.to_lowercase();
    let mut score: f32 = 0.0;
    
    // Question patterns
    let question_patterns = [
        "what is", "how do", "how to", "why does", "when should", "where can",
        "which one", "who is", "can you", "could you", "would you", "should i",
        "help me", "explain", "show me", "tell me", "find", "calculate",
        "solve", "create", "generate", "write", "design", "build",
    ];
    
    for pattern in &question_patterns {
        if text_lower.contains(pattern) {
            score += 0.4;
        }
    }
    
    // Question marks
    if text.contains('?') {
        score += 0.3;
    }
    
    // Instruction patterns
    let instruction_patterns = [
        "please", "need to", "want to", "how can", "step by step",
        "tutorial", "guide", "example", "demo", "walkthrough",
    ];
    
    for pattern in &instruction_patterns {
        if text_lower.contains(pattern) {
            score += 0.2;
        }
    }
    
    // Problem-solving indicators
    let problem_patterns = [
        "error", "problem", "issue", "bug", "not working", "failed",
        "trouble", "difficulty", "stuck", "confused",
    ];
    
    for pattern in &problem_patterns {
        if text_lower.contains(pattern) {
            score += 0.2;
        }
    }
    
    // Length penalty for very short or very long text
    let length = text.len();
    if length < 20 || length > 200 {
        score *= 0.5;
    }
    
    // Cap at 1.0
    score.min(1.0)
}

#[tauri::command]
pub async fn set_active_window(window_id: String) -> Result<(), String> {
    unsafe {
        if let Some(state) = &BROWSER_STATE {
            let mut state_guard = state.lock().unwrap();
            state_guard.active_window_id = Some(window_id);
            
            // Recalculate priorities
            let mut all_prompts = Vec::new();
            for window in state_guard.windows.values() {
                for mut prompt in window.detected_prompts.clone() {
                    if Some(&window.id) == state_guard.active_window_id.as_ref() {
                        prompt.priority *= 2.0;
                    }
                    all_prompts.push(prompt);
                }
            }
            
            all_prompts.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
            all_prompts.truncate(5);
            state_guard.top_prompts = all_prompts;
        }
    }
    Ok(())
}