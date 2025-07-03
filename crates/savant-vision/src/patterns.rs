use anyhow::Result;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub confidence_threshold: f32,
    pub description: String,
    pub indicators: Vec<PatternIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ApplicationSignature,
    UIElement,
    Layout,
    Interaction,
    Content,
    Navigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternIndicator {
    pub indicator_type: IndicatorType,
    pub value: String,
    pub weight: f32,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorType {
    Color,
    Shape,
    Text,
    Position,
    Size,
    Texture,
    Layout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub confidence: f32,
    pub matched_indicators: Vec<String>,
    pub bounding_box: Option<crate::BoundingBox>,
}

#[derive(Debug)]
pub struct PatternMatcher {
    patterns: HashMap<String, VisualPattern>,
    app_patterns: AppPatternDatabase,
}

impl PatternMatcher {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Load built-in patterns
        Self::load_builtin_patterns(&mut patterns);
        
        Self {
            patterns,
            app_patterns: AppPatternDatabase::new(),
        }
    }

    fn load_builtin_patterns(patterns: &mut HashMap<String, VisualPattern>) {
        // Zoom application patterns
        patterns.insert("zoom_meeting".to_string(), VisualPattern {
            pattern_id: "zoom_meeting".to_string(),
            pattern_type: PatternType::ApplicationSignature,
            confidence_threshold: 0.7,
            description: "Zoom video conferencing interface".to_string(),
            indicators: vec![
                PatternIndicator {
                    indicator_type: IndicatorType::Color,
                    value: "#2D8CFF".to_string(), // Zoom blue
                    weight: 0.3,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "Zoom".to_string(),
                    weight: 0.4,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "Participants".to_string(),
                    weight: 0.2,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Layout,
                    value: "video_grid".to_string(),
                    weight: 0.3,
                    required: false,
                },
            ],
        });

        // VS Code patterns
        patterns.insert("vscode_ide".to_string(), VisualPattern {
            pattern_id: "vscode_ide".to_string(),
            pattern_type: PatternType::ApplicationSignature,
            confidence_threshold: 0.6,
            description: "Visual Studio Code IDE interface".to_string(),
            indicators: vec![
                PatternIndicator {
                    indicator_type: IndicatorType::Color,
                    value: "#1E1E1E".to_string(), // VS Code dark theme
                    weight: 0.2,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "Explorer".to_string(),
                    weight: 0.3,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Layout,
                    value: "sidebar_editor_panel".to_string(),
                    weight: 0.4,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "Visual Studio Code".to_string(),
                    weight: 0.5,
                    required: false,
                },
            ],
        });

        // Chrome browser patterns
        patterns.insert("chrome_browser".to_string(), VisualPattern {
            pattern_id: "chrome_browser".to_string(),
            pattern_type: PatternType::ApplicationSignature,
            confidence_threshold: 0.5,
            description: "Google Chrome browser interface".to_string(),
            indicators: vec![
                PatternIndicator {
                    indicator_type: IndicatorType::Shape,
                    value: "rounded_tabs".to_string(),
                    weight: 0.3,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "Chrome".to_string(),
                    weight: 0.2,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Layout,
                    value: "address_bar_tabs".to_string(),
                    weight: 0.4,
                    required: false,
                },
            ],
        });

        // Terminal patterns
        patterns.insert("terminal_app".to_string(), VisualPattern {
            pattern_id: "terminal_app".to_string(),
            pattern_type: PatternType::ApplicationSignature,
            confidence_threshold: 0.6,
            description: "Terminal/command line interface".to_string(),
            indicators: vec![
                PatternIndicator {
                    indicator_type: IndicatorType::Color,
                    value: "#000000".to_string(), // Black background
                    weight: 0.3,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "$".to_string(), // Command prompt
                    weight: 0.4,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "Terminal".to_string(),
                    weight: 0.3,
                    required: false,
                },
            ],
        });

        // UI element patterns
        patterns.insert("dialog_box".to_string(), VisualPattern {
            pattern_id: "dialog_box".to_string(),
            pattern_type: PatternType::UIElement,
            confidence_threshold: 0.7,
            description: "Modal dialog box".to_string(),
            indicators: vec![
                PatternIndicator {
                    indicator_type: IndicatorType::Shape,
                    value: "rectangular_border".to_string(),
                    weight: 0.3,
                    required: true,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Position,
                    value: "center_screen".to_string(),
                    weight: 0.2,
                    required: false,
                },
                PatternIndicator {
                    indicator_type: IndicatorType::Text,
                    value: "OK|Cancel|Apply".to_string(),
                    weight: 0.4,
                    required: false,
                },
            ],
        });
    }

    pub async fn match_patterns(&self, image: &DynamicImage, context: &PatternMatchContext) -> Result<Vec<PatternMatch>> {
        let mut matches = Vec::new();

        for pattern in self.patterns.values() {
            if let Some(pattern_match) = self.evaluate_pattern(image, pattern, context).await? {
                matches.push(pattern_match);
            }
        }

        // Sort by confidence
        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(matches)
    }

    async fn evaluate_pattern(&self, image: &DynamicImage, pattern: &VisualPattern, context: &PatternMatchContext) -> Result<Option<PatternMatch>> {
        let mut total_score = 0.0;
        let mut matched_indicators = Vec::new();
        let mut required_met = true;

        for indicator in &pattern.indicators {
            let indicator_score = self.evaluate_indicator(image, indicator, context).await?;
            
            if indicator_score > 0.0 {
                total_score += indicator_score * indicator.weight;
                matched_indicators.push(indicator.value.clone());
            } else if indicator.required {
                required_met = false;
                break;
            }
        }

        if required_met && total_score >= pattern.confidence_threshold {
            Ok(Some(PatternMatch {
                pattern_id: pattern.pattern_id.clone(),
                confidence: total_score.min(1.0),
                matched_indicators,
                bounding_box: None, // Would be calculated based on matched elements
            }))
        } else {
            Ok(None)
        }
    }

    async fn evaluate_indicator(&self, image: &DynamicImage, indicator: &PatternIndicator, context: &PatternMatchContext) -> Result<f32> {
        match indicator.indicator_type {
            IndicatorType::Color => self.evaluate_color_indicator(image, &indicator.value).await,
            IndicatorType::Text => self.evaluate_text_indicator(&indicator.value, context).await,
            IndicatorType::Shape => self.evaluate_shape_indicator(image, &indicator.value).await,
            IndicatorType::Position => self.evaluate_position_indicator(&indicator.value, context).await,
            IndicatorType::Size => self.evaluate_size_indicator(&indicator.value, context).await,
            IndicatorType::Layout => self.evaluate_layout_indicator(&indicator.value, context).await,
            IndicatorType::Texture => self.evaluate_texture_indicator(image, &indicator.value).await,
        }
    }

    async fn evaluate_color_indicator(&self, image: &DynamicImage, target_color: &str) -> Result<f32> {
        // Parse hex color
        let target_rgb = if target_color.starts_with('#') && target_color.len() == 7 {
            let r = u8::from_str_radix(&target_color[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&target_color[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&target_color[5..7], 16).unwrap_or(0);
            [r, g, b]
        } else {
            return Ok(0.0);
        };

        let rgba_image = image.to_rgba8();
        let total_pixels = (image.width() * image.height()) as f32;
        let mut matching_pixels = 0;

        for pixel in rgba_image.pixels() {
            if self.colors_similar(&[pixel[0], pixel[1], pixel[2]], &target_rgb, 30) {
                matching_pixels += 1;
            }
        }

        let color_ratio = matching_pixels as f32 / total_pixels;
        Ok((color_ratio * 10.0).min(1.0)) // Scale and cap at 1.0
    }

    fn colors_similar(&self, color1: &[u8; 3], color2: &[u8; 3], tolerance: u8) -> bool {
        let diff_r = (color1[0] as i16 - color2[0] as i16).abs();
        let diff_g = (color1[1] as i16 - color2[1] as i16).abs();
        let diff_b = (color1[2] as i16 - color2[2] as i16).abs();
        
        diff_r <= tolerance as i16 && diff_g <= tolerance as i16 && diff_b <= tolerance as i16
    }

    async fn evaluate_text_indicator(&self, pattern: &str, context: &PatternMatchContext) -> Result<f32> {
        // Check if any extracted text matches the pattern
        for text_block in &context.extracted_text {
            if text_block.to_lowercase().contains(&pattern.to_lowercase()) {
                return Ok(1.0);
            }
        }

        // Check with regex if pattern contains regex syntax
        if pattern.contains('|') || pattern.contains('[') {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for text_block in &context.extracted_text {
                    if regex.is_match(text_block) {
                        return Ok(1.0);
                    }
                }
            }
        }

        Ok(0.0)
    }

    async fn evaluate_shape_indicator(&self, _image: &DynamicImage, _pattern: &str) -> Result<f32> {
        // Placeholder for shape detection
        // Would use computer vision techniques to detect shapes
        Ok(0.0)
    }

    async fn evaluate_position_indicator(&self, pattern: &str, context: &PatternMatchContext) -> Result<f32> {
        match pattern {
            "center_screen" => {
                // Check if any elements are in the center area
                let screen_center_x = context.screen_width / 2;
                let screen_center_y = context.screen_height / 2;
                
                for element in &context.visual_elements {
                    let element_center_x = element.x + element.width / 2;
                    let element_center_y = element.y + element.height / 2;
                    
                    let distance_from_center = ((element_center_x as f32 - screen_center_x as f32).powi(2) + 
                                               (element_center_y as f32 - screen_center_y as f32).powi(2)).sqrt();
                    
                    let max_distance = ((context.screen_width * context.screen_width + 
                                       context.screen_height * context.screen_height) as f32).sqrt();
                    
                    let center_score = 1.0 - (distance_from_center / max_distance);
                    if center_score > 0.7 {
                        return Ok(center_score);
                    }
                }
                Ok(0.0)
            }
            _ => Ok(0.0),
        }
    }

    async fn evaluate_size_indicator(&self, _pattern: &str, _context: &PatternMatchContext) -> Result<f32> {
        // Placeholder for size-based pattern matching
        Ok(0.0)
    }

    async fn evaluate_layout_indicator(&self, pattern: &str, context: &PatternMatchContext) -> Result<f32> {
        match pattern {
            "video_grid" => {
                // Look for grid-like arrangement of rectangular elements
                let rectangular_elements: Vec<_> = context.visual_elements.iter()
                    .filter(|e| e.width > 100 && e.height > 80) // Video-sized elements
                    .collect();
                
                if rectangular_elements.len() >= 4 {
                    Ok(0.8)
                } else if rectangular_elements.len() >= 2 {
                    Ok(0.6)
                } else {
                    Ok(0.0)
                }
            }
            "sidebar_editor_panel" => {
                // Look for typical IDE layout with sidebar and main panel
                let left_elements = context.visual_elements.iter()
                    .filter(|e| e.x < context.screen_width / 4)
                    .count();
                let center_elements = context.visual_elements.iter()
                    .filter(|e| e.x > context.screen_width / 4 && e.x < 3 * context.screen_width / 4)
                    .count();
                
                if left_elements > 0 && center_elements > 0 {
                    Ok(0.7)
                } else {
                    Ok(0.0)
                }
            }
            "address_bar_tabs" => {
                // Look for browser-like layout with tabs at top
                let top_elements = context.visual_elements.iter()
                    .filter(|e| e.y < context.screen_height / 10)
                    .count();
                
                if top_elements > 3 {
                    Ok(0.6)
                } else {
                    Ok(0.0)
                }
            }
            _ => Ok(0.0),
        }
    }

    async fn evaluate_texture_indicator(&self, _image: &DynamicImage, _pattern: &str) -> Result<f32> {
        // Placeholder for texture analysis
        Ok(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct PatternMatchContext {
    pub extracted_text: Vec<String>,
    pub visual_elements: Vec<ElementBounds>,
    pub screen_width: u32,
    pub screen_height: u32,
}

#[derive(Debug, Clone)]
pub struct ElementBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub element_type: String,
}

#[derive(Debug)]
struct AppPatternDatabase {
    patterns: HashMap<String, AppPattern>,
}

#[derive(Debug, Clone)]
struct AppPattern {
    app_name: String,
    visual_signatures: Vec<VisualSignature>,
    confidence_threshold: f32,
}

#[derive(Debug, Clone)]
struct VisualSignature {
    signature_type: SignatureType,
    data: Vec<u8>,
    weight: f32,
}

#[derive(Debug, Clone)]
enum SignatureType {
    ColorHistogram,
    EdgePattern,
    TextureFeature,
    TemplateMatch,
}

impl AppPatternDatabase {
    fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    // Methods for loading and matching app-specific patterns would go here
}