use savant_video::coding_problem_detector::*;
use savant_ocr::{ComprehensiveOCRResult, WordData, LineData, ParagraphData, BoundingBox as OcrBoundingBox, TextType};
use savant_vision::{
    ScreenAnalysis, DetectedApp, ActivityClassification, VisualContext, AppContext, AppType, 
    IDEType, WindowState, Activity, Evidence, ThemeInfo, ImageMetadata, BoundingBox as VisionBoundingBox
};
use chrono::Utc;

fn create_test_ocr_result(text_content: &str) -> ComprehensiveOCRResult {
    let words: Vec<WordData> = text_content
        .split_whitespace()
        .enumerate()
        .map(|(i, word)| WordData {
            text: word.to_string(),
            confidence: 0.95,
            bounding_box: OcrBoundingBox {
                x: (i * 50) as u32,
                y: 100,
                width: 40,
                height: 20,
            },
            font_size_estimate: Some(12.0),
            text_type: None,
            line_id: 0,
            paragraph_id: 0,
        })
        .collect();

    let paragraph = ParagraphData {
        text: text_content.to_string(),
        bounding_box: BoundingBox {
            x: 0,
            y: 100,
            width: 800,
            height: 100,
        },
        confidence: 0.95,
        line_count: 1,
    };

    ComprehensiveOCRResult {
        raw_text: text_content.to_string(),
        words: words.clone(),
        lines: vec![LineData {
            text: text_content.to_string(),
            bounding_box: paragraph.bounding_box.clone(),
            confidence: 0.95,
            word_count: words.len(),
        }],
        paragraphs: vec![paragraph],
        screen_regions: vec![],
        confidence: 0.95,
        processing_time_ms: 100,
    }
}

fn create_test_vision_analysis() -> ScreenAnalysis {
    ScreenAnalysis {
        timestamp: Utc::now(),
        visual_elements: vec![],
        app_context: AppContext {
            detected_applications: vec![DetectedApp {
                app_type: AppType::IDE(IDEType::VSCode),
                app_name: Some("Visual Studio Code".to_string()),
                confidence: 0.9,
                visual_indicators: vec![],
                screen_region: BoundingBox {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080
                },
                window_state: WindowState::Active,
            }],
            active_windows: vec![],
            browser_context: None,
            ide_context: None,
            meeting_context: None,
            desktop_environment: None,
        },
        activity_classification: ActivityClassification {
            primary_activity: Activity::Coding,
            secondary_activities: vec![],
            context_indicators: vec![Evidence {
                description: "IDE detected".to_string(),
                confidence: 0.85,
                source: "visual".to_string(),
            }],
        },
        visual_context: VisualContext {
            layout_analysis: "Standard IDE layout".to_string(),
            attention_areas: vec![],
            interaction_elements: vec![],
            content_regions: vec![],
            theme_info: ThemeInfo {
                is_dark_mode: true,
                accent_color: None,
                wallpaper_type: None,
            },
        },
        processing_time_ms: 50,
        image_metadata: ImageMetadata {
            width: 1920,
            height: 1080,
            format: "png".to_string(),
            file_size: None,
        },
    }
}

#[tokio::test]
async fn test_detect_compilation_error() {
    let mut detector = CodingProblemDetector::new(DetectionConfig::default());

    let error_text = "error: expected `;`, found `}` 
        --> src/main.rs:10:5
        |
        10 |     println!(\"Hello, world!\")
        |                               ^ expected `;`
        |
        = help: add `;` here";

    let ocr_result = create_test_ocr_result(error_text);
    let vision_analysis = create_test_vision_analysis();

    let problems = detector.detect_problems(&ocr_result, &vision_analysis).await.unwrap();

    assert_eq!(problems.len(), 1);
    assert!(matches!(problems[0].problem_type, CodingProblemType::CompilationError));
    assert!(problems[0].description.contains("expected `;`"));
    assert!(problems[0].confidence >= 0.7);
}

#[tokio::test]
async fn test_detect_runtime_error() {
    let mut detector = CodingProblemDetector::new(DetectionConfig::default());

    let error_text = "Traceback (most recent call last):
        File \"main.py\", line 42, in <module>
            result = divide(10, 0)
        File \"main.py\", line 5, in divide
            return a / b
        ZeroDivisionError: division by zero";

    let ocr_result = create_test_ocr_result(error_text);
    let vision_analysis = create_test_vision_analysis();

    let problems = detector.detect_problems(&ocr_result, &vision_analysis).await.unwrap();

    assert_eq!(problems.len(), 1);
    assert!(matches!(problems[0].problem_type, CodingProblemType::RuntimeError | CodingProblemType::CompilationError));
    assert!(problems[0].description.contains("ZeroDivisionError"));
    assert_eq!(problems[0].language, ProgrammingLanguage::Python);
}

#[tokio::test]
async fn test_detect_hackerrank_challenge() {
    let mut detector = CodingProblemDetector::new(DetectionConfig::default());

    let challenge_text = "HackerRank Problem Statement

        Given an array of integers, find the sum of its elements.

        Sample Input
        6
        1 2 3 4 10 11

        Sample Output
        31

        Constraints:
        - 1 <= n <= 1000
        - 0 <= arr[i] <= 1000";

    let mut ocr_result = create_test_ocr_result(challenge_text);
    // Add HackerRank-specific visual cues
    ocr_result.paragraphs[0].bounding_box.y = 50; // Near top of screen

    let mut vision_analysis = create_test_vision_analysis();
    vision_analysis.detected_applications[0].name = "Chrome - HackerRank".to_string();

    let problems = detector.detect_problems(&ocr_result, &vision_analysis).await.unwrap();

    assert_eq!(problems.len(), 1);
    assert!(matches!(problems[0].problem_type, CodingProblemType::AlgorithmChallenge));
    assert_eq!(problems[0].platform, Some(CodingPlatform::HackerRank));
    assert!(problems[0].description.contains("sum of its elements"));
    assert!(!problems[0].test_cases.is_empty());
    assert!(!problems[0].constraints.is_empty());
}

#[tokio::test]
async fn test_detect_leetcode_challenge() {
    let mut detector = CodingProblemDetector::new(DetectionConfig::default());

    let challenge_text = "1. Two Sum

        Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target.

        Example 1:
        Input: nums = [2,7,11,15], target = 9
        Output: [0,1]
        Explanation: Because nums[0] + nums[1] == 9, we return [0, 1].

        Example 2:
        Input: nums = [3,2,4], target = 6
        Output: [1,2]";

    let ocr_result = create_test_ocr_result(challenge_text);
    let mut vision_analysis = create_test_vision_analysis();
    vision_analysis.detected_applications[0].name = "Chrome - LeetCode".to_string();

    let problems = detector.detect_problems(&ocr_result, &vision_analysis).await.unwrap();

    assert_eq!(problems.len(), 1);
    assert!(matches!(problems[0].problem_type, CodingProblemType::AlgorithmChallenge));
    assert_eq!(problems[0].platform, Some(CodingPlatform::LeetCode));
    assert!(problems[0].title.contains("Two Sum"));
    assert_eq!(problems[0].test_cases.len(), 2);
}

#[tokio::test]
async fn test_detect_test_failure() {
    let mut detector = CodingProblemDetector::new(DetectionConfig::default());

    let test_output = "Running tests...

        test result: FAILED. 2 passed; 1 failed; 0 ignored

        ---- test_addition stdout ----
        thread 'test_addition' panicked at 'assertion failed: `(left == right)`
          left: `5`,
         right: `4`', src/lib.rs:10:5

        failures:
            test_addition";

    let ocr_result = create_test_ocr_result(test_output);
    let vision_analysis = create_test_vision_analysis();

    let problems = detector.detect_problems(&ocr_result, &vision_analysis).await.unwrap();

    assert_eq!(problems.len(), 1);
    assert!(matches!(problems[0].problem_type, CodingProblemType::TestFailure));
    assert!(problems[0].description.contains("assertion failed"));
    assert!(problems[0].description.contains("left: `5`"));
}

#[tokio::test]
async fn test_programming_language_detection() {
    let detector = CodingProblemDetector::new(DetectionConfig::default());

    // Test Python detection
    let python_code = "def factorial(n):
        if n == 0:
            return 1
        return n * factorial(n - 1)";
    let lang = detector.detect_programming_language(python_code);
    assert_eq!(lang, ProgrammingLanguage::Python);

    // Test JavaScript detection
    let js_code = "function fibonacci(n) {
        const arr = [0, 1];
        for (let i = 2; i <= n; i++) {
            arr[i] = arr[i - 1] + arr[i - 2];
        }
        return arr[n];
    }";
    let lang = detector.detect_programming_language(js_code);
    assert_eq!(lang, ProgrammingLanguage::JavaScript);

    // Test Java detection
    let java_code = "public class Solution {
        public static void main(String[] args) {
            System.out.println(\"Hello, World!\");
        }
    }";
    let lang = detector.detect_programming_language(java_code);
    assert_eq!(lang, ProgrammingLanguage::Java);

    // Test Rust detection
    let rust_code = "fn main() {
        let mut vec = Vec::new();
        vec.push(42);
        println!(\"{:?}\", vec);
    }";
    let lang = detector.detect_programming_language(rust_code);
    assert_eq!(lang, ProgrammingLanguage::Rust);
}

#[tokio::test]
async fn test_context_buffer_management() {
    let mut detector = CodingProblemDetector::new(DetectionConfig {
        buffer_size: 3,
        ..Default::default()
    });

    // Add multiple screens to buffer
    for i in 0..5 {
        let ocr_result = create_test_ocr_result(&format!("Screen {}", i));
        let vision_analysis = create_test_vision_analysis();
        detector.update_context_buffer(ocr_result, vision_analysis);
    }

    // Buffer should only keep last 3 screens
    assert_eq!(detector.context_buffer.recent_screens.len(), 3);
}

#[tokio::test]
async fn test_confidence_thresholds() {
    let mut config = DetectionConfig::default();
    config.min_confidence_threshold = 0.9; // High threshold

    let mut detector = CodingProblemDetector::new(config);

    // Low confidence error (should not be detected)
    let vague_error = "something went wrong maybe error occurred";
    let ocr_result = create_test_ocr_result(vague_error);
    let vision_analysis = create_test_vision_analysis();

    let problems = detector.detect_problems(&ocr_result, &vision_analysis).await.unwrap();
    assert_eq!(problems.len(), 0);
}

#[tokio::test]
async fn test_disabled_detection_types() {
    let mut config = DetectionConfig::default();
    config.enable_error_detection = false;
    config.enable_algorithm_detection = true;

    let mut detector = CodingProblemDetector::new(config);

    // Add both error and algorithm challenge text
    let mixed_content = "error: compilation failed

        HackerRank Problem Statement
        Find the maximum element in an array.

        Sample Input: 5 1 2 3 4 5
        Sample Output: 5";

    let ocr_result = create_test_ocr_result(mixed_content);
    let vision_analysis = create_test_vision_analysis();

    let problems = detector.detect_problems(&ocr_result, &vision_analysis).await.unwrap();

    // Should only detect algorithm challenge, not error
    assert_eq!(problems.len(), 1);
    assert!(matches!(problems[0].problem_type, CodingProblemType::AlgorithmChallenge));
}
