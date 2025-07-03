use anyhow::Result;
use async_trait::async_trait;
use image::DynamicImage;
use crate::{BoundingBox, TextBlock, TextType};

#[async_trait]
pub trait OCREngine: Send {
    async fn extract_text(&self, image: &DynamicImage) -> Result<Vec<TextBlock>>;
    fn get_supported_languages(&self) -> Vec<String>;
    fn set_language(&mut self, languages: &[String]) -> Result<()>;
    fn clone_engine(&self) -> Result<Box<dyn OCREngine>>;
}

pub struct TesseractEngine {
    languages: Vec<String>,
}

impl TesseractEngine {
    pub fn new(languages: &[String]) -> Result<Self> {
        Ok(Self {
            languages: languages.to_vec(),
        })
    }
}

#[async_trait]
impl OCREngine for TesseractEngine {
    async fn extract_text(&self, image: &DynamicImage) -> Result<Vec<TextBlock>> {
        use std::io::Cursor;

        // Convert to PNG bytes for Tesseract (more reliable than raw pixels)
        let mut png_data = Vec::new();
        image.write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
            .map_err(|e| anyhow::anyhow!("Failed to encode image as PNG: {}", e))?;

        // Create a temporary Tesseract instance for this operation
        let api = tesseract::Tesseract::new(None, Some(&self.languages.join("+")))?;
        let api = api.set_variable("tessedit_create_tsv", "1")?;

        // Set image data from PNG bytes
        let mut api = api.set_image_from_mem(&png_data)?;

        // Get TSV data for detailed block information
        let tsv_data = api.get_tsv_text(1)?;
        let text_blocks = self.parse_tsv_output(&tsv_data)?;

        Ok(text_blocks)
    }

    fn get_supported_languages(&self) -> Vec<String> {
        // Common Tesseract language codes
        vec![
            "eng".to_string(),
            "spa".to_string(),
            "fra".to_string(),
            "deu".to_string(),
            "ita".to_string(),
            "por".to_string(),
            "rus".to_string(),
            "jpn".to_string(),
            "kor".to_string(),
            "chi_sim".to_string(),
            "chi_tra".to_string(),
        ]
    }

    fn set_language(&mut self, languages: &[String]) -> Result<()> {
        self.languages = languages.to_vec();
        Ok(())
    }

    fn clone_engine(&self) -> Result<Box<dyn OCREngine>> {
        Ok(Box::new(TesseractEngine::new(&self.languages)?))
    }
}

impl TesseractEngine {
    fn parse_tsv_output(&self, tsv_data: &str) -> Result<Vec<TextBlock>> {
        let mut text_blocks = Vec::new();
        let lines: Vec<&str> = tsv_data.lines().skip(1).collect(); // Skip header

        for line in lines {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 12 {
                continue;
            }

            // Parse TSV fields: level, page_num, block_num, par_num, line_num, word_num,
            // left, top, width, height, conf, text
            let level: i32 = fields[0].parse().unwrap_or(0);
            let left: u32 = fields[6].parse().unwrap_or(0);
            let top: u32 = fields[7].parse().unwrap_or(0);
            let width: u32 = fields[8].parse().unwrap_or(0);
            let height: u32 = fields[9].parse().unwrap_or(0);
            let confidence: f32 = fields[10].parse().unwrap_or(0.0) / 100.0; // Convert to 0-1 range
            let text = fields[11].trim();

            // Only process word-level (level 5) with actual text
            if level == 5 && !text.is_empty() && confidence > 0.0 {
                let text_block = TextBlock {
                    text: text.to_string(),
                    confidence,
                    bounding_box: BoundingBox {
                        x: left,
                        y: top,
                        width,
                        height,
                    },
                    font_info: None, // Tesseract doesn't provide font info directly
                    semantic_type: TextType::Unknown, // Will be classified later
                    language: Some(self.languages.first().unwrap_or(&"eng".to_string()).clone()),
                };
                text_blocks.push(text_block);
            }
        }

        Ok(text_blocks)
    }
}
