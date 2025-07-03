/*!
Mock End-to-End Coding Problem Detection Demo

This demonstrates the complete workflow with simulated data:
1. Simulated OCR extraction (Two Sum problem)
2. Simulated computer vision analysis  
3. Coding problem detection
4. LLM solution generation
5. Database storage

Usage: cargo run -p e2e-coding-detection --bin mock_demo
*/

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

// Import all the modules we need
use savant_ocr::{OCRResult, TextBlock, BoundingBox, TextType, ImageMetadata, StructuredContent};
use savant_vision::{ScreenAnalysis, AppContext, DetectedApp, AppType, IDEType, VisualElement, ElementType, BoundingBox as VisionBoundingBox, ElementProperties, ActivityClassification, Activity, VisualContext};
use savant_vision::classifier::{ContextIndicator, IndicatorType, IndicatorSource, Evidence, EvidenceType, SiteCategory};
use savant_vision::analyzer::{LayoutAnalysis, LayoutType, ContentArea, ContentType, AttentionArea, AttentionReason, InteractionElement, InteractionType, InteractionState, ContentRegion, ContentDensity, ThemeInfo};
use savant_video::{CodingProblemDetector, DetectionConfig, SolutionGenerator, SolutionConfig, DetectedCodingProblem, GeneratedSolution};
use savant_video::coding_problem_detector::{CodingProblemType, ProgrammingLanguage, ScreenRegion, CodeContext, CodingPlatform};
use savant_video::solution_generator::TestValidationResult;
use savant_video::llm_provider::LLMProvider;
use savant_db::{TranscriptDatabase, visual_data::VisualDataManager};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Mock End-to-End Coding Problem Detection Demo");
    println!("===============================================");
    println!("This demonstrates the complete Savant AI pipeline for real-time coding assistance\n");
    
    // Test configuration
    let test_session_id = "mock-demo-session";
    
    // Step 1: Simulated OCR Analysis
    println!("📖 Step 1: OCR Text Extraction (Simulated)");
    println!("-------------------------------------------");
    
    let ocr_result = create_mock_ocr_result();
    println!("📝 Extracted {} text elements from Two Sum problem page", ocr_result.text_blocks.len());
    
    // Display key text extractions
    for (i, block) in ocr_result.text_blocks.iter().take(10).enumerate() {
        println!("  {}. '{}' (confidence: {:.2})", 
            i + 1, block.text, block.confidence);
    }
    
    // Step 2: Simulated Computer Vision Analysis  
    println!("\n👁️  Step 2: Computer Vision Analysis (Simulated)");
    println!("-----------------------------------------------");
    
    let vision_result = create_mock_vision_analysis();
    println!("🖥️  Detected {} applications", vision_result.app_context.detected_applications.len());
    println!("📊 Activity classification: {:?}", vision_result.activity_classification);
    
    // Display detected applications
    for app in &vision_result.app_context.detected_applications {
        println!("  🎯 App: {:?} (confidence: {:.2})", app.app_type, app.confidence);
    }
    
    // Step 3: Coding Problem Detection
    println!("\n🧩 Step 3: Coding Problem Detection");
    println!("----------------------------------");
    
    let problem = detect_coding_problem(&ocr_result, &vision_result).await?;
    
    let detected_problem = if let Some(problem) = problem {
        println!("✅ Coding problem detected!");
        println!("   Type: {:?}", problem.problem_type);
        println!("   Title: {}", problem.title);
        println!("   Confidence: {:.2}%", problem.confidence * 100.0);
        println!("   Platform: {:?}", problem.platform);
        println!("   Language: {:?}", problem.language);
        problem
    } else {
        println!("❌ No coding problem detected");
        return Ok(());
    };
    
    // Step 4: LLM Solution Generation
    println!("\n🤖 Step 4: LLM Solution Generation");
    println!("---------------------------------");
    
    let solution = generate_solution(&detected_problem).await?;
    println!("✅ Solution generated!");
    println!("   Confidence: {:.2}%", solution.confidence_score * 100.0);
    println!("   Time complexity: {:?}", solution.time_complexity);
    println!("   Space complexity: {:?}", solution.space_complexity);
    
    // Display solution code (first 200 chars)
    let code_preview = if solution.solution_code.len() > 200 {
        format!("{}...", &solution.solution_code[..200])
    } else {
        solution.solution_code.clone()
    };
    println!("   Code preview:\n{}", code_preview);
    
    // Display explanation
    if let Some(explanation) = &solution.explanation {
        let explanation_preview = if explanation.len() > 150 {
            format!("{}...", &explanation[..150])
        } else {
            explanation.clone()
        };
        println!("   Explanation: {}", explanation_preview);
    }
    
    // Step 5: Database Storage
    println!("\n💾 Step 5: Database Storage");
    println!("-------------------------");
    
    store_detection_results(test_session_id, &ocr_result, &detected_problem, &solution).await?;
    println!("✅ Results stored in database");
    
    // Step 6: Performance Metrics
    println!("\n📊 Step 6: Performance Summary");
    println!("-----------------------------");
    
    let total_time = std::time::Duration::from_millis(850); // Mock timing
    println!("⏱️  Total processing time: {:?}", total_time);
    println!("🎯 Detection accuracy: {:.1}%", 96.5);
    println!("🚀 Real-time capable: {}", total_time.as_millis() < 2000);
    println!("💡 LLM model: {}", solution.model_used);
    println!("🧪 Test cases: {} passed", solution.test_results.len());
    
    // Step 7: Simulated UI Display
    println!("\n🖼️  Step 7: UI Display Simulation");
    println!("--------------------------------");
    
    simulate_ui_display(&detected_problem, &solution);
    
    println!("\n🎉 End-to-End Demo Completed Successfully!");
    println!("==========================================");
    println!("✨ The Savant AI system successfully:");
    println!("   • Detected a Two Sum coding problem from screen content");
    println!("   • Generated an optimized O(n) hash map solution"); 
    println!("   • Provided comprehensive explanation and complexity analysis");
    println!("   • Stored all data for future learning and improvement");
    println!("   • Displayed results in real-time overlay for immediate assistance");
    
    Ok(())
}

fn create_mock_ocr_result() -> OCRResult {
    let text_blocks = vec![
        TextBlock {
            text: "LeetCode".to_string(),
            confidence: 0.98,
            bounding_box: BoundingBox { x: 50, y: 30, width: 100, height: 25 },
            font_info: None,
            semantic_type: TextType::BrowserUI,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "1. Two Sum".to_string(),
            confidence: 0.96,
            bounding_box: BoundingBox { x: 100, y: 120, width: 150, height: 30 },
            font_info: None,
            semantic_type: TextType::DocumentContent,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "Given an array of integers nums and an integer target".to_string(),
            confidence: 0.94,
            bounding_box: BoundingBox { x: 100, y: 180, width: 500, height: 20 },
            font_info: None,
            semantic_type: TextType::DocumentContent,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "return indices of the two numbers such that they add up to target".to_string(),
            confidence: 0.93,
            bounding_box: BoundingBox { x: 100, y: 205, width: 480, height: 20 },
            font_info: None,
            semantic_type: TextType::DocumentContent,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "Example 1:".to_string(),
            confidence: 0.97,
            bounding_box: BoundingBox { x: 100, y: 250, width: 80, height: 20 },
            font_info: None,
            semantic_type: TextType::DocumentContent,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "Input: nums = [2,7,11,15], target = 9".to_string(),
            confidence: 0.95,
            bounding_box: BoundingBox { x: 120, y: 275, width: 300, height: 18 },
            font_info: None,
            semantic_type: TextType::CodeSnippet,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "Output: [0,1]".to_string(),
            confidence: 0.96,
            bounding_box: BoundingBox { x: 120, y: 298, width: 120, height: 18 },
            font_info: None,
            semantic_type: TextType::CodeSnippet,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "def twoSum(self, nums, target):".to_string(),
            confidence: 0.89,
            bounding_box: BoundingBox { x: 150, y: 450, width: 250, height: 18 },
            font_info: None,
            semantic_type: TextType::CodeSnippet,
            language: Some("en".to_string()),
        },
        TextBlock {
            text: "# Write your solution here".to_string(),
            confidence: 0.87,
            bounding_box: BoundingBox { x: 170, y: 475, width: 200, height: 18 },
            font_info: None,
            semantic_type: TextType::CodeSnippet,
            language: Some("en".to_string()),
        },
    ];
    
    OCRResult {
        text_blocks,
        structured_content: StructuredContent::default(),
        overall_confidence: 0.94,
        processing_time_ms: 850,
        detected_language: "en".to_string(),
        image_metadata: ImageMetadata {
            width: 1920,
            height: 1080,
            format: "PNG".to_string(),
            file_size: Some(2048000),
            timestamp: Utc::now(),
        },
    }
}

fn create_mock_vision_analysis() -> ScreenAnalysis {
    let detected_app = DetectedApp {
        app_type: AppType::Browser(savant_vision::BrowserType::Chrome),
        app_name: Some("Google Chrome".to_string()),
        confidence: 0.92,
        visual_indicators: vec![],
        screen_region: VisionBoundingBox { x: 0, y: 0, width: 1920, height: 1080, confidence: 0.95 },
        window_state: savant_vision::WindowState::Focused,
    };
    
    let visual_elements = vec![
        VisualElement {
            element_type: ElementType::Browser,
            bounding_box: VisionBoundingBox { x: 0, y: 0, width: 1920, height: 1080, confidence: 0.95 },
            properties: ElementProperties {
                color_scheme: None,
                text_content: Some("LeetCode - Two Sum".to_string()),
                is_interactive: true,
                state: Some("active".to_string()),
                app_context: Some("browser".to_string()),
            },
            confidence: 0.92,
        },
        VisualElement {
            element_type: ElementType::Text,
            bounding_box: VisionBoundingBox { x: 100, y: 120, width: 500, height: 200, confidence: 0.88 },
            properties: ElementProperties {
                color_scheme: None,
                text_content: Some("coding problem description".to_string()),
                is_interactive: false,
                state: None,
                app_context: Some("leetcode".to_string()),
            },
            confidence: 0.89,
        },
    ];
    
    ScreenAnalysis {
        timestamp: Utc::now(),
        visual_elements,
        app_context: AppContext {
            detected_applications: vec![detected_app],
            active_windows: vec![],
            browser_context: Some(savant_vision::BrowserContext {
                browser_type: savant_vision::BrowserType::Chrome,
                visible_tabs: vec![
                    savant_vision::TabInfo {
                        title: "Two Sum - LeetCode".to_string(),
                        url: Some("https://leetcode.com/problems/two-sum/".to_string()),
                        is_active: true,
                        favicon: None,
                    }
                ],
                current_url: Some("https://leetcode.com/problems/two-sum/".to_string()),
                page_type: savant_vision::PageType::Development,
                navigation_elements: vec![],
            }),
            ide_context: None,
            meeting_context: None,
            desktop_environment: None,
        },
        activity_classification: ActivityClassification {
            primary_activity: Activity::Coding {
                language: Some("Python".to_string()),
                editor: "Chrome Browser".to_string(),
                project_type: Some("Algorithm Challenge".to_string()),
                debugging: false,
            },
            confidence: 0.94,
            secondary_activities: vec![Activity::WebBrowsing {
                site_category: savant_vision::classifier::SiteCategory::Development,
                primary_domain: "leetcode.com".to_string(),
                tab_count: Some(1),
            }],
            context_indicators: vec![
                ContextIndicator {
                    indicator_type: IndicatorType::TextPattern,
                    value: "algorithm challenge".to_string(),
                    confidence: 0.9,
                    source: IndicatorSource::OCR,
                },
                ContextIndicator {
                    indicator_type: IndicatorType::URLBar,
                    value: "coding platform".to_string(),
                    confidence: 0.85,
                    source: IndicatorSource::VisualAnalysis,
                },
            ],
            evidence: vec![
                Evidence {
                    evidence_type: EvidenceType::TextContent,
                    description: "Contains coding problem description and examples".to_string(),
                    confidence: 0.95,
                    weight: 0.8,
                },
                Evidence {
                    evidence_type: EvidenceType::VisualPattern,
                    description: "URL matches LeetCode problem page pattern".to_string(),
                    confidence: 0.98,
                    weight: 0.9,
                },
            ],
        },
        visual_context: VisualContext {
            dominant_colors: vec!["#ffffff".to_string(), "#f8f9fa".to_string(), "#333333".to_string()],
            layout_analysis: LayoutAnalysis {
                layout_type: LayoutType::SingleColumn,
                grid_structure: None,
                primary_content_area: Some(ContentArea {
                    x: 100,
                    y: 120,
                    width: 800,
                    height: 600,
                    content_type: ContentType::Text,
                }),
                sidebar_present: false,
                header_present: true,
                footer_present: false,
            },
            attention_areas: vec![
                AttentionArea {
                    region: ContentArea {
                        x: 100,
                        y: 120,
                        width: 500,
                        height: 200,
                        content_type: ContentType::Text,
                    },
                    attention_score: 0.8,
                    reason: AttentionReason::CenterPosition,
                },
            ],
            interaction_elements: vec![
                InteractionElement {
                    element_type: InteractionType::Button,
                    position: ContentArea {
                        x: 600,
                        y: 500,
                        width: 100,
                        height: 30,
                        content_type: ContentType::Unknown,
                    },
                    state: InteractionState::Normal,
                    accessibility_score: 0.8,
                },
            ],
            content_regions: vec![
                ContentRegion {
                    region: ContentArea {
                        x: 100,
                        y: 120,
                        width: 800,
                        height: 600,
                        content_type: ContentType::Text,
                    },
                    content_type: ContentType::Text,
                    density: ContentDensity::Medium,
                    scroll_position: None,
                },
            ],
            theme_info: ThemeInfo {
                is_dark_mode: false,
                primary_color: Some("#007bff".to_string()),
                secondary_color: Some("#6c757d".to_string()),
                accent_color: Some("#28a745".to_string()),
                background_color: "#ffffff".to_string(),
                text_color: "#333333".to_string(),
                contrast_ratio: 7.0,
            },
        },
        processing_time_ms: 180,
        image_metadata: savant_vision::ImageMetadata {
            width: 1920,
            height: 1080,
            format: "Screenshot".to_string(),
            file_size: Some(2048000),
        },
    }
}

async fn detect_coding_problem(ocr_result: &OCRResult, _vision_result: &ScreenAnalysis) -> Result<Option<DetectedCodingProblem>> {
    println!("  🕵️  Analyzing for coding problems...");
    
    let config = DetectionConfig::default();
    let _detector = CodingProblemDetector::new(config);
    
    // Look for coding problem indicators in the text
    let all_text = ocr_result.text_blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    
    println!("  📝 Analyzing extracted text: '{}'", 
        if all_text.len() > 100 { &all_text[..100] } else { &all_text });
    
    // Check for problem indicators
    let has_two_sum = all_text.to_lowercase().contains("two sum");
    let has_array = all_text.to_lowercase().contains("array");
    let has_target = all_text.to_lowercase().contains("target");
    let has_leetcode = all_text.to_lowercase().contains("leetcode");
    
    if has_two_sum || (has_array && has_target) {
        println!("  ✅ Two Sum problem detected!");
        
        let problem = DetectedCodingProblem {
            id: "twosum-detection-1".to_string(),
            problem_type: CodingProblemType::AlgorithmChallenge,
            title: "Two Sum".to_string(),
            description: "Find two numbers in an array that add up to a target sum".to_string(),
            code_context: CodeContext {
                visible_code: extract_code_from_ocr(ocr_result),
                focused_function: Some("twoSum".to_string()),
                imports: vec![],
                class_context: None,
                line_numbers: Some((1, 5)),
                cursor_position: Some((2, 8)),
                selected_text: None,
            },
            error_details: None,
            platform: if has_leetcode { Some(CodingPlatform::LeetCode) } else { None },
            language: ProgrammingLanguage::Python,
            starter_code: Some("def twoSum(self, nums, target):\n    # Write your solution here\n    pass".to_string()),
            test_cases: vec![
                savant_video::coding_problem_detector::TestCase {
                    input: "nums = [2,7,11,15], target = 9".to_string(),
                    expected_output: "[0,1]".to_string(),
                    actual_output: None,
                    passed: None,
                    execution_time: None,
                },
                savant_video::coding_problem_detector::TestCase {
                    input: "nums = [3,2,4], target = 6".to_string(),
                    expected_output: "[1,2]".to_string(),
                    actual_output: None,
                    passed: None,
                    execution_time: None,
                },
            ],
            constraints: vec![
                "2 ≤ nums.length ≤ 10^4".to_string(),
                "-10^9 ≤ nums[i] ≤ 10^9".to_string(),
                "-10^9 ≤ target ≤ 10^9".to_string(),
                "Only one valid answer exists".to_string(),
            ],
            confidence: if has_two_sum { 0.96 } else { 0.82 },
            detected_at: Utc::now(),
            screen_region: ScreenRegion {
                x: 100, y: 120, width: 800, height: 600
            },
        };
        
        Ok(Some(problem))
    } else {
        println!("  ❌ No recognizable coding problem detected");
        Ok(None)
    }
}

fn extract_code_from_ocr(ocr_result: &OCRResult) -> String {
    ocr_result.text_blocks
        .iter()
        .filter(|block| {
            matches!(block.semantic_type, TextType::CodeSnippet)
        })
        .map(|block| block.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

async fn generate_solution(problem: &DetectedCodingProblem) -> Result<GeneratedSolution> {
    println!("  🧠 Generating solution with LLM...");
    
    let config = SolutionConfig::default();
    let llm_provider = LLMProvider::new_ollama("http://localhost:11434".to_string(), Some("llama3.2".to_string()));
    let _generator = SolutionGenerator::new(config, llm_provider);
    
    // Create a comprehensive prompt for the Two Sum problem
    let prompt = format!(
        "Solve this coding problem: {}\n\nDescription: {}\n\nRequirements:\n- Provide a complete solution in Python\n- Include time and space complexity analysis\n- Add a clear explanation\n- Make it efficient and readable",
        problem.title, problem.description
    );
    
    println!("  💭 LLM prompt: {}", &prompt[..100]);
    
    // Generate optimized solution (mock LLM response)
    let solution = GeneratedSolution {
        id: Uuid::new_v4().to_string(),
        problem_id: problem.id.clone(),
        solution_code: r#"def twoSum(nums, target):
    """
    Find two numbers in the array that add up to target.
    
    Args:
        nums: List of integers
        target: Target sum
        
    Returns:
        List of two indices that add up to target
    """
    num_map = {}
    
    for i, num in enumerate(nums):
        complement = target - num
        if complement in num_map:
            return [num_map[complement], i]
        num_map[num] = i
    
    return []  # No solution found"#.to_string(),
        language: ProgrammingLanguage::Python,
        explanation: Some("This solution uses a hash map to store numbers we've seen and their indices. For each number, we calculate its complement (target - current number) and check if we've seen it before. If yes, we found our pair! This approach only requires one pass through the array, making it much more efficient than the brute force O(n²) nested loop approach.".to_string()),
        time_complexity: Some("O(n)".to_string()),
        space_complexity: Some("O(n)".to_string()),
        test_results: vec![
            TestValidationResult {
                test_case_id: "test1".to_string(),
                input: "[2,7,11,15], target=9".to_string(),
                expected_output: "[0,1]".to_string(),
                actual_output: "[0,1]".to_string(),
                passed: true,
                execution_time_ms: Some(1),
                error_message: None,
            },
            TestValidationResult {
                test_case_id: "test2".to_string(),
                input: "[3,2,4], target=6".to_string(),
                expected_output: "[1,2]".to_string(),
                actual_output: "[1,2]".to_string(),
                passed: true,
                execution_time_ms: Some(1),
                error_message: None,
            },
            TestValidationResult {
                test_case_id: "test3".to_string(),
                input: "[3,3], target=6".to_string(),
                expected_output: "[0,1]".to_string(),
                actual_output: "[0,1]".to_string(),
                passed: true,
                execution_time_ms: Some(1),
                error_message: None,
            },
        ],
        confidence_score: 0.94,
        generation_time_ms: 750,
        model_used: "llama3.2:latest".to_string(),
        alternative_solutions: vec![],
        generated_at: Utc::now(),
    };
    
    println!("  ✅ Solution generated successfully");
    
    Ok(solution)
}

async fn store_detection_results(
    session_id: &str,
    ocr_result: &OCRResult,
    problem: &DetectedCodingProblem,
    solution: &GeneratedSolution
) -> Result<()> {
    println!("  💾 Storing results in database...");
    
    // Initialize database
    let db = TranscriptDatabase::new(None).await?;
    let visual_manager = VisualDataManager::new(db.pool.clone());
    
    // Store high-frequency frame
    let timestamp_ms = Utc::now().timestamp_millis();
    let frame = savant_db::visual_data::HighFrequencyFrame {
        timestamp_ms,
        session_id: session_id.to_string(),
        frame_hash: "mock-demo-frame".to_string(),
        change_score: 0.95,
        file_path: Some("test-data/screenshots/twosum.png".to_string()),
        screen_resolution: Some("1920x1080".to_string()),
        active_app: Some("Chrome".to_string()),
        processing_flags: 0,
    };
    
    visual_manager.store_hf_frame(&frame).await?;
    
    // Store text extractions
    for (i, block) in ocr_result.text_blocks.iter().enumerate().take(5) {
        let text_extraction = savant_db::visual_data::TextExtraction {
            frame_id: "mock-demo-frame".to_string(),
            word_text: block.text.clone(),
            confidence: block.confidence as f64,
            bbox_x: block.bounding_box.x as i32,
            bbox_y: block.bounding_box.y as i32,
            bbox_width: block.bounding_box.width as i32,
            bbox_height: block.bounding_box.height as i32,
            font_size_estimate: block.font_info.as_ref().and_then(|f| f.size).map(|s| s as f64),
            text_type: Some(format!("{:?}", block.semantic_type)),
            line_id: i as i32,
            paragraph_id: 0,
        };
        
        visual_manager.store_text_extraction(&text_extraction).await?;
    }
    
    // Store detected task
    let detected_task = savant_db::visual_data::DetectedTask {
        frame_id: "mock-demo-frame".to_string(),
        task_type: "CodingProblem".to_string(),
        confidence: problem.confidence as f64,
        description: problem.description.clone(),
        evidence_text: serde_json::to_string(&problem)?,
        bounding_regions: Some(serde_json::to_string(&problem.screen_region)?),
        assistance_suggestions: serde_json::to_string(&solution)?,
    };
    
    visual_manager.store_detected_task(&detected_task).await?;
    
    println!("  ✅ All data stored successfully");
    
    Ok(())
}

fn simulate_ui_display(problem: &DetectedCodingProblem, solution: &GeneratedSolution) {
    println!("┌─ Solution Assistant ──────────────────────────────────┐");
    println!("│ 🧩 Problem: {}                               │", problem.title);
    println!("│ 🎯 Platform: {:?}                                │", problem.platform.as_ref().unwrap_or(&CodingPlatform::Unknown));
    println!("│ 📊 Confidence: {:.1}%                              │", problem.confidence * 100.0);
    println!("├────────────────────────────────────────────────────────┤");
    println!("│ 🚀 Generated Solution (Confidence: {:.1}%)           │", solution.confidence_score * 100.0);
    println!("│                                                        │");
    println!("│ ⏱️  Time: {} | 💾 Space: {}                │", 
        solution.time_complexity.as_ref().unwrap_or(&"O(?)".to_string()),
        solution.space_complexity.as_ref().unwrap_or(&"O(?)".to_string())
    );
    println!("│                                                        │");
    println!("│ 🧪 Tests: {}/{} passed                              │", 
        solution.test_results.iter().filter(|t| t.passed).count(),
        solution.test_results.len()
    );
    println!("│                                                        │");
    println!("│ [📋 Copy]  [🔄 Regenerate]  [✨ Apply]  [❌ Dismiss]  │");
    println!("└────────────────────────────────────────────────────────┘");
    println!("\n💡 The solution overlay would appear as a draggable window");
    println!("   positioned near the coding problem for immediate assistance.");
}