//! Error types for Savant AI components

use serde::{Deserialize, Serialize};

/// Standard error type for all Savant AI components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavantError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
}

/// Categories of errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    Config,
    Network,
    LlmProvider,
    BrowserAccess,
    FileSystem,
    Serialization,
    Permission,
    NotFound,
    InvalidInput,
}

impl SavantError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Config, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Network, message)
    }

    pub fn llm_provider(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::LlmProvider, message)
    }

    pub fn browser_access(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BrowserAccess, message)
    }

    pub fn file_system(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::FileSystem, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, message)
    }
}

impl std::fmt::Display for SavantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(context) = &self.context {
            write!(f, "{}: {} ({})", self.kind_str(), self.message, context)
        } else {
            write!(f, "{}: {}", self.kind_str(), self.message)
        }
    }
}

impl SavantError {
    fn kind_str(&self) -> &'static str {
        match self.kind {
            ErrorKind::Config => "Config Error",
            ErrorKind::Network => "Network Error",
            ErrorKind::LlmProvider => "LLM Provider Error",
            ErrorKind::BrowserAccess => "Browser Access Error",
            ErrorKind::FileSystem => "File System Error",
            ErrorKind::Serialization => "Serialization Error",
            ErrorKind::Permission => "Permission Error",
            ErrorKind::NotFound => "Not Found",
            ErrorKind::InvalidInput => "Invalid Input",
        }
    }
}

impl std::error::Error for SavantError {}

impl From<std::io::Error> for SavantError {
    fn from(err: std::io::Error) -> Self {
        SavantError::file_system(err.to_string())
    }
}

impl From<serde_json::Error> for SavantError {
    fn from(err: serde_json::Error) -> Self {
        SavantError::new(ErrorKind::Serialization, err.to_string())
    }
}

/// Result type alias for convenience
pub type SavantResult<T> = Result<T, SavantError>;