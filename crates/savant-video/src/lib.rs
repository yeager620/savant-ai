use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod capture;
pub mod config;
pub mod privacy;
pub mod storage;

pub use capture::VideoCapture;
pub use config::{CaptureConfig, ImageQuality, VideoConfig};
pub use privacy::{PrivacyController, PrivacySettings};
pub use storage::{StorageManager, StorageSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub file_path: PathBuf,
    pub resolution: (u32, u32),
    pub file_size: u64,
    pub image_hash: String,
    pub metadata: FrameMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub session_id: String,
    pub display_id: Option<String>,
    pub active_application: Option<String>,
    pub window_title: Option<String>,
    pub change_detected: bool,
    pub ocr_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub frame_count: u32,
    pub total_size_bytes: u64,
    pub config: CaptureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureEvent {
    Started { session_id: String },
    Stopped { session_id: String },
    FrameCaptured { frame: VideoFrame },
    Error { message: String },
}

/// Create platform-specific video capture instance
pub fn create_video_capture() -> Result<Box<dyn VideoCapture>> {
    capture::create_platform_capture()
}