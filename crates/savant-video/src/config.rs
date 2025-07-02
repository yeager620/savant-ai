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
    pub interval_seconds: u32,
    pub enabled_hours: Option<TimeRange>,
    pub quality: ImageQuality,
    pub notify_user: bool,
    pub stealth_mode: bool, // For invisibility to external capture
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 60,
            enabled_hours: None,
            quality: ImageQuality::Medium,
            notify_user: true,
            stealth_mode: true, // Default to stealth for privacy
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