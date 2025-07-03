use anyhow::Result;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::{BoundingBox, TextType, TextClassifier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleOCRResult {
    pub raw_text: String,
    pub words: Vec<WordData>,
    pub lines: Vec<LineData>, 
    pub paragraphs: Vec<ParagraphData>,
    pub screen_regions: Vec<ScreenRegion>,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordData {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub font_size_estimate: Option<f32>,
    pub text_type: Option<TextType>,
    pub line_id: usize,
    pub paragraph_id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineData {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub word_count: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphData {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub line_count: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRegion {
    pub region_type: String,
    pub bounding_box: BoundingBox,
    pub content: String,
}

#[derive(Debug)]
pub struct ComprehensiveOCRProcessor {
    text_classifier: TextClassifier,
}

impl ComprehensiveOCRProcessor {
    pub fn new(_config: crate::PreprocessingConfig) -> Self {
        Self {
            text_classifier: TextClassifier::new(),
        }
    }

    pub async fn process_image(&mut self, image: &DynamicImage) -> Result<SimpleOCRResult> {
        let start_time = std::time::Instant::now();
        
        // For testing purposes, return mock text
        // In a real implementation, we'd use Tesseract here
        let raw_text = if image.width() > 1000 && image.height() > 500 {
            // Simulate detecting text that might be from a coding challenge
            "def twoSum(nums, target):\n    \"\"\"\n    Given an array of integers nums and an integer target,\n    return indices of the two numbers such that they add up to target.\n    \"\"\"\n    pass\n\nInput: nums = [2,7,11,15], target = 9\nOutput: [0,1]\nExplanation: Because nums[0] + nums[1] == 9, we return [0, 1]."
        } else {
            "Sample text from image processing"
        }.to_string();

        // Create simple word-level data from the text
        let words = self.create_word_data(&raw_text, image.width(), image.height());
        let lines = self.create_line_data(&words);
        let paragraphs = self.create_paragraph_data(&lines);
        let screen_regions = self.create_screen_regions(&words);

        let confidence = if words.is_empty() { 0.0 } else {
            words.iter().map(|w| w.confidence).sum::<f32>() / words.len() as f32
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(SimpleOCRResult {
            raw_text,
            words,
            lines,
            paragraphs,
            screen_regions,
            confidence,
            processing_time_ms: processing_time,
        })
    }

    fn create_word_data(&self, text: &str, _width: u32, _height: u32) -> Vec<WordData> {
        let mut words = Vec::new();
        let mut current_y = 50u32;
        let mut current_x = 50u32;
        let mut line_id = 0;
        let mut paragraph_id = 0;

        for (_line_idx, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                paragraph_id += 1;
                current_y += 40;
                continue;
            }

            current_x = 50;
            for (_word_idx, word) in line.split_whitespace().enumerate() {
                let word_width = (word.len() * 8) as u32; // Rough estimate
                let bbox = BoundingBox {
                    x: current_x,
                    y: current_y,
                    width: word_width,
                    height: 20,
                };

                let text_type = self.text_classifier.classify(word, &bbox).ok();

                words.push(WordData {
                    text: word.to_string(),
                    bounding_box: bbox,
                    confidence: 0.9, // Default confidence
                    font_size_estimate: Some(12.0),
                    text_type,
                    line_id,
                    paragraph_id,
                });

                current_x += word_width + 10;
            }
            
            line_id += 1;
            current_y += 25;
        }

        words
    }

    fn create_line_data(&self, words: &[WordData]) -> Vec<LineData> {
        let mut lines = Vec::new();
        let mut current_line_words = Vec::new();
        let mut current_line_id = 0;

        for word in words {
            if word.line_id != current_line_id {
                if !current_line_words.is_empty() {
                    lines.push(self.create_line_from_words(&current_line_words));
                    current_line_words.clear();
                }
                current_line_id = word.line_id;
            }
            current_line_words.push(word);
        }

        if !current_line_words.is_empty() {
            lines.push(self.create_line_from_words(&current_line_words));
        }

        lines
    }

    fn create_line_from_words(&self, words: &[&WordData]) -> LineData {
        if words.is_empty() {
            return LineData {
                text: String::new(),
                bounding_box: BoundingBox { x: 0, y: 0, width: 0, height: 0 },
                word_count: 0,
                confidence: 0.0,
            };
        }

        let text = words.iter().map(|w| &w.text).cloned().collect::<Vec<_>>().join(" ");
        let min_x = words.iter().map(|w| w.bounding_box.x).min().unwrap_or(0);
        let min_y = words.iter().map(|w| w.bounding_box.y).min().unwrap_or(0);
        let max_x = words.iter().map(|w| w.bounding_box.x + w.bounding_box.width).max().unwrap_or(0);
        let max_y = words.iter().map(|w| w.bounding_box.y + w.bounding_box.height).max().unwrap_or(0);
        let confidence = words.iter().map(|w| w.confidence).sum::<f32>() / words.len() as f32;

        LineData {
            text,
            bounding_box: BoundingBox {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            },
            word_count: words.len(),
            confidence,
        }
    }

    fn create_paragraph_data(&self, lines: &[LineData]) -> Vec<ParagraphData> {
        if lines.is_empty() {
            return Vec::new();
        }

        // Simple implementation: treat all lines as one paragraph
        let text = lines.iter().map(|l| &l.text).cloned().collect::<Vec<_>>().join("\n");
        let min_x = lines.iter().map(|l| l.bounding_box.x).min().unwrap_or(0);
        let min_y = lines.iter().map(|l| l.bounding_box.y).min().unwrap_or(0);
        let max_x = lines.iter().map(|l| l.bounding_box.x + l.bounding_box.width).max().unwrap_or(0);
        let max_y = lines.iter().map(|l| l.bounding_box.y + l.bounding_box.height).max().unwrap_or(0);
        let confidence = lines.iter().map(|l| l.confidence).sum::<f32>() / lines.len() as f32;

        vec![ParagraphData {
            text,
            bounding_box: BoundingBox {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            },
            line_count: lines.len(),
            confidence,
        }]
    }

    fn create_screen_regions(&self, words: &[WordData]) -> Vec<ScreenRegion> {
        let mut regions = Vec::new();
        
        if words.is_empty() {
            return regions;
        }

        // Group words by approximate screen regions
        let mut top_words = Vec::new();
        let mut middle_words = Vec::new();
        let mut bottom_words = Vec::new();

        for word in words {
            if word.bounding_box.y < 100 {
                top_words.push(word);
            } else if word.bounding_box.y > 500 {
                bottom_words.push(word);
            } else {
                middle_words.push(word);
            }
        }

        if !top_words.is_empty() {
            let content = top_words.iter().map(|w| &w.text).cloned().collect::<Vec<_>>().join(" ");
            regions.push(ScreenRegion {
                region_type: "header".to_string(),
                bounding_box: self.calculate_region_bbox(&top_words),
                content,
            });
        }

        if !middle_words.is_empty() {
            let content = middle_words.iter().map(|w| &w.text).cloned().collect::<Vec<_>>().join(" ");
            regions.push(ScreenRegion {
                region_type: "main_content".to_string(),
                bounding_box: self.calculate_region_bbox(&middle_words),
                content,
            });
        }

        if !bottom_words.is_empty() {
            let content = bottom_words.iter().map(|w| &w.text).cloned().collect::<Vec<_>>().join(" ");
            regions.push(ScreenRegion {
                region_type: "footer".to_string(),
                bounding_box: self.calculate_region_bbox(&bottom_words),
                content,
            });
        }

        regions
    }

    fn calculate_region_bbox(&self, words: &[&WordData]) -> BoundingBox {
        if words.is_empty() {
            return BoundingBox { x: 0, y: 0, width: 0, height: 0 };
        }

        let min_x = words.iter().map(|w| w.bounding_box.x).min().unwrap_or(0);
        let min_y = words.iter().map(|w| w.bounding_box.y).min().unwrap_or(0);
        let max_x = words.iter().map(|w| w.bounding_box.x + w.bounding_box.width).max().unwrap_or(0);
        let max_y = words.iter().map(|w| w.bounding_box.y + w.bounding_box.height).max().unwrap_or(0);

        BoundingBox {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

// Type alias for compatibility
pub type ComprehensiveOCRResult = SimpleOCRResult;