use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod capture;
pub mod config;
pub mod privacy;
pub mod storage;
pub mod analyzer;
pub mod multimodal;
pub mod processor;
pub mod real_time_analyzer;
pub mod change_detector;
pub mod coding_problem_detector;
pub mod solution_generator;
pub mod integrated_processor;
pub mod llm_provider;

pub use capture::VideoCapture;
pub use config::{CaptureConfig, ImageQuality, VideoConfig};
pub use privacy::{PrivacyController, PrivacySettings};
pub use storage::{StorageManager, StorageSettings};
pub use analyzer::{EnhancedVideoAnalyzer, VideoAnalysisResult, CodeSnippet, InteractionOpportunity};
pub use multimodal::{MultimodalFrame, MultimodalAnalyzer};
pub use processor::{VideoProcessor, ProcessingCommand, ProcessingEvent as VideoProcessingEvent, CompressedFrame};
pub use real_time_analyzer::{RealTimeAnalyzer, TaskDetectionResult, DetectedTask, DetectedQuestion, AssistanceOpportunity};
pub use change_detector::{ChangeDetector, ChangeDetectionResult, ChangedRegion, ChangeDetectorConfig};
pub use coding_problem_detector::{CodingProblemDetector, DetectedCodingProblem, CodingProblemType, DetectionConfig};
pub use solution_generator::{SolutionGenerator, GeneratedSolution, SolutionConfig};
pub use integrated_processor::{IntegratedProcessor, ProcessorConfig, ProcessingEvent, ProcessingResult};

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
    pub enhanced_analysis: Option<VideoAnalysisResult>,
    pub detected_applications: Vec<savant_vision::DetectedApp>,
    pub activity_classification: Option<savant_vision::ActivityClassification>,
    pub visual_context: Option<savant_vision::VisualContext>,
}

impl Default for FrameMetadata {
    fn default() -> Self {
        Self {
            session_id: String::from("default-session"),
            display_id: None,
            active_application: None,
            window_title: None,
            change_detected: false,
            ocr_text: None,
            enhanced_analysis: None,
            detected_applications: Vec::new(),
            activity_classification: None,
            visual_context: None,
        }
    }
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
