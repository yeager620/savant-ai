//! WASM-compatible types for the Leptos frontend
//! These mirror the types from savant-video but can compile to WASM

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Java,
    Cpp,
    C,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub visible_code: String,
    pub focused_function: Option<String>,
    pub imports: Vec<String>,
    pub class_context: Option<String>,
    pub line_numbers: Option<(u32, u32)>,
    pub cursor_position: Option<(u32, u32)>,
    pub selected_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodingProblemType {
    AlgorithmChallenge,
    DataStructure,
    SystemDesign,
    Debugging,
    CodeReview,
    Optimization,
    Testing,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedCodingProblem {
    pub id: String,
    pub problem_type: CodingProblemType,
    pub title: String,
    pub description: String,
    pub code_context: CodeContext,
    pub error_details: Option<String>,
    pub platform: Option<String>,
    pub language: ProgrammingLanguage,
    pub starter_code: Option<String>,
    pub test_cases: Vec<String>,
    pub constraints: Vec<String>,
    pub confidence: f32,
    pub detected_at: DateTime<Utc>,
    pub screen_region: ScreenRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSolution {
    pub problem_id: String,
    pub solution_code: String,
    pub explanation: Option<String>,
    pub approach: String,
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub test_results: Vec<TestResult>,
    pub confidence_score: f32,
    pub generated_at: DateTime<Utc>,
    pub llm_model: String,
    pub processing_time_ms: u64,
}