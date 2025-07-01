//! Core data types shared across all Savant AI components

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Chat message with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub id: Uuid,
    pub content: String,
    pub is_user: bool,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<MessageMetadata>,
}

impl ChatMessage {
    pub fn new_user(content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            is_user: true,
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    pub fn new_assistant(content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            is_user: false,
            timestamp: Utc::now(),
            metadata: None,
        }
    }
}

/// Additional metadata for messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageMetadata {
    pub model: Option<String>,
    pub tokens_used: Option<u32>,
    pub processing_time_ms: Option<u64>,
    pub source: Option<MessageSource>,
}

/// Source of a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageSource {
    Manual,
    BrowserDetection { url: String, tab_title: String },
    VoiceInput,
    ScreenCapture,
}

/// LLM request with all parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    pub model: String,
    pub provider: LlmProvider,
    pub options: LlmOptions,
    pub context: Option<Vec<ChatMessage>>,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LlmProvider {
    Ollama { url: String },
    OpenAI { api_key: String },
    DeepSeek { api_key: String },
    Anthropic { api_key: String },
}

/// LLM generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmOptions {
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

impl Default for LlmOptions {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 4096,
            stream: false,
        }
    }
}

/// LLM response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
    pub tokens_used: Option<u32>,
    pub processing_time_ms: u64,
    pub finished: bool,
}

/// Browser tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub is_active: bool,
    pub browser_name: String,
}

/// Detected question/prompt from browser content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPrompt {
    pub id: Uuid,
    pub text: String,
    pub confidence: f32,
    pub source_tab: BrowserTab,
    pub position: Option<TextPosition>,
    pub context: Option<String>,
}

/// Text position within content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    pub start: usize,
    pub end: usize,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub browser: BrowserConfig,
    pub stealth: StealthConfig,
    pub ui: UiConfig,
}

/// LLM-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub default_provider: LlmProvider,
    pub default_model: String,
    pub default_options: LlmOptions,
}

/// Browser monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub enabled: bool,
    pub scan_interval_ms: u64,
    pub supported_browsers: Vec<String>,
    pub question_detection_threshold: f32,
}

/// Stealth mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfig {
    pub hide_from_screenshots: bool,
    pub hide_from_taskbar: bool,
    pub always_on_top: bool,
    pub transparency: f32,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub position: WindowPosition,
    pub auto_hide: bool,
}

/// Window position settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}