use savant_video::solution_generator::*;
use savant_video::coding_problem_detector::{
    DetectedCodingProblem, CodingProblemType, CodingPlatform, 
    ProgrammingLanguage, CodeContext, ErrorDetails, TestCase, ScreenRegion
};
use savant_video::llm_provider::{LLMProvider, MockLLMProvider};
use chrono::Utc;

fn create_test_problem() -> DetectedCodingProblem {
    DetectedCodingProblem {
        id: "test-problem-1".to_string(),
        problem_type: CodingProblemType::AlgorithmChallenge,
        title: "Two Sum".to_string(),
        description: "Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target.".to_string(),
        code_context: CodeContext {
            visible_code: "def twoSum(nums, target):\n    pass".to_string(),
            focused_function: Some("twoSum".to_string()),
            imports: vec![],
            class_context: None,
            line_numbers: Some((1, 2)),
            cursor_position: None,
            selected_text: None,
        },
        error_details: None,
        platform: Some(CodingPlatform::LeetCode),
        language: ProgrammingLanguage::Python,
        starter_code: Some("def twoSum(nums, target):\n    pass".to_string()),
        test_cases: vec![
            TestCase {
                input: "nums = [2,7,11,15], target = 9".to_string(),
                expected_output: "[0,1]".to_string(),
                actual_output: None,
                passed: None,
                execution_time: None,
            },
            TestCase {
                input: "nums = [3,2,4], target = 6".to_string(),
                expected_output: "[1,2]".to_string(),
                actual_output: None,
                passed: None,
                execution_time: None,
            },
        ],
        constraints: vec![
            "2 <= nums.length <= 10^4".to_string(),
            "-10^9 <= nums[i] <= 10^9".to_string(),
        ],
        confidence: 0.95,
        detected_at: Utc::now(),
        screen_region: ScreenRegion { x: 0, y: 0, width: 1920, height: 1080 },
    }
}

fn create_mock_llm_provider() -> MockLLMProvider {
    let mut mock = MockLLMProvider::new();
    
    // Set up mock responses
    mock.set_response(
        "two sum",
        r#"```solution
def twoSum(nums, target):
    seen = {}
    for i, num in enumerate(nums):
        complement = target - num
        if complement in seen:
            return [seen[complement], i]
        seen[num] = i
    return []
```

```explanation
This solution uses a hash map to store numbers we've seen along with their indices. For each number, we check if its complement (target - num) exists in the hash map. If it does, we've found our pair and return their indices.
```

```time_complexity
O(n) - We traverse the list once, and hash map lookups are O(1) on average.
```

```space_complexity
O(n) - In the worst case, we store all n elements in the hash map.
```"#,
    );
    
    mock
}

#[tokio::test]
async fn test_generate_solution_for_algorithm_challenge() {
    let config = SolutionConfig::default();
    let llm_provider = LLMProvider::Mock(create_mock_llm_provider());
    let generator = SolutionGenerator::new(config, llm_provider);
    
    let problem = create_test_problem();
    let solution = generator.generate_solution(&problem).await.unwrap();
    
    assert_eq!(solution.problem_id, problem.id);
    assert!(solution.solution_code.contains("def twoSum"));
    assert!(solution.solution_code.contains("seen = {}"));
    assert!(solution.explanation.is_some());
    assert!(solution.time_complexity.is_some());
    assert!(solution.space_complexity.is_some());
    assert!(solution.confidence_score > 0.5);
}

#[tokio::test]
async fn test_generate_solution_for_compilation_error() {
    let config = SolutionConfig::default();
    let mut mock = MockLLMProvider::new();
    mock.set_response(
        "syntax error",
        r#"```solution
print("Hello, world!")
```

```explanation
The error was caused by a missing semicolon. In Python, statements don't require semicolons, but the print statement was missing its closing parenthesis.
```"#,
    );
    
    let llm_provider = LLMProvider::Mock(mock);
    let generator = SolutionGenerator::new(config, llm_provider);
    
    let mut problem = create_test_problem();
    problem.problem_type = CodingProblemType::CompilationError;
    problem.error_details = Some(ErrorDetails {
        error_type: "SyntaxError".to_string(),
        error_message: "invalid syntax".to_string(),
        stack_trace: None,
        error_line: Some(1),
        error_column: Some(20),
        suggestions: vec![],
    });
    
    let solution = generator.generate_solution(&problem).await.unwrap();
    
    assert!(solution.solution_code.contains("print"));
    assert!(solution.explanation.unwrap().contains("semicolon") || solution.explanation.unwrap().contains("parenthesis"));
}

#[tokio::test]
async fn test_solution_caching() {
    let mut config = SolutionConfig::default();
    config.cache_solutions = true;
    
    let llm_provider = LLMProvider::Mock(create_mock_llm_provider());
    let generator = SolutionGenerator::new(config, llm_provider);
    
    let problem = create_test_problem();
    
    // First generation
    let start = std::time::Instant::now();
    let solution1 = generator.generate_solution(&problem).await.unwrap();
    let first_time = start.elapsed();
    
    // Second generation (should be cached)
    let start = std::time::Instant::now();
    let solution2 = generator.generate_solution(&problem).await.unwrap();
    let cached_time = start.elapsed();
    
    assert_eq!(solution1.solution_code, solution2.solution_code);
    assert!(cached_time < first_time); // Cached should be faster
}

#[tokio::test]
async fn test_prompt_generation_includes_all_problem_details() {
    let config = SolutionConfig::default();
    let generator = SolutionGenerator::new(config, LLMProvider::Mock(MockLLMProvider::new()));
    
    let problem = create_test_problem();
    let prompt = generator.generate_prompt(&problem).unwrap();
    
    // Check that prompt includes all necessary information
    assert!(prompt.contains("Two Sum"));
    assert!(prompt.contains("LeetCode"));
    assert!(prompt.contains("Python"));
    assert!(prompt.contains("nums = [2,7,11,15], target = 9"));
    assert!(prompt.contains("[0,1]"));
    assert!(prompt.contains("2 <= nums.length <= 10^4"));
}

#[tokio::test]
async fn test_parse_solution_response() {
    let config = SolutionConfig {
        include_explanations: true,
        include_time_complexity: true,
        include_space_complexity: true,
        ..Default::default()
    };
    let generator = SolutionGenerator::new(config, LLMProvider::Mock(MockLLMProvider::new()));
    
    let problem = create_test_problem();
    let response = r#"
```solution
def solution(n):
    return n * 2
```

```explanation
This solution simply doubles the input.
```

```time_complexity
O(1) - Constant time operation
```

```space_complexity
O(1) - No extra space used
```
"#;
    
    let solution = generator.parse_solution_response(&problem, response, "test-model").unwrap();
    
    assert_eq!(solution.solution_code.trim(), "def solution(n):\n    return n * 2");
    assert_eq!(solution.explanation.unwrap().trim(), "This solution simply doubles the input.");
    assert_eq!(solution.time_complexity.unwrap().trim(), "O(1) - Constant time operation");
    assert_eq!(solution.space_complexity.unwrap().trim(), "O(1) - No extra space used");
}

#[tokio::test]
async fn test_confidence_score_calculation() {
    let config = SolutionConfig::default();
    let generator = SolutionGenerator::new(config, LLMProvider::Mock(MockLLMProvider::new()));
    
    // Full solution with all components
    let full_score = generator.calculate_confidence_score(
        "def solution(): return 42",
        &Some("This is a detailed explanation".to_string()),
        &Some("O(1)".to_string()),
        &Some("O(1)".to_string()),
    );
    assert!(full_score > 0.9);
    
    // Minimal solution
    let minimal_score = generator.calculate_confidence_score(
        "pass",
        &None,
        &None,
        &None,
    );
    assert!(minimal_score < 0.6);
}

#[tokio::test]
async fn test_multiple_model_fallback() {
    let mut config = SolutionConfig::default();
    config.preferred_models = vec![
        "model-that-fails".to_string(),
        "model-that-works".to_string(),
    ];
    
    let mut mock = MockLLMProvider::new();
    mock.set_response("model-that-works", "```solution\nprint('success')\n```");
    mock.fail_for_model("model-that-fails");
    
    let generator = SolutionGenerator::new(config, LLMProvider::Mock(mock));
    let problem = create_test_problem();
    
    let solution = generator.generate_solution(&problem).await.unwrap();
    assert_eq!(solution.model_used, "model-that-works");
    assert!(solution.solution_code.contains("success"));
}

#[tokio::test]
async fn test_test_case_validation() {
    let mut config = SolutionConfig::default();
    config.include_test_validation = true;
    
    let generator = SolutionGenerator::new(config, LLMProvider::Mock(create_mock_llm_provider()));
    let problem = create_test_problem();
    
    let solution = generator.generate_solution(&problem).await.unwrap();
    
    // Should have test results for each test case
    assert_eq!(solution.test_results.len(), problem.test_cases.len());
    for result in &solution.test_results {
        assert!(result.passed);
        assert!(result.execution_time_ms.is_some());
    }
}