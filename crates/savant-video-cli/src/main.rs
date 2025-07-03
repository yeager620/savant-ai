use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use savant_video::{
    create_video_capture, CaptureConfig, CaptureEvent, FrameMetadata, PrivacyController,
    PrivacySettings, StorageManager, StorageSettings, VideoConfig, VideoFrame, VideoSession,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "savant-video",
    about = "Video capture daemon with privacy controls and stealth mode"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start video capture
    Start {
        /// Capture interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u32,

        /// Duration in seconds (0 for continuous)
        #[arg(short, long, default_value = "0")]
        duration: u32,

        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Disable stealth mode
        #[arg(long)]
        no_stealth: bool,
    },

    /// Stop video capture
    Stop,

    /// Get capture status
    Status,

    /// Extract text from recent captures
    Ocr {
        /// Time range (e.g., "1 hour ago", "today")
        #[arg(long)]
        since: Option<String>,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// List recent captures
    List {
        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Search captured text
    Search {
        /// Search query
        query: String,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Configure privacy settings
    Config {
        /// Set recording schedule (e.g., "09:00-17:00")
        #[arg(long)]
        schedule: Option<String>,

        /// Add blocked application
        #[arg(long)]
        block_app: Option<String>,

        /// Remove blocked application
        #[arg(long)]
        unblock_app: Option<String>,
    },

    /// Clean up old captures
    Cleanup {
        /// Delete captures older than N days
        #[arg(long)]
        older_than: u32,
    },

    /// Export session data
    Export {
        /// Session ID
        #[arg(long)]
        session: String,

        /// Export format (json, frames)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "savant_video=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            interval,
            duration,
            format,
            no_stealth,
        } => {
            start_capture(interval, duration, format, !no_stealth).await?;
        }
        Commands::Stop => {
            stop_capture().await?;
        }
        Commands::Status => {
            get_status().await?;
        }
        Commands::Ocr { since, limit } => {
            extract_text(since, limit).await?;
        }
        Commands::List { limit } => {
            list_captures(limit).await?;
        }
        Commands::Search { query, limit } => {
            search_captures(&query, limit).await?;
        }
        Commands::Config {
            schedule,
            block_app,
            unblock_app,
        } => {
            configure_privacy(schedule, block_app, unblock_app).await?;
        }
        Commands::Cleanup { older_than } => {
            cleanup_old_captures(older_than).await?;
        }
        Commands::Export {
            session,
            format,
            output,
        } => {
            export_session(&session, &format, output).await?;
        }
    }

    Ok(())
}

async fn start_capture(
    interval_seconds: u32,
    duration_seconds: u32,
    format: String,
    stealth_mode: bool,
) -> Result<()> {
    info!("Starting video capture with interval {}s", interval_seconds);

    // Create capture instance
    let capture = create_video_capture()?;
    capture.set_stealth_mode(stealth_mode).await?;

    // Initialize storage
    let storage_settings = StorageSettings::default();
    let storage = Arc::new(StorageManager::new(storage_settings));
    storage.initialize().await?;

    // Initialize privacy controller
    let privacy_settings = PrivacySettings::default();
    let privacy = Arc::new(Mutex::new(PrivacyController::new(privacy_settings)));

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let session = VideoSession {
        id: session_id.clone(),
        start_time: chrono::Utc::now(),
        end_time: None,
        frame_count: 0,
        total_size_bytes: 0,
        config: CaptureConfig {
            interval_milliseconds: interval_seconds * 1000, // Convert seconds to milliseconds
            enabled_hours: None,
            quality: savant_video::config::ImageQuality::Medium,
            notify_user: true,
            stealth_mode,
            continuous_mode: true,
            auto_compress: true,
            max_resolution: Some((1920, 1080)),
            enable_processing: true,
            processing_interval: 1,
            change_detection_threshold: 0.05,
            enable_full_text_extraction: true,
            enable_real_time_analysis: true,
            buffer_size: 10,
        },
    };

    // Save session metadata
    let session_json = json!({
        "session": session,
        "started_at": chrono::Utc::now(),
    });
    storage.save_metadata(&session_id, &session_json).await?;

    // Capture loop
    let mut interval_timer = interval(Duration::from_secs(interval_seconds as u64));
    let start_time = std::time::Instant::now();
    let mut frame_count = 0u32;

    loop {
        interval_timer.tick().await;

        // Check duration limit
        if duration_seconds > 0
            && start_time.elapsed().as_secs() >= duration_seconds as u64
        {
            info!("Duration limit reached, stopping capture");
            break;
        }

        // Check privacy settings
        let app_info = capture.get_active_application().await?;
        let app_name = app_info.as_ref().map(|a| a.name.as_str());

        let should_capture = privacy.lock().await.should_capture(app_name);
        if !should_capture {
            info!("Skipping capture due to privacy settings");
            continue;
        }

        // Capture screenshot
        match capture.capture_screen().await {
            Ok(screen_capture) => {
                // Convert to PNG bytes
                let mut png_bytes = Vec::new();
                {
                    let mut cursor = std::io::Cursor::new(&mut png_bytes);
                    screen_capture
                        .image
                        .write_to(&mut cursor, image::ImageFormat::Png)?;
                }

                // Calculate image hash
                let mut hasher = Sha256::new();
                hasher.update(&png_bytes);
                let hash = hex::encode(hasher.finalize());

                // Save frame
                let file_path = storage.save_frame(&session_id, &png_bytes).await?;
                frame_count += 1;

                // Create frame metadata
                let frame = VideoFrame {
                    id: Uuid::new_v4().to_string(),
                    timestamp: screen_capture.timestamp,
                    file_path: file_path.clone(),
                    resolution: (
                        screen_capture.image.width(),
                        screen_capture.image.height(),
                    ),
                    file_size: png_bytes.len() as u64,
                    image_hash: hash,
                    metadata: FrameMetadata {
                        session_id: session_id.clone(),
                        display_id: screen_capture.display_id,
                        active_application: app_info.as_ref().map(|a| a.name.clone()),
                        window_title: app_info.and_then(|a| a.window_title),
                        change_detected: true, // TODO: Implement change detection
                        ocr_text: None,        // TODO: Implement OCR
                        enhanced_analysis: None,
                        detected_applications: Vec::new(),
                        activity_classification: None,
                        visual_context: None,
                    },
                };

                // Output frame info
                if format == "json" {
                    println!("{}", serde_json::to_string(&frame)?);
                } else {
                    println!(
                        "Frame {}: {} - {}",
                        frame_count,
                        frame.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        file_path.display()
                    );
                }

                // Check if we should notify
                if privacy.lock().await.should_notify() {
                    info!("Video capture is active - {} frames captured", frame_count);
                }
            }
            Err(e) => {
                error!("Failed to capture screen: {}", e);
            }
        }
    }

    // Update session metadata
    let final_session = VideoSession {
        end_time: Some(chrono::Utc::now()),
        frame_count,
        ..session
    };
    storage
        .save_metadata(&session_id, &serde_json::to_value(final_session)?)
        .await?;

    info!("Video capture stopped. {} frames captured", frame_count);
    Ok(())
}

async fn stop_capture() -> Result<()> {
    // TODO: Implement daemon stop via PID file
    println!("Stopping video capture daemon...");
    Ok(())
}

async fn get_status() -> Result<()> {
    // TODO: Check daemon status via PID file
    let status = json!({
        "running": false,
        "message": "Video capture daemon not running"
    });
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

async fn extract_text(_since: Option<String>, _limit: Option<usize>) -> Result<()> {
    // TODO: Implement OCR extraction
    println!("OCR extraction not yet implemented");
    Ok(())
}

async fn list_captures(limit: usize) -> Result<()> {
    // TODO: List captures from storage
    println!("Listing {} most recent captures:", limit);
    Ok(())
}

async fn search_captures(query: &str, limit: usize) -> Result<()> {
    // TODO: Search captures using database
    println!("Searching for '{}' (limit: {})", query, limit);
    Ok(())
}

async fn configure_privacy(
    schedule: Option<String>,
    block_app: Option<String>,
    unblock_app: Option<String>,
) -> Result<()> {
    // TODO: Update privacy configuration
    if let Some(schedule) = schedule {
        println!("Setting recording schedule: {}", schedule);
    }
    if let Some(app) = block_app {
        println!("Blocking application: {}", app);
    }
    if let Some(app) = unblock_app {
        println!("Unblocking application: {}", app);
    }
    Ok(())
}

async fn cleanup_old_captures(days: u32) -> Result<()> {
    let storage = StorageManager::new(StorageSettings::default());
    storage.initialize().await?;

    println!("Cleaning up captures older than {} days...", days);
    storage.cleanup_old_files().await?;

    let usage = storage.get_storage_usage().await?;
    println!("Current storage usage: {} MB", usage / 1024 / 1024);
    Ok(())
}

async fn export_session(session_id: &str, format: &str, output: Option<PathBuf>) -> Result<()> {
    // TODO: Export session data
    println!(
        "Exporting session {} in {} format",
        session_id, format
    );
    if let Some(output_path) = output {
        println!("Output directory: {}", output_path.display());
    }
    Ok(())
}
