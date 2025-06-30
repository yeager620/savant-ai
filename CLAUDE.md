# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

This is a production-ready Tauri + Leptos AI assistant application with real-time screen analysis and stealth capabilities. The project implements a sophisticated multi-layered architecture:

- **Frontend**: Leptos 0.7 WASM application with reactive components for dashboard and overlay UI
- **Backend**: Tauri 2.0 Rust application with modular command system for AI, OCR, and system operations
- **AI Integration**: Multi-provider LLM support (Ollama local + OpenAI/DeepSeek/Anthropic cloud APIs)
- **Stealth System**: OS-level window manipulation for screenshot invisibility and system tray operation
- **Build System**: Trunk for WASM bundling, Cargo workspace for cross-compilation

The application operates as an invisible AI assistant that detects questions on screen via OCR and provides contextual answers through floating overlay bubbles.

## Development Commands

### Full Development Workflow
```bash
# Primary development command (recommended)
cargo tauri dev

# Alternative frontend-only development
trunk serve  # Frontend runs on localhost:1420

# Production builds
cargo tauri build --release
trunk build --release  # Frontend only
```

### Testing and Quality
```bash
# Run all tests across workspace
cargo test --workspace

# Test specific modules
cargo test -p savant-ai-lib  # Backend tests
cargo test -p savant-ai-ui   # Frontend tests (if any)

# Code quality checks
cargo check --workspace
cargo clippy --workspace
```

### Platform-Specific Notes
- **macOS**: Requires `macOSPrivateApi: true` in tauri.conf.json for stealth features
- **Windows**: Uses Win32 APIs for window manipulation
- **Dependencies**: Ensure Tesseract OCR is installed system-wide

## Command System Architecture

The backend implements a modular command system in `src-tauri/src/commands/`:

### Core Command Modules
- **`llm.rs`**: Multi-provider AI integration (Ollama, OpenAI, DeepSeek, Anthropic)
- **`ocr.rs`**: Tesseract-based text detection and question identification
- **`system.rs`**: OS-level stealth operations (screenshot hiding, transparency, window management)
- **`hotkey.rs`**: Global keyboard shortcuts (Cmd+Shift+A/S/D)
- **`config.rs`**: Persistent configuration management with TOML storage

### Frontend Component Structure
- **`components/dashboard.rs`**: Configuration UI with AI provider settings
- **`components/overlay.rs`**: Invisible question detection and answer bubble system
- **`utils/`**: Shared utilities for OCR and LLM communication

## Critical Implementation Details

### Stealth Mode Implementation
Uses platform-specific APIs to hide from screenshots:
- **macOS**: `setSharingType: 0` (NSWindowSharingNone) via Objective-C runtime
- **Windows**: `WS_EX_LAYERED` and `WS_EX_TOOLWINDOW` flags
- **System Tray**: Always-present tray icon with show/hide controls

### AI Provider Integration Pattern
All LLM providers implement standardized request/response through `LlmRequest`/`LlmResponse` structs:
- Local Ollama: Direct HTTP API calls to localhost:11434
- Cloud APIs: Authenticated HTTPS with timeout handling
- Fallback mechanisms for provider failures

### OCR Question Detection Logic
Implements intelligent question detection using:
- Text ending patterns (question marks)
- Question word starters (what, how, why, etc.)
- Imperative command detection (help, explain, show)
- Confidence scoring and position tracking

### Global Hotkey System
- **Cmd+Shift+A**: Toggle AI overlay scanning
- **Cmd+Shift+S**: Trigger immediate screenshot analysis
- **Cmd+Shift+D**: Show/focus dashboard window
- Hotkey validation and conflict detection

## Configuration Management

Application state persists to `~/.config/savant-ai/config.toml` with:
- AI provider settings and API keys
- Stealth mode preferences
- Scanning intervals and hotkey configurations
- Window transparency and positioning

## Frontend-Backend Communication

Uses type-safe Tauri command invocation pattern:
```rust
// Frontend (WASM)
let result = invoke("command_name", serde_wasm_bindgen::to_value(&args)?).await;

// Backend (Rust)
#[tauri::command]
async fn command_name(args: SomeStruct) -> Result<ReturnType, String> { ... }
```

All commands return `Result<T, String>` for consistent error handling across the application.

## Development Workflow Notes

- Use `cargo tauri dev` for full-stack development with hot reload
- Frontend changes trigger automatic WASM rebuilds via Trunk
- Backend changes require Tauri restart but preserve frontend state
- System tray persists between restarts for testing stealth functionality
- OCR requires actual screen content for testing - use external applications with text