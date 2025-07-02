use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};

use crate::{AppContext, VisualElement};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualContext {
    pub dominant_colors: Vec<String>,
    pub layout_analysis: LayoutAnalysis,
    pub attention_areas: Vec<AttentionArea>,
    pub interaction_elements: Vec<InteractionElement>,
    pub content_regions: Vec<ContentRegion>,
    pub theme_info: ThemeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutAnalysis {
    pub layout_type: LayoutType,
    pub grid_structure: Option<GridStructure>,
    pub primary_content_area: Option<ContentArea>,
    pub sidebar_present: bool,
    pub header_present: bool,
    pub footer_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    SingleColumn,
    TwoColumn,
    ThreeColumn,
    Grid,
    Dashboard,
    Fullscreen,
    Split,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridStructure {
    pub rows: u32,
    pub columns: u32,
    pub cell_size: (u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub content_type: ContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Video,
    Code,
    Form,
    Navigation,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionArea {
    pub region: ContentArea,
    pub attention_score: f32,
    pub reason: AttentionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttentionReason {
    BrightColors,
    Movement,
    ContrastDifference,
    CenterPosition,
    LargeSize,
    UnusualShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionElement {
    pub element_type: InteractionType,
    pub position: ContentArea,
    pub state: InteractionState,
    pub accessibility_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Button,
    Link,
    Input,
    Dropdown,
    Slider,
    Checkbox,
    RadioButton,
    Tab,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionState {
    Normal,
    Hover,
    Active,
    Disabled,
    Selected,
    Loading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRegion {
    pub region: ContentArea,
    pub content_type: ContentType,
    pub density: ContentDensity,
    pub scroll_position: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentDensity {
    Sparse,
    Medium,
    Dense,
    Overcrowded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub is_dark_mode: bool,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub background_color: String,
    pub text_color: String,
    pub contrast_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub confidence: f32,
    pub location: Option<ContentArea>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    ColorAnalysis,
    ShapeDetection,
    TextPresence,
    LayoutPattern,
    InteractionCue,
    BrandingElement,
    NavigationStructure,
}

pub struct ContextAnalyzer {
    color_analyzer: ColorAnalyzer,
    layout_analyzer: LayoutAnalyzer,
    attention_analyzer: AttentionAnalyzer,
}

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self {
            color_analyzer: ColorAnalyzer::new(),
            layout_analyzer: LayoutAnalyzer::new(),
            attention_analyzer: AttentionAnalyzer::new(),
        }
    }

    pub async fn analyze_context(
        &self,
        image: &DynamicImage,
        _app_context: &AppContext,
        visual_elements: &[VisualElement],
    ) -> Result<VisualContext> {
        
        // Analyze colors and theme
        let theme_info = self.color_analyzer.analyze_theme(image)?;
        let dominant_colors = self.color_analyzer.extract_dominant_colors(image)?;

        // Analyze layout structure
        let layout_analysis = self.layout_analyzer.analyze_layout(image, visual_elements)?;

        // Identify attention areas
        let attention_areas = self.attention_analyzer.find_attention_areas(image, visual_elements)?;

        // Analyze interaction elements
        let interaction_elements = self.analyze_interaction_elements(visual_elements)?;

        // Analyze content regions
        let content_regions = self.analyze_content_regions(image, visual_elements)?;

        Ok(VisualContext {
            dominant_colors,
            layout_analysis,
            attention_areas,
            interaction_elements,
            content_regions,
            theme_info,
        })
    }

    fn analyze_interaction_elements(&self, visual_elements: &[VisualElement]) -> Result<Vec<InteractionElement>> {
        let mut interaction_elements = Vec::new();

        for element in visual_elements {
            if element.properties.is_interactive {
                let interaction_type = match element.element_type {
                    crate::ElementType::Button => InteractionType::Button,
                    crate::ElementType::TextField => InteractionType::Input,
                    _ => continue,
                };

                interaction_elements.push(InteractionElement {
                    element_type: interaction_type,
                    position: ContentArea {
                        x: element.bounding_box.x,
                        y: element.bounding_box.y,
                        width: element.bounding_box.width,
                        height: element.bounding_box.height,
                        content_type: ContentType::Unknown,
                    },
                    state: InteractionState::Normal,
                    accessibility_score: self.calculate_accessibility_score(element)?,
                });
            }
        }

        Ok(interaction_elements)
    }

    fn calculate_accessibility_score(&self, element: &VisualElement) -> Result<f32> {
        let mut score: f32 = 0.5; // Base score

        // Check size (larger elements are more accessible)
        let area = element.bounding_box.width * element.bounding_box.height;
        if area > 2000 { // Reasonable button size
            score += 0.2;
        }

        // Check position (elements not at screen edges are more accessible)
        if element.bounding_box.x > 50 && element.bounding_box.y > 50 {
            score += 0.1;
        }

        // Check if element has text content
        if element.properties.text_content.is_some() {
            score += 0.2;
        }

        Ok(score.min(1.0_f32))
    }

    fn analyze_content_regions(&self, image: &DynamicImage, visual_elements: &[VisualElement]) -> Result<Vec<ContentRegion>> {
        let mut regions = Vec::new();
        let (width, height) = image.dimensions();

        // Divide screen into regions and analyze content density
        let region_size = 200u32;
        let mut y = 0u32;
        while y < height {
            let mut x = 0u32;
            while x < width {
                let region_width = region_size.min(width - x);
                let region_height = region_size.min(height - y);

                let elements_in_region = visual_elements.iter()
                    .filter(|element| {
                        element.bounding_box.x >= x && 
                        element.bounding_box.y >= y &&
                        element.bounding_box.x < x + region_width &&
                        element.bounding_box.y < y + region_height
                    })
                    .count();

                let density = match elements_in_region {
                    0..=2 => ContentDensity::Sparse,
                    3..=5 => ContentDensity::Medium,
                    6..=10 => ContentDensity::Dense,
                    _ => ContentDensity::Overcrowded,
                };

                regions.push(ContentRegion {
                    region: ContentArea {
                        x,
                        y,
                        width: region_width,
                        height: region_height,
                        content_type: ContentType::Unknown,
                    },
                    content_type: ContentType::Unknown,
                    density,
                    scroll_position: None,
                });
                
                x += region_size;
            }
            y += region_size;
        }

        Ok(regions)
    }
}

struct ColorAnalyzer;

impl ColorAnalyzer {
    fn new() -> Self {
        Self
    }

    fn analyze_theme(&self, image: &DynamicImage) -> Result<ThemeInfo> {
        let rgba_image = image.to_rgba8();
        let mut color_counts: std::collections::HashMap<[u8; 3], u32> = std::collections::HashMap::new();
        
        // Sample pixels to determine dominant colors
        let sample_rate = 10;
        for (i, pixel) in rgba_image.pixels().enumerate() {
            if i % sample_rate == 0 {
                let rgb = [pixel[0], pixel[1], pixel[2]];
                *color_counts.entry(rgb).or_insert(0) += 1;
            }
        }

        // Find dominant background color
        let most_common_color = color_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(color, _)| *color)
            .unwrap_or([255, 255, 255]);

        let background_color = format!("#{:02x}{:02x}{:02x}", 
            most_common_color[0], most_common_color[1], most_common_color[2]);

        // Determine if dark mode based on background brightness
        let brightness = (most_common_color[0] as f32 * 0.299 + 
                         most_common_color[1] as f32 * 0.587 + 
                         most_common_color[2] as f32 * 0.114) / 255.0;
        let is_dark_mode = brightness < 0.5;

        // Estimate text color based on theme
        let text_color = if is_dark_mode {
            "#ffffff".to_string()
        } else {
            "#000000".to_string()
        };

        // Calculate contrast ratio (simplified)
        let contrast_ratio = if is_dark_mode { 7.0 } else { 5.0 };

        Ok(ThemeInfo {
            is_dark_mode,
            primary_color: None,
            secondary_color: None,
            accent_color: None,
            background_color,
            text_color,
            contrast_ratio,
        })
    }

    fn extract_dominant_colors(&self, image: &DynamicImage) -> Result<Vec<String>> {
        let rgba_image = image.to_rgba8();
        let mut color_counts: std::collections::HashMap<[u8; 3], u32> = std::collections::HashMap::new();
        
        // Sample pixels
        let sample_rate = 20;
        for (i, pixel) in rgba_image.pixels().enumerate() {
            if i % sample_rate == 0 {
                let rgb = [pixel[0], pixel[1], pixel[2]];
                *color_counts.entry(rgb).or_insert(0) += 1;
            }
        }

        // Get top 5 most common colors
        let mut sorted_colors: Vec<_> = color_counts.iter().collect();
        sorted_colors.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        let dominant_colors = sorted_colors
            .iter()
            .take(5)
            .map(|(color, _)| format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]))
            .collect();

        Ok(dominant_colors)
    }
}

struct LayoutAnalyzer;

impl LayoutAnalyzer {
    fn new() -> Self {
        Self
    }

    fn analyze_layout(&self, image: &DynamicImage, visual_elements: &[VisualElement]) -> Result<LayoutAnalysis> {
        let (width, height) = image.dimensions();
        
        // Simple layout detection based on element distribution
        let layout_type = self.detect_layout_type(width, height, visual_elements)?;
        
        // Check for common UI regions
        let header_present = self.has_header_region(height, visual_elements);
        let footer_present = self.has_footer_region(height, visual_elements);
        let sidebar_present = self.has_sidebar_region(width, visual_elements);

        // Find primary content area
        let primary_content_area = self.find_primary_content_area(width, height, visual_elements)?;

        Ok(LayoutAnalysis {
            layout_type,
            grid_structure: None,
            primary_content_area,
            sidebar_present,
            header_present,
            footer_present,
        })
    }

    fn detect_layout_type(&self, width: u32, _height: u32, visual_elements: &[VisualElement]) -> Result<LayoutType> {
        // Simple heuristics based on element distribution
        let left_elements = visual_elements.iter().filter(|e| e.bounding_box.x < width / 3).count();
        let center_elements = visual_elements.iter().filter(|e| e.bounding_box.x >= width / 3 && e.bounding_box.x < 2 * width / 3).count();
        let right_elements = visual_elements.iter().filter(|e| e.bounding_box.x >= 2 * width / 3).count();

        if left_elements > 0 && center_elements > 0 && right_elements > 0 {
            Ok(LayoutType::ThreeColumn)
        } else if (left_elements > 0 && center_elements > 0) || (center_elements > 0 && right_elements > 0) {
            Ok(LayoutType::TwoColumn)
        } else if visual_elements.len() > 20 {
            Ok(LayoutType::Grid)
        } else {
            Ok(LayoutType::SingleColumn)
        }
    }

    fn has_header_region(&self, height: u32, visual_elements: &[VisualElement]) -> bool {
        let header_threshold = height / 10;
        visual_elements.iter().any(|e| e.bounding_box.y < header_threshold)
    }

    fn has_footer_region(&self, height: u32, visual_elements: &[VisualElement]) -> bool {
        let footer_threshold = height - height / 10;
        visual_elements.iter().any(|e| e.bounding_box.y > footer_threshold)
    }

    fn has_sidebar_region(&self, width: u32, visual_elements: &[VisualElement]) -> bool {
        let sidebar_threshold = width / 6;
        visual_elements.iter().any(|e| e.bounding_box.x < sidebar_threshold || e.bounding_box.x > width - sidebar_threshold)
    }

    fn find_primary_content_area(&self, width: u32, height: u32, _visual_elements: &[VisualElement]) -> Result<Option<ContentArea>> {
        // Simple heuristic: assume center area is primary content
        Ok(Some(ContentArea {
            x: width / 4,
            y: height / 6,
            width: width / 2,
            height: 2 * height / 3,
            content_type: ContentType::Unknown,
        }))
    }
}

struct AttentionAnalyzer;

impl AttentionAnalyzer {
    fn new() -> Self {
        Self
    }

    fn find_attention_areas(&self, image: &DynamicImage, visual_elements: &[VisualElement]) -> Result<Vec<AttentionArea>> {
        let mut attention_areas = Vec::new();
        let (width, height) = image.dimensions();

        // Center bias - elements in center get higher attention score
        let center_x = width / 2;
        let center_y = height / 2;

        for element in visual_elements {
            let element_center_x = element.bounding_box.x + element.bounding_box.width / 2;
            let element_center_y = element.bounding_box.y + element.bounding_box.height / 2;

            let distance_from_center = ((element_center_x as f32 - center_x as f32).powi(2) + 
                                       (element_center_y as f32 - center_y as f32).powi(2)).sqrt();
            
            let max_distance = ((width * width + height * height) as f32).sqrt();
            let center_score = 1.0 - (distance_from_center / max_distance);

            // Size bias - larger elements get more attention
            let element_area = element.bounding_box.width * element.bounding_box.height;
            let total_area = width * height;
            let size_score = (element_area as f32 / total_area as f32) * 10.0; // Scale up

            // Combine scores
            let attention_score = (center_score * 0.4 + size_score * 0.6).min(1.0);

            if attention_score > 0.3 {
                attention_areas.push(AttentionArea {
                    region: ContentArea {
                        x: element.bounding_box.x,
                        y: element.bounding_box.y,
                        width: element.bounding_box.width,
                        height: element.bounding_box.height,
                        content_type: ContentType::Unknown,
                    },
                    attention_score,
                    reason: if center_score > 0.7 {
                        AttentionReason::CenterPosition
                    } else {
                        AttentionReason::LargeSize
                    },
                });
            }
        }

        Ok(attention_areas)
    }
}