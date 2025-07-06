# End-to-End Coding Problem Detection Test Framework

A flexible, modular testing system for validating coding problem detection and solution generation using devstral:latest LLM integration.

## 🏗️ Framework Architecture

Built following UNIX philosophy and project design principles:

- **Single Purpose**: Test coding problem detection pipeline
- **Composable**: OCR → Vision → Detection → LLM → Validation
- **Standard I/O**: JSON configuration and results  
- **Extensible**: Easily add new problems and platforms

## 🧪 Available Tests

### 1. Two Sum Test (Optimized Demo)
```bash
cargo run -p e2e-coding-detection --bin optimized_demo
```
- **Screenshot**: `test-data/screenshots/twosum.png`
- **Platform**: LeetCode
- **LLM**: devstral:latest integration
- **Validation**: 7 comprehensive test cases
- **Performance**: Real-time OCR + vision analysis

### 2. HackerRank Hard Problem Test
```bash
cargo run -p e2e-coding-detection --bin hackerrank_hard_test
```
- **Screenshot**: `test-data/screenshots/hackerrank_hard_01.png`
- **Platform**: HackerRank
- **Difficulty**: Hard
- **Validation**: Flexible test case generation
- **Performance**: Extended timeouts for complex problems

### 3. Test Suite Runner
```bash
cargo run -p e2e-coding-detection --bin test_suite_runner
```
- **Multi-test execution**: Runs multiple test configurations
- **Comprehensive reporting**: Performance metrics and success rates
- **CI/CD Integration**: JSON export for automated pipelines

### 4. Quick Demo
```bash
cargo run -p e2e-coding-detection --bin quick_demo
```
- **Framework Overview**: Capabilities and architecture
- **Usage Examples**: How to add new tests
- **No OCR Processing**: Fast demonstration mode

## 📋 Test Configuration Structure

### TestConfiguration
```rust
pub struct TestConfiguration {
    pub test_name: String,
    pub description: String,
    pub screenshot_path: String,
    pub expected_problem: ExpectedProblem,
    pub validation_config: ValidationConfig,
    pub performance_targets: PerformanceTargets,
}
```

### ExpectedProblem
```rust
pub struct ExpectedProblem {
    pub problem_type: String,           // "algorithm", "data_structure", "debugging"
    pub title_contains: Vec<String>,    // Keywords for detection
    pub platform: Option<String>,      // "leetcode", "hackerrank", "codeforces"
    pub language: String,               // "python", "java", "javascript"
    pub difficulty: Option<String>,     // "easy", "medium", "hard"
    pub min_confidence: f64,            // 0.5-0.9 detection threshold
}
```

### ValidationConfig
```rust
pub struct ValidationConfig {
    pub test_cases: Vec<TestCase>,      // Solution validation tests
    pub min_success_rate: f64,          // 0.7-0.9 required pass rate
    pub min_performance_score: f64,     // 6.0-8.0 quality threshold
    pub timeout_ms: u64,                // 30000-45000ms max execution
}
```

## 🎯 Performance Targets

| Component | Target Time | Acceptable | Notes |
|-----------|-------------|------------|-------|
| OCR Processing | < 10s | < 15s | Tesseract with optimization |
| Vision Analysis | < 10s | < 15s | Computer vision pipeline |
| Problem Detection | < 1s | < 2s | Pattern matching algorithms |
| LLM Generation | < 15s | < 20s | devstral:latest via ollama |
| **Total Pipeline** | **< 30s** | **< 45s** | **Real-time target** |

## 🧪 Adding New Tests

### 1. Using TestConfigurationBuilder (Recommended)
```rust
let config = TestConfigurationBuilder::hackerrank_hard_test("path/to/screenshot.png");
let framework = FlexibleTestFramework::new()?;
let result = framework.run_test(config).await?;
```

### 2. Custom Configuration
```rust
let config = TestConfiguration {
    test_name: "My Custom Test".to_string(),
    description: "Custom problem detection test".to_string(),
    screenshot_path: "test-data/screenshots/my_problem.png".to_string(),
    expected_problem: ExpectedProblem {
        problem_type: "algorithm".to_string(),
        title_contains: vec!["binary search".to_string(), "tree".to_string()],
        platform: Some("leetcode".to_string()),
        language: "python".to_string(),
        difficulty: Some("medium".to_string()),
        min_confidence: 0.7,
    },
    validation_config: ValidationConfig {
        test_cases: vec![
            TestCase {
                input: "nums=[1,2,3,4], target=3".to_string(),
                expected_output: "2".to_string(),
                description: "Basic search test".to_string(),
            }
        ],
        min_success_rate: 0.8,
        min_performance_score: 7.0,
        timeout_ms: 30000,
    },
    performance_targets: PerformanceTargets {
        max_total_time_ms: 30000,
        max_ocr_time_ms: 10000,
        max_vision_time_ms: 10000,
        max_llm_time_ms: 15000,
    },
};
```

## 🔌 Extending for New Platforms

### 1. Add Platform Support
```rust
// In savant-video/src/coding_problem_detector.rs
pub enum CodingPlatform {
    LeetCode,
    HackerRank,
    Codeforces,
    AtCoder,        // Add new platform
    Unknown,
}
```

### 2. Update Detection Patterns
```rust
let platform = match expected.platform.as_deref() {
    Some("leetcode") => CodingPlatform::LeetCode,
    Some("hackerrank") => CodingPlatform::HackerRank,
    Some("codeforces") => CodingPlatform::Codeforces,
    Some("atcoder") => CodingPlatform::AtCoder,    // Add mapping
    _ => CodingPlatform::Unknown,
};
```

### 3. Create Platform-Specific Builder
```rust
impl TestConfigurationBuilder {
    pub fn atcoder_test(screenshot_path: &str) -> TestConfiguration {
        TestConfiguration {
            test_name: "AtCoder Contest Problem".to_string(),
            expected_problem: ExpectedProblem {
                platform: Some("atcoder".to_string()),
                title_contains: vec!["atcoder".to_string(), "contest".to_string()],
                // ... other configuration
            },
            // ... rest of config
        }
    }
}
```

## 📊 Test Results Structure

### TestResult
```rust
pub struct TestResult {
    pub test_name: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,                          // Overall pass/fail
    pub error_message: Option<String>,
    pub performance_metrics: PerformanceMetrics,
    pub problem_detection: ProblemDetectionResult,
    pub solution_validation: Option<SolutionValidationReport>,
    pub overall_score: f64,                     // 0.0-10.0 quality score
}
```

### Success Criteria
- **Problem Detection**: Confidence ≥ threshold, keywords match
- **Solution Generation**: Valid syntax, logical correctness
- **Test Validation**: Success rate ≥ minimum threshold
- **Performance**: Total time within targets
- **Overall Score**: ≥ 6.0/10 for passing grade

## 🚀 CI/CD Integration

### Automated Testing
```bash
# Run all tests and export results
cargo run -p e2e-coding-detection --bin test_suite_runner

# Check results
cat test-results/ci_results.env
# TESTS_PASSED=2
# TESTS_TOTAL=2  
# SUCCESS_RATE=100.0
# AVERAGE_SCORE=8.5
# SUITE_STATUS=PASS
```

### JSON Export
```bash
# Detailed results in JSON format
cat test-results/test_suite_results.json

# Human-readable summary
cat test-results/test_suite_summary.txt
```

## 🎉 Framework Benefits

### ✅ Modular Design
- Easy test addition via configuration structs
- Reusable across different screenshots and problems
- Flexible validation criteria per problem type
- Configurable performance targets

### ✅ UNIX Philosophy
- Single-purpose components that do one thing well
- Composable pipeline: OCR → Vision → Detection → LLM → Validation
- Standard I/O with JSON configuration and results
- Tool chain integration with existing Savant AI ecosystem

### ✅ Production Ready
- Comprehensive error handling and fallbacks
- Performance monitoring with detailed metrics
- Real LLM integration (devstral:latest)
- CI/CD support with JSON export
- Extensive logging and debugging capabilities

### ✅ Extensible Architecture
- Support for any coding platform (LeetCode, HackerRank, etc.)
- Multiple programming languages (Python, Java, C++, etc.)
- Custom problem types and validation criteria
- Plugin architecture for additional LLM models

## 🔧 Development Workflow

1. **Add Screenshot**: Place in `test-data/screenshots/`
2. **Create Configuration**: Use `TestConfigurationBuilder` or custom config
3. **Define Test Cases**: Add validation test cases for solution
4. **Set Performance Targets**: Configure acceptable time limits
5. **Run Test**: Execute with `FlexibleTestFramework::run_test()`
6. **Validate Results**: Check success criteria and performance metrics
7. **Export for CI/CD**: Use JSON results for automated pipelines

The framework is now ready for production use with comprehensive testing capabilities for any coding problem detection scenario!