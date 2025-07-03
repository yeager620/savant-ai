use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::real_time_analyzer::{DetectedTask, TaskType};
use savant_ocr::{ComprehensiveOCRResult, TextType};
use savant_vision::ScreenAnalysis;

#[derive(Debug, Clone)]
pub struct CodingProblemDetector {
    pub detection_config: DetectionConfig,
    pub pattern_matchers: Vec<ProblemPatternMatcher>,
    pub context_buffer: ContextBuffer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub enable_error_detection: bool,
    pub enable_algorithm_detection: bool,
    pub enable_test_failure_detection: bool,
    pub enable_hackerrank_detection: bool,
    pub enable_leetcode_detection: bool,
    pub min_confidence_threshold: f32,
    pub context_lines_before: usize,
    pub context_lines_after: usize,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            enable_error_detection: true,
            enable_algorithm_detection: true,
            enable_test_failure_detection: true,
            enable_hackerrank_detection: true,
            enable_leetcode_detection: true,
            min_confidence_threshold: 0.7,
            context_lines_before: 10,
            context_lines_after: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedCodingProblem {
    pub id: String,
    pub problem_type: CodingProblemType,
    pub title: String,
    pub description: String,
    pub code_context: CodeContext,
    pub error_details: Option<ErrorDetails>,
    pub platform: Option<CodingPlatform>,
    pub language: ProgrammingLanguage,
    pub starter_code: Option<String>,
    pub test_cases: Vec<TestCase>,
    pub constraints: Vec<String>,
    pub confidence: f32,
    pub detected_at: DateTime<Utc>,
    pub screen_region: ScreenRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodingProblemType {
    CompilationError,
    RuntimeError,
    TestFailure,
    AlgorithmChallenge,
    DataStructureChallenge,
    SystemDesignProblem,
    DebugChallenge,
    OptimizationProblem,
    SyntaxError,
    LogicError,
    PerformanceIssue,
}

impl CodingProblemType {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::CompilationError => "Compilation Error",
            Self::RuntimeError => "Runtime Error",
            Self::TestFailure => "Test Failure",
            Self::AlgorithmChallenge => "Algorithm Challenge",
            Self::DataStructureChallenge => "Data Structure Challenge",
            Self::SystemDesignProblem => "System Design Problem",
            Self::DebugChallenge => "Debug Challenge",
            Self::OptimizationProblem => "Optimization Problem",
            Self::SyntaxError => "Syntax Error",
            Self::LogicError => "Logic Error",
            Self::PerformanceIssue => "Performance Issue",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CodingPlatform {
    HackerRank,
    LeetCode,
    CodeSignal,
    TopCoder,
    Codeforces,
    ProjectEuler,
    LocalIDE,
    Terminal,
    JupyterNotebook,
    OnlineCompiler,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Python,
    JavaScript,
    TypeScript,
    Java,
    Cpp,
    C,
    Rust,
    Go,
    Swift,
    Kotlin,
    Ruby,
    PHP,
    CSharp,
    Scala,
    Haskell,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub visible_code: String,
    pub focused_function: Option<String>,
    pub imports: Vec<String>,
    pub class_context: Option<String>,
    pub line_numbers: Option<(usize, usize)>,
    pub cursor_position: Option<(usize, usize)>,
    pub selected_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub error_line: Option<usize>,
    pub error_column: Option<usize>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub expected_output: String,
    pub actual_output: Option<String>,
    pub passed: Option<bool>,
    pub execution_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemPatternMatcher {
    pub pattern_type: PatternType,
    pub keywords: Vec<String>,
    pub regex_pattern_strings: Vec<String>,
    pub confidence_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ErrorPattern,
    ProblemStatement,
    TestCase,
    Constraints,
    StarterCode,
    PlatformSpecific,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBuffer {
    pub recent_screens: Vec<ScreenContext>,
    pub max_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenContext {
    pub timestamp: DateTime<Utc>,
    pub ocr_result: ComprehensiveOCRResult,
    pub vision_analysis: ScreenAnalysis,
}

impl CodingProblemDetector {
    pub fn new(config: DetectionConfig) -> Self {
        Self {
            detection_config: config,
            pattern_matchers: Self::initialize_pattern_matchers(),
            context_buffer: ContextBuffer {
                recent_screens: Vec::new(),
                max_buffer_size: 10,
            },
        }
    }

    fn initialize_pattern_matchers() -> Vec<ProblemPatternMatcher> {
        vec![
            // Error patterns
            ProblemPatternMatcher {
                pattern_type: PatternType::ErrorPattern,
                keywords: vec![
                    "error".to_string(),
                    "exception".to_string(),
                    "failed".to_string(),
                    "compilation error".to_string(),
                    "runtime error".to_string(),
                    "syntax error".to_string(),
                ],
                regex_pattern_strings: vec![
                    r"(?i)(error|exception):\s*(.+)".to_string(),
                    r"(?i)line\s*(\d+).*error".to_string(),
                    r"(?i)traceback.*most recent call".to_string(),
                ],
                confidence_boost: 0.3,
            },
            // HackerRank patterns
            ProblemPatternMatcher {
                pattern_type: PatternType::PlatformSpecific,
                keywords: vec![
                    "hackerrank".to_string(),
                    "problem statement".to_string(),
                    "sample input".to_string(),
                    "sample output".to_string(),
                    "constraints".to_string(),
                ],
                regex_pattern_strings: vec![
                    r"(?i)sample\s+(input|output)\s*\d*".to_string(),
                    r"(?i)constraint[s]?:".to_string(),
                    r"(?i)input\s*format:".to_string(),
                ],
                confidence_boost: 0.4,
            },
            // LeetCode patterns
            ProblemPatternMatcher {
                pattern_type: PatternType::PlatformSpecific,
                keywords: vec![
                    "leetcode".to_string(),
                    "example".to_string(),
                    "input:".to_string(),
                    "output:".to_string(),
                    "explanation:".to_string(),
                ],
                regex_pattern_strings: vec![
                    r"(?i)example\s*\d*:".to_string(),
                    r"(?i)input:\s*(.+)".to_string(),
                    r"(?i)output:\s*(.+)".to_string(),
                ],
                confidence_boost: 0.4,
            },
        ]
    }

    pub async fn detect_problems(
        &mut self,
        ocr_result: &ComprehensiveOCRResult,
        vision_analysis: &ScreenAnalysis,
    ) -> Result<Vec<DetectedCodingProblem>> {
        // Update context buffer
        self.update_context_buffer(ocr_result.clone(), vision_analysis.clone());

        let mut detected_problems = Vec::new();

        // Detect compilation/runtime errors
        if self.detection_config.enable_error_detection {
            if let Some(problem) = self.detect_error_problem(ocr_result, vision_analysis).await? {
                detected_problems.push(problem);
            }
        }

        // Detect algorithm challenges (HackerRank, LeetCode, etc.)
        if self.detection_config.enable_algorithm_detection {
            if let Some(problem) = self.detect_algorithm_challenge(ocr_result, vision_analysis).await? {
                detected_problems.push(problem);
            }
        }

        // Detect test failures
        if self.detection_config.enable_test_failure_detection {
            if let Some(problem) = self.detect_test_failure(ocr_result, vision_analysis).await? {
                detected_problems.push(problem);
            }
        }

        Ok(detected_problems)
    }

    async fn detect_error_problem(
        &self,
        ocr_result: &ComprehensiveOCRResult,
        vision_analysis: &ScreenAnalysis,
    ) -> Result<Option<DetectedCodingProblem>> {
        let error_patterns = &self.pattern_matchers[0]; // Error pattern matcher
        
        // Check for error keywords and patterns
        let mut error_confidence = 0.0;
        let mut error_details = None;
        let mut error_context = String::new();

        for paragraph in &ocr_result.paragraphs {
            let text = paragraph.text.to_lowercase();
            
            // Check keywords
            for keyword in &error_patterns.keywords {
                if text.contains(keyword) {
                    error_confidence += 0.2;
                }
            }

            // Check regex patterns
            for pattern_str in &error_patterns.regex_pattern_strings {
                if let Ok(pattern) = regex::Regex::new(pattern_str) {
                    if pattern.is_match(&paragraph.text) {
                        error_confidence += 0.3;
                        
                        // Extract error details
                        if let Some(captures) = pattern.captures(&paragraph.text) {
                            if error_details.is_none() {
                                error_details = Some(ErrorDetails {
                                    error_type: "CompilationError".to_string(),
                                error_message: captures.get(1)
                                    .map(|m| m.as_str().to_string())
                                    .unwrap_or_default(),
                                stack_trace: None,
                                error_line: None,
                                error_column: None,
                                suggestions: vec![],
                            });
                        }
                    }
                }
            }
            }

            // Collect context
            if error_confidence > 0.0 {
                error_context.push_str(&paragraph.text);
                error_context.push('\n');
            }
        }

        if error_confidence >= self.detection_config.min_confidence_threshold {
            let code_context = self.extract_code_context(ocr_result)?;
            let language = self.detect_programming_language(&code_context.visible_code);

            Ok(Some(DetectedCodingProblem {
                id: uuid::Uuid::new_v4().to_string(),
                problem_type: CodingProblemType::CompilationError,
                title: "Compilation Error Detected".to_string(),
                description: error_context.trim().to_string(),
                code_context,
                error_details,
                platform: self.detect_platform(vision_analysis),
                language,
                starter_code: None,
                test_cases: vec![],
                constraints: vec![],
                confidence: error_confidence.min(1.0),
                detected_at: Utc::now(),
                screen_region: self.calculate_problem_region(ocr_result),
            }))
        } else {
            Ok(None)
        }
    }

    async fn detect_algorithm_challenge(
        &self,
        ocr_result: &ComprehensiveOCRResult,
        vision_analysis: &ScreenAnalysis,
    ) -> Result<Option<DetectedCodingProblem>> {
        let mut platform_confidence = HashMap::new();
        let mut problem_elements = ProblemElements::default();

        // Check for platform-specific patterns
        for matcher in &self.pattern_matchers[1..] {
            if matches!(matcher.pattern_type, PatternType::PlatformSpecific) {
                let mut confidence = 0.0;
                
                for paragraph in &ocr_result.paragraphs {
                    let text = paragraph.text.to_lowercase();
                    
                    // Check keywords
                    for keyword in &matcher.keywords {
                        if text.contains(keyword) {
                            confidence += 0.1;
                        }
                    }

                    // Extract problem elements
                    if text.contains("problem statement") || text.contains("description") {
                        problem_elements.description.push_str(&paragraph.text);
                        problem_elements.description.push('\n');
                    }

                    if text.contains("sample input") || text.contains("example") {
                        problem_elements.sample_inputs.push(paragraph.text.clone());
                    }

                    if text.contains("sample output") || text.contains("expected") {
                        problem_elements.sample_outputs.push(paragraph.text.clone());
                    }

                    if text.contains("constraint") {
                        problem_elements.constraints.push(paragraph.text.clone());
                    }
                }

                if confidence > 0.0 {
                    let platform = self.platform_from_matcher(matcher);
                    platform_confidence.insert(platform, confidence);
                }
            }
        }

        // Find the most likely platform
        let best_platform = platform_confidence.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(platform, confidence)| (platform.clone(), *confidence));

        if let Some((platform, confidence)) = best_platform {
            if confidence >= self.detection_config.min_confidence_threshold {
                let code_context = self.extract_code_context(ocr_result)?;
                let language = self.detect_programming_language(&code_context.visible_code);
                let test_cases = self.extract_test_cases(&problem_elements);

                Ok(Some(DetectedCodingProblem {
                    id: uuid::Uuid::new_v4().to_string(),
                    problem_type: CodingProblemType::AlgorithmChallenge,
                    title: self.extract_problem_title(ocr_result),
                    description: problem_elements.description.trim().to_string(),
                    code_context,
                    error_details: None,
                    platform: Some(platform),
                    language,
                    starter_code: self.extract_starter_code(ocr_result),
                    test_cases,
                    constraints: problem_elements.constraints,
                    confidence: confidence.min(1.0),
                    detected_at: Utc::now(),
                    screen_region: self.calculate_problem_region(ocr_result),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn detect_test_failure(
        &self,
        ocr_result: &ComprehensiveOCRResult,
        vision_analysis: &ScreenAnalysis,
    ) -> Result<Option<DetectedCodingProblem>> {
        // Look for test failure patterns
        let test_patterns = vec![
            "test failed",
            "assertion error",
            "expected:",
            "actual:",
            "tests passed:",
            "tests failed:",
        ];

        let mut test_confidence = 0.0;
        let mut failed_tests = Vec::new();

        for paragraph in &ocr_result.paragraphs {
            let text = paragraph.text.to_lowercase();
            
            for pattern in &test_patterns {
                if text.contains(pattern) {
                    test_confidence += 0.2;
                    failed_tests.push(paragraph.text.clone());
                }
            }
        }

        if test_confidence >= self.detection_config.min_confidence_threshold {
            let code_context = self.extract_code_context(ocr_result)?;
            let language = self.detect_programming_language(&code_context.visible_code);

            Ok(Some(DetectedCodingProblem {
                id: uuid::Uuid::new_v4().to_string(),
                problem_type: CodingProblemType::TestFailure,
                title: "Test Failure Detected".to_string(),
                description: failed_tests.join("\n"),
                code_context,
                error_details: None,
                platform: self.detect_platform(vision_analysis),
                language,
                starter_code: None,
                test_cases: self.extract_failed_test_cases(&failed_tests),
                constraints: vec![],
                confidence: test_confidence.min(1.0),
                detected_at: Utc::now(),
                screen_region: self.calculate_problem_region(ocr_result),
            }))
        } else {
            Ok(None)
        }
    }

    fn extract_code_context(&self, ocr_result: &ComprehensiveOCRResult) -> Result<CodeContext> {
        let mut visible_code = String::new();
        let mut imports = Vec::new();
        let mut line_numbers = None;

        for paragraph in &ocr_result.paragraphs {
            // Check if this looks like code
            if paragraph.text_type == Some(TextType::CodeSnippet) || 
               self.looks_like_code(&paragraph.text) {
                visible_code.push_str(&paragraph.text);
                visible_code.push('\n');

                // Extract imports
                if paragraph.text.contains("import") || paragraph.text.contains("require") {
                    imports.push(paragraph.text.trim().to_string());
                }

                // Try to extract line numbers
                if line_numbers.is_none() {
                    if let Some((start, end)) = self.extract_line_numbers(&paragraph.text) {
                        line_numbers = Some((start, end));
                    }
                }
            }
        }

        Ok(CodeContext {
            visible_code: visible_code.trim().to_string(),
            focused_function: self.extract_focused_function(&visible_code),
            imports,
            class_context: self.extract_class_context(&visible_code),
            line_numbers,
            cursor_position: None,
            selected_text: None,
        })
    }

    fn detect_programming_language(&self, code: &str) -> ProgrammingLanguage {
        // Simple heuristics for language detection
        if code.contains("def ") && code.contains("import") {
            ProgrammingLanguage::Python
        } else if code.contains("function") || code.contains("const ") || code.contains("let ") {
            ProgrammingLanguage::JavaScript
        } else if code.contains("public class") || code.contains("import java") {
            ProgrammingLanguage::Java
        } else if code.contains("fn ") && code.contains("let mut") {
            ProgrammingLanguage::Rust
        } else if code.contains("#include") && code.contains("std::") {
            ProgrammingLanguage::Cpp
        } else if code.contains("package main") || code.contains("func ") {
            ProgrammingLanguage::Go
        } else {
            ProgrammingLanguage::Unknown
        }
    }

    fn detect_platform(&self, vision_analysis: &ScreenAnalysis) -> Option<CodingPlatform> {
        // Check detected applications
        for app in &vision_analysis.detected_applications {
            match app.name.to_lowercase().as_str() {
                name if name.contains("hackerrank") => return Some(CodingPlatform::HackerRank),
                name if name.contains("leetcode") => return Some(CodingPlatform::LeetCode),
                name if name.contains("codesignal") => return Some(CodingPlatform::CodeSignal),
                name if name.contains("vscode") || name.contains("visual studio") => {
                    return Some(CodingPlatform::LocalIDE)
                },
                name if name.contains("terminal") || name.contains("iterm") => {
                    return Some(CodingPlatform::Terminal)
                },
                name if name.contains("jupyter") => return Some(CodingPlatform::JupyterNotebook),
                _ => {}
            }
        }

        None
    }

    fn extract_problem_title(&self, ocr_result: &ComprehensiveOCRResult) -> String {
        // Look for title patterns
        for paragraph in &ocr_result.paragraphs {
            // Check if this looks like a title (larger font, at top of screen)
            if paragraph.bounding_box.y < 200 && paragraph.words.len() < 10 {
                let text = paragraph.text.trim();
                if !text.is_empty() && text.len() < 100 {
                    return text.to_string();
                }
            }
        }

        "Untitled Problem".to_string()
    }

    fn extract_starter_code(&self, ocr_result: &ComprehensiveOCRResult) -> Option<String> {
        // Look for code editor regions or starter code patterns
        for paragraph in &ocr_result.paragraphs {
            if paragraph.text_type == Some(TextType::CodeSnippet) {
                let text = paragraph.text.trim();
                if text.contains("class Solution") || 
                   text.contains("def solution") ||
                   text.contains("function solution") {
                    return Some(text.to_string());
                }
            }
        }

        None
    }

    fn extract_test_cases(&self, elements: &ProblemElements) -> Vec<TestCase> {
        let mut test_cases = Vec::new();

        for (i, input) in elements.sample_inputs.iter().enumerate() {
            let output = elements.sample_outputs.get(i)
                .map(|s| s.clone())
                .unwrap_or_default();

            test_cases.push(TestCase {
                input: input.clone(),
                expected_output: output,
                actual_output: None,
                passed: None,
                execution_time: None,
            });
        }

        test_cases
    }

    fn extract_failed_test_cases(&self, failed_tests: &[String]) -> Vec<TestCase> {
        let mut test_cases = Vec::new();

        for test_output in failed_tests {
            // Parse test output for expected vs actual
            if test_output.contains("expected:") && test_output.contains("actual:") {
                let parts: Vec<&str> = test_output.split('\n').collect();
                let mut test_case = TestCase {
                    input: String::new(),
                    expected_output: String::new(),
                    actual_output: None,
                    passed: Some(false),
                    execution_time: None,
                };

                for part in parts {
                    if part.contains("expected:") {
                        test_case.expected_output = part.replace("expected:", "").trim().to_string();
                    } else if part.contains("actual:") {
                        test_case.actual_output = Some(part.replace("actual:", "").trim().to_string());
                    }
                }

                test_cases.push(test_case);
            }
        }

        test_cases
    }

    fn looks_like_code(&self, text: &str) -> bool {
        // Simple heuristics to identify code
        let code_indicators = vec![
            "function", "def", "class", "import", "const", "let", "var",
            "public", "private", "return", "if", "else", "for", "while",
            "{", "}", "(", ")", ";", "=>", "->", "::",
        ];

        let mut indicator_count = 0;
        for indicator in code_indicators {
            if text.contains(indicator) {
                indicator_count += 1;
            }
        }

        indicator_count >= 2
    }

    fn extract_line_numbers(&self, text: &str) -> Option<(usize, usize)> {
        // Look for line number patterns
        let line_num_regex = regex::Regex::new(r"^\s*(\d+)\s*[|:]").unwrap();
        
        let mut min_line = usize::MAX;
        let mut max_line = 0;

        for line in text.lines() {
            if let Some(captures) = line_num_regex.captures(line) {
                if let Ok(line_num) = captures[1].parse::<usize>() {
                    min_line = min_line.min(line_num);
                    max_line = max_line.max(line_num);
                }
            }
        }

        if min_line != usize::MAX {
            Some((min_line, max_line))
        } else {
            None
        }
    }

    fn extract_focused_function(&self, code: &str) -> Option<String> {
        // Look for function definitions
        let function_patterns = vec![
            regex::Regex::new(r"def\s+(\w+)\s*\(").unwrap(),
            regex::Regex::new(r"function\s+(\w+)\s*\(").unwrap(),
            regex::Regex::new(r"fn\s+(\w+)\s*\(").unwrap(),
            regex::Regex::new(r"public\s+\w+\s+(\w+)\s*\(").unwrap(),
        ];

        for pattern in function_patterns {
            if let Some(captures) = pattern.captures(code) {
                return Some(captures[1].to_string());
            }
        }

        None
    }

    fn extract_class_context(&self, code: &str) -> Option<String> {
        // Look for class definitions
        let class_patterns = vec![
            regex::Regex::new(r"class\s+(\w+)").unwrap(),
            regex::Regex::new(r"public\s+class\s+(\w+)").unwrap(),
            regex::Regex::new(r"struct\s+(\w+)").unwrap(),
        ];

        for pattern in class_patterns {
            if let Some(captures) = pattern.captures(code) {
                return Some(captures[1].to_string());
            }
        }

        None
    }

    fn calculate_problem_region(&self, ocr_result: &ComprehensiveOCRResult) -> ScreenRegion {
        // Calculate bounding box around all text
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = 0;
        let mut max_y = 0;

        for paragraph in &ocr_result.paragraphs {
            min_x = min_x.min(paragraph.bounding_box.x);
            min_y = min_y.min(paragraph.bounding_box.y);
            max_x = max_x.max(paragraph.bounding_box.x + paragraph.bounding_box.width);
            max_y = max_y.max(paragraph.bounding_box.y + paragraph.bounding_box.height);
        }

        ScreenRegion {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    fn platform_from_matcher(&self, matcher: &ProblemPatternMatcher) -> CodingPlatform {
        // Determine platform based on keywords
        if matcher.keywords.iter().any(|k| k.contains("hackerrank")) {
            CodingPlatform::HackerRank
        } else if matcher.keywords.iter().any(|k| k.contains("leetcode")) {
            CodingPlatform::LeetCode
        } else {
            CodingPlatform::Unknown
        }
    }

    fn update_context_buffer(&mut self, ocr_result: ComprehensiveOCRResult, vision_analysis: ScreenAnalysis) {
        let context = ScreenContext {
            timestamp: Utc::now(),
            ocr_result,
            vision_analysis,
        };

        self.context_buffer.recent_screens.push(context);

        // Keep buffer size limited
        if self.context_buffer.recent_screens.len() > self.context_buffer.max_buffer_size {
            self.context_buffer.recent_screens.remove(0);
        }
    }
}

#[derive(Default)]
struct ProblemElements {
    description: String,
    sample_inputs: Vec<String>,
    sample_outputs: Vec<String>,
    constraints: Vec<String>,
}

// Re-export for convenience
pub use self::{
    CodingProblemType::*,
    CodingPlatform::*,
    ProgrammingLanguage::*,
};