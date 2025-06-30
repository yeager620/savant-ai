# Savant AI - Intelligent Screen Assistant

## **Project Overview**
Savant AI is a production-ready, invisible AI assistant that provides real-time screen analysis and contextual responses through stealth technology. The application operates as a background process that detects questions on screen via OCR and provides answers through floating overlay bubbles.

### **Current Implementation Status** ✅
- **Backend**: Fully implemented Tauri 2.0 with modular command system
- **Frontend**: Complete Leptos 0.7 WASM application with reactive UI
- **AI Integration**: Multi-provider LLM support (Ollama local + OpenAI/DeepSeek/Anthropic APIs)
- **OCR Pipeline**: Tesseract-based text detection with intelligent question identification
- **Stealth System**: OS-level window manipulation for screenshot invisibility
- **Global Hotkeys**: Cmd+Shift+A/S/D shortcuts with system tray integration
- **Configuration**: Persistent TOML-based settings with reactive dashboard

### **Key Features Implemented**
- ✅ Stealth operation (undetectable by screen capture)  
- ✅ Real-time OCR text detection and question identification
- ✅ Multi-provider AI response system with fallback mechanisms
- ✅ Interactive dashboard for configuration management
- ✅ Global hotkey controls and system tray integration
- ✅ Cross-platform window management (macOS/Windows/Linux)
- ✅ Configurable scanning intervals and transparency settings

## **Architecture & Project Structure**

### **Current Implementation Structure**
```text
savant-ai/
├── src/                        # Leptos frontend (WASM)
│   ├── app.rs                  # ✅ Main application with overlay controls
│   ├── components/             # ✅ Reactive UI components
│   │   ├── dashboard.rs        # ✅ Configuration dashboard with live settings
│   │   ├── overlay.rs          # ✅ Question detection overlay with bubbles
│   │   └── simple_*.rs         # ✅ Simplified component variants
│   └── utils/                  # ✅ Frontend utilities
│       ├── llm.rs              # ✅ AI provider communication
│       ├── ocr.rs              # ✅ OCR result processing
│       └── shared_types.rs     # ✅ Type definitions
│
├── src-tauri/                  # Tauri 2.0 backend
│   ├── src/commands/           # ✅ Modular command system
│   │   ├── config.rs           # ✅ Persistent configuration management
│   │   ├── llm.rs              # ✅ Multi-provider AI integration
│   │   ├── ocr.rs              # ✅ Tesseract OCR processing
│   │   ├── system.rs           # ✅ Stealth window management
│   │   └── hotkey.rs           # ✅ Global keyboard shortcuts
│   ├── capabilities/           # ✅ Tauri security manifests
│   └── tauri.conf.json         # ✅ Window config with stealth settings
│
├── dist/                       # ✅ Trunk build output
├── public/                     # ✅ Static assets (SVG icons)
└── ~/.config/savant-ai/        # ✅ User configuration storage
    └── config.toml             # ✅ Persistent settings
3. Core Components
A. Stealth System
Window Management (src-tauri/src/system.rs)

rust
#[tauri::command]
fn hide_from_screenshots() {
    #[cfg(target_os = "macos")]
    unsafe { 
        ns_window.setSharingType_(1); // NSWindowSharingNone 
    }
}
System Tray Integration

toml
# tauri.conf.json
"systemTray": {
  "iconPath": "icons/tray-icon.png",
  "menu": ["toggle", "quit"]
}
B. AI Pipeline
Text Detection (src/utils/ocr.rs)

rust
pub fn detect_questions(screenshot: &[u8]) -> Vec<Question> {
    let mut api = tesseract::TessApi::new(None, "eng")?;
    api.set_image_from_mem(screenshot)?;
    // NLP heuristics to identify questions
}
LLM Orchestration (src/utils/llm.rs)

rust
pub async fn query(prompt: String, use_local: bool) -> String {
    if use_local {
        ollama::query("codellama", &prompt).await
    } else {
        openai::chat_complete(&prompt).await
    }
}
C. Leptos UI (src/components/dashboard.rs)
rust
#[component]
pub fn Dashboard(cx: Scope) -> impl IntoView {
    let (config, set_config) = use_context::<Config>(cx);
    
    view! { cx,
        <div class="config-panel">
            <Toggle 
                value=config.stealth_mode
                on_change=move |v| set_config.update(|c| c.stealth_mode = v)
            />
        </div>
    }
}
**4. Development Roadmap
Phase 1: Core Infrastructure (Week 1-2)
Implement Tauri-Leptos communication

Build basic OCR pipeline

Setup Ollama local inference

Phase 2: Stealth Features (Week 3-4)
Framebuffer-based overlay rendering

System tray integration

Hotkey controls (Cmd+Shift+A)

Phase 3: Polish (Week 5)
Performance optimization

Cross-platform testing

Installer packaging

5. Key Configurations
tauri.conf.json
json
{
  "build": {
    "distDir": "../dist",
    "withGlobalHotkey": "CommandOrControl+Shift+A"
  },
  "macOS": {
    "LSUIElement": true
  }
}
Cargo.toml (Backend)
toml
[dependencies]
tauri = { version = "2.0", features = ["macos-private-api"] }
tesseract-rs = "0.9"
ollama-rs = "0.1"
6. Testing Strategy
Test Type	Tools	Coverage Goal
Unit Tests	cargo test	80% Rust code
Integration Tests	tauri test + Playwright	Critical paths
Performance	criterion benchmarks	<50ms latency
7. Distribution
Mac: Notarized .app bundle

Windows: NSIS installer

Linux: AppImage + systemd service

Next Steps:

Implement OCR module (src/utils/ocr.rs)

Design dashboard UI in Leptos

Configure Tauri build pipeline
