use anyhow::Result;
use chrono::{DateTime, Utc};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod engine;
pub mod preprocessor;
pub mod classifier;
pub mod analyzer;
pub mod simple_extractor;
pub mod fast_config;

pub use engine::{OCREngine, TesseractEngine};
pub use preprocessor::{ImagePreprocessor, PreprocessingConfig};
pub use classifier::{TextClassifier, TextType};
pub use analyzer::{StructuredContentAnalyzer, StructuredContent};
pub use simple_extractor::{ComprehensiveOCRProcessor, ComprehensiveOCRResult, WordData, LineData, ParagraphData};
pub use fast_config::{FastOCRConfig, FastOCRProcessor, OCRPresets, OCRPerformanceMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub size: Option<f32>,
    pub family: Option<String>,
    pub style: Option<String>,
    pub weight: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
    pub font_info: Option<FontInfo>,
    pub semantic_type: TextType,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRResult {
    pub text_blocks: Vec<TextBlock>,
    pub structured_content: StructuredContent,
    pub overall_confidence: f32,
    pub processing_time_ms: u64,
    pub detected_language: String,
    pub image_metadata: ImageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRConfig {
    pub engine: String, // "tesseract", "easyocr", etc.
    pub languages: Vec<String>,
    pub preprocessing: PreprocessingConfig,
    pub min_confidence: f32,
    pub enable_text_classification: bool,
    pub enable_structure_analysis: bool,
    pub parallel_processing: bool,
}

impl Default for OCRConfig {
    fn default() -> Self {
        Self {
            engine: "tesseract".to_string(),
            languages: vec!["eng".to_string()],
            preprocessing: PreprocessingConfig::default(),
            min_confidence: 0.5,
            enable_text_classification: true,
            enable_structure_analysis: true,
            parallel_processing: true,
        }
    }
}

pub struct OCRProcessor {
    pub engine: Box<dyn OCREngine>,
    pub preprocessor: ImagePreprocessor,
    pub text_classifier: TextClassifier,
    pub content_analyzer: StructuredContentAnalyzer,
    pub config: OCRConfig,
}

impl OCRProcessor {
    pub fn new(config: OCRConfig) -> Result<Self> {
        let engine: Box<dyn OCREngine> = match config.engine.as_str() {
            "tesseract" => Box::new(TesseractEngine::new(&config.languages)?),
            _ => anyhow::bail!("Unsupported OCR engine: {}", config.engine),
        };

        Ok(Self {
            engine,
            preprocessor: ImagePreprocessor::new(config.preprocessing.clone()),
            text_classifier: TextClassifier::new(),
            content_analyzer: StructuredContentAnalyzer::new(),
            config,
        })
    }

    pub async fn process_image(&self, image: &DynamicImage) -> Result<OCRResult> {
        let start_time = std::time::Instant::now();
        
        // Preprocess image
        let processed_image = if self.config.preprocessing.enabled {
            self.preprocessor.process(image)?
        } else {
            image.clone()
        };

        // Extract text
        let text_blocks = self.engine.extract_text(&processed_image).await?;

        // Filter by confidence
        let filtered_blocks: Vec<TextBlock> = text_blocks
            .into_iter()
            .filter(|block| block.confidence >= self.config.min_confidence)
            .collect();

        // Classify text types
        let classified_blocks = if self.config.enable_text_classification {
            self.classify_text_blocks(filtered_blocks)?
        } else {
            filtered_blocks
        };

        // Analyze structure
        let structured_content = if self.config.enable_structure_analysis {
            self.content_analyzer.analyze(&classified_blocks)?
        } else {
            StructuredContent::default()
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        let overall_confidence = classified_blocks
            .iter()
            .map(|b| b.confidence)
            .sum::<f32>() / classified_blocks.len().max(1) as f32;

        let detected_language = self.detect_primary_language(&classified_blocks);

        Ok(OCRResult {
            text_blocks: classified_blocks,
            structured_content,
            overall_confidence,
            processing_time_ms: processing_time,
            detected_language,
            image_metadata: ImageMetadata {
                width: image.width(),
                height: image.height(),
                format: "DynamicImage".to_string(),
                file_size: None,
                timestamp: Utc::now(),
            },
        })
    }

    fn classify_text_blocks(&self, blocks: Vec<TextBlock>) -> Result<Vec<TextBlock>> {
        let mut classified_blocks = Vec::new();
        
        for mut block in blocks {
            let text_type = self.text_classifier.classify(&block.text, &block.bounding_box)?;
            block.semantic_type = text_type;
            classified_blocks.push(block);
        }
        
        Ok(classified_blocks)
    }

    fn detect_primary_language(&self, blocks: &[TextBlock]) -> String {
        let mut language_counts: HashMap<String, usize> = HashMap::new();
        
        for block in blocks {
            if let Some(ref lang) = block.language {
                *language_counts.entry(lang.clone()).or_insert(0) += 1;
            }
        }
        
        language_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
            .unwrap_or_else(|| "eng".to_string())
    }
}