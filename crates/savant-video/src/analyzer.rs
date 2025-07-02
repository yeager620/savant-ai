use anyhow::Result;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use savant_ocr::{OCRProcessor, OCRConfig, OCRResult};
use savant_vision::{VisionAnalyzer, ScreenAnalysis, ActivityClassification, VisualContext, DetectedApp};

use crate::FrameMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAnalysisResult {
    pub ocr_result: Option<OCRResult>,
    pub screen_analysis: Option<ScreenAnalysis>,
    pub application_context: ApplicationContext,
    pub text_summary: TextSummary,
    pub interaction_opportunities: Vec<InteractionOpportunity>,
    pub processing_stats: ProcessingStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationContext {
    pub primary_application: Option<DetectedApp>,
    pub secondary_applications: Vec<DetectedApp>,
    pub browser_context: Option<BrowserContext>,
    pub ide_context: Option<IDEContext>,
    pub meeting_context: Option<MeetingContext>,
    pub productivity_context: Option<ProductivityContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserContext {
    pub browser_type: String,
    pub estimated_url: Option<String>,
    pub page_category: PageCategory,
    pub tab_count_estimate: Option<u32>,
    pub navigation_visible: bool,
    pub bookmarks_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageCategory {
    Search,
    SocialMedia,
    Documentation,
    VideoStreaming,
    Shopping,
    News,
    Email,
    Productivity,
    Development,
    Entertainment,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDEContext {
    pub ide_type: String,
    pub active_file_extension: Option<String>,
    pub programming_language: Option<String>,
    pub code_snippets: Vec<CodeSnippet>,
    pub error_messages: Vec<ErrorMessage>,
    pub debug_mode_active: bool,
    pub terminal_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub language: Option<String>,
    pub content: String,
    pub line_range: Option<(u32, u32)>,
    pub complexity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub severity: ErrorSeverity,
    pub message: String,
    pub line_number: Option<u32>,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingContext {
    pub platform: String,
    pub estimated_participants: Option<u32>,
    pub screen_sharing_active: bool,
    pub recording_active: bool,
    pub chat_visible: bool,
    pub camera_status: CameraStatus,
    pub microphone_status: MicrophoneStatus,
    pub meeting_controls_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraStatus {
    On,
    Off,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MicrophoneStatus {
    Unmuted,
    Muted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityContext {
    pub app_category: ProductivityCategory,
    pub document_type: Option<String>,
    pub editing_mode: EditingMode,
    pub collaboration_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductivityCategory {
    TextEditor,
    Spreadsheet,
    Presentation,
    NoteTaking,
    ProjectManagement,
    Design,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditingMode {
    Reading,
    Writing,
    Reviewing,
    Presenting,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSummary {
    pub total_text_blocks: usize,
    pub code_blocks: usize,
    pub ui_elements: usize,
    pub document_content: usize,
    pub chat_messages: usize,
    pub email_content: usize,
    pub dominant_language: String,
    pub key_phrases: Vec<String>,
    pub technical_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOpportunity {
    pub opportunity_type: OpportunityType,
    pub description: String,
    pub confidence: f32,
    pub suggested_action: String,
    pub context: String,
    pub urgency: Urgency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    CodingAssistance,
    DocumentationHelp,
    MeetingSupport,
    ProductivityOptimization,
    LearningOpportunity,
    TroubleshootingHelp,
    AutomationSuggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Urgency {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_processing_time_ms: u64,
    pub ocr_time_ms: Option<u64>,
    pub vision_analysis_time_ms: Option<u64>,
    pub context_analysis_time_ms: u64,
    pub opportunity_detection_time_ms: u64,
}

pub struct EnhancedVideoAnalyzer {
    ocr_processor: OCRProcessor,
    vision_analyzer: VisionAnalyzer,
    context_analyzer: ContextAnalyzer,
    opportunity_detector: OpportunityDetector,
}

impl EnhancedVideoAnalyzer {
    pub fn new() -> Result<Self> {
        let ocr_config = OCRConfig {
            enable_text_classification: true,
            enable_structure_analysis: true,
            ..Default::default()
        };

        Ok(Self {
            ocr_processor: OCRProcessor::new(ocr_config)?,
            vision_analyzer: VisionAnalyzer::new()?,
            context_analyzer: ContextAnalyzer::new(),
            opportunity_detector: OpportunityDetector::new(),
        })
    }

    pub async fn analyze_frame(&self, image: &DynamicImage, frame_metadata: &FrameMetadata) -> Result<VideoAnalysisResult> {
        let start_time = std::time::Instant::now();

        // Perform OCR analysis
        let ocr_start = std::time::Instant::now();
        let ocr_result = match self.ocr_processor.process_image(image).await {
            Ok(result) => Some(result),
            Err(e) => {
                tracing::warn!("OCR processing failed: {}", e);
                None
            }
        };
        let ocr_time = ocr_start.elapsed().as_millis() as u64;

        // Perform vision analysis
        let vision_start = std::time::Instant::now();
        let screen_analysis = match self.vision_analyzer.analyze_screenshot(image).await {
            Ok(analysis) => Some(analysis),
            Err(e) => {
                tracing::warn!("Vision analysis failed: {}", e);
                None
            }
        };
        let vision_time = vision_start.elapsed().as_millis() as u64;

        // Analyze application context
        let context_start = std::time::Instant::now();
        let application_context = self.context_analyzer.analyze_application_context(
            &ocr_result,
            &screen_analysis,
            frame_metadata
        ).await?;
        let context_time = context_start.elapsed().as_millis() as u64;

        // Generate text summary
        let text_summary = self.generate_text_summary(&ocr_result, &screen_analysis)?;

        // Detect interaction opportunities
        let opportunity_start = std::time::Instant::now();
        let interaction_opportunities = self.opportunity_detector.detect_opportunities(
            &ocr_result,
            &screen_analysis,
            &application_context,
            frame_metadata
        ).await?;
        let opportunity_time = opportunity_start.elapsed().as_millis() as u64;

        let total_time = start_time.elapsed().as_millis() as u64;

        Ok(VideoAnalysisResult {
            ocr_result,
            screen_analysis,
            application_context,
            text_summary,
            interaction_opportunities,
            processing_stats: ProcessingStats {
                total_processing_time_ms: total_time,
                ocr_time_ms: Some(ocr_time),
                vision_analysis_time_ms: Some(vision_time),
                context_analysis_time_ms: context_time,
                opportunity_detection_time_ms: opportunity_time,
            },
        })
    }

    fn generate_text_summary(&self, ocr_result: &Option<OCRResult>, screen_analysis: &Option<ScreenAnalysis>) -> Result<TextSummary> {
        let mut summary = TextSummary {
            total_text_blocks: 0,
            code_blocks: 0,
            ui_elements: 0,
            document_content: 0,
            chat_messages: 0,
            email_content: 0,
            dominant_language: "eng".to_string(),
            key_phrases: Vec::new(),
            technical_terms: Vec::new(),
        };

        if let Some(ocr) = ocr_result {
            summary.total_text_blocks = ocr.text_blocks.len();
            summary.dominant_language = ocr.detected_language.clone();

            // Count different types of content
            for block in &ocr.text_blocks {
                match block.semantic_type {
                    savant_ocr::TextType::CodeSnippet => summary.code_blocks += 1,
                    savant_ocr::TextType::UIElement => summary.ui_elements += 1,
                    savant_ocr::TextType::DocumentContent => summary.document_content += 1,
                    savant_ocr::TextType::ChatMessage => summary.chat_messages += 1,
                    savant_ocr::TextType::EmailContent => summary.email_content += 1,
                    _ => {}
                }
            }

            // Extract key phrases and technical terms
            summary.key_phrases = self.extract_key_phrases(&ocr.text_blocks)?;
            summary.technical_terms = self.extract_technical_terms(&ocr.text_blocks)?;
        }

        Ok(summary)
    }

    fn extract_key_phrases(&self, text_blocks: &[savant_ocr::TextBlock]) -> Result<Vec<String>> {
        let mut phrases = Vec::new();
        
        for block in text_blocks {
            // Simple key phrase extraction - look for capitalized phrases
            let words: Vec<&str> = block.text.split_whitespace().collect();
            for window in words.windows(2) {
                if window.len() == 2 && 
                   window[0].chars().next().map_or(false, |c| c.is_uppercase()) &&
                   window[1].chars().next().map_or(false, |c| c.is_uppercase()) {
                    phrases.push(format!("{} {}", window[0], window[1]));
                }
            }
        }

        // Remove duplicates and limit to top 20
        phrases.sort();
        phrases.dedup();
        phrases.truncate(20);
        
        Ok(phrases)
    }

    fn extract_technical_terms(&self, text_blocks: &[savant_ocr::TextBlock]) -> Result<Vec<String>> {
        let mut terms = Vec::new();
        
        // Common technical term patterns
        let technical_patterns = [
            r"\b[A-Z]{2,}\b", // Acronyms
            r"\b\w+\(\)", // Function calls
            r"\b\w+\.\w+\b", // Method calls or file extensions
            r"\b\d+\.\d+\.\d+\b", // Version numbers
            r"\bhttps?://\S+\b", // URLs
        ];

        for block in text_blocks {
            for pattern in &technical_patterns {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    for mat in regex.find_iter(&block.text) {
                        terms.push(mat.as_str().to_string());
                    }
                }
            }
        }

        // Remove duplicates and limit
        terms.sort();
        terms.dedup();
        terms.truncate(15);
        
        Ok(terms)
    }
}

struct ContextAnalyzer;

impl ContextAnalyzer {
    fn new() -> Self {
        Self
    }

    async fn analyze_application_context(
        &self,
        ocr_result: &Option<OCRResult>,
        screen_analysis: &Option<ScreenAnalysis>,
        frame_metadata: &FrameMetadata,
    ) -> Result<ApplicationContext> {
        let mut context = ApplicationContext {
            primary_application: None,
            secondary_applications: Vec::new(),
            browser_context: None,
            ide_context: None,
            meeting_context: None,
            productivity_context: None,
        };

        if let Some(analysis) = screen_analysis {
            // Extract detected applications
            context.primary_application = analysis.app_context.detected_applications.first().cloned();
            context.secondary_applications = analysis.app_context.detected_applications.iter().skip(1).cloned().collect();

            // Analyze specific application contexts
            context.browser_context = self.analyze_browser_context(ocr_result, analysis).await?;
            context.ide_context = self.analyze_ide_context(ocr_result, analysis).await?;
            context.meeting_context = self.analyze_meeting_context(ocr_result, analysis).await?;
            context.productivity_context = self.analyze_productivity_context(ocr_result, analysis).await?;
        }

        Ok(context)
    }

    async fn analyze_browser_context(&self, ocr_result: &Option<OCRResult>, screen_analysis: &ScreenAnalysis) -> Result<Option<BrowserContext>> {
        // Check if any detected app is a browser
        let has_browser = screen_analysis.app_context.detected_applications.iter()
            .any(|app| matches!(app.app_type, savant_vision::AppType::Browser(_)));

        if !has_browser {
            return Ok(None);
        }

        let mut context = BrowserContext {
            browser_type: "Unknown".to_string(),
            estimated_url: None,
            page_category: PageCategory::Unknown,
            tab_count_estimate: None,
            navigation_visible: false,
            bookmarks_visible: false,
        };

        // Detect browser type from applications
        for app in &screen_analysis.app_context.detected_applications {
            if let savant_vision::AppType::Browser(browser_type) = &app.app_type {
                context.browser_type = format!("{:?}", browser_type);
                break;
            }
        }

        // Extract URL from OCR if available
        if let Some(ocr) = ocr_result {
            for block in &ocr.text_blocks {
                if block.text.starts_with("http") || block.text.contains("www.") {
                    context.estimated_url = Some(block.text.clone());
                    break;
                }
            }
        }

        // Estimate page category based on visual elements and text
        context.page_category = self.classify_page_category(ocr_result, screen_analysis);

        Ok(Some(context))
    }

    async fn analyze_ide_context(&self, ocr_result: &Option<OCRResult>, screen_analysis: &ScreenAnalysis) -> Result<Option<IDEContext>> {
        // Check if any detected app is an IDE
        let ide_app = screen_analysis.app_context.detected_applications.iter()
            .find(|app| matches!(app.app_type, savant_vision::AppType::IDE(_)));

        if ide_app.is_none() {
            return Ok(None);
        }

        let mut context = IDEContext {
            ide_type: "Unknown".to_string(),
            active_file_extension: None,
            programming_language: None,
            code_snippets: Vec::new(),
            error_messages: Vec::new(),
            debug_mode_active: false,
            terminal_visible: false,
        };

        // Set IDE type
        if let Some(app) = ide_app {
            if let savant_vision::AppType::IDE(ide_type) = &app.app_type {
                context.ide_type = format!("{:?}", ide_type);
            }
        }

        // Extract code snippets and analyze programming language
        if let Some(ocr) = ocr_result {
            for block in &ocr.structured_content.code_blocks {
                context.code_snippets.push(CodeSnippet {
                    language: block.language.clone(),
                    content: block.content.clone(),
                    line_range: None,
                    complexity_score: self.calculate_code_complexity(&block.content),
                });

                if context.programming_language.is_none() {
                    context.programming_language = block.language.clone();
                }
            }
        }

        // Check for terminal visibility
        context.terminal_visible = screen_analysis.visual_elements.iter()
            .any(|element| matches!(element.element_type, savant_vision::ElementType::Terminal));

        Ok(Some(context))
    }

    async fn analyze_meeting_context(&self, _ocr_result: &Option<OCRResult>, screen_analysis: &ScreenAnalysis) -> Result<Option<MeetingContext>> {
        // Check if any detected app is a video conferencing tool
        let meeting_app = screen_analysis.app_context.detected_applications.iter()
            .find(|app| matches!(app.app_type, savant_vision::AppType::VideoConferencing(_)));

        if meeting_app.is_none() {
            return Ok(None);
        }

        let mut context = MeetingContext {
            platform: "Unknown".to_string(),
            estimated_participants: None,
            screen_sharing_active: false,
            recording_active: false,
            chat_visible: false,
            camera_status: CameraStatus::Unknown,
            microphone_status: MicrophoneStatus::Unknown,
            meeting_controls_visible: false,
        };

        // Set platform
        if let Some(app) = meeting_app {
            if let savant_vision::AppType::VideoConferencing(platform) = &app.app_type {
                context.platform = format!("{:?}", platform);
            }
        }

        // Analyze meeting UI elements
        context.meeting_controls_visible = screen_analysis.visual_elements.iter()
            .any(|element| matches!(element.element_type, savant_vision::ElementType::VideoCall));

        Ok(Some(context))
    }

    async fn analyze_productivity_context(&self, _ocr_result: &Option<OCRResult>, screen_analysis: &ScreenAnalysis) -> Result<Option<ProductivityContext>> {
        // Check if any detected app is a productivity tool
        let productivity_app = screen_analysis.app_context.detected_applications.iter()
            .find(|app| matches!(app.app_type, savant_vision::AppType::Productivity(_)));

        if productivity_app.is_none() {
            return Ok(None);
        }

        let context = ProductivityContext {
            app_category: ProductivityCategory::Unknown,
            document_type: None,
            editing_mode: EditingMode::Unknown,
            collaboration_indicators: Vec::new(),
        };

        Ok(Some(context))
    }

    fn classify_page_category(&self, _ocr_result: &Option<OCRResult>, _screen_analysis: &ScreenAnalysis) -> PageCategory {
        // Placeholder for page category classification
        PageCategory::Unknown
    }

    fn calculate_code_complexity(&self, code: &str) -> f32 {
        // Simple complexity calculation based on nesting and keywords
        let lines = code.lines().count();
        let nesting_chars = code.matches('{').count() + code.matches('(').count();
        let keywords = ["if", "for", "while", "switch", "try", "catch"].iter()
            .map(|&keyword| code.matches(keyword).count())
            .sum::<usize>();

        ((nesting_chars + keywords) as f32 / lines.max(1) as f32).min(10.0)
    }
}

struct OpportunityDetector;

impl OpportunityDetector {
    fn new() -> Self {
        Self
    }

    async fn detect_opportunities(
        &self,
        ocr_result: &Option<OCRResult>,
        screen_analysis: &Option<ScreenAnalysis>,
        application_context: &ApplicationContext,
        _frame_metadata: &FrameMetadata,
    ) -> Result<Vec<InteractionOpportunity>> {
        let mut opportunities = Vec::new();

        // Detect coding assistance opportunities
        if let Some(ide_context) = &application_context.ide_context {
            opportunities.extend(self.detect_coding_opportunities(ide_context, ocr_result)?);
        }

        // Detect meeting support opportunities
        if let Some(meeting_context) = &application_context.meeting_context {
            opportunities.extend(self.detect_meeting_opportunities(meeting_context, ocr_result)?);
        }

        // Detect productivity optimization opportunities
        if let Some(productivity_context) = &application_context.productivity_context {
            opportunities.extend(self.detect_productivity_opportunities(productivity_context, ocr_result)?);
        }

        // Detect documentation help opportunities
        opportunities.extend(self.detect_documentation_opportunities(ocr_result, screen_analysis)?);

        Ok(opportunities)
    }

    fn detect_coding_opportunities(&self, ide_context: &IDEContext, _ocr_result: &Option<OCRResult>) -> Result<Vec<InteractionOpportunity>> {
        let mut opportunities = Vec::new();

        // Check for complex code that might need refactoring
        for snippet in &ide_context.code_snippets {
            if snippet.complexity_score > 7.0 {
                opportunities.push(InteractionOpportunity {
                    opportunity_type: OpportunityType::CodingAssistance,
                    description: "Complex code detected that could benefit from refactoring".to_string(),
                    confidence: 0.8,
                    suggested_action: "Suggest code simplification and refactoring patterns".to_string(),
                    context: format!("Code complexity score: {:.1}", snippet.complexity_score),
                    urgency: Urgency::Medium,
                });
            }
        }

        // Check for error messages
        if !ide_context.error_messages.is_empty() {
            opportunities.push(InteractionOpportunity {
                opportunity_type: OpportunityType::TroubleshootingHelp,
                description: "Compilation or runtime errors detected".to_string(),
                confidence: 0.9,
                suggested_action: "Provide error explanation and solution suggestions".to_string(),
                context: format!("{} errors found", ide_context.error_messages.len()),
                urgency: Urgency::High,
            });
        }

        Ok(opportunities)
    }

    fn detect_meeting_opportunities(&self, meeting_context: &MeetingContext, _ocr_result: &Option<OCRResult>) -> Result<Vec<InteractionOpportunity>> {
        let mut opportunities = Vec::new();

        if meeting_context.screen_sharing_active {
            opportunities.push(InteractionOpportunity {
                opportunity_type: OpportunityType::MeetingSupport,
                description: "Screen sharing detected - ready to assist with presentation".to_string(),
                confidence: 0.7,
                suggested_action: "Offer to help with slide navigation or content explanation".to_string(),
                context: "Screen sharing mode active".to_string(),
                urgency: Urgency::Low,
            });
        }

        Ok(opportunities)
    }

    fn detect_productivity_opportunities(&self, _productivity_context: &ProductivityContext, _ocr_result: &Option<OCRResult>) -> Result<Vec<InteractionOpportunity>> {
        // Placeholder for productivity opportunity detection
        Ok(Vec::new())
    }

    fn detect_documentation_opportunities(&self, ocr_result: &Option<OCRResult>, _screen_analysis: &Option<ScreenAnalysis>) -> Result<Vec<InteractionOpportunity>> {
        let mut opportunities = Vec::new();

        if let Some(ocr) = ocr_result {
            // Look for technical terms that might need explanation
            let technical_blocks: Vec<_> = ocr.text_blocks.iter()
                .filter(|block| matches!(block.semantic_type, savant_ocr::TextType::CodeSnippet))
                .collect();

            if technical_blocks.len() > 3 {
                opportunities.push(InteractionOpportunity {
                    opportunity_type: OpportunityType::DocumentationHelp,
                    description: "Multiple technical code blocks detected".to_string(),
                    confidence: 0.6,
                    suggested_action: "Offer to explain code functionality and provide documentation".to_string(),
                    context: format!("{} code blocks found", technical_blocks.len()),
                    urgency: Urgency::Low,
                });
            }
        }

        Ok(opportunities)
    }
}