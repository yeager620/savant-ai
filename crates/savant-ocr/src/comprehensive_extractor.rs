use anyhow::Result;
use image::{DynamicImage, GrayImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tesseract::{Tesseract, PageSegMode};
use tracing::{debug, warn};

use crate::{OCRResult, TextBlock, TextType, BoundingBox};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveOCRResult {
    pub raw_text: String,
    pub word_level_data: Vec<WordData>,
    pub line_level_data: Vec<LineData>,
    pub paragraph_level_data: Vec<ParagraphData>,
    pub text_regions: Vec<TextRegion>,
    pub confidence_map: HashMap<String, f32>,
    pub processing_time_ms: u64,
    pub screen_layout: ScreenLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordData {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub font_size: Option<u32>,
    pub is_bold: Option<bool>,
    pub line_id: String,
    pub paragraph_id: String,
    pub semantic_type: TextType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineData {
    pub id: String,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub words: Vec<String>, // Word IDs
    pub average_confidence: f32,
    pub text_alignment: TextAlignment,
    pub is_heading: bool,
    pub font_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphData {
    pub id: String,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub lines: Vec<String>, // Line IDs
    pub semantic_type: TextType,
    pub reading_order: u32,
    pub text_direction: TextDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    pub id: String,
    pub region_type: RegionType,
    pub bounding_box: BoundingBox,
    pub paragraphs: Vec<String>, // Paragraph IDs
    pub ui_context: UIContext,
    pub interaction_elements: Vec<InteractionElement>,
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
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIContext {
    pub window_title: Option<String>,
    pub application_name: Option<String>,
    pub ui_framework: Option<String>, // Qt, Electron, Web, Native, etc.
    pub theme: Option<String>, // Dark, Light, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionElement {
    pub element_type: ElementType,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub is_clickable: bool,
    pub has_focus: bool,
    pub state: ElementState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Button,
    Link,
    TextField,
    Checkbox,
    RadioButton,
    Dropdown,
    Tab,
    MenuItem,
    Icon,
    Label,
    Error,
    Warning,
    Success,
    Progress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementState {
    Normal,
    Hover,
    Active,
    Disabled,
    Selected,
    Checked,
    Unchecked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenLayout {
    pub screen_resolution: (u32, u32),
    pub effective_area: BoundingBox, // Excluding OS chrome
    pub regions: Vec<LayoutRegion>,
    pub navigation_elements: Vec<NavigationElement>,
    pub content_hierarchy: ContentHierarchy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRegion {
    pub id: String,
    pub region_type: RegionType,
    pub bounding_box: BoundingBox,
    pub z_order: u32,
    pub is_scrollable: bool,
    pub scroll_position: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationElement {
    pub element_type: NavigationType,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub is_active: bool,
    pub parent_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationType {
    BreadcrumbItem,
    TabItem,
    MenuPath,
    FilePathSegment,
    URL,
    PageIndicator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHierarchy {
    pub main_heading: Option<String>,
    pub subheadings: Vec<HeadingData>,
    pub content_sections: Vec<ContentSection>,
    pub information_density: f32, // Text density per region
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingData {
    pub level: u32, // H1, H2, etc.
    pub text: String,
    pub bounding_box: BoundingBox,
    pub section_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSection {
    pub id: String,
    pub title: Option<String>,
    pub content_type: ContentType,
    pub bounding_box: BoundingBox,
    pub word_count: u32,
    pub reading_time_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Article,
    Code,
    Chat,
    Email,
    Form,
    List,
    Table,
    Media,
    Navigation,
    Advertisement,
    Footer,
}

pub struct ComprehensiveOCRProcessor {
    tesseract: Tesseract,
    word_tesseract: Tesseract,
    layout_analyzer: LayoutAnalyzer,
    semantic_classifier: SemanticClassifier,
    ui_detector: UIDetector,
}

impl ComprehensiveOCRProcessor {
    pub fn new() -> Result<Self> {
        let mut tesseract = Tesseract::new(None, Some("eng"))?;
        tesseract.set_page_seg_mode(PageSegMode::PsmAutoOsd)?;
        tesseract.set_variable("preserve_interword_spaces", "1")?;

        let mut word_tesseract = Tesseract::new(None, Some("eng"))?;
        word_tesseract.set_page_seg_mode(PageSegMode::PsmSingleWord)?;

        Ok(Self {
            tesseract,
            word_tesseract,
            layout_analyzer: LayoutAnalyzer::new(),
            semantic_classifier: SemanticClassifier::new(),
            ui_detector: UIDetector::new(),
        })
    }

    pub async fn extract_comprehensive_text(&mut self, image: &DynamicImage) -> Result<ComprehensiveOCRResult> {
        let start_time = std::time::Instant::now();
        
        debug!("Starting comprehensive OCR extraction for {}x{} image", image.width(), image.height());

        // Convert to grayscale for processing
        let gray_image = image.to_luma8();
        
        // Analyze screen layout first
        let screen_layout = self.layout_analyzer.analyze_layout(image).await?;
        
        // Extract text at multiple levels
        let raw_text = self.extract_raw_text(&gray_image).await?;
        let word_data = self.extract_word_level_data(&gray_image, &screen_layout).await?;
        let line_data = self.extract_line_level_data(&word_data).await?;
        let paragraph_data = self.extract_paragraph_level_data(&line_data).await?;
        
        // Analyze text regions and UI elements
        let text_regions = self.analyze_text_regions(&paragraph_data, &screen_layout).await?;
        
        // Build confidence map
        let confidence_map = self.build_confidence_map(&word_data);
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        debug!(
            "Comprehensive OCR complete: {} words, {} lines, {} paragraphs in {}ms",
            word_data.len(),
            line_data.len(),
            paragraph_data.len(),
            processing_time
        );

        Ok(ComprehensiveOCRResult {
            raw_text,
            word_level_data: word_data,
            line_level_data: line_data,
            paragraph_level_data: paragraph_data,
            text_regions,
            confidence_map,
            processing_time_ms: processing_time,
            screen_layout,
        })
    }

    async fn extract_raw_text(&mut self, image: &GrayImage) -> Result<String> {
        self.tesseract.set_image_from_mem(image.as_raw())?;
        Ok(self.tesseract.get_text()?)
    }

    async fn extract_word_level_data(
        &mut self,
        image: &GrayImage,
        layout: &ScreenLayout,
    ) -> Result<Vec<WordData>> {
        let mut words = Vec::new();
        
        self.tesseract.set_image_from_mem(image.as_raw())?;
        
        // Use Tesseract's iterator to get word-level data
        let boxes = self.tesseract.get_component_images(tesseract::PageIteratorLevel::Word, true)?;
        
        for (i, word_info) in boxes.iter().enumerate() {
            if let Ok(text) = self.tesseract.get_utf8_text(tesseract::PageIteratorLevel::Word) {
                if !text.trim().is_empty() {
                    let bbox = BoundingBox {
                        x: word_info.x as f32,
                        y: word_info.y as f32,
                        width: word_info.w as f32,
                        height: word_info.h as f32,
                    };
                    
                    let confidence = self.tesseract.mean_text_conf() as f32 / 100.0;
                    let semantic_type = self.semantic_classifier.classify_word(&text, &bbox, layout);
                    
                    words.push(WordData {
                        text: text.trim().to_string(),
                        bounding_box: bbox,
                        confidence,
                        font_size: self.estimate_font_size(&bbox),
                        is_bold: self.detect_bold_text(&text, &bbox),
                        line_id: format!("line_{}", i / 10), // Rough grouping
                        paragraph_id: format!("para_{}", i / 50), // Rough grouping
                        semantic_type,
                    });
                }
            }
        }
        
        Ok(words)
    }

    async fn extract_line_level_data(&self, words: &[WordData]) -> Result<Vec<LineData>> {
        let mut lines = Vec::new();
        let mut line_groups: HashMap<String, Vec<&WordData>> = HashMap::new();
        
        // Group words by line
        for word in words {
            line_groups.entry(word.line_id.clone()).or_default().push(word);
        }
        
        for (line_id, line_words) in line_groups {
            if line_words.is_empty() {
                continue;
            }
            
            // Calculate line bounding box
            let min_x = line_words.iter().map(|w| w.bounding_box.x).fold(f32::INFINITY, f32::min);
            let min_y = line_words.iter().map(|w| w.bounding_box.y).fold(f32::INFINITY, f32::min);
            let max_x = line_words.iter().map(|w| w.bounding_box.x + w.bounding_box.width).fold(0.0, f32::max);
            let max_y = line_words.iter().map(|w| w.bounding_box.y + w.bounding_box.height).fold(0.0, f32::max);
            
            let text = line_words.iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ");
            let avg_confidence = line_words.iter().map(|w| w.confidence).sum::<f32>() / line_words.len() as f32;
            
            lines.push(LineData {
                id: line_id,
                text,
                bounding_box: BoundingBox {
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                },
                words: line_words.iter().map(|w| w.text.clone()).collect(),
                average_confidence: avg_confidence,
                text_alignment: self.determine_text_alignment(&line_words),
                is_heading: self.is_heading_line(&text, &line_words),
                font_size: line_words.first().and_then(|w| w.font_size),
            });
        }
        
        Ok(lines)
    }

    async fn extract_paragraph_level_data(&self, lines: &[LineData]) -> Result<Vec<ParagraphData>> {
        let mut paragraphs = Vec::new();
        let mut para_groups: HashMap<String, Vec<&LineData>> = HashMap::new();
        
        // Group lines by paragraph (simplified grouping based on proximity)
        for line in lines {
            let para_id = self.determine_paragraph_id(line, lines);
            para_groups.entry(para_id).or_default().push(line);
        }
        
        for (para_id, para_lines) in para_groups {
            if para_lines.is_empty() {
                continue;
            }
            
            // Calculate paragraph bounding box
            let min_x = para_lines.iter().map(|l| l.bounding_box.x).fold(f32::INFINITY, f32::min);
            let min_y = para_lines.iter().map(|l| l.bounding_box.y).fold(f32::INFINITY, f32::min);
            let max_x = para_lines.iter().map(|l| l.bounding_box.x + l.bounding_box.width).fold(0.0, f32::max);
            let max_y = para_lines.iter().map(|l| l.bounding_box.y + l.bounding_box.height).fold(0.0, f32::max);
            
            let text = para_lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n");
            let semantic_type = self.semantic_classifier.classify_paragraph(&text);
            
            paragraphs.push(ParagraphData {
                id: para_id,
                text,
                bounding_box: BoundingBox {
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                },
                lines: para_lines.iter().map(|l| l.id.clone()).collect(),
                semantic_type,
                reading_order: paragraphs.len() as u32,
                text_direction: TextDirection::LeftToRight, // Default for now
            });
        }
        
        Ok(paragraphs)
    }

    async fn analyze_text_regions(
        &self,
        paragraphs: &[ParagraphData],
        layout: &ScreenLayout,
    ) -> Result<Vec<TextRegion>> {
        let mut regions = Vec::new();
        
        for layout_region in &layout.regions {
            let region_paragraphs: Vec<String> = paragraphs
                .iter()
                .filter(|p| self.bbox_overlap(&p.bounding_box, &layout_region.bounding_box) > 0.5)
                .map(|p| p.id.clone())
                .collect();
            
            if !region_paragraphs.is_empty() {
                let ui_context = self.ui_detector.analyze_ui_context(&layout_region.bounding_box);
                let interaction_elements = self.ui_detector.detect_interaction_elements(&layout_region.bounding_box);
                
                regions.push(TextRegion {
                    id: layout_region.id.clone(),
                    region_type: layout_region.region_type.clone(),
                    bounding_box: layout_region.bounding_box.clone(),
                    paragraphs: region_paragraphs,
                    ui_context,
                    interaction_elements,
                });
            }
        }
        
        Ok(regions)
    }

    fn build_confidence_map(&self, words: &[WordData]) -> HashMap<String, f32> {
        let mut confidence_map = HashMap::new();
        
        // Overall confidence
        let overall_confidence = words.iter().map(|w| w.confidence).sum::<f32>() / words.len() as f32;
        confidence_map.insert("overall".to_string(), overall_confidence);
        
        // Per semantic type confidence
        let mut type_confidences: HashMap<String, Vec<f32>> = HashMap::new();
        for word in words {
            type_confidences
                .entry(format!("{:?}", word.semantic_type))
                .or_default()
                .push(word.confidence);
        }
        
        for (type_name, confidences) in type_confidences {
            let avg_confidence = confidences.iter().sum::<f32>() / confidences.len() as f32;
            confidence_map.insert(type_name, avg_confidence);
        }
        
        confidence_map
    }

    // Helper methods
    fn estimate_font_size(&self, bbox: &BoundingBox) -> Option<u32> {
        Some((bbox.height * 0.75) as u32) // Rough estimation
    }

    fn detect_bold_text(&self, _text: &str, _bbox: &BoundingBox) -> Option<bool> {
        None // TODO: Implement based on character thickness analysis
    }

    fn determine_text_alignment(&self, words: &[&WordData]) -> TextAlignment {
        if words.is_empty() {
            return TextAlignment::Left;
        }
        
        // Simple heuristic based on word positions
        let first_x = words[0].bounding_box.x;
        let last_x = words.last().unwrap().bounding_box.x + words.last().unwrap().bounding_box.width;
        let center_screen = 960.0; // Assuming 1920px wide screen
        
        if (first_x - 50.0..first_x + 50.0).contains(&center_screen) {
            TextAlignment::Center
        } else if last_x > center_screen * 1.5 {
            TextAlignment::Right
        } else {
            TextAlignment::Left
        }
    }

    fn is_heading_line(&self, text: &str, words: &[&WordData]) -> bool {
        // Simple heuristics for heading detection
        if words.is_empty() {
            return false;
        }
        
        let avg_font_size = words.iter()
            .filter_map(|w| w.font_size)
            .sum::<u32>() as f32 / words.len() as f32;
        
        text.len() < 100 && // Short text
        avg_font_size > 16.0 && // Larger font
        text.chars().filter(|c| c.is_uppercase()).count() as f32 / text.len() as f32 > 0.3 // Many capitals
    }

    fn determine_paragraph_id(&self, line: &LineData, _all_lines: &[LineData]) -> String {
        // Simplified: group by vertical proximity
        format!("para_{}", (line.bounding_box.y / 100.0) as u32)
    }

    fn bbox_overlap(&self, bbox1: &BoundingBox, bbox2: &BoundingBox) -> f32 {
        let overlap_x = (bbox1.x + bbox1.width).min(bbox2.x + bbox2.width) - bbox1.x.max(bbox2.x);
        let overlap_y = (bbox1.y + bbox1.height).min(bbox2.y + bbox2.height) - bbox1.y.max(bbox2.y);
        
        if overlap_x <= 0.0 || overlap_y <= 0.0 {
            return 0.0;
        }
        
        let overlap_area = overlap_x * overlap_y;
        let bbox1_area = bbox1.width * bbox1.height;
        
        overlap_area / bbox1_area
    }
}

// Supporting analyzer structs
struct LayoutAnalyzer;

impl LayoutAnalyzer {
    fn new() -> Self {
        Self
    }

    async fn analyze_layout(&self, image: &DynamicImage) -> Result<ScreenLayout> {
        // TODO: Implement comprehensive layout analysis
        Ok(ScreenLayout {
            screen_resolution: (image.width(), image.height()),
            effective_area: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: image.width() as f32,
                height: image.height() as f32,
            },
            regions: vec![
                LayoutRegion {
                    id: "main_content".to_string(),
                    region_type: RegionType::MainContent,
                    bounding_box: BoundingBox {
                        x: 0.0,
                        y: 0.0,
                        width: image.width() as f32,
                        height: image.height() as f32,
                    },
                    z_order: 1,
                    is_scrollable: true,
                    scroll_position: None,
                }
            ],
            navigation_elements: Vec::new(),
            content_hierarchy: ContentHierarchy {
                main_heading: None,
                subheadings: Vec::new(),
                content_sections: Vec::new(),
                information_density: 0.5,
            },
        })
    }
}

struct SemanticClassifier;

impl SemanticClassifier {
    fn new() -> Self {
        Self
    }

    fn classify_word(&self, text: &str, _bbox: &BoundingBox, _layout: &ScreenLayout) -> TextType {
        // Enhanced classification logic
        if text.contains("error") || text.contains("Error") {
            TextType::ErrorMessage
        } else if text.contains("def ") || text.contains("function") || text.contains("=>") {
            TextType::CodeSnippet
        } else if text.contains("@") && text.contains(".") {
            TextType::EmailContent
        } else {
            TextType::UIElement
        }
    }

    fn classify_paragraph(&self, text: &str) -> TextType {
        if text.contains("def ") || text.contains("function") || text.contains("import") {
            TextType::CodeSnippet
        } else if text.contains("@") && text.contains(".com") {
            TextType::EmailContent
        } else if text.len() > 200 {
            TextType::DocumentContent
        } else {
            TextType::UIElement
        }
    }
}

struct UIDetector;

impl UIDetector {
    fn new() -> Self {
        Self
    }

    fn analyze_ui_context(&self, _bbox: &BoundingBox) -> UIContext {
        UIContext {
            window_title: None,
            application_name: None,
            ui_framework: None,
            theme: None,
        }
    }

    fn detect_interaction_elements(&self, _bbox: &BoundingBox) -> Vec<InteractionElement> {
        Vec::new() // TODO: Implement UI element detection
    }
}