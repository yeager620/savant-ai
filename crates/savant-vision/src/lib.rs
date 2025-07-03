use anyhow::Result;
use chrono::{DateTime, Utc};
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};

pub mod detector;
pub mod classifier;
pub mod analyzer;
pub mod patterns;

pub use detector::{ObjectDetector, UIDetector, AppDetector, DetectionResult};
pub use classifier::{ActivityClassifier, Activity, ActivityClassification};
pub use analyzer::{ContextAnalyzer, VisualContext, Evidence};
pub use patterns::{PatternMatcher, VisualPattern};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualElement {
    pub element_type: ElementType,
    pub bounding_box: BoundingBox,
    pub properties: ElementProperties,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Window,
    Button,
    TextField,
    Image,
    Video,
    Menu,
    Icon,
    Text,
    StatusBar,
    Toolbar,
    Browser,
    IDE,
    VideoCall,
    Chat,
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementProperties {
    pub color_scheme: Option<ColorScheme>,
    pub text_content: Option<String>,
    pub is_interactive: bool,
    pub state: Option<String>, // "active", "disabled", "selected", etc.
    pub app_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub dominant_colors: Vec<String>, // Hex color codes
    pub is_dark_theme: bool,
    pub accent_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    pub detected_applications: Vec<DetectedApp>,
    pub active_windows: Vec<WindowInfo>,
    pub browser_context: Option<BrowserContext>,
    pub ide_context: Option<IDEContext>,
    pub meeting_context: Option<MeetingContext>,
    pub desktop_environment: Option<DesktopContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedApp {
    pub app_type: AppType,
    pub app_name: Option<String>,
    pub confidence: f32,
    pub visual_indicators: Vec<VisualIndicator>,
    pub screen_region: BoundingBox,
    pub window_state: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppType {
    VideoConferencing(VideoConferencingApp),
    IDE(IDEType),
    Browser(BrowserType),
    Productivity(ProductivityApp),
    Entertainment(EntertainmentApp),
    Communication(CommunicationApp),
    Development(DevelopmentApp),
    SystemUtility(SystemUtilityApp),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoConferencingApp {
    Zoom,
    GoogleMeet,
    MicrosoftTeams,
    Slack,
    Discord,
    Webex,
    Skype,
    FaceTime,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IDEType {
    VSCode,
    IntelliJ,
    Xcode,
    Sublime,
    Vim,
    Emacs,
    Atom,
    RustRover,
    PyCharm,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Safari,
    Firefox,
    Edge,
    Arc,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductivityApp {
    Word,
    Excel,
    PowerPoint,
    Notion,
    Obsidian,
    Notes,
    Pages,
    Numbers,
    Keynote,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntertainmentApp {
    YouTube,
    Netflix,
    Spotify,
    AppleMusic,
    Twitch,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationApp {
    Messages,
    WhatsApp,
    Telegram,
    Signal,
    Email,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevelopmentApp {
    Terminal,
    Docker,
    Postman,
    GitClient,
    DatabaseTool,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemUtilityApp {
    Finder,
    FileExplorer,
    TaskManager,
    SystemPreferences,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualIndicator {
    pub indicator_type: IndicatorType,
    pub value: String,
    pub position: BoundingBox,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorType {
    WindowTitle,
    Logo,
    UILayout,
    ColorScheme,
    URLBar,
    StatusIndicator,
    ButtonText,
    IconShape,
    MenuStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub app_name: String,
    pub bounds: BoundingBox,
    pub window_level: i32,
    pub is_focused: bool,
    pub is_minimized: bool,
    pub is_fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowState {
    Focused,
    Background,
    Minimized,
    Fullscreen,
    Split,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserContext {
    pub browser_type: BrowserType,
    pub visible_tabs: Vec<TabInfo>,
    pub current_url: Option<String>,
    pub page_type: PageType,
    pub navigation_elements: Vec<VisualElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub title: String,
    pub url: Option<String>,
    pub is_active: bool,
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageType {
    Search,
    SocialMedia,
    Documentation,
    VideoStreaming,
    Shopping,
    News,
    Email,
    Productivity,
    Development,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDEContext {
    pub ide_type: IDEType,
    pub active_file: Option<String>,
    pub language: Option<String>,
    pub project_structure: Option<Vec<String>>,
    pub errors_visible: bool,
    pub debug_mode: bool,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingContext {
    pub platform: VideoConferencingApp,
    pub participant_count: Option<u32>,
    pub is_screen_sharing: bool,
    pub is_recording: bool,
    pub camera_on: bool,
    pub microphone_on: bool,
    pub chat_visible: bool,
    pub participants_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopContext {
    pub desktop_environment: String, // "macOS", "Windows", "GNOME", etc.
    pub theme: ThemeInfo,
    pub dock_visible: bool,
    pub menu_bar_visible: bool,
    pub notification_center_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub is_dark_mode: bool,
    pub accent_color: Option<String>,
    pub wallpaper_type: Option<String>,
}

#[derive(Debug)]
pub struct VisionAnalyzer {
    pub object_detector: ObjectDetector,
    pub ui_detector: UIDetector,
    pub app_detector: AppDetector,
    pub activity_classifier: ActivityClassifier,
    pub context_analyzer: ContextAnalyzer,
    pub pattern_matcher: PatternMatcher,
}

impl VisionAnalyzer {
    pub fn new(_config: VisionConfig) -> Result<Self> {
        Ok(Self {
            object_detector: ObjectDetector::new(),
            ui_detector: UIDetector::new(),
            app_detector: AppDetector::new(),
            activity_classifier: ActivityClassifier::new(),
            context_analyzer: ContextAnalyzer::new(),
            pattern_matcher: PatternMatcher::new(),
        })
    }

    pub async fn analyze_screen(&self, image: &DynamicImage) -> Result<ScreenAnalysis> {
        let start_time = std::time::Instant::now();

        // Detect visual elements
        let visual_elements = self.ui_detector.detect_elements(image).await?;
        
        // Detect applications
        let app_context = self.app_detector.detect_applications(image, &visual_elements).await?;
        
        // Classify current activity
        let activity_classification = self.activity_classifier.classify_activity(image, &app_context, &visual_elements).await?;
        
        // Analyze overall context
        let visual_context = self.context_analyzer.analyze_context(image, &app_context, &visual_elements).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(ScreenAnalysis {
            timestamp: Utc::now(),
            visual_elements,
            app_context,
            activity_classification,
            visual_context,
            processing_time_ms: processing_time,
            image_metadata: ImageMetadata {
                width: image.width(),
                height: image.height(),
                format: "DynamicImage".to_string(),
                file_size: None,
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub timestamp: DateTime<Utc>,
    pub visual_elements: Vec<VisualElement>,
    pub app_context: AppContext,
    pub activity_classification: ActivityClassification,
    pub visual_context: VisualContext,
    pub processing_time_ms: u64,
    pub image_metadata: ImageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    pub enable_app_detection: bool,
    pub enable_activity_classification: bool,
    pub enable_ui_analysis: bool,
    pub pattern_matching_threshold: f32,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            enable_app_detection: true,
            enable_activity_classification: true,
            enable_ui_analysis: true,
            pattern_matching_threshold: 0.6,
        }
    }
}