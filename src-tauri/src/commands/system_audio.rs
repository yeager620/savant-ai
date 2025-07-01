//! System audio capture automation and management

use anyhow::{anyhow, Result};
use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemAudioStatus {
    pub daemon_running: bool,
    pub blackhole_available: bool,
    pub setup_completed: bool,
    pub capture_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureInfo {
    pub filename: String,
    pub timestamp: String,
    pub size_bytes: u64,
    pub preview: String, // First few lines of transcript
}

/// Check system audio setup status
#[tauri::command]
pub async fn check_system_audio_status() -> Result<SystemAudioStatus, String> {
    match internal_check_status().await {
        Ok(status) => Ok(status),
        Err(e) => Err(e.to_string()),
    }
}

/// Automatically setup system audio capture
#[tauri::command]
pub async fn setup_system_audio() -> Result<String, String> {
    match internal_setup().await {
        Ok(msg) => Ok(msg),
        Err(e) => Err(e.to_string()),
    }
}

/// Start the system audio capture daemon
#[tauri::command]
pub async fn start_audio_daemon() -> Result<String, String> {
    match internal_start_daemon().await {
        Ok(msg) => Ok(msg),
        Err(e) => Err(e.to_string()),
    }
}

/// Stop the system audio capture daemon
#[tauri::command]
pub async fn stop_audio_daemon() -> Result<String, String> {
    match internal_stop_daemon().await {
        Ok(msg) => Ok(msg),
        Err(e) => Err(e.to_string()),
    }
}

/// Get list of captured transcripts
#[tauri::command]
pub async fn list_captures() -> Result<Vec<CaptureInfo>, String> {
    match internal_list_captures().await {
        Ok(captures) => Ok(captures),
        Err(e) => Err(e.to_string()),
    }
}

/// Search through captured transcripts
#[tauri::command]
pub async fn search_captures(query: String) -> Result<Vec<String>, String> {
    match internal_search_captures(&query).await {
        Ok(results) => Ok(results),
        Err(e) => Err(e.to_string()),
    }
}

/// Get recent daemon logs
#[tauri::command]
pub async fn get_daemon_logs(lines: Option<u32>) -> Result<Vec<String>, String> {
    match internal_get_logs(lines.unwrap_or(20)).await {
        Ok(logs) => Ok(logs),
        Err(e) => Err(e.to_string()),
    }
}

// Internal implementation functions

async fn internal_check_status() -> Result<SystemAudioStatus> {
    let mut status = SystemAudioStatus {
        daemon_running: false,
        blackhole_available: false,
        setup_completed: false,
        capture_count: 0,
        last_error: None,
    };

    // Check if daemon is running
    if let Ok(output) = Command::new("sudo")
        .args(&["launchctl", "list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        status.daemon_running = stdout.contains("com.savant.audio.daemon");
    }

    // Check BlackHole availability
    if let Ok(output) = Command::new("cargo")
        .args(&["run", "--package", "savant-audio", "--bin", "list-devices"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        status.blackhole_available = stdout.to_lowercase().contains("blackhole");
    }

    // Check if setup files exist
    let daemon_plist = Path::new("/Library/LaunchAgents/com.savant.audio.daemon.plist");
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let daemon_script = Path::new(&format!("{}/savant-audio-daemon.sh", home_dir));
    
    status.setup_completed = daemon_plist.exists() && daemon_script.exists();

    // Count captures
    let capture_dir = format!("{}/savant-audio-captures", home_dir);
    if let Ok(entries) = std::fs::read_dir(&capture_dir) {
        status.capture_count = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "md")
                    .unwrap_or(false)
            })
            .count() as u32;
    }

    Ok(status)
}

async fn internal_setup() -> Result<String> {
    // Check if we're on macOS
    if !cfg!(target_os = "macos") {
        return Err(anyhow!("System audio capture is only supported on macOS"));
    }

    // Run the automated setup script
    let output = Command::new("bash")
        .arg("auto-setup-system-audio.sh")
        .output()
        .map_err(|e| anyhow!("Failed to run setup script: {}", e))?;

    if output.status.success() {
        Ok("System audio setup completed successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Setup failed: {}", stderr))
    }
}

async fn internal_start_daemon() -> Result<String> {
    let output = Command::new("sudo")
        .args(&["launchctl", "load", "/Library/LaunchAgents/com.savant.audio.daemon.plist"])
        .output()
        .map_err(|e| anyhow!("Failed to start daemon: {}", e))?;

    if output.status.success() {
        Ok("Audio capture daemon started".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to start daemon: {}", stderr))
    }
}

async fn internal_stop_daemon() -> Result<String> {
    let output = Command::new("sudo")
        .args(&["launchctl", "unload", "/Library/LaunchAgents/com.savant.audio.daemon.plist"])
        .output()
        .map_err(|e| anyhow!("Failed to stop daemon: {}", e))?;

    if output.status.success() {
        Ok("Audio capture daemon stopped".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to stop daemon: {}", stderr))
    }
}

async fn internal_list_captures() -> Result<Vec<CaptureInfo>> {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let capture_dir = format!("{}/savant-audio-captures", home_dir);
    
    let mut captures = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&capture_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let preview = content
                            .lines()
                            .take(3)
                            .collect::<Vec<_>>()
                            .join("\n");
                        
                        captures.push(CaptureInfo {
                            filename: path.file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            timestamp: format!("{:?}", metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)),
                            size_bytes: metadata.len(),
                            preview,
                        });
                    }
                }
            }
        }
    }
    
    // Sort by timestamp (newest first)
    captures.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(captures)
}

async fn internal_search_captures(query: &str) -> Result<Vec<String>> {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let capture_dir = format!("{}/savant-audio-captures", home_dir);
    
    let output = Command::new("grep")
        .args(&["-r", "-i", "-n", query, &format!("{}/*.md", capture_dir)])
        .output()
        .map_err(|e| anyhow!("Search failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<String> = stdout
        .lines()
        .map(|line| line.to_string())
        .collect();

    Ok(results)
}

async fn internal_get_logs(lines: u32) -> Result<Vec<String>> {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let log_file = format!("{}/savant-audio-daemon.log", home_dir);
    
    if !Path::new(&log_file).exists() {
        return Ok(vec!["No log file found".to_string()]);
    }

    let output = Command::new("tail")
        .args(&["-n", &lines.to_string(), &log_file])
        .output()
        .map_err(|e| anyhow!("Failed to read logs: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let logs: Vec<String> = stdout
        .lines()
        .map(|line| line.to_string())
        .collect();

    Ok(logs)
}