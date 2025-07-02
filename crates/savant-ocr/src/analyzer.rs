use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{BoundingBox, TextBlock, TextType};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredContent {
    pub code_blocks: Vec<CodeBlock>,
    pub ui_elements: Vec<UIElement>,
    pub chat_messages: Vec<ChatMessage>,
    pub document_structure: Option<DocumentStructure>,
    pub browser_content: Option<BrowserContent>,
    pub ide_context: Option<IDEContext>,
    pub meeting_context: Option<MeetingContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub content: String,
    pub line_numbers: Option<Vec<u32>>,
    pub bounding_box: BoundingBox,
    pub syntax_elements: Vec<SyntaxElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxElement {
    pub element_type: SyntaxElementType,
    pub value: String,
    pub position: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyntaxElementType {
    Keyword,
    Identifier,
    String,
    Number,
    Comment,
    Operator,
    Function,
    Class,
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub element_type: UIElementType,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub is_interactive: bool,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIElementType {
    Button,
    Label,
    TextField,
    MenuItem,
    Tab,
    Link,
    Icon,
    StatusIndicator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: Option<String>,
    pub timestamp: Option<String>,
    pub content: String,
    pub message_type: MessageType,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    UserMessage,
    BotResponse,
    SystemMessage,
    StatusUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructure {
    pub title: Option<String>,
    pub headings: Vec<Heading>,
    pub paragraphs: Vec<Paragraph>,
    pub lists: Vec<List>,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub list_type: ListType,
    pub items: Vec<String>,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListType {
    Bullet,
    Numbered,
    Checklist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserContent {
    pub url: Option<String>,
    pub title: Option<String>,
    pub navigation_elements: Vec<UIElement>,
    pub main_content: Vec<TextBlock>,
    pub sidebar_content: Vec<TextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDEContext {
    pub active_file: Option<String>,
    pub language: Option<String>,
    pub line_count: Option<u32>,
    pub current_line: Option<u32>,
    pub errors: Vec<IDEError>,
    pub suggestions: Vec<IDESuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDEError {
    pub line: u32,
    pub column: Option<u32>,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDESuggestion {
    pub line: u32,
    pub suggestion: String,
    pub auto_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingContext {
    pub platform: Option<String>,
    pub participants: Vec<String>,
    pub meeting_controls: Vec<UIElement>,
    pub chat_panel: Option<Vec<ChatMessage>>,
    pub screen_sharing: bool,
}

pub struct StructuredContentAnalyzer {
    syntax_patterns: HashMap<String, Vec<regex::Regex>>,
}

impl StructuredContentAnalyzer {
    pub fn new() -> Self {
        let mut syntax_patterns = HashMap::new();
        
        // Common programming language patterns
        syntax_patterns.insert("rust".to_string(), vec![
            regex::Regex::new(r"\b(fn|let|mut|const|struct|enum|impl|trait|use|mod|pub|crate)\b").unwrap(),
        ]);
        
        syntax_patterns.insert("javascript".to_string(), vec![
            regex::Regex::new(r"\b(function|const|let|var|class|async|await|import|export)\b").unwrap(),
        ]);
        
        syntax_patterns.insert("python".to_string(), vec![
            regex::Regex::new(r"\b(def|class|import|from|if|else|elif|for|while|try|except)\b").unwrap(),
        ]);

        Self { syntax_patterns }
    }

    pub fn analyze(&self, text_blocks: &[TextBlock]) -> Result<StructuredContent> {
        let mut content = StructuredContent::default();

        // Group text blocks by type
        let mut blocks_by_type: HashMap<TextType, Vec<&TextBlock>> = HashMap::new();
        for block in text_blocks {
            blocks_by_type.entry(block.semantic_type.clone()).or_default().push(block);
        }

        // Analyze code blocks
        if let Some(code_blocks) = blocks_by_type.get(&TextType::CodeSnippet) {
            content.code_blocks = self.analyze_code_blocks(code_blocks)?;
        }

        // Analyze UI elements
        let ui_types = [TextType::UIElement, TextType::Button, TextType::Label, TextType::TextField];
        for ui_type in &ui_types {
            if let Some(ui_blocks) = blocks_by_type.get(ui_type) {
                content.ui_elements.extend(self.analyze_ui_elements(ui_blocks)?);
            }
        }

        // Analyze chat messages
        if let Some(chat_blocks) = blocks_by_type.get(&TextType::ChatMessage) {
            content.chat_messages = self.analyze_chat_messages(chat_blocks)?;
        }

        // Analyze document structure
        if let Some(doc_blocks) = blocks_by_type.get(&TextType::DocumentContent) {
            content.document_structure = self.analyze_document_structure(doc_blocks)?;
        }

        // Analyze browser content
        if let Some(web_blocks) = blocks_by_type.get(&TextType::WebPageContent) {
            content.browser_content = self.analyze_browser_content(web_blocks, &blocks_by_type)?;
        }

        // Analyze IDE context
        if let Some(ide_blocks) = blocks_by_type.get(&TextType::IDEContent) {
            content.ide_context = self.analyze_ide_context(ide_blocks, &blocks_by_type)?;
        }

        // Analyze meeting context
        if let Some(meeting_blocks) = blocks_by_type.get(&TextType::MeetingContent) {
            content.meeting_context = self.analyze_meeting_context(meeting_blocks, &blocks_by_type)?;
        }

        Ok(content)
    }

    fn analyze_code_blocks(&self, blocks: &[&TextBlock]) -> Result<Vec<CodeBlock>> {
        let mut code_blocks = Vec::new();

        // Group consecutive code blocks
        let mut current_block_text = String::new();
        let mut current_bounding_box: Option<BoundingBox> = None;
        let mut detected_language: Option<String> = None;

        for block in blocks {
            // Detect programming language
            if detected_language.is_none() {
                detected_language = self.detect_programming_language(&block.text);
            }

            current_block_text.push_str(&block.text);
            current_block_text.push('\n');

            if let Some(ref mut bbox) = current_bounding_box {
                // Expand bounding box
                let right = bbox.x + bbox.width;
                let bottom = bbox.y + bbox.height;
                let new_right = block.bounding_box.x + block.bounding_box.width;
                let new_bottom = block.bounding_box.y + block.bounding_box.height;

                bbox.x = bbox.x.min(block.bounding_box.x);
                bbox.y = bbox.y.min(block.bounding_box.y);
                bbox.width = new_right.max(right) - bbox.x;
                bbox.height = new_bottom.max(bottom) - bbox.y;
            } else {
                current_bounding_box = Some(block.bounding_box.clone());
            }
        }

        if !current_block_text.trim().is_empty() {
            let syntax_elements = self.analyze_syntax_elements(&current_block_text, &detected_language);
            
            code_blocks.push(CodeBlock {
                language: detected_language,
                content: current_block_text.trim().to_string(),
                line_numbers: None,
                bounding_box: current_bounding_box.unwrap_or_default(),
                syntax_elements,
            });
        }

        Ok(code_blocks)
    }

    fn detect_programming_language(&self, text: &str) -> Option<String> {
        for (language, patterns) in &self.syntax_patterns {
            for pattern in patterns {
                if pattern.is_match(text) {
                    return Some(language.clone());
                }
            }
        }
        None
    }

    fn analyze_syntax_elements(&self, code: &str, language: &Option<String>) -> Vec<SyntaxElement> {
        let mut elements = Vec::new();

        if let Some(lang) = language {
            if let Some(patterns) = self.syntax_patterns.get(lang) {
                for pattern in patterns {
                    for mat in pattern.find_iter(code) {
                        elements.push(SyntaxElement {
                            element_type: SyntaxElementType::Keyword,
                            value: mat.as_str().to_string(),
                            position: BoundingBox { x: 0, y: 0, width: 0, height: 0 }, // Would need more sophisticated positioning
                        });
                    }
                }
            }
        }

        elements
    }

    fn analyze_ui_elements(&self, blocks: &[&TextBlock]) -> Result<Vec<UIElement>> {
        let mut ui_elements = Vec::new();

        for block in blocks {
            let element_type = self.classify_ui_element(&block.text);
            let is_interactive = self.is_interactive_element(&block.text, &element_type);

            ui_elements.push(UIElement {
                element_type,
                text: block.text.clone(),
                bounding_box: block.bounding_box.clone(),
                is_interactive,
                context: None,
            });
        }

        Ok(ui_elements)
    }

    fn classify_ui_element(&self, text: &str) -> UIElementType {
        let text_lower = text.to_lowercase();
        
        // Button patterns
        if text_lower.contains("click") || text_lower.contains("submit") || 
           text_lower.contains("save") || text_lower.contains("cancel") {
            return UIElementType::Button;
        }

        // Link patterns
        if text_lower.starts_with("http") || text_lower.contains("www.") {
            return UIElementType::Link;
        }

        // Default to label
        UIElementType::Label
    }

    fn is_interactive_element(&self, _text: &str, element_type: &UIElementType) -> bool {
        matches!(element_type, UIElementType::Button | UIElementType::TextField | UIElementType::Link | UIElementType::Tab)
    }

    fn analyze_chat_messages(&self, blocks: &[&TextBlock]) -> Result<Vec<ChatMessage>> {
        let mut messages = Vec::new();

        for block in blocks {
            let (sender, content) = self.parse_chat_message(&block.text);
            let message_type = self.classify_message_type(&content);

            messages.push(ChatMessage {
                sender,
                timestamp: None, // Would need better parsing
                content,
                message_type,
                bounding_box: block.bounding_box.clone(),
            });
        }

        Ok(messages)
    }

    fn parse_chat_message(&self, text: &str) -> (Option<String>, String) {
        if let Some(colon_pos) = text.find(':') {
            let potential_sender = &text[..colon_pos].trim();
            if potential_sender.len() < 50 && !potential_sender.contains(' ') {
                return (Some(potential_sender.to_string()), text[colon_pos + 1..].trim().to_string());
            }
        }
        (None, text.to_string())
    }

    fn classify_message_type(&self, content: &str) -> MessageType {
        if content.to_lowercase().contains("bot") || content.to_lowercase().contains("assistant") {
            MessageType::BotResponse
        } else {
            MessageType::UserMessage
        }
    }

    fn analyze_document_structure(&self, _blocks: &[&TextBlock]) -> Result<Option<DocumentStructure>> {
        // Placeholder for document structure analysis
        Ok(None)
    }

    fn analyze_browser_content(&self, _blocks: &[&TextBlock], _all_blocks: &HashMap<TextType, Vec<&TextBlock>>) -> Result<Option<BrowserContent>> {
        // Placeholder for browser content analysis
        Ok(None)
    }

    fn analyze_ide_context(&self, _blocks: &[&TextBlock], _all_blocks: &HashMap<TextType, Vec<&TextBlock>>) -> Result<Option<IDEContext>> {
        // Placeholder for IDE context analysis
        Ok(None)
    }

    fn analyze_meeting_context(&self, _blocks: &[&TextBlock], _all_blocks: &HashMap<TextType, Vec<&TextBlock>>) -> Result<Option<MeetingContext>> {
        // Placeholder for meeting context analysis
        Ok(None)
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self { x: 0, y: 0, width: 0, height: 0 }
    }
}