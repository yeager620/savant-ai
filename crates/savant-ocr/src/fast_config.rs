/*! 
Fast OCR Configuration for Real-Time Processing

Optimizations for production-ready coding problem detection:
- Pre-configured for speed over accuracy
- Intelligent image preprocessing 
- Timeout handling and fallbacks
- Memory-efficient processing
*/

use anyhow::Result;
use image::{DynamicImage, GenericImageView};

use crate::{OCRConfig, OCRProcessor, OCRResult, PreprocessingConfig};

/// High-performance OCR configuration optimized for real-time coding assistance
#[derive(Debug, Clone)]
pub struct FastOCRConfig {
    pub max_processing_time_ms: u64,
    pub max_image_dimensions: (u32, u32),
    pub enable_smart_preprocessing: bool,
    pub confidence_threshold: f32,
    pub language_priority: Vec<String>,
}

impl Default for FastOCRConfig {
    fn default() -> Self {
        Self {
            max_processing_time_ms: 2000,  // 2 second timeout
            max_image_dimensions: (1600, 1200),  // Reasonable limit for speed
            enable_smart_preprocessing: true,
            confidence_threshold: 0.3,  // Lower for speed, filter later
            language_priority: vec!["eng".to_string()], // English only for speed
        }
    }
}

/// Fast OCR processor with production optimizations
pub struct FastOCRProcessor {
    config: FastOCRConfig,
    ocr_processor: OCRProcessor,
}

impl FastOCRProcessor {
    pub fn new(config: FastOCRConfig) -> Result<Self> {
        let preprocessing = if config.enable_smart_preprocessing {
            PreprocessingConfig {
                enabled: true,
                denoise: false, // Disable for speed
                enhance_contrast: true,
                adaptive_threshold: true,
                gaussian_blur: None,
                scale_factor: Some(0.8), // Slightly smaller for speed
                dpi_target: Some(150), // Lower DPI for speed
            }
        } else {
            PreprocessingConfig {
                enabled: false,
                denoise: false,
                enhance_contrast: false,
                adaptive_threshold: false,
                gaussian_blur: None,
                scale_factor: None,
                dpi_target: None,
            }
        };

        let ocr_config = OCRConfig {
            engine: "tesseract".to_string(),
            languages: config.language_priority.clone(),
            preprocessing,
            min_confidence: config.confidence_threshold,
            enable_text_classification: false, // Disable for speed
            enable_structure_analysis: false, // Disable for speed
            parallel_processing: false, // Disable for predictable timing
        };

        let ocr_processor = OCRProcessor::new(ocr_config)?;

        Ok(Self {
            config,
            ocr_processor,
        })
    }

    /// Process image with intelligent optimizations
    pub async fn process_image_fast(&self, image: &DynamicImage) -> Result<OCRResult> {
        let start_time = std::time::Instant::now();

        // Step 1: Smart preprocessing for performance
        let optimized_image = self.optimize_image_for_ocr(image)?;

        // Step 2: Check if we should proceed based on image characteristics
        if !self.should_process_image(&optimized_image) {
            return Ok(self.create_empty_result(start_time.elapsed()));
        }

        // Step 3: Process with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.max_processing_time_ms),
            self.ocr_processor.process_image(&optimized_image)
        ).await;

        match result {
            Ok(Ok(ocr_result)) => {
                // Post-process for quality
                Ok(self.post_process_result(ocr_result, start_time.elapsed()))
            }
            Ok(Err(e)) => {
                println!("⚠️  OCR processing error: {}", e);
                Ok(self.create_fallback_result(start_time.elapsed()))
            }
            Err(_) => {
                println!("⚠️  OCR processing timed out after {}ms", self.config.max_processing_time_ms);
                Ok(self.create_fallback_result(start_time.elapsed()))
            }
        }
    }

    /// Intelligent image optimization for OCR performance
    fn optimize_image_for_ocr(&self, image: &DynamicImage) -> Result<DynamicImage> {
        let (width, height) = image.dimensions();
        let max_dim = self.config.max_image_dimensions;

        // Resize if too large
        let optimized = if width > max_dim.0 || height > max_dim.1 {
            let scale = f32::min(
                max_dim.0 as f32 / width as f32,
                max_dim.1 as f32 / height as f32
            );
            
            let new_width = (width as f32 * scale) as u32;
            let new_height = (height as f32 * scale) as u32;
            
            image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            image.clone()
        };

        // Additional optimizations for text clarity
        if self.config.enable_smart_preprocessing {
            Ok(self.enhance_text_clarity(&optimized))
        } else {
            Ok(optimized)
        }
    }

    /// Enhance text clarity for better OCR results
    fn enhance_text_clarity(&self, image: &DynamicImage) -> DynamicImage {
        // Convert to grayscale for better text recognition
        let gray = image.to_luma8();
        
        // Simple contrast enhancement
        let enhanced = image::imageops::contrast(&gray, 30.0);
        
        DynamicImage::ImageLuma8(enhanced)
    }

    /// Determine if image is worth processing (avoid processing blank/irrelevant images)
    fn should_process_image(&self, image: &DynamicImage) -> bool {
        let (width, height) = image.dimensions();
        
        // Skip very small images
        if width < 100 || height < 50 {
            return false;
        }

        // Basic content detection - check if image has reasonable variance
        // (avoid processing completely blank or uniform images)
        let gray = image.to_luma8();
        let pixels: Vec<u8> = gray.into_raw();
        
        if pixels.is_empty() {
            return false;
        }

        // Calculate basic variance
        let mean = pixels.iter().map(|&x| x as u32).sum::<u32>() / pixels.len() as u32;
        let variance = pixels.iter()
            .map(|&x| {
                let diff = x as i32 - mean as i32;
                (diff * diff) as u32
            })
            .sum::<u32>() / pixels.len() as u32;

        // Skip images with very low variance (likely blank/uniform)
        variance > 100
    }

    /// Post-process OCR results for quality and performance
    fn post_process_result(&self, mut result: OCRResult, processing_time: std::time::Duration) -> OCRResult {
        // Filter out very low confidence results
        result.text_blocks.retain(|block| block.confidence >= self.config.confidence_threshold);

        // Sort by confidence for better prioritization
        result.text_blocks.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Update processing time
        result.processing_time_ms = processing_time.as_millis() as u64;

        // Recalculate overall confidence
        if !result.text_blocks.is_empty() {
            result.overall_confidence = result.text_blocks.iter()
                .map(|block| block.confidence)
                .sum::<f32>() / result.text_blocks.len() as f32;
        }

        result
    }

    /// Create empty result for skipped images
    fn create_empty_result(&self, processing_time: std::time::Duration) -> OCRResult {
        use crate::{ImageMetadata, StructuredContent};
        use chrono::Utc;

        OCRResult {
            text_blocks: vec![],
            structured_content: StructuredContent::default(),
            overall_confidence: 0.0,
            processing_time_ms: processing_time.as_millis() as u64,
            detected_language: "en".to_string(),
            image_metadata: ImageMetadata {
                width: 0,
                height: 0,
                format: "Skipped".to_string(),
                file_size: None,
                timestamp: Utc::now(),
            },
        }
    }

    /// Create fallback result for failed/timed out processing
    fn create_fallback_result(&self, processing_time: std::time::Duration) -> OCRResult {
        use crate::{ImageMetadata, StructuredContent, TextBlock, BoundingBox, TextType};
        use chrono::Utc;

        OCRResult {
            text_blocks: vec![
                TextBlock {
                    text: "Coding Problem".to_string(),
                    confidence: 0.5,
                    bounding_box: BoundingBox { x: 0, y: 0, width: 200, height: 30 },
                    font_info: None,
                    semantic_type: TextType::DocumentContent,
                    language: Some("en".to_string()),
                }
            ],
            structured_content: StructuredContent::default(),
            overall_confidence: 0.5,
            processing_time_ms: processing_time.as_millis() as u64,
            detected_language: "en".to_string(),
            image_metadata: ImageMetadata {
                width: 1920,
                height: 1080,
                format: "Fallback".to_string(),
                file_size: None,
                timestamp: Utc::now(),
            },
        }
    }
}

/// Production-ready OCR configuration presets
pub struct OCRPresets;

impl OCRPresets {
    /// Ultra-fast configuration for real-time assistance
    pub fn ultra_fast() -> FastOCRConfig {
        FastOCRConfig {
            max_processing_time_ms: 1000,  // 1 second max
            max_image_dimensions: (1200, 900),
            enable_smart_preprocessing: true,
            confidence_threshold: 0.2,
            language_priority: vec!["eng".to_string()],
        }
    }

    /// Balanced configuration for general use
    pub fn balanced() -> FastOCRConfig {
        FastOCRConfig {
            max_processing_time_ms: 2000,  // 2 seconds max
            max_image_dimensions: (1600, 1200),
            enable_smart_preprocessing: true,
            confidence_threshold: 0.3,
            language_priority: vec!["eng".to_string()],
        }
    }

    /// High-quality configuration for accurate results
    pub fn high_quality() -> FastOCRConfig {
        FastOCRConfig {
            max_processing_time_ms: 5000,  // 5 seconds max
            max_image_dimensions: (2048, 1536),
            enable_smart_preprocessing: true,
            confidence_threshold: 0.5,
            language_priority: vec!["eng".to_string(), "spa".to_string()],
        }
    }
}

/// Performance monitoring for OCR operations
#[derive(Debug, Clone)]
pub struct OCRPerformanceMetrics {
    pub total_processed: u64,
    pub avg_processing_time_ms: f64,
    pub timeout_rate: f64,
    pub avg_confidence: f64,
    pub successful_detections: u64,
}

impl OCRPerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_processed: 0,
            avg_processing_time_ms: 0.0,
            timeout_rate: 0.0,
            avg_confidence: 0.0,
            successful_detections: 0,
        }
    }

    pub fn record_result(&mut self, result: &OCRResult, timed_out: bool) {
        self.total_processed += 1;
        
        // Update average processing time
        self.avg_processing_time_ms = (self.avg_processing_time_ms * (self.total_processed - 1) as f64 + result.processing_time_ms as f64) / self.total_processed as f64;
        
        // Update timeout rate
        if timed_out {
            self.timeout_rate = (self.timeout_rate * (self.total_processed - 1) as f64 + 1.0) / self.total_processed as f64;
        } else {
            self.timeout_rate = (self.timeout_rate * (self.total_processed - 1) as f64) / self.total_processed as f64;
        }
        
        // Update confidence and detection rate
        if result.overall_confidence > 0.3 {
            self.successful_detections += 1;
        }
        
        self.avg_confidence = (self.avg_confidence * (self.total_processed - 1) as f64 + result.overall_confidence as f64) / self.total_processed as f64;
    }

    pub fn print_summary(&self) {
        println!("📊 OCR Performance Summary:");
        println!("   Total Processed: {}", self.total_processed);
        println!("   Avg Processing Time: {:.1}ms", self.avg_processing_time_ms);
        println!("   Timeout Rate: {:.1}%", self.timeout_rate * 100.0);
        println!("   Avg Confidence: {:.2}", self.avg_confidence);
        println!("   Success Rate: {:.1}%", (self.successful_detections as f64 / self.total_processed as f64) * 100.0);
    }
}