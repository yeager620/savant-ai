use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::debug;

use crate::VideoFrame;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetectionResult {
    pub frame_id: String,
    pub previous_frame_id: Option<String>,
    pub change_score: f32, // 0.0 = no change, 1.0 = complete change
    pub pixel_diff_percentage: f32,
    pub text_diff_percentage: f32,
    pub ui_diff_percentage: f32,
    pub changed_regions: Vec<ChangedRegion>,
    pub significant_change: bool,
    pub change_summary: String,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedRegion {
    pub region_id: String,
    pub region_type: RegionType,
    pub bounding_box: BoundingBox,
    pub change_type: ChangeType,
    pub change_intensity: f32, // 0.0-1.0
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionType {
    MenuBar,
    Sidebar,
    MainContent,
    StatusBar,
    Dialog,
    Popup,
    Toolbar,
    CodeEditor,
    Terminal,
    Browser,
    ChatWindow,
    FileExplorer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    TextChange,
    UIElementChange,
    ColorChange,
    SizeChange,
    PositionChange,
    AppearanceChange,
    DisappearanceChange,
    ContentScrolling,
    WindowResize,
    ApplicationSwitch,
}

#[derive(Debug, Clone)]
struct FrameBuffer {
    frame: VideoFrame,
    image_data: Vec<u8>,
    image_hash: String,
    processed_regions: Option<Vec<ProcessedRegion>>,
    text_content: Option<String>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct ProcessedRegion {
    region_type: RegionType,
    bounding_box: BoundingBox,
    pixel_hash: String,
    text_content: String,
    element_count: u32,
}

#[derive(Debug, Clone)]
pub struct ChangeDetectorConfig {
    pub buffer_size: usize,
    pub pixel_diff_threshold: f32,
    pub text_diff_threshold: f32,
    pub significant_change_threshold: f32,
    pub enable_region_analysis: bool,
    pub enable_text_comparison: bool,
    pub enable_ui_element_tracking: bool,
    pub hash_comparison_enabled: bool,
    pub adaptive_threshold: bool,
}

impl Default for ChangeDetectorConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10,
            pixel_diff_threshold: 0.05, // 5% pixel difference
            text_diff_threshold: 0.1,   // 10% text difference
            significant_change_threshold: 0.15, // 15% overall change
            enable_region_analysis: true,
            enable_text_comparison: true,
            enable_ui_element_tracking: true,
            hash_comparison_enabled: true,
            adaptive_threshold: true,
        }
    }
}

#[derive(Debug)]
pub struct ChangeDetector {
    config: ChangeDetectorConfig,
    frame_buffer: VecDeque<FrameBuffer>,
    region_analyzer: RegionAnalyzer,
    hash_cache: HashMap<String, DateTime<Utc>>,
    adaptive_thresholds: AdaptiveThresholds,
}

#[derive(Debug, Clone)]
struct AdaptiveThresholds {
    current_pixel_threshold: f32,
    current_text_threshold: f32,
    recent_change_scores: VecDeque<f32>,
    baseline_noise_level: f32,
}

impl ChangeDetector {
    pub fn new(config: ChangeDetectorConfig) -> Self {
        Self {
            config: config.clone(),
            frame_buffer: VecDeque::with_capacity(config.buffer_size),
            region_analyzer: RegionAnalyzer::new(),
            hash_cache: HashMap::new(),
            adaptive_thresholds: AdaptiveThresholds {
                current_pixel_threshold: config.pixel_diff_threshold,
                current_text_threshold: config.text_diff_threshold,
                recent_change_scores: VecDeque::with_capacity(20),
                baseline_noise_level: 0.02, // 2% baseline noise
            },
        }
    }

    pub async fn detect_changes(
        &mut self,
        frame: VideoFrame,
        image_data: Vec<u8>,
        text_content: Option<String>,
    ) -> Result<ChangeDetectionResult> {
        let start_time = std::time::Instant::now();
        let frame_hash = self.calculate_image_hash(&image_data)?;

        // Quick hash-based deduplication check
        if self.config.hash_comparison_enabled {
            if let Some(last_seen) = self.hash_cache.get(&frame_hash) {
                let time_since = Utc::now().signed_duration_since(*last_seen);
                if time_since.num_seconds() < 1 {
                    // Exact duplicate within 1 second - skip processing
                    return Ok(ChangeDetectionResult {
                        frame_id: frame.id.clone(),
                        previous_frame_id: self.get_last_frame_id(),
                        change_score: 0.0,
                        pixel_diff_percentage: 0.0,
                        text_diff_percentage: 0.0,
                        ui_diff_percentage: 0.0,
                        changed_regions: Vec::new(),
                        significant_change: false,
                        change_summary: "Duplicate frame".to_string(),
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
            }
        }

        // Create frame buffer entry
        let frame_buffer = FrameBuffer {
            frame: frame.clone(),
            image_data: image_data.clone(),
            image_hash: frame_hash.clone(),
            processed_regions: None,
            text_content,
            timestamp: Utc::now(),
        };

        // Compare with previous frame if available
        let change_result = if let Some(previous_frame) = self.frame_buffer.back() {
            self.compare_frames(&frame_buffer, previous_frame).await?
        } else {
            // First frame - no comparison possible
            ChangeDetectionResult {
                frame_id: frame.id.clone(),
                previous_frame_id: None,
                change_score: 1.0, // First frame is always "significant change"
                pixel_diff_percentage: 1.0,
                text_diff_percentage: 1.0,
                ui_diff_percentage: 1.0,
                changed_regions: Vec::new(),
                significant_change: true,
                change_summary: "Initial frame".to_string(),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            }
        };

        // Add frame to buffer
        self.add_frame_to_buffer(frame_buffer);

        // Update hash cache
        self.hash_cache.insert(frame_hash, Utc::now());

        // Update adaptive thresholds
        if self.config.adaptive_threshold {
            self.update_adaptive_thresholds(change_result.change_score);
        }

        // Clean up old hash entries
        self.cleanup_hash_cache();

        debug!(
            "Change detection for frame {}: score={:.3}, significant={}",
            frame.id, change_result.change_score, change_result.significant_change
        );

        Ok(change_result)
    }

    async fn compare_frames(
        &self,
        current: &FrameBuffer,
        previous: &FrameBuffer,
    ) -> Result<ChangeDetectionResult> {
        let mut changed_regions = Vec::new();
        let mut total_change_score = 0.0;

        // 1. Pixel-level comparison
        let pixel_diff = self.calculate_pixel_difference(&current.image_data, &previous.image_data)?;
        
        // 2. Text content comparison
        let text_diff = self.calculate_text_difference(
            &current.text_content,
            &previous.text_content,
        );

        // 3. Region-based analysis
        let (ui_diff, region_changes) = if self.config.enable_region_analysis {
            self.analyze_regional_changes(current, previous).await?
        } else {
            (0.0, Vec::new())
        };

        changed_regions.extend(region_changes);

        // Calculate overall change score (weighted average)
        total_change_score = (pixel_diff * 0.4) + (text_diff * 0.4) + (ui_diff * 0.2);

        // Apply adaptive thresholds
        let effective_threshold = if self.config.adaptive_threshold {
            self.adaptive_thresholds.current_pixel_threshold
        } else {
            self.config.significant_change_threshold
        };

        let significant_change = total_change_score > effective_threshold;

        // Generate change summary
        let change_summary = self.generate_change_summary(
            pixel_diff,
            text_diff,
            ui_diff,
            &changed_regions,
        );

        Ok(ChangeDetectionResult {
            frame_id: current.frame.id.clone(),
            previous_frame_id: Some(previous.frame.id.clone()),
            change_score: total_change_score,
            pixel_diff_percentage: pixel_diff,
            text_diff_percentage: text_diff,
            ui_diff_percentage: ui_diff,
            changed_regions,
            significant_change,
            change_summary,
            processing_time_ms: 0, // Will be set by caller
        })
    }

    fn calculate_pixel_difference(&self, current: &[u8], previous: &[u8]) -> Result<f32> {
        if current.len() != previous.len() {
            return Ok(1.0); // Complete difference if sizes don't match
        }

        let total_pixels = current.len() / 4; // RGBA format
        let mut different_pixels = 0;
        let threshold = 30; // Pixel difference threshold (0-255)

        for i in (0..current.len()).step_by(4) {
            let curr_r = current[i] as i32;
            let curr_g = current[i + 1] as i32;
            let curr_b = current[i + 2] as i32;
            
            let prev_r = previous[i] as i32;
            let prev_g = previous[i + 1] as i32;
            let prev_b = previous[i + 2] as i32;

            let diff = ((curr_r - prev_r).pow(2) + 
                       (curr_g - prev_g).pow(2) + 
                       (curr_b - prev_b).pow(2)) as f32;
            
            if diff.sqrt() > threshold as f32 {
                different_pixels += 1;
            }
        }

        Ok(different_pixels as f32 / total_pixels as f32)
    }

    fn calculate_text_difference(
        &self,
        current_text: &Option<String>,
        previous_text: &Option<String>,
    ) -> f32 {
        if !self.config.enable_text_comparison {
            return 0.0;
        }

        match (current_text, previous_text) {
            (Some(current), Some(previous)) => {
                if current == previous {
                    0.0
                } else {
                    // Use Levenshtein distance ratio
                    let distance = levenshtein_distance(current, previous);
                    let max_len = current.len().max(previous.len());
                    if max_len == 0 {
                        0.0
                    } else {
                        distance as f32 / max_len as f32
                    }
                }
            }
            (Some(_), None) | (None, Some(_)) => 1.0, // Complete change
            (None, None) => 0.0, // No change
        }
    }

    async fn analyze_regional_changes(
        &self,
        current: &FrameBuffer,
        previous: &FrameBuffer,
    ) -> Result<(f32, Vec<ChangedRegion>)> {
        // Simplified region analysis
        // In a full implementation, this would use computer vision to detect UI regions
        
        let mut changed_regions = Vec::new();
        let mut total_ui_change = 0.0;

        // Check for window size changes
        if current.frame.resolution != previous.frame.resolution {
            changed_regions.push(ChangedRegion {
                region_id: "window".to_string(),
                region_type: RegionType::MainContent,
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: current.frame.resolution.0 as f32,
                    height: current.frame.resolution.1 as f32,
                },
                change_type: ChangeType::WindowResize,
                change_intensity: 1.0,
                description: "Window size changed".to_string(),
            });
            total_ui_change += 0.5;
        }

        // Check for application changes
        if current.frame.metadata.active_application != previous.frame.metadata.active_application {
            changed_regions.push(ChangedRegion {
                region_id: "application".to_string(),
                region_type: RegionType::MainContent,
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: current.frame.resolution.0 as f32,
                    height: current.frame.resolution.1 as f32,
                },
                change_type: ChangeType::ApplicationSwitch,
                change_intensity: 1.0,
                description: format!(
                    "Application changed from {:?} to {:?}",
                    previous.frame.metadata.active_application,
                    current.frame.metadata.active_application
                ),
            });
            total_ui_change = 1.0; // Application switch is major change
        }

        Ok((total_ui_change, changed_regions))
    }

    fn generate_change_summary(
        &self,
        pixel_diff: f32,
        text_diff: f32,
        ui_diff: f32,
        regions: &[ChangedRegion],
    ) -> String {
        let mut summary_parts = Vec::new();

        if pixel_diff > 0.2 {
            summary_parts.push(format!("Major visual change ({:.1}%)", pixel_diff * 100.0));
        } else if pixel_diff > 0.05 {
            summary_parts.push(format!("Minor visual change ({:.1}%)", pixel_diff * 100.0));
        }

        if text_diff > 0.1 {
            summary_parts.push(format!("Text content changed ({:.1}%)", text_diff * 100.0));
        }

        if ui_diff > 0.1 {
            summary_parts.push("UI layout changed".to_string());
        }

        // Add specific region changes
        for region in regions {
            match region.change_type {
                ChangeType::ApplicationSwitch => {
                    summary_parts.push("Application switched".to_string());
                }
                ChangeType::WindowResize => {
                    summary_parts.push("Window resized".to_string());
                }
                ChangeType::ContentScrolling => {
                    summary_parts.push("Content scrolled".to_string());
                }
                _ => {}
            }
        }

        if summary_parts.is_empty() {
            "No significant changes detected".to_string()
        } else {
            summary_parts.join(", ")
        }
    }

    fn calculate_image_hash(&self, image_data: &[u8]) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Sample every 16th pixel for performance
        for i in (0..image_data.len()).step_by(64) { // Every 16th pixel * 4 bytes
            if i + 3 < image_data.len() {
                let pixel = [image_data[i], image_data[i + 1], image_data[i + 2], image_data[i + 3]];
                pixel.hash(&mut hasher);
            }
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    fn add_frame_to_buffer(&mut self, frame: FrameBuffer) {
        if self.frame_buffer.len() >= self.config.buffer_size {
            self.frame_buffer.pop_front();
        }
        self.frame_buffer.push_back(frame);
    }

    fn get_last_frame_id(&self) -> Option<String> {
        self.frame_buffer.back().map(|f| f.frame.id.clone())
    }

    fn update_adaptive_thresholds(&mut self, change_score: f32) {
        self.adaptive_thresholds.recent_change_scores.push_back(change_score);
        
        if self.adaptive_thresholds.recent_change_scores.len() > 20 {
            self.adaptive_thresholds.recent_change_scores.pop_front();
        }

        // Calculate adaptive baseline
        if self.adaptive_thresholds.recent_change_scores.len() >= 10 {
            let avg_change: f32 = self.adaptive_thresholds.recent_change_scores.iter().sum::<f32>() 
                / self.adaptive_thresholds.recent_change_scores.len() as f32;
            
            // Adjust thresholds based on recent activity
            self.adaptive_thresholds.current_pixel_threshold = 
                (self.config.pixel_diff_threshold + avg_change * 0.5).clamp(0.01, 0.5);
            
            self.adaptive_thresholds.current_text_threshold = 
                (self.config.text_diff_threshold + avg_change * 0.3).clamp(0.05, 0.8);
        }
    }

    fn cleanup_hash_cache(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::minutes(5);
        self.hash_cache.retain(|_, timestamp| *timestamp > cutoff);
    }

    pub fn is_duplicate_frame(&self, image_hash: &str) -> bool {
        if let Some(last_seen) = self.hash_cache.get(image_hash) {
            let time_since = Utc::now().signed_duration_since(*last_seen);
            time_since.num_milliseconds() < 1000 // Within 1 second
        } else {
            false
        }
    }

    pub fn get_buffer_info(&self) -> (usize, usize) {
        (self.frame_buffer.len(), self.hash_cache.len())
    }

    pub fn get_adaptive_thresholds(&self) -> (f32, f32) {
        (
            self.adaptive_thresholds.current_pixel_threshold,
            self.adaptive_thresholds.current_text_threshold,
        )
    }
}

// Supporting structs and functions

#[derive(Debug)]
struct RegionAnalyzer;

impl RegionAnalyzer {
    fn new() -> Self {
        Self
    }
}

// Simple Levenshtein distance implementation
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let s1_len = s1_chars.len();
    let s2_len = s2_chars.len();

    if s1_len == 0 {
        return s2_len;
    }
    if s2_len == 0 {
        return s1_len;
    }

    let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];

    // Initialize first row and column
    for i in 0..=s1_len {
        matrix[i][0] = i;
    }
    for j in 0..=s2_len {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=s1_len {
        for j in 1..=s2_len {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[s1_len][s2_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("test", ""), 4);
    }

    #[test]
    fn test_change_detector_config() {
        let config = ChangeDetectorConfig::default();
        assert_eq!(config.buffer_size, 10);
        assert_eq!(config.pixel_diff_threshold, 0.05);
        assert!(config.enable_region_analysis);
    }

    #[tokio::test]
    async fn test_change_detector_creation() {
        let config = ChangeDetectorConfig::default();
        let detector = ChangeDetector::new(config);
        assert_eq!(detector.frame_buffer.len(), 0);
        assert_eq!(detector.hash_cache.len(), 0);
    }
}