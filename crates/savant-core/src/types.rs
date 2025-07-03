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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_chat_message_new_user() {
        let content = "Hello, world!".to_string();
        let message = ChatMessage::new_user(content.clone());
        
        assert_eq!(message.content, content);
        assert!(message.is_user);
        assert!(message.timestamp <= Utc::now());
        assert!(message.metadata.is_none());
        assert!(!message.id.is_nil());
    }

    #[test]
    fn test_chat_message_new_assistant() {
        let content = "Hello! How can I help you?".to_string();
        let message = ChatMessage::new_assistant(content.clone());
        
        assert_eq!(message.content, content);
        assert!(!message.is_user);
        assert!(message.timestamp <= Utc::now());
        assert!(message.metadata.is_none());
        assert!(!message.id.is_nil());
    }

    #[test]
    fn test_chat_message_serialization() {
        let message = ChatMessage::new_user("Test message".to_string());
        
        // Test serialization
        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("Test message"));
        
        // Test deserialization
        let deserialized: ChatMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, deserialized);
    }

    #[test]
    fn test_llm_options_default() {
        let options = LlmOptions::default();
        
        assert_eq!(options.temperature, 0.7);
        assert_eq!(options.max_tokens, 4096);
        assert!(!options.stream);
    }

    #[test]
    fn test_llm_response_creation() {
        let response = LlmResponse {
            content: "Generated response".to_string(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
            tokens_used: Some(150),
            processing_time_ms: 1500,
            finished: true,
        };

        assert_eq!(response.content, "Generated response");
        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.provider, "openai");
        assert_eq!(response.tokens_used, Some(150));
        assert_eq!(response.processing_time_ms, 1500);
        assert!(response.finished);
    }

    #[test]
    fn test_browser_tab_serialization() {
        let tab = BrowserTab {
            id: "tab123".to_string(),
            title: "Example Website".to_string(),
            url: "https://example.com".to_string(),
            content: "Some page content".to_string(),
            is_active: true,
            browser_name: "Chrome".to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&tab).unwrap();
        assert!(serialized.contains("tab123"));
        assert!(serialized.contains("Example Website"));
        
        // Test deserialization
        let deserialized: BrowserTab = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tab.id, deserialized.id);
        assert_eq!(tab.title, deserialized.title);
        assert_eq!(tab.url, deserialized.url);
        assert_eq!(tab.content, deserialized.content);
        assert_eq!(tab.is_active, deserialized.is_active);
        assert_eq!(tab.browser_name, deserialized.browser_name);
    }

    #[test]
    fn test_window_position() {
        let position = WindowPosition {
            x: 100,
            y: 200,
            width: 800,
            height: 600,
        };

        assert_eq!(position.x, 100);
        assert_eq!(position.y, 200);
        assert_eq!(position.width, 800);
        assert_eq!(position.height, 600);
    }

    #[test]
    fn test_ui_config() {
        let config = UiConfig {
            theme: "dark".to_string(),
            position: WindowPosition {
                x: 0,
                y: 0,
                width: 1200,
                height: 800,
            },
            auto_hide: true,
        };

        assert_eq!(config.theme, "dark");
        assert!(config.auto_hide);
        assert_eq!(config.position.width, 1200);
    }

    #[test]
    fn test_message_source_variants() {
        // Test different message source variants
        let manual_source = MessageSource::Manual;
        let voice_source = MessageSource::VoiceInput;
        let screen_source = MessageSource::ScreenCapture;
        let browser_source = MessageSource::BrowserDetection {
            url: "https://example.com".to_string(),
            tab_title: "Example Page".to_string(),
        };

        // Test serialization of enum variants
        let manual_json = serde_json::to_string(&manual_source).unwrap();
        assert!(manual_json.contains("Manual"));

        let voice_json = serde_json::to_string(&voice_source).unwrap();
        assert!(voice_json.contains("VoiceInput"));

        let screen_json = serde_json::to_string(&screen_source).unwrap();
        assert!(screen_json.contains("ScreenCapture"));

        let browser_json = serde_json::to_string(&browser_source).unwrap();
        assert!(browser_json.contains("BrowserDetection"));
        assert!(browser_json.contains("example.com"));
    }

    #[test]
    fn test_message_metadata() {
        let metadata = MessageMetadata {
            model: Some("gpt-4".to_string()),
            tokens_used: Some(250),
            processing_time_ms: Some(2000),
            source: Some(MessageSource::VoiceInput),
        };

        assert_eq!(metadata.model, Some("gpt-4".to_string()));
        assert_eq!(metadata.tokens_used, Some(250));
        assert_eq!(metadata.processing_time_ms, Some(2000));
        assert!(matches!(metadata.source, Some(MessageSource::VoiceInput)));
    }
}