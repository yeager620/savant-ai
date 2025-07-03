use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use savant_ocr::ComprehensiveOCRResult;
use savant_vision::ScreenAnalysis;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetectionResult {
    pub detected_tasks: Vec<DetectedTask>,
    pub detected_questions: Vec<DetectedQuestion>,
    pub assistance_opportunities: Vec<AssistanceOpportunity>,
    pub context_changes: Vec<ContextChange>,
    pub user_intent_signals: Vec<IntentSignal>,
    pub confidence_score: f32,
    pub analysis_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTask {
    pub id: String,
    pub task_type: TaskType,
    pub description: String,
    pub context: TaskContext,
    pub estimated_complexity: ComplexityLevel,
    pub suggested_assistance: Vec<String>,
    pub confidence: f32,
    pub bounding_box: Option<savant_ocr::BoundingBox>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    // Development tasks
    DebuggingError,
    WritingCode,
    CodeReview,
    TestingCode,
    RefactoringCode,
    DocumentingCode,
    
    // Research tasks
    SearchingInformation,
    ReadingDocumentation,
    ComparingOptions,
    LearningConcept,
    
    // Communication tasks
    WritingEmail,
    ReviewingMessage,
    SchedulingMeeting,
    ParticipatingInMeeting,
    
    // Content creation
    WritingDocument,
    EditingContent,
    CreatingPresentation,
    DesigningInterface,
    
    // Problem solving
    TroubleshootingIssue,
    ConfiguringSystem,
    AnalyzingData,
    PlanningProject,
    
    // General
    OrganizingFiles,
    ManagingTasks,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedQuestion {
    pub id: String,
    pub question_type: QuestionType,
    pub question_text: String,
    pub context: QuestionContext,
    pub suggested_answers: Vec<SuggestedAnswer>,
    pub confidence: f32,
    pub urgency: UrgencyLevel,
    pub bounding_box: Option<savant_ocr::BoundingBox>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    ErrorExplanation,
    HowTo,
    Conceptual,
    Troubleshooting,
    Comparison,
    BestPractice,
    Syntax,
    Configuration,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistanceOpportunity {
    pub id: String,
    pub opportunity_type: AssistanceType,
    pub description: String,
    pub trigger_context: String,
    pub suggested_action: String,
    pub confidence: f32,
    pub priority: PriorityLevel,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssistanceType {
    // Proactive assistance
    ErrorPrevention,
    OptimizationSuggestion,
    LearningOpportunity,
    ProductivityTip,
    
    // Reactive assistance
    ErrorResolution,
    ContextualHelp,
    QuickAction,
    ResourceSuggestion,
    
    // Workflow assistance
    AutoCompletion,
    TemplateOffer,
    WorkflowOptimization,
    IntegrationSuggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChange {
    pub change_type: ChangeType,
    pub from_context: String,
    pub to_context: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub related_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    ApplicationSwitch,
    TaskTransition,
    FocusShift,
    WorkflowStage,
    AttentionChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSignal {
    pub signal_type: IntentType,
    pub strength: f32,
    pub evidence: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentType {
    SeekingHelp,
    Frustrated,
    Confused,
    Focused,
    Exploring,
    Comparing,
    Deciding,
    Learning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub application: String,
    pub domain: String,
    pub related_files: Vec<String>,
    pub current_action: String,
    pub session_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionContext {
    pub source: QuestionSource,
    pub domain: String,
    pub related_concepts: Vec<String>,
    pub user_expertise_level: ExpertiseLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionSource {
    ErrorMessage,
    Documentation,
    SearchQuery,
    ChatMessage,
    CodeComment,
    UIElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAnswer {
    pub answer_type: AnswerType,
    pub content: String,
    pub confidence: f32,
    pub source: String,
    pub action_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerType {
    DirectAnswer,
    Tutorial,
    CodeExample,
    Documentation,
    StackOverflowLink,
    OfficialGuide,
    VideoTutorial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug)]
pub struct RealTimeAnalyzer {
    frame_buffer: VecDeque<AnalysisFrame>,
    task_patterns: TaskPatternMatcher,
    question_detector: QuestionDetector,
    context_tracker: ContextTracker,
    intent_analyzer: IntentAnalyzer,
    assistance_engine: AssistanceEngine,
    config: AnalyzerConfig,
}

#[derive(Debug, Clone)]
struct AnalysisFrame {
    timestamp: DateTime<Utc>,
    ocr_result: ComprehensiveOCRResult,
    vision_result: Option<ScreenAnalysis>,
    frame_hash: String,
}

#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    pub buffer_size: usize,
    pub min_confidence_threshold: f32,
    pub context_window_seconds: u64,
    pub enable_proactive_assistance: bool,
    pub enable_intent_tracking: bool,
    pub debug_mode: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 20, // Keep 20 frames (10 seconds at 2 FPS)
            min_confidence_threshold: 0.6,
            context_window_seconds: 30,
            enable_proactive_assistance: true,
            enable_intent_tracking: true,
            debug_mode: false,
        }
    }
}

impl RealTimeAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            frame_buffer: VecDeque::with_capacity(config.buffer_size),
            task_patterns: TaskPatternMatcher::new(),
            question_detector: QuestionDetector::new(),
            context_tracker: ContextTracker::new(),
            intent_analyzer: IntentAnalyzer::new(),
            assistance_engine: AssistanceEngine::new(),
            config,
        }
    }

    pub async fn analyze_frame(
        &mut self,
        ocr_result: ComprehensiveOCRResult,
        vision_result: Option<ScreenAnalysis>,
    ) -> Result<TaskDetectionResult> {
        let timestamp = Utc::now();
        let frame_hash = self.calculate_frame_hash(&ocr_result);

        // Add frame to buffer
        let frame = AnalysisFrame {
            timestamp,
            ocr_result,
            vision_result,
            frame_hash,
        };

        self.add_frame(frame.clone());

        // Perform real-time analysis
        let detected_tasks = self.detect_tasks(&frame).await?;
        let detected_questions = self.detect_questions(&frame).await?;
        let context_changes = self.track_context_changes(&frame).await?;
        let user_intent_signals = self.analyze_user_intent(&frame).await?;
        let assistance_opportunities = self.generate_assistance_opportunities(
            &detected_tasks,
            &detected_questions,
            &context_changes,
            &user_intent_signals,
        ).await?;

        // Calculate overall confidence
        let confidence_score = self.calculate_overall_confidence(
            &detected_tasks,
            &detected_questions,
            &assistance_opportunities,
        );

        if self.config.debug_mode {
            debug!(
                "Real-time analysis: {} tasks, {} questions, {} opportunities",
                detected_tasks.len(),
                detected_questions.len(),
                assistance_opportunities.len()
            );
        }

        Ok(TaskDetectionResult {
            detected_tasks,
            detected_questions,
            assistance_opportunities,
            context_changes,
            user_intent_signals,
            confidence_score,
            analysis_timestamp: timestamp,
        })
    }

    fn add_frame(&mut self, frame: AnalysisFrame) {
        if self.frame_buffer.len() >= self.config.buffer_size {
            self.frame_buffer.pop_front();
        }
        self.frame_buffer.push_back(frame);
    }

    async fn detect_tasks(&self, frame: &AnalysisFrame) -> Result<Vec<DetectedTask>> {
        let mut tasks = Vec::new();

        // Analyze text content for task indicators
        for word_data in &frame.ocr_result.words {
            if let Some(task) = self.task_patterns.match_word_pattern(&word_data.text, word_data) {
                tasks.push(task);
            }
        }

        // Analyze paragraph-level content
        for para in &frame.ocr_result.paragraphs {
            if let Some(task) = self.task_patterns.match_paragraph_pattern(&para.text, para) {
                tasks.push(task);
            }
        }

        // Analyze application context
        if let Some(vision) = &frame.vision_result {
            for app in &vision.app_context.detected_applications {
                if let Some(task) = self.task_patterns.match_application_context(app, &frame.ocr_result) {
                    tasks.push(task);
                }
            }
        }

        Ok(tasks)
    }

    async fn detect_questions(&self, frame: &AnalysisFrame) -> Result<Vec<DetectedQuestion>> {
        let mut questions = Vec::new();

        // Scan all text for question patterns
        for para in &frame.ocr_result.paragraphs {
            if let Some(question) = self.question_detector.analyze_text(&para.text, para) {
                questions.push(question);
            }
        }

        // Check for error messages that imply questions
        for word_data in &frame.ocr_result.words {
            if word_data.text_type == Some(savant_ocr::TextType::ErrorMessage) {
                if let Some(question) = self.question_detector.error_to_question(&word_data.text, word_data) {
                    questions.push(question);
                }
            }
        }

        Ok(questions)
    }

    async fn track_context_changes(&mut self, frame: &AnalysisFrame) -> Result<Vec<ContextChange>> {
        self.context_tracker.update(frame).await
    }

    async fn analyze_user_intent(&self, frame: &AnalysisFrame) -> Result<Vec<IntentSignal>> {
        if !self.config.enable_intent_tracking {
            return Ok(Vec::new());
        }

        self.intent_analyzer.analyze_frame(frame, &self.frame_buffer).await
    }

    async fn generate_assistance_opportunities(
        &self,
        tasks: &[DetectedTask],
        questions: &[DetectedQuestion],
        context_changes: &[ContextChange],
        intent_signals: &[IntentSignal],
    ) -> Result<Vec<AssistanceOpportunity>> {
        if !self.config.enable_proactive_assistance {
            return Ok(Vec::new());
        }

        self.assistance_engine.generate_opportunities(
            tasks,
            questions,
            context_changes,
            intent_signals,
        ).await
    }

    fn calculate_frame_hash(&self, ocr_result: &ComprehensiveOCRResult) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        ocr_result.raw_text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn calculate_overall_confidence(
        &self,
        tasks: &[DetectedTask],
        questions: &[DetectedQuestion],
        opportunities: &[AssistanceOpportunity],
    ) -> f32 {
        let total_items = tasks.len() + questions.len() + opportunities.len();
        if total_items == 0 {
            return 0.0;
        }

        let total_confidence = tasks.iter().map(|t| t.confidence).sum::<f32>()
            + questions.iter().map(|q| q.confidence).sum::<f32>()
            + opportunities.iter().map(|o| o.confidence).sum::<f32>();

        total_confidence / total_items as f32
    }
}

// Supporting analyzer structs
#[derive(Debug)]
struct TaskPatternMatcher {
    patterns: HashMap<TaskType, Vec<String>>,
}

impl TaskPatternMatcher {
    fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Development task patterns
        patterns.insert(TaskType::DebuggingError, vec![
            "error".to_string(),
            "exception".to_string(),
            "failed".to_string(),
            "debug".to_string(),
            "traceback".to_string(),
            "stack trace".to_string(),
        ]);
        
        patterns.insert(TaskType::WritingCode, vec![
            "function".to_string(),
            "class".to_string(),
            "def ".to_string(),
            "import".to_string(),
            "const ".to_string(),
            "let ".to_string(),
        ]);

        patterns.insert(TaskType::TestingCode, vec![
            "test".to_string(),
            "assert".to_string(),
            "expect".to_string(),
            "describe".to_string(),
            "it(".to_string(),
        ]);

        // Add more patterns...
        
        Self { patterns }
    }

    fn match_word_pattern(&self, text: &str, word_data: &savant_ocr::WordData) -> Option<DetectedTask> {
        for (task_type, patterns) in &self.patterns {
            for pattern in patterns {
                if text.to_lowercase().contains(pattern) {
                    return Some(DetectedTask {
                        id: uuid::Uuid::new_v4().to_string(),
                        task_type: task_type.clone(),
                        description: format!("Detected {} activity: {}", format!("{:?}", task_type), text),
                        context: TaskContext {
                            application: "Unknown".to_string(),
                            domain: "General".to_string(),
                            related_files: Vec::new(),
                            current_action: text.to_string(),
                            session_duration: 0,
                        },
                        estimated_complexity: ComplexityLevel::Simple,
                        suggested_assistance: self.get_assistance_for_task(task_type),
                        confidence: word_data.confidence,
                        bounding_box: Some(word_data.bounding_box.clone()),
                        detected_at: Utc::now(),
                    });
                }
            }
        }
        None
    }

    fn match_paragraph_pattern(&self, text: &str, _para: &savant_ocr::ParagraphData) -> Option<DetectedTask> {
        // More sophisticated paragraph-level analysis
        if text.contains("How to") || text.contains("how to") {
            return Some(DetectedTask {
                id: uuid::Uuid::new_v4().to_string(),
                task_type: TaskType::LearningConcept,
                description: format!("Learning task detected: {}", text),
                context: TaskContext {
                    application: "Unknown".to_string(),
                    domain: "Learning".to_string(),
                    related_files: Vec::new(),
                    current_action: "Learning".to_string(),
                    session_duration: 0,
                },
                estimated_complexity: ComplexityLevel::Moderate,
                suggested_assistance: vec![
                    "Provide step-by-step tutorial".to_string(),
                    "Show relevant examples".to_string(),
                ],
                confidence: 0.8,
                bounding_box: None,
                detected_at: Utc::now(),
            });
        }
        None
    }

    fn match_application_context(
        &self,
        app: &savant_vision::DetectedApp,
        _ocr_result: &ComprehensiveOCRResult,
    ) -> Option<DetectedTask> {
        match &app.app_type {
            savant_vision::AppType::IDE(_) => Some(DetectedTask {
                id: uuid::Uuid::new_v4().to_string(),
                task_type: TaskType::WritingCode,
                description: "Code development in IDE".to_string(),
                context: TaskContext {
                    application: format!("{:?}", app.app_type),
                    domain: "Development".to_string(),
                    related_files: Vec::new(),
                    current_action: "Coding".to_string(),
                    session_duration: 0,
                },
                estimated_complexity: ComplexityLevel::Moderate,
                suggested_assistance: vec![
                    "Code completion".to_string(),
                    "Error checking".to_string(),
                    "Refactoring suggestions".to_string(),
                ],
                confidence: app.confidence,
                bounding_box: None,
                detected_at: Utc::now(),
            }),
            _ => None,
        }
    }

    fn get_assistance_for_task(&self, task_type: &TaskType) -> Vec<String> {
        match task_type {
            TaskType::DebuggingError => vec![
                "Explain error message".to_string(),
                "Suggest debugging steps".to_string(),
                "Show similar issues".to_string(),
            ],
            TaskType::WritingCode => vec![
                "Code completion".to_string(),
                "Best practices".to_string(),
                "Code review".to_string(),
            ],
            _ => vec!["General assistance available".to_string()],
        }
    }
}

#[derive(Debug)]
struct QuestionDetector;

impl QuestionDetector {
    fn new() -> Self {
        Self
    }

    fn analyze_text(&self, text: &str, para: &savant_ocr::ParagraphData) -> Option<DetectedQuestion> {
        if text.contains("?") || text.starts_with("How") || text.starts_with("What") || text.starts_with("Why") {
            Some(DetectedQuestion {
                id: uuid::Uuid::new_v4().to_string(),
                question_type: self.classify_question(text),
                question_text: text.to_string(),
                context: QuestionContext {
                    source: QuestionSource::UIElement,
                    domain: "General".to_string(),
                    related_concepts: Vec::new(),
                    user_expertise_level: ExpertiseLevel::Intermediate,
                },
                suggested_answers: self.generate_answers(text),
                confidence: 0.7,
                urgency: UrgencyLevel::Medium,
                bounding_box: Some(para.bounding_box.clone()),
                detected_at: Utc::now(),
            })
        } else {
            None
        }
    }

    fn error_to_question(&self, error_text: &str, word_data: &savant_ocr::WordData) -> Option<DetectedQuestion> {
        Some(DetectedQuestion {
            id: uuid::Uuid::new_v4().to_string(),
            question_type: QuestionType::ErrorExplanation,
            question_text: format!("What does this error mean: {}", error_text),
            context: QuestionContext {
                source: QuestionSource::ErrorMessage,
                domain: "Error Resolution".to_string(),
                related_concepts: Vec::new(),
                user_expertise_level: ExpertiseLevel::Intermediate,
            },
            suggested_answers: vec![
                SuggestedAnswer {
                    answer_type: AnswerType::DirectAnswer,
                    content: "Error explanation and solution".to_string(),
                    confidence: 0.8,
                    source: "Built-in knowledge".to_string(),
                    action_required: true,
                }
            ],
            confidence: word_data.confidence,
            urgency: UrgencyLevel::High,
            bounding_box: Some(word_data.bounding_box.clone()),
            detected_at: Utc::now(),
        })
    }

    fn classify_question(&self, text: &str) -> QuestionType {
        if text.to_lowercase().contains("how") {
            QuestionType::HowTo
        } else if text.to_lowercase().contains("what") {
            QuestionType::Conceptual
        } else if text.to_lowercase().contains("error") {
            QuestionType::ErrorExplanation
        } else {
            QuestionType::Unknown
        }
    }

    fn generate_answers(&self, _text: &str) -> Vec<SuggestedAnswer> {
        vec![
            SuggestedAnswer {
                answer_type: AnswerType::DirectAnswer,
                content: "AI-generated response".to_string(),
                confidence: 0.7,
                source: "Built-in AI".to_string(),
                action_required: false,
            }
        ]
    }
}

#[derive(Debug)]
struct ContextTracker {
    current_context: Option<String>,
}

impl ContextTracker {
    fn new() -> Self {
        Self {
            current_context: None,
        }
    }

    async fn update(&mut self, _frame: &AnalysisFrame) -> Result<Vec<ContextChange>> {
        // TODO: Implement context tracking
        Ok(Vec::new())
    }
}

#[derive(Debug)]
struct IntentAnalyzer;

impl IntentAnalyzer {
    fn new() -> Self {
        Self
    }

    async fn analyze_frame(
        &self,
        _frame: &AnalysisFrame,
        _buffer: &VecDeque<AnalysisFrame>,
    ) -> Result<Vec<IntentSignal>> {
        // TODO: Implement intent analysis
        Ok(Vec::new())
    }
}

#[derive(Debug)]
struct AssistanceEngine;

impl AssistanceEngine {
    fn new() -> Self {
        Self
    }

    async fn generate_opportunities(
        &self,
        tasks: &[DetectedTask],
        questions: &[DetectedQuestion],
        _context_changes: &[ContextChange],
        _intent_signals: &[IntentSignal],
    ) -> Result<Vec<AssistanceOpportunity>> {
        let mut opportunities = Vec::new();

        // Generate opportunities from detected tasks
        for task in tasks {
            opportunities.push(AssistanceOpportunity {
                id: uuid::Uuid::new_v4().to_string(),
                opportunity_type: AssistanceType::ContextualHelp,
                description: format!("Assistance available for: {}", task.description),
                trigger_context: format!("{:?}", task.task_type),
                suggested_action: task.suggested_assistance.first().cloned().unwrap_or_default(),
                confidence: task.confidence,
                priority: PriorityLevel::Medium,
                expires_at: Some(Utc::now() + chrono::Duration::minutes(30)),
            });
        }

        // Generate opportunities from questions
        for question in questions {
            opportunities.push(AssistanceOpportunity {
                id: uuid::Uuid::new_v4().to_string(),
                opportunity_type: AssistanceType::QuickAction,
                description: format!("Answer available for: {}", question.question_text),
                trigger_context: format!("{:?}", question.question_type),
                suggested_action: "Provide detailed answer".to_string(),
                confidence: question.confidence,
                priority: match question.urgency {
                    UrgencyLevel::Critical => PriorityLevel::Critical,
                    UrgencyLevel::High => PriorityLevel::High,
                    UrgencyLevel::Medium => PriorityLevel::Medium,
                    UrgencyLevel::Low => PriorityLevel::Low,
                },
                expires_at: Some(Utc::now() + chrono::Duration::minutes(15)),
            });
        }

        Ok(opportunities)
    }
}