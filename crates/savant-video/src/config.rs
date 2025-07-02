use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub capture: CaptureConfig,
    pub privacy: crate::PrivacySettings,
    pub storage: crate::StorageSettings,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            capture: CaptureConfig::default(),
            privacy: crate::PrivacySettings::default(),
            storage: crate::StorageSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub interval_milliseconds: u32, // Changed to milliseconds for sub-second precision
    pub enabled_hours: Option<TimeRange>,
    pub quality: ImageQuality,
    pub notify_user: bool,
    pub stealth_mode: bool, // For invisibility to external capture
    pub continuous_mode: bool, // Enable continuous high-frequency capture
    pub auto_compress: bool, // Automatically compress captured images
    pub max_resolution: Option<(u32, u32)>, // Maximum resolution for compression
    pub enable_processing: bool, // Enable automated OCR/vision processing
    pub processing_interval: u32, // Process every Nth frame
    pub change_detection_threshold: f32, // Minimum change to trigger processing (0.0-1.0)
    pub enable_full_text_extraction: bool, // Extract ALL text with positions
    pub enable_real_time_analysis: bool, // Real-time task/question detection
    pub buffer_size: usize, // Number of frames to buffer for change detection
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            interval_milliseconds: 500, // 500ms = 2 FPS for continuous monitoring
            enabled_hours: None,
            quality: ImageQuality::Medium, // Balance quality vs speed
            notify_user: false, // Disable notifications for continuous mode
            stealth_mode: true, // Default to stealth for privacy
            continuous_mode: true, // Enable continuous high-frequency capture
            auto_compress: true, // Automatically compress captured images
            max_resolution: Some((1920, 1080)), // Higher res for better text extraction
            enable_processing: true, // Enable automated OCR/vision processing
            processing_interval: 1, // Process every frame for full monitoring
            change_detection_threshold: 0.05, // 5% change threshold
            enable_full_text_extraction: true, // Extract ALL text with positions
            enable_real_time_analysis: true, // Real-time task/question detection
            buffer_size: 10, // Keep 10 frames for change detection
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageQuality {
    Low,    // 50% quality, faster
    Medium, // 75% quality, balanced
    High,   // 90% quality, larger files
}

impl ImageQuality {
    pub fn jpeg_quality(&self) -> u8 {
        match self {
            ImageQuality::Low => 50,
            ImageQuality::Medium => 75,
            ImageQuality::High => 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
}

impl TimeRange {
    pub fn new(start: (u8, u8), end: (u8, u8)) -> Self {
        Self {
            start_hour: start.0,
            start_minute: start.1,
            end_hour: end.0,
            end_minute: end.1,
        }
    }

    pub fn is_within_range(&self, hour: u8, minute: u8) -> bool {
        let current = hour * 60 + minute;
        let start = self.start_hour * 60 + self.start_minute;
        let end = self.end_hour * 60 + self.end_minute;

        if start <= end {
            current >= start && current <= end
        } else {
            // Handle overnight ranges
            current >= start || current <= end
        }
    }
}