use anyhow::Result;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{AppContext, AppType, VisualElement, ElementType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityClassification {
    pub primary_activity: Activity,
    pub secondary_activities: Vec<Activity>,
    pub context_indicators: Vec<ContextIndicator>,
    pub confidence: f32,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Activity {
    Coding {
        language: Option<String>,
        editor: String,
        project_type: Option<String>,
        debugging: bool,
    },
    VideoConferencing {
        platform: String,
        participant_count: Option<u32>,
        screen_sharing: bool,
        recording: bool,
    },
    WebBrowsing {
        site_category: SiteCategory,
        primary_domain: String,
        tab_count: Option<u32>,
    },
    Documentation {
        doc_type: DocumentationType,
        reading_vs_writing: ReadingWritingRatio,
        platform: String,
    },
    Entertainment {
        platform: String,
        content_type: ContentType,
        is_streaming: bool,
    },
    Communication {
        platform: String,
        message_type: MessageType,
        active_conversation: bool,
    },
    Productivity {
        app_category: ProductivityCategory,
        document_type: Option<String>,
        collaboration: bool,
    },
    Gaming {
        game_name: Option<String>,
        platform: String,
        multiplayer: bool,
    },
    SystemManagement {
        task_type: SystemTaskType,
        tools_used: Vec<String>,
    },
    Idle {
        screen_saver: bool,
        last_activity_mins: Option<u32>,
    },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiteCategory {
    SocialMedia,
    News,
    Shopping,
    Education,
    Entertainment,
    Productivity,
    Development,
    Research,
    Email,
    Documentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationType {
    TechnicalDoc,
    UserManual,
    API,
    Tutorial,
    Blog,
    Academic,
    Legal,
    Business,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadingWritingRatio {
    MostlyReading,
    MostlyWriting,
    Balanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Video,
    Music,
    Podcast,
    Game,
    Article,
    SocialFeed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    DirectMessage,
    GroupChat,
    Email,
    VideoCall,
    VoiceCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductivityCategory {
    TextEditing,
    Spreadsheet,
    Presentation,
    ProjectManagement,
    Design,
    DataAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemTaskType {
    FileManagement,
    SystemConfiguration,
    Monitoring,
    Troubleshooting,
    Installation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextIndicator {
    pub indicator_type: IndicatorType,
    pub value: String,
    pub confidence: f32,
    pub source: IndicatorSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorType {
    WindowTitle,
    URLBar,
    UILayout,
    VisualElement,
    TextPattern,
    ColorScheme,
    ApplicationPresence,
    UserInteraction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorSource {
    SystemAPI,
    VisualAnalysis,
    OCR,
    PatternMatching,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub confidence: f32,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    ApplicationDetection,
    UIElementPresence,
    TextContent,
    VisualPattern,
    ColorAnalysis,
    WindowLayout,
    UserBehavior,
}

pub struct ActivityClassifier {
    classification_rules: Vec<ClassificationRule>,
    context_analyzers: HashMap<String, ContextAnalyzer>,
}

#[derive(Debug, Clone)]
struct ClassificationRule {
    activity: Activity,
    required_indicators: Vec<RequiredIndicator>,
    optional_indicators: Vec<OptionalIndicator>,
    exclusion_patterns: Vec<ExclusionPattern>,
    confidence_threshold: f32,
}

#[derive(Debug, Clone)]
struct RequiredIndicator {
    indicator_type: IndicatorType,
    pattern: String,
    weight: f32,
}

#[derive(Debug, Clone)]
struct OptionalIndicator {
    indicator_type: IndicatorType,
    pattern: String,
    bonus_weight: f32,
}

#[derive(Debug, Clone)]
struct ExclusionPattern {
    indicator_type: IndicatorType,
    pattern: String,
}

struct ContextAnalyzer {
    analyzer_type: String,
}

impl ActivityClassifier {
    pub fn new() -> Self {
        let mut classification_rules = Vec::new();
        
        // Coding activity rules
        classification_rules.push(ClassificationRule {
            activity: Activity::Coding {
                language: None,
                editor: "Unknown".to_string(),
                project_type: None,
                debugging: false,
            },
            required_indicators: vec![
                RequiredIndicator {
                    indicator_type: IndicatorType::ApplicationPresence,
                    pattern: "IDE|Editor".to_string(),
                    weight: 0.8,
                }
            ],
            optional_indicators: vec![
                OptionalIndicator {
                    indicator_type: IndicatorType::TextPattern,
                    pattern: r"(function|class|def|import|var|let|const)".to_string(),
                    bonus_weight: 0.3,
                },
                OptionalIndicator {
                    indicator_type: IndicatorType::VisualElement,
                    pattern: "Terminal|Console".to_string(),
                    bonus_weight: 0.2,
                }
            ],
            exclusion_patterns: Vec::new(),
            confidence_threshold: 0.6,
        });

        // Video conferencing rules
        classification_rules.push(ClassificationRule {
            activity: Activity::VideoConferencing {
                platform: "Unknown".to_string(),
                participant_count: None,
                screen_sharing: false,
                recording: false,
            },
            required_indicators: vec![
                RequiredIndicator {
                    indicator_type: IndicatorType::ApplicationPresence,
                    pattern: "VideoConferencing".to_string(),
                    weight: 0.9,
                }
            ],
            optional_indicators: vec![
                OptionalIndicator {
                    indicator_type: IndicatorType::VisualElement,
                    pattern: "Mute|Camera|Share".to_string(),
                    bonus_weight: 0.3,
                }
            ],
            exclusion_patterns: Vec::new(),
            confidence_threshold: 0.7,
        });

        // Web browsing rules
        classification_rules.push(ClassificationRule {
            activity: Activity::WebBrowsing {
                site_category: SiteCategory::Research,
                primary_domain: "Unknown".to_string(),
                tab_count: None,
            },
            required_indicators: vec![
                RequiredIndicator {
                    indicator_type: IndicatorType::ApplicationPresence,
                    pattern: "Browser".to_string(),
                    weight: 0.8,
                }
            ],
            optional_indicators: vec![
                OptionalIndicator {
                    indicator_type: IndicatorType::URLBar,
                    pattern: r"https?://".to_string(),
                    bonus_weight: 0.3,
                }
            ],
            exclusion_patterns: Vec::new(),
            confidence_threshold: 0.5,
        });

        Self {
            classification_rules,
            context_analyzers: HashMap::new(),
        }
    }

    pub async fn classify_activity(
        &self,
        _image: &DynamicImage,
        app_context: &AppContext,
        visual_elements: &[VisualElement],
    ) -> Result<ActivityClassification> {
        
        let mut activity_scores: HashMap<String, f32> = HashMap::new();
        let mut all_evidence: Vec<Evidence> = Vec::new();
        let mut context_indicators: Vec<ContextIndicator> = Vec::new();

        // Analyze application context
        for detected_app in &app_context.detected_applications {
            let app_evidence = self.analyze_app_context(detected_app)?;
            all_evidence.extend(app_evidence);
        }

        // Analyze visual elements
        let visual_evidence = self.analyze_visual_elements(visual_elements)?;
        all_evidence.extend(visual_evidence);

        // Apply classification rules
        for rule in &self.classification_rules {
            let score = self.evaluate_rule(rule, &all_evidence, app_context)?;
            if score >= rule.confidence_threshold {
                let activity_key = self.activity_to_key(&rule.activity);
                activity_scores.insert(activity_key, score);
            }
        }

        // Determine primary and secondary activities
        let mut scored_activities: Vec<(String, f32)> = activity_scores.into_iter().collect();
        scored_activities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let primary_activity = if !scored_activities.is_empty() {
            self.key_to_activity(&scored_activities[0].0, app_context)?
        } else {
            Activity::Unknown
        };

        let secondary_activities: Vec<Activity> = scored_activities
            .iter()
            .skip(1)
            .take(3)
            .filter_map(|(key, _)| self.key_to_activity(key, app_context).ok())
            .collect();

        let overall_confidence = scored_activities
            .first()
            .map(|(_, score)| *score)
            .unwrap_or(0.0);

        // Generate context indicators
        context_indicators.extend(self.generate_context_indicators(app_context, visual_elements)?);

        Ok(ActivityClassification {
            primary_activity,
            secondary_activities,
            context_indicators,
            confidence: overall_confidence,
            evidence: all_evidence,
        })
    }

    fn analyze_app_context(&self, detected_app: &crate::DetectedApp) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        evidence.push(Evidence {
            evidence_type: EvidenceType::ApplicationDetection,
            description: format!("Detected application: {:?}", detected_app.app_type),
            confidence: detected_app.confidence,
            weight: 0.8,
        });

        // Analyze specific app types
        match &detected_app.app_type {
            AppType::IDE(_) => {
                evidence.push(Evidence {
                    evidence_type: EvidenceType::ApplicationDetection,
                    description: "IDE detected - likely coding activity".to_string(),
                    confidence: detected_app.confidence * 0.9,
                    weight: 0.9,
                });
            }
            AppType::VideoConferencing(_) => {
                evidence.push(Evidence {
                    evidence_type: EvidenceType::ApplicationDetection,
                    description: "Video conferencing app detected".to_string(),
                    confidence: detected_app.confidence * 0.95,
                    weight: 0.95,
                });
            }
            AppType::Browser(_) => {
                evidence.push(Evidence {
                    evidence_type: EvidenceType::ApplicationDetection,
                    description: "Browser detected - web browsing activity".to_string(),
                    confidence: detected_app.confidence * 0.7,
                    weight: 0.7,
                });
            }
            _ => {}
        }

        Ok(evidence)
    }

    fn analyze_visual_elements(&self, visual_elements: &[VisualElement]) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        for element in visual_elements {
            match element.element_type {
                ElementType::Terminal => {
                    evidence.push(Evidence {
                        evidence_type: EvidenceType::UIElementPresence,
                        description: "Terminal window detected".to_string(),
                        confidence: element.confidence,
                        weight: 0.6,
                    });
                }
                ElementType::VideoCall => {
                    evidence.push(Evidence {
                        evidence_type: EvidenceType::UIElementPresence,
                        description: "Video call interface detected".to_string(),
                        confidence: element.confidence,
                        weight: 0.8,
                    });
                }
                ElementType::Chat => {
                    evidence.push(Evidence {
                        evidence_type: EvidenceType::UIElementPresence,
                        description: "Chat interface detected".to_string(),
                        confidence: element.confidence,
                        weight: 0.5,
                    });
                }
                _ => {}
            }
        }

        Ok(evidence)
    }

    fn evaluate_rule(&self, rule: &ClassificationRule, evidence: &[Evidence], _app_context: &AppContext) -> Result<f32> {
        let mut score = 0.0;
        let mut required_met = 0;
        let required_count = rule.required_indicators.len();

        // Check required indicators
        for required in &rule.required_indicators {
            if self.evidence_matches_indicator(evidence, required) {
                score += required.weight;
                required_met += 1;
            }
        }

        // Must meet all required indicators
        if required_met < required_count {
            return Ok(0.0);
        }

        // Check optional indicators
        for optional in &rule.optional_indicators {
            if self.evidence_matches_optional(evidence, optional) {
                score += optional.bonus_weight;
            }
        }

        // Check exclusion patterns
        for exclusion in &rule.exclusion_patterns {
            if self.evidence_matches_exclusion(evidence, exclusion) {
                score *= 0.5; // Reduce confidence if exclusion pattern matches
            }
        }

        Ok(score.min(1.0))
    }

    fn evidence_matches_indicator(&self, evidence: &[Evidence], indicator: &RequiredIndicator) -> bool {
        evidence.iter().any(|e| {
            match indicator.indicator_type {
                IndicatorType::ApplicationPresence => {
                    e.description.contains(&indicator.pattern)
                }
                _ => false,
            }
        })
    }

    fn evidence_matches_optional(&self, evidence: &[Evidence], indicator: &OptionalIndicator) -> bool {
        evidence.iter().any(|e| {
            e.description.contains(&indicator.pattern)
        })
    }

    fn evidence_matches_exclusion(&self, evidence: &[Evidence], exclusion: &ExclusionPattern) -> bool {
        evidence.iter().any(|e| {
            e.description.contains(&exclusion.pattern)
        })
    }

    fn activity_to_key(&self, activity: &Activity) -> String {
        match activity {
            Activity::Coding { .. } => "coding".to_string(),
            Activity::VideoConferencing { .. } => "video_conferencing".to_string(),
            Activity::WebBrowsing { .. } => "web_browsing".to_string(),
            Activity::Documentation { .. } => "documentation".to_string(),
            Activity::Entertainment { .. } => "entertainment".to_string(),
            Activity::Communication { .. } => "communication".to_string(),
            Activity::Productivity { .. } => "productivity".to_string(),
            Activity::Gaming { .. } => "gaming".to_string(),
            Activity::SystemManagement { .. } => "system_management".to_string(),
            Activity::Idle { .. } => "idle".to_string(),
            Activity::Unknown => "unknown".to_string(),
        }
    }

    fn key_to_activity(&self, key: &str, app_context: &AppContext) -> Result<Activity> {
        match key {
            "coding" => {
                let (language, editor) = self.detect_coding_context(app_context)?;
                Ok(Activity::Coding {
                    language,
                    editor,
                    project_type: None,
                    debugging: false,
                })
            }
            "video_conferencing" => {
                let platform = self.detect_meeting_platform(app_context)?;
                Ok(Activity::VideoConferencing {
                    platform,
                    participant_count: None,
                    screen_sharing: false,
                    recording: false,
                })
            }
            "web_browsing" => {
                Ok(Activity::WebBrowsing {
                    site_category: SiteCategory::Research,
                    primary_domain: "Unknown".to_string(),
                    tab_count: None,
                })
            }
            _ => Ok(Activity::Unknown),
        }
    }

    fn detect_coding_context(&self, app_context: &AppContext) -> Result<(Option<String>, String)> {
        for app in &app_context.detected_applications {
            if let AppType::IDE(ide_type) = &app.app_type {
                let editor = match ide_type {
                    crate::IDEType::VSCode => "Visual Studio Code".to_string(),
                    crate::IDEType::IntelliJ => "IntelliJ IDEA".to_string(),
                    crate::IDEType::Xcode => "Xcode".to_string(),
                    crate::IDEType::Other(name) => name.clone(),
                    _ => "Unknown IDE".to_string(),
                };
                return Ok((None, editor));
            }
        }
        Ok((None, "Unknown".to_string()))
    }

    fn detect_meeting_platform(&self, app_context: &AppContext) -> Result<String> {
        for app in &app_context.detected_applications {
            if let AppType::VideoConferencing(platform) = &app.app_type {
                let platform_name = match platform {
                    crate::VideoConferencingApp::Zoom => "Zoom".to_string(),
                    crate::VideoConferencingApp::GoogleMeet => "Google Meet".to_string(),
                    crate::VideoConferencingApp::MicrosoftTeams => "Microsoft Teams".to_string(),
                    crate::VideoConferencingApp::Other(name) => name.clone(),
                    _ => "Unknown".to_string(),
                };
                return Ok(platform_name);
            }
        }
        Ok("Unknown".to_string())
    }

    fn generate_context_indicators(&self, app_context: &AppContext, visual_elements: &[VisualElement]) -> Result<Vec<ContextIndicator>> {
        let mut indicators = Vec::new();

        // Generate indicators from detected applications
        for app in &app_context.detected_applications {
            indicators.push(ContextIndicator {
                indicator_type: IndicatorType::ApplicationPresence,
                value: format!("{:?}", app.app_type),
                confidence: app.confidence,
                source: IndicatorSource::VisualAnalysis,
            });
        }

        // Generate indicators from visual elements
        for element in visual_elements {
            indicators.push(ContextIndicator {
                indicator_type: IndicatorType::VisualElement,
                value: format!("{:?}", element.element_type),
                confidence: element.confidence,
                source: IndicatorSource::VisualAnalysis,
            });
        }

        Ok(indicators)
    }
}