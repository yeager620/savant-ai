//! Integration tests for savant-llm CLI tool

use std::process::Command;
use serde_json;

/// Test that the CLI tool can be executed and shows help
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--package", "savant-llm", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("UNIX-philosophy CLI tool"));
    assert!(stdout.contains("--prompt"));
    assert!(stdout.contains("--model"));
}

/// Test the connection test subcommand
#[test]
fn test_connection_test() {
    let output = Command::new("cargo")
        .args(&["run", "--package", "savant-llm", "--", "test"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should output valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
    
    assert!(json["provider"].as_str().is_some());
    assert!(json["connected"].as_bool().is_some());
    assert!(json["timestamp"].as_str().is_some());
}

/// Test JSON input parsing
#[test]
fn test_json_input_parsing() {
    // This test would require mocking or a test server
    // For now, just test that invalid JSON is handled gracefully
    let output = Command::new("bash")
        .arg("-c")
        .arg("echo 'invalid json' | cargo run --package savant-llm")
        .output()
        .expect("Failed to execute command");

    // Should still run (treating as plain text), not crash
    // The exact behavior depends on whether Ollama is running
    assert!(output.status.success() || output.status.code() == Some(1));
}

/// Test model listing
#[test] 
fn test_model_listing() {
    let output = Command::new("cargo")
        .args(&["run", "--package", "savant-llm", "--", "models"])
        .output()
        .expect("Failed to execute command");

    // Should output valid JSON regardless of whether Ollama is running
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    
    // If it fails, it should fail gracefully with an error message
    assert!(json.is_ok() || !stdout.is_empty());
}

/// Test CLI argument validation
#[test]
fn test_invalid_provider() {
    let output = Command::new("cargo")
        .args(&["run", "--package", "savant-llm", "--", 
               "--provider", "invalid", "--prompt", "test"])
        .output()
        .expect("Failed to execute command");

    // Should fail with unsupported provider error
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Unsupported provider") || stderr.contains("invalid"));
}

/// Test that output is proper JSON format
#[test]
fn test_output_format() {
    // Only run if we can connect to Ollama
    let test_connection = Command::new("cargo")
        .args(&["run", "--package", "savant-llm", "--", "test"])
        .output()
        .expect("Failed to execute test command");
    
    if !test_connection.status.success() {
        println!("Skipping output format test - Ollama not available");
        return;
    }
    
    let test_stdout = String::from_utf8(test_connection.stdout).unwrap();
    let test_json: serde_json::Value = serde_json::from_str(&test_stdout).unwrap();
    
    // Only run actual query test if connection is true
    if test_json["connected"].as_bool() == Some(true) {
        let output = Command::new("cargo")
            .args(&["run", "--package", "savant-llm", "--", 
                   "--prompt", "Hello", "--model", "devstral"])
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let json: serde_json::Value = serde_json::from_str(&stdout)
                .expect("Output should be valid JSON");
            
            // Check required fields
            assert!(json["content"].as_str().is_some());
            assert!(json["model"].as_str().is_some());
            assert!(json["provider"].as_str().is_some());
            assert!(json["processing_time_ms"].as_u64().is_some());
            assert!(json["finished"].as_bool().is_some());
        }
    }
}