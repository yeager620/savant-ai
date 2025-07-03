use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::BoundingBox;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TextType {
    UIElement,
    CodeSnippet,
    DocumentContent,
    ChatMessage,
    EmailContent,
    WebPageContent,
    TerminalOutput,
    MeetingContent,
    BrowserUI,
    IDEContent,
    SystemDialog,
    MenuBar,
    StatusBar,
    Button,
    Label,
    TextField,
    ErrorMessage,
    Unknown,
}

#[derive(Debug)]
pub struct TextClassifier {
    patterns: HashMap<TextType, Vec<Regex>>,
    ui_position_classifier: UIPositionClassifier,
}

impl TextClassifier {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Code patterns
        patterns.insert(TextType::CodeSnippet, vec![
            Regex::new(r"^\s*(def|function|class|interface|struct|impl|fn|let|const|var)\s+").unwrap(),
            Regex::new(r"^\s*[a-zA-Z_][a-zA-Z0-9_]*\s*[=:]\s*").unwrap(),
            Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*\([^)]*\)\s*[{:]?").unwrap(),
            Regex::new(r"^\s*[{}]\s*$").unwrap(),
            Regex::new(r"^\s*//|^\s*/\*|^\s*#|^\s*<!--|^\s*%").unwrap(),
        ]);

        // Terminal patterns
        patterns.insert(TextType::TerminalOutput, vec![
            Regex::new(r"^\$\s+").unwrap(),
            Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*@[a-zA-Z0-9_-]+:").unwrap(),
            Regex::new(r"^[A-Z]:\\.*>").unwrap(),
            Regex::new(r"^\[.*\]\s+").unwrap(),
            Regex::new(r"^Error:|^Warning:|^Info:").unwrap(),
        ]);

        // Chat/messaging patterns
        patterns.insert(TextType::ChatMessage, vec![
            Regex::new(r"^[A-Za-z\s]+:\s*").unwrap(),
            Regex::new(r"^\d{1,2}:\d{2}\s*[AP]M").unwrap(),
            Regex::new(r"^You:|^Me:").unwrap(),
            Regex::new(r"^@[a-zA-Z0-9_]+").unwrap(),
        ]);

        // Email patterns
        patterns.insert(TextType::EmailContent, vec![
            Regex::new(r"^From:|^To:|^Subject:|^Date:").unwrap(),
            Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            Regex::new(r"^Re:|^Fwd:").unwrap(),
        ]);

        // Web content patterns
        patterns.insert(TextType::WebPageContent, vec![
            Regex::new(r"^https?://").unwrap(),
            Regex::new(r"www\.[a-zA-Z0-9.-]+").unwrap(),
            Regex::new(r"^Home|About|Contact|FAQ|Login|Sign Up|Register").unwrap(),
        ]);

        // Meeting content patterns
        patterns.insert(TextType::MeetingContent, vec![
            Regex::new(r"^Meeting|^Conference|^Call").unwrap(),
            Regex::new(r"^Participants:|^Attendees:").unwrap(),
            Regex::new(r"^Zoom|^Teams|^Meet|^Webex").unwrap(),
            Regex::new(r"^Mute|^Unmute|^Camera|^Share Screen").unwrap(),
        ]);

        // Browser UI patterns
        patterns.insert(TextType::BrowserUI, vec![
            Regex::new(r"^Back|^Forward|^Refresh|^Home|^Bookmarks").unwrap(),
            Regex::new(r"^New Tab|^Private|^Incognito").unwrap(),
            Regex::new(r"^Downloads|^History|^Settings").unwrap(),
        ]);

        // IDE patterns
        patterns.insert(TextType::IDEContent, vec![
            Regex::new(r"^File|^Edit|^View|^Run|^Debug|^Tools").unwrap(),
            Regex::new(r"^Problems|^Output|^Terminal|^Explorer").unwrap(),
            Regex::new(r"^\d+\s+\|\s*").unwrap(), // Line numbers
        ]);

        // Error message patterns
        patterns.insert(TextType::ErrorMessage, vec![
            Regex::new(r"^Error:|^ERROR:|^Exception:|^EXCEPTION:").unwrap(),
            Regex::new(r"^SyntaxError|^TypeError|^ValueError|^RuntimeError").unwrap(),
            Regex::new(r"^Fatal:|^FATAL:|^Failure:|^FAILURE:").unwrap(),
            Regex::new(r"^\s*at\s+.*:\d+:\d+").unwrap(), // Stack trace lines
        ]);

        // UI element patterns
        patterns.insert(TextType::Button, vec![
            Regex::new(r"^OK$|^Cancel$|^Apply$|^Submit$|^Save$|^Delete$").unwrap(),
            Regex::new(r"^Yes$|^No$|^Close$|^Exit$|^Next$|^Previous$").unwrap(),
        ]);

        Self {
            patterns,
            ui_position_classifier: UIPositionClassifier::new(),
        }
    }

    pub fn classify(&self, text: &str, bounding_box: &BoundingBox) -> Result<TextType> {
        // First check position-based classification
        if let Some(ui_type) = self.ui_position_classifier.classify_by_position(text, bounding_box) {
            return Ok(ui_type);
        }

        // Then check pattern-based classification
        for (text_type, regexes) in &self.patterns {
            for regex in regexes {
                if regex.is_match(text) {
                    return Ok(text_type.clone());
                }
            }
        }

        // Check content-based heuristics
        self.classify_by_content_heuristics(text)
    }

    fn classify_by_content_heuristics(&self, text: &str) -> Result<TextType> {
        let text_lower = text.to_lowercase();

        // Check for programming-related keywords
        let code_keywords = [
            "function", "return", "import", "export", "class", "interface",
            "struct", "enum", "const", "let", "var", "def", "async", "await"
        ];
        if code_keywords.iter().any(|&keyword| text_lower.contains(keyword)) {
            return Ok(TextType::CodeSnippet);
        }

        // Check for UI elements by common words
        let ui_keywords = ["click", "button", "menu", "dialog", "window", "tab"];
        if ui_keywords.iter().any(|&keyword| text_lower.contains(keyword)) {
            return Ok(TextType::UIElement);
        }

        // Check text length and structure for document content
        if text.len() > 50 && text.contains(' ') && !text.contains('{') && !text.contains(';') {
            return Ok(TextType::DocumentContent);
        }

        // Default classification
        Ok(TextType::Unknown)
    }
}

#[derive(Debug)]
struct UIPositionClassifier {
    #[allow(dead_code)]
    screen_regions: HashMap<ScreenRegion, TextType>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
#[allow(dead_code)]
enum ScreenRegion {
    TopBar,
    BottomBar,
    LeftSidebar,
    RightSidebar,
    Center,
}

impl UIPositionClassifier {
    fn new() -> Self {
        let mut screen_regions = HashMap::new();
        screen_regions.insert(ScreenRegion::TopBar, TextType::MenuBar);
        screen_regions.insert(ScreenRegion::BottomBar, TextType::StatusBar);

        Self { screen_regions }
    }

    fn classify_by_position(&self, text: &str, bounding_box: &BoundingBox) -> Option<TextType> {
        // This would need screen dimensions to be truly effective
        // For now, use simple heuristics based on relative position

        // Very small height suggests UI element
        if bounding_box.height < 30 {
            return Some(TextType::UIElement);
        }

        // Top 50 pixels could be menu bar
        if bounding_box.y < 50 && bounding_box.height < 50 {
            return Some(TextType::MenuBar);
        }

        // Single words that are short could be buttons/labels
        if text.split_whitespace().count() == 1 && text.len() < 20 {
            return Some(TextType::UIElement);
        }

        None
    }
}
