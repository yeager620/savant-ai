/*! 
Test Suite Runner

Demonstrates running multiple coding problem tests in sequence.
Shows how the flexible framework can handle different problem types and platforms.

Usage: cargo run -p e2e-coding-detection --bin test_suite_runner
*/

use anyhow::Result;
use std::time::Instant;

mod solution_validator;
mod flexible_test_framework;

use flexible_test_framework::{FlexibleTestFramework, TestConfigurationBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🏃 Coding Problem Detection Test Suite");
    println!("======================================");
    println!("Running multiple tests to demonstrate framework flexibility\n");
    
    let total_start = Instant::now();
    
    // Initialize the flexible test framework
    let framework = FlexibleTestFramework::new()?;
    
    // Create test configurations for different problems
    let test_configs = vec![
        TestConfigurationBuilder::two_sum_test("test-data/screenshots/twosum.png"),
        TestConfigurationBuilder::hackerrank_hard_test("test-data/screenshots/hackerrank_hard_01.png"),
        // Add more tests here as needed
    ];
    
    println!("📋 Test Suite Configuration:");
    println!("   Total Tests: {}", test_configs.len());
    for (i, config) in test_configs.iter().enumerate() {
        println!("   {}. {} - {}", i + 1, config.test_name, config.screenshot_path);
    }
    println!();
    
    // Run all tests
    let mut results = Vec::new();
    for (i, config) in test_configs.iter().enumerate() {
        println!("\n{}", "=".repeat(80));
        println!("🧪 Running Test {}/{}: {}", i + 1, test_configs.len(), config.test_name);
        println!("{}", "=".repeat(80));
        
        let result = framework.run_test(config.clone()).await?;
        results.push(result);
        
        println!("\n✅ Test {} completed - Score: {:.1}/10", i + 1, results[i].overall_score);
    }
    
    // Generate comprehensive summary
    println!("\n{}", "=".repeat(80));
    println!("📊 TEST SUITE SUMMARY");
    println!("{}", "=".repeat(80));
    
    let passed_tests = results.iter().filter(|r| r.success).count();
    let total_tests = results.len();
    let avg_score = results.iter().map(|r| r.overall_score).sum::<f64>() / total_tests as f64;
    let total_time_ms = results.iter().map(|r| r.performance_metrics.total_time_ms).sum::<u64>();
    
    println!("🎯 Overall Results:");
    println!("   Tests Passed: {}/{} ({:.1}%)", passed_tests, total_tests, (passed_tests as f64 / total_tests as f64) * 100.0);
    println!("   Average Score: {:.1}/10", avg_score);
    println!("   Total Execution Time: {:.2}s", total_time_ms as f64 / 1000.0);
    println!("   Suite Status: {}", if passed_tests == total_tests { "✅ PASS" } else { "❌ FAIL" });
    
    println!("\n📋 Individual Test Results:");
    for (i, result) in results.iter().enumerate() {
        let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
        println!("   {}. {} - {} ({:.1}/10, {}ms)", 
            i + 1, 
            result.test_name, 
            status, 
            result.overall_score,
            result.performance_metrics.total_time_ms
        );
        
        if result.problem_detection.detected {
            println!("      Problem: {} (confidence: {:.1}%)", 
                result.problem_detection.title,
                result.problem_detection.confidence * 100.0
            );
        } else {
            println!("      No problem detected");
        }
        
        if let Some(validation) = &result.solution_validation {
            println!("      Solution: {:.1}% success rate, {:.1}/10 performance", 
                validation.overall_success_rate * 100.0,
                validation.performance_score
            );
        }
    }
    
    // Performance analysis
    println!("\n⏱️  Performance Analysis:");
    let avg_ocr_time = results.iter().map(|r| r.performance_metrics.ocr_time_ms).sum::<u64>() / total_tests as u64;
    let avg_vision_time = results.iter().map(|r| r.performance_metrics.vision_time_ms).sum::<u64>() / total_tests as u64;
    let avg_detection_time = results.iter().map(|r| r.performance_metrics.detection_time_ms).sum::<u64>() / total_tests as u64;
    let avg_llm_time = results.iter().map(|r| r.performance_metrics.llm_time_ms).sum::<u64>() / total_tests as u64;
    
    println!("   Average OCR Time: {}ms", avg_ocr_time);
    println!("   Average Vision Time: {}ms", avg_vision_time);
    println!("   Average Detection Time: {}ms", avg_detection_time);
    println!("   Average LLM Time: {}ms", avg_llm_time);
    
    let meets_targets = results.iter().filter(|r| r.performance_metrics.meets_targets).count();
    println!("   Performance Targets Met: {}/{} ({:.1}%)", 
        meets_targets, total_tests, (meets_targets as f64 / total_tests as f64) * 100.0);
    
    // Framework capabilities demonstration
    println!("\n{}", "=".repeat(80));
    println!("🏗️  FRAMEWORK CAPABILITIES DEMONSTRATED");
    println!("{}", "=".repeat(80));
    
    println!("✅ Multi-Platform Support:");
    let platforms: std::collections::HashSet<_> = results.iter()
        .filter_map(|r| r.problem_detection.platform.as_ref())
        .collect();
    for platform in &platforms {
        println!("   - {}", platform);
    }
    
    println!("\n✅ Problem Types Detected:");
    let problem_types: std::collections::HashSet<_> = results.iter()
        .filter(|r| r.problem_detection.detected)
        .map(|r| &r.problem_detection.problem_type)
        .collect();
    for problem_type in &problem_types {
        println!("   - {}", problem_type);
    }
    
    println!("\n✅ Modular Architecture Benefits:");
    println!("   - Easy test addition (just add new TestConfiguration)");
    println!("   - Flexible validation criteria per problem type");
    println!("   - Configurable performance targets");
    println!("   - Reusable across different screenshots and problems");
    println!("   - JSON export for CI/CD integration");
    
    println!("\n✅ Production Features:");
    println!("   - Comprehensive error handling");
    println!("   - Performance monitoring and benchmarking");
    println!("   - Detailed result reporting");
    println!("   - Extensible for new platforms and problem types");
    
    // Export comprehensive results
    export_suite_results(&results)?;
    
    let total_elapsed = total_start.elapsed();
    println!("\n🎉 Test suite completed in {:.2}s", total_elapsed.as_secs_f64());
    println!("💾 Results exported to test-results/ directory");
    
    // Exit with appropriate code for CI/CD
    if passed_tests == total_tests {
        println!("🏆 All tests passed! Framework is ready for production use.");
        std::process::exit(0);
    } else {
        println!("⚠️  Some tests failed. Check results for details.");
        std::process::exit(1);
    }
}

fn export_suite_results(results: &[flexible_test_framework::TestResult]) -> Result<()> {
    // Create results directory
    std::fs::create_dir_all("test-results")?;
    
    // Export full results as JSON
    let json_content = serde_json::to_string_pretty(results)?;
    std::fs::write("test-results/test_suite_results.json", json_content)?;
    
    // Create summary report
    let passed_tests = results.iter().filter(|r| r.success).count();
    let total_tests = results.len();
    let avg_score = results.iter().map(|r| r.overall_score).sum::<f64>() / total_tests as f64;
    let total_time_ms = results.iter().map(|r| r.performance_metrics.total_time_ms).sum::<u64>();
    
    let summary = format!(
        "Coding Problem Detection Test Suite Summary\n\
        ==========================================\n\
        \n\
        Overall Results:\n\
        - Tests Passed: {}/{} ({:.1}%)\n\
        - Average Score: {:.1}/10\n\
        - Total Time: {:.2}s\n\
        - Suite Status: {}\n\
        \n\
        Individual Results:\n",
        passed_tests, total_tests, 
        (passed_tests as f64 / total_tests as f64) * 100.0,
        avg_score,
        total_time_ms as f64 / 1000.0,
        if passed_tests == total_tests { "PASS" } else { "FAIL" }
    );
    
    let mut full_summary = summary;
    for (i, result) in results.iter().enumerate() {
        let status = if result.success { "PASS" } else { "FAIL" };
        full_summary.push_str(&format!(
            "{}. {} - {} ({:.1}/10)\n\
            \   Problem Detected: {}\n\
            \   Performance: {}ms\n\
            \   Meets Targets: {}\n\n",
            i + 1,
            result.test_name,
            status,
            result.overall_score,
            if result.problem_detection.detected { "YES" } else { "NO" },
            result.performance_metrics.total_time_ms,
            if result.performance_metrics.meets_targets { "YES" } else { "NO" }
        ));
    }
    
    std::fs::write("test-results/test_suite_summary.txt", full_summary)?;
    
    // Create CI/CD friendly format
    let ci_summary = format!(
        "TESTS_PASSED={}\n\
        TESTS_TOTAL={}\n\
        SUCCESS_RATE={:.1}\n\
        AVERAGE_SCORE={:.1}\n\
        SUITE_STATUS={}\n",
        passed_tests,
        total_tests,
        (passed_tests as f64 / total_tests as f64) * 100.0,
        avg_score,
        if passed_tests == total_tests { "PASS" } else { "FAIL" }
    );
    
    std::fs::write("test-results/ci_results.env", ci_summary)?;
    
    Ok(())
}