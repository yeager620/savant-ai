/*! 
Solution Validation System for Coding Problem Detection

Uses devstral:latest to generate solutions and validates them against hardcoded test cases.
Provides comprehensive testing for Two Sum and other coding problems.
*/

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub expected_output: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub test_case_id: String,
    pub passed: bool,
    pub actual_output: String,
    pub expected_output: String,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionValidationReport {
    pub solution_code: String,
    pub language: String,
    pub model_used: String,
    pub generation_time_ms: u64,
    pub test_results: Vec<ValidationResult>,
    pub overall_success_rate: f64,
    pub performance_score: f64,
    pub correctness_verified: bool,
}

pub struct SolutionValidator {
    pub model_name: String,
}

impl SolutionValidator {
    pub fn new() -> Self {
        Self {
            model_name: "devstral:latest".to_string(),
        }
    }

    /// Generate solution using devstral:latest
    pub async fn generate_solution(&self, problem_description: &str) -> Result<String> {
        let prompt = format!(
            r#"You are a coding expert. Generate a Python solution for this problem:

{}

Requirements:
- Provide ONLY the function implementation
- No explanations, comments, or test cases
- Use the exact function signature expected
- Write clean, efficient code
- For Two Sum: function should be named 'twoSum' and take (nums, target) parameters

Return only the Python code:"#,
            problem_description
        );

        let start_time = std::time::Instant::now();
        
        // Call ollama with devstral model
        let output = Command::new("ollama")
            .arg("run")
            .arg(&self.model_name)
            .arg(&prompt)
            .output()?;

        let generation_time = start_time.elapsed().as_millis() as u64;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Ollama generation failed: {}", error));
        }

        let solution = String::from_utf8_lossy(&output.stdout);
        let cleaned_solution = self.clean_solution_code(&solution);
        
        println!("🤖 Generated solution in {}ms using {}", generation_time, self.model_name);
        println!("📝 Solution code:\n{}", cleaned_solution);
        
        Ok(cleaned_solution)
    }

    /// Clean and format the generated solution
    fn clean_solution_code(&self, raw_solution: &str) -> String {
        // Remove common prefixes/suffixes from LLM output
        let cleaned = raw_solution
            .trim()
            .replace("```python", "")
            .replace("```", "")
            .replace("Here's the solution:", "")
            .replace("Here is the solution:", "")
            .trim()
            .to_string();

        // Ensure proper indentation
        let lines: Vec<&str> = cleaned.lines().collect();
        let mut formatted_lines = Vec::new();
        
        for line in lines {
            if line.trim().is_empty() {
                formatted_lines.push("".to_string());
            } else if line.starts_with("def ") || line.starts_with("class ") {
                formatted_lines.push(line.to_string());
            } else {
                // Ensure proper indentation for function body
                if !line.starts_with("    ") && !line.trim().is_empty() {
                    formatted_lines.push(format!("    {}", line.trim()));
                } else {
                    formatted_lines.push(line.to_string());
                }
            }
        }

        formatted_lines.join("\n")
    }

    /// Validate solution against Two Sum test cases
    pub async fn validate_two_sum_solution(&self, solution_code: &str) -> Result<SolutionValidationReport> {
        let test_cases = self.get_two_sum_test_cases();
        let mut test_results = Vec::new();
        
        println!("\n🧪 Validating solution against {} test cases...", test_cases.len());
        
        for (i, test_case) in test_cases.iter().enumerate() {
            println!("   Running test case {}: {}", i + 1, test_case.description);
            
            let result = self.run_test_case(solution_code, test_case).await?;
            
            if result.passed {
                println!("   ✅ PASSED - Expected: {}, Got: {}", result.expected_output, result.actual_output);
            } else {
                println!("   ❌ FAILED - Expected: {}, Got: {}", result.expected_output, result.actual_output);
                if let Some(error) = &result.error_message {
                    println!("      Error: {}", error);
                }
            }
            
            test_results.push(result);
        }

        let passed_tests = test_results.iter().filter(|r| r.passed).count();
        let total_tests = test_results.len();
        let success_rate = passed_tests as f64 / total_tests as f64;
        
        let performance_score = self.calculate_performance_score(&test_results);
        let correctness_verified = success_rate >= 0.8; // 80% pass rate required
        
        println!("\n📊 Validation Results:");
        println!("   Success Rate: {}/{} ({:.1}%)", passed_tests, total_tests, success_rate * 100.0);
        println!("   Performance Score: {:.1}/10", performance_score);
        println!("   Correctness Verified: {}", if correctness_verified { "✅ YES" } else { "❌ NO" });

        Ok(SolutionValidationReport {
            solution_code: solution_code.to_string(),
            language: "python".to_string(),
            model_used: self.model_name.clone(),
            generation_time_ms: 0, // Set by caller
            test_results,
            overall_success_rate: success_rate,
            performance_score,
            correctness_verified,
        })
    }

    /// Get comprehensive Two Sum test cases
    fn get_two_sum_test_cases(&self) -> Vec<TestCase> {
        vec![
            TestCase {
                input: "nums=[2,7,11,15], target=9".to_string(),
                expected_output: "[0, 1]".to_string(),
                description: "Basic case - first two elements".to_string(),
            },
            TestCase {
                input: "nums=[3,2,4], target=6".to_string(),
                expected_output: "[1, 2]".to_string(),
                description: "Different positions".to_string(),
            },
            TestCase {
                input: "nums=[3,3], target=6".to_string(),
                expected_output: "[0, 1]".to_string(),
                description: "Duplicate numbers".to_string(),
            },
            TestCase {
                input: "nums=[2,7,11,15], target=26".to_string(),
                expected_output: "[2, 3]".to_string(),
                description: "Last two elements".to_string(),
            },
            TestCase {
                input: "nums=[1,2,3,4,5], target=9".to_string(),
                expected_output: "[3, 4]".to_string(),
                description: "Middle elements".to_string(),
            },
            TestCase {
                input: "nums=[-1,-2,-3,-4,-5], target=-8".to_string(),
                expected_output: "[2, 4]".to_string(),
                description: "Negative numbers".to_string(),
            },
            TestCase {
                input: "nums=[0,4,3,0], target=0".to_string(),
                expected_output: "[0, 3]".to_string(),
                description: "Zero target".to_string(),
            },
        ]
    }

    /// Run a single test case
    async fn run_test_case(&self, solution_code: &str, test_case: &TestCase) -> Result<ValidationResult> {
        let start_time = std::time::Instant::now();
        
        // Parse input
        let (nums, target) = self.parse_two_sum_input(&test_case.input)?;
        
        // Create Python test script
        let test_script = format!(
            r#"
{}

# Test execution
nums = {}
target = {}
result = twoSum(nums, target)
print(result)
"#,
            solution_code,
            self.format_python_list(&nums),
            target
        );

        // Write to temporary file and execute
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(test_script.as_bytes())?;
        
        let output = Command::new("python3")
            .arg(temp_file.path())
            .output()?;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Ok(ValidationResult {
                test_case_id: Uuid::new_v4().to_string(),
                passed: false,
                actual_output: "".to_string(),
                expected_output: test_case.expected_output.clone(),
                execution_time_ms: execution_time,
                error_message: Some(error.to_string()),
            });
        }

        let actual_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let expected_output = test_case.expected_output.clone();
        
        // Compare results (normalize format)
        let passed = self.compare_outputs(&actual_output, &expected_output);
        
        Ok(ValidationResult {
            test_case_id: Uuid::new_v4().to_string(),
            passed,
            actual_output,
            expected_output,
            execution_time_ms: execution_time,
            error_message: None,
        })
    }

    /// Parse Two Sum input string
    fn parse_two_sum_input(&self, input: &str) -> Result<(Vec<i32>, i32)> {
        // Parse "nums=[2,7,11,15], target=9"
        let parts: Vec<&str> = input.split(", ").collect();
        
        let nums_str = parts[0].replace("nums=", "");
        let target_str = parts[1].replace("target=", "");
        
        let nums: Vec<i32> = nums_str
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to parse nums: {}", e))?;
        
        let target: i32 = target_str.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse target: {}", e))?;
        
        Ok((nums, target))
    }

    /// Format Vec<i32> as Python list
    fn format_python_list(&self, nums: &[i32]) -> String {
        format!("[{}]", nums.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", "))
    }

    /// Compare actual vs expected output
    fn compare_outputs(&self, actual: &str, expected: &str) -> bool {
        // Normalize both outputs
        let normalize = |s: &str| -> String {
            s.replace(" ", "").replace("[", "").replace("]", "").to_lowercase()
        };
        
        normalize(actual) == normalize(expected)
    }

    /// Calculate performance score based on test results
    fn calculate_performance_score(&self, results: &[ValidationResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }

        let mut score = 10.0;
        let passed_count = results.iter().filter(|r| r.passed).count();
        let total_count = results.len();
        
        // Success rate penalty
        let success_rate = passed_count as f64 / total_count as f64;
        if success_rate < 0.5 {
            score -= 5.0;
        } else if success_rate < 0.8 {
            score -= 2.0;
        }
        
        // Performance penalty (if execution times are too high)
        let avg_execution_time = results.iter().map(|r| r.execution_time_ms).sum::<u64>() / results.len() as u64;
        if avg_execution_time > 100 {
            score -= 1.0;
        }
        
        // Error penalty
        let error_count = results.iter().filter(|r| r.error_message.is_some()).count();
        if error_count > 0 {
            score -= error_count as f64 * 0.5;
        }
        
        score.max(0.0).min(10.0)
    }
}

/// Generate comprehensive Two Sum problem description for devstral
pub fn create_two_sum_problem_description() -> String {
    r#"Two Sum Problem

Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target.

You may assume that each input would have exactly one solution, and you may not use the same element twice.

You can return the answer in any order.

Example 1:
Input: nums = [2,7,11,15], target = 9
Output: [0,1]
Explanation: Because nums[0] + nums[1] == 9, we return [0, 1].

Example 2:
Input: nums = [3,2,4], target = 6
Output: [1,2]

Example 3:
Input: nums = [3,3], target = 6
Output: [0,1]

Constraints:
- 2 <= nums.length <= 10^4
- -10^9 <= nums[i] <= 10^9
- -10^9 <= target <= 10^9
- Only one valid answer exists.

Write a function called twoSum that takes nums and target as parameters and returns a list of two indices."#.to_string()
}