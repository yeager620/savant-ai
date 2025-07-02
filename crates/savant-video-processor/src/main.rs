use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use savant_core::config::Config;
use savant_db::{TranscriptDatabase, visual_data::{VisualDataManager, VideoQuery}};
use savant_video::{
    CaptureConfig, VideoProcessor, ProcessingCommand, ProcessingEvent,
    processor::{batch_process_existing_files, create_processing_pipeline}
};

#[derive(Parser)]
#[command(name = "savant-video-processor")]
#[command(about = "Continuous video processing daemon and batch processor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start continuous processing daemon
    Daemon {
        /// Directory to monitor for new images
        #[arg(short, long)]
        input_dir: PathBuf,
        
        /// Processing interval in seconds
        #[arg(short = 'i', long, default_value = "5")]
        interval: u64,
        
        /// Enable OCR processing
        #[arg(long)]
        enable_ocr: bool,
        
        /// Enable vision analysis
        #[arg(long)]
        enable_vision: bool,
        
        /// Database path
        #[arg(short, long)]
        db_path: Option<PathBuf>,
    },
    
    /// Batch process existing PNG files
    Batch {
        /// Directory containing PNG files
        #[arg(short, long)]
        input_dir: PathBuf,
        
        /// Enable OCR processing
        #[arg(long)]
        enable_ocr: bool,
        
        /// Enable vision analysis
        #[arg(long)]
        enable_vision: bool,
        
        /// Process every Nth file for analysis
        #[arg(long, default_value = "3")]
        process_interval: u32,
        
        /// Database path
        #[arg(short, long)]
        db_path: Option<PathBuf>,
        
        /// Show progress
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Query processed video data
    Query {
        /// Database path
        #[arg(short, long)]
        db_path: Option<PathBuf>,
        
        /// Text to search for
        #[arg(short, long)]
        text: Option<String>,
        
        /// Application filter
        #[arg(short, long)]
        app: Option<String>,
        
        /// Start time (ISO 8601)
        #[arg(long)]
        start: Option<String>,
        
        /// End time (ISO 8601)
        #[arg(long)]
        end: Option<String>,
        
        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: i64,
        
        /// Output format (json, table)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Show statistics
    Stats {
        /// Database path
        #[arg(short, long)]
        db_path: Option<PathBuf>,
    },
    
    /// Clean up old data
    Cleanup {
        /// Database path
        #[arg(short, long)]
        db_path: Option<PathBuf>,
        
        /// Days to keep (delete older)
        #[arg(short, long, default_value = "30")]
        keep_days: i64,
        
        /// Dry run (don't actually delete)
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { 
            input_dir, 
            interval: interval_secs, 
            enable_ocr, 
            enable_vision, 
            db_path 
        } => {
            run_daemon(input_dir, interval_secs, enable_ocr, enable_vision, db_path).await
        }
        Commands::Batch { 
            input_dir, 
            enable_ocr, 
            enable_vision, 
            process_interval, 
            db_path, 
            verbose 
        } => {
            run_batch_processing(input_dir, enable_ocr, enable_vision, process_interval, db_path, verbose).await
        }
        Commands::Query { 
            db_path, 
            text, 
            app, 
            start, 
            end, 
            limit, 
            format 
        } => {
            run_query(db_path, text, app, start, end, limit, format).await
        }
        Commands::Stats { db_path } => {
            show_stats(db_path).await
        }
        Commands::Cleanup { db_path, keep_days, dry_run } => {
            run_cleanup(db_path, keep_days, dry_run).await
        }
    }
}

async fn run_daemon(
    input_dir: PathBuf,
    interval_secs: u64,
    enable_ocr: bool,
    enable_vision: bool,
    db_path: Option<PathBuf>,
) -> Result<()> {
    info!("Starting video processing daemon");
    info!("  Input directory: {}", input_dir.display());
    info!("  Processing interval: {}s", interval_secs);
    info!("  OCR enabled: {}", enable_ocr);
    info!("  Vision enabled: {}", enable_vision);

    // Initialize database
    let db = TranscriptDatabase::new(db_path).await?;
    let visual_db = VisualDataManager::new(db.pool.clone());

    // Create session
    let config = CaptureConfig {
        interval_seconds: interval_secs as u32,
        continuous_mode: true,
        auto_compress: true,
        max_resolution: Some((1400, 1050)),
        enable_processing: enable_ocr || enable_vision,
        processing_interval: 3,
        ..Default::default()
    };

    let session_id = visual_db.create_session(Some(&serde_json::to_string(&config)?)).await?;
    info!("Created processing session: {}", session_id);

    // Create processing pipeline
    let (cmd_sender, mut event_receiver, _handle) = create_processing_pipeline(config.clone())?;

    // Start file monitoring
    let input_dir_clone = input_dir.clone();
    let cmd_sender_clone = cmd_sender.clone();
    let monitoring_task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(interval_secs));
        let mut processed_files = std::collections::HashSet::new();

        loop {
            interval.tick().await;
            
            match scan_for_new_files(&input_dir_clone, &mut processed_files).await {
                Ok(new_files) => {
                    for (file_path, image_data) in new_files {
                        let frame = create_frame_from_file(&file_path, &session_id, &image_data).await;
                        let _ = cmd_sender_clone.send(ProcessingCommand::ProcessFrame {
                            frame,
                            image_data,
                        }).await;
                    }
                }
                Err(e) => {
                    error!("Error scanning for files: {}", e);
                }
            }
        }
    });

    // Handle processing events
    let visual_db = Arc::new(visual_db);
    let event_handling_task = {
        let visual_db = visual_db.clone();
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                match event {
                    ProcessingEvent::FrameProcessed(compressed_frame) => {
                        if let Err(e) = visual_db.store_compressed_frame(&compressed_frame).await {
                            error!("Failed to store frame: {}", e);
                        } else {
                            info!("Stored frame: {}", compressed_frame.original_frame.id);
                        }
                    }
                    ProcessingEvent::Error(err) => {
                        error!("Processing error: {}", err);
                    }
                    ProcessingEvent::ProcessingComplete => {
                        info!("Processing complete");
                        break;
                    }
                }
            }
        })
    };

    // Setup graceful shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal, stopping daemon");
            let _ = cmd_sender.send(ProcessingCommand::Stop).await;
        }
        _ = monitoring_task => {
            warn!("File monitoring task completed unexpectedly");
        }
        _ = event_handling_task => {
            warn!("Event handling task completed unexpectedly");
        }
    }

    // End session
    visual_db.end_session(&session_id).await?;
    info!("Video processing daemon stopped");

    Ok(())
}

async fn run_batch_processing(
    input_dir: PathBuf,
    enable_ocr: bool,
    enable_vision: bool,
    process_interval: u32,
    db_path: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    info!("Starting batch processing of directory: {}", input_dir.display());

    let config = CaptureConfig {
        auto_compress: true,
        max_resolution: Some((1400, 1050)),
        enable_processing: enable_ocr || enable_vision,
        processing_interval: process_interval,
        ..Default::default()
    };

    let progress_callback = if verbose {
        Some(Box::new(move |current: usize, total: usize| {
            println!("Processing: {}/{} ({:.1}%)", current, total, (current as f32 / total as f32) * 100.0);
        }) as Box<dyn Fn(usize, usize) + Send + Sync>)
    } else {
        None
    };

    let results = batch_process_existing_files(input_dir, config, progress_callback).await?;

    info!("Batch processing complete: {} files processed", results.len());

    // Store results in database if provided
    if let Some(_db_path) = db_path {
        let db = TranscriptDatabase::new(db_path).await?;
        let visual_db = VisualDataManager::new(db.pool.clone());
        let session_id = visual_db.create_session(Some("batch_processing")).await?;

        for compressed_frame in results {
            if let Err(e) = visual_db.store_compressed_frame(&compressed_frame).await {
                error!("Failed to store frame: {}", e);
            }
        }

        visual_db.end_session(&session_id).await?;
        info!("Results stored in database");
    }

    Ok(())
}

async fn run_query(
    db_path: Option<PathBuf>,
    text: Option<String>,
    app: Option<String>,
    start: Option<String>,
    end: Option<String>,
    limit: i64,
    format: String,
) -> Result<()> {
    let db = TranscriptDatabase::new(db_path).await?;
    let visual_db = VisualDataManager::new(db.pool.clone());

    let start_time = if let Some(s) = start {
        Some(chrono::DateTime::parse_from_rfc3339(&s)?.with_timezone(&chrono::Utc))
    } else {
        None
    };

    let end_time = if let Some(e) = end {
        Some(chrono::DateTime::parse_from_rfc3339(&e)?.with_timezone(&chrono::Utc))
    } else {
        None
    };

    let query = VideoQuery {
        text_contains: text.clone(),
        active_application: app,
        start_time,
        end_time,
        limit: Some(limit),
        ..Default::default()
    };

    let results = if let Some(text_query) = text {
        // Use full-text search if text is provided
        visual_db.search_text(&text_query, Some(limit)).await?
    } else {
        // Use frame query
        visual_db.query_frames(&query).await?
    };

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        "table" => {
            println!("{:<20} {:<20} {:<30} {:<50}", "Timestamp", "Application", "Frame ID", "Content");
            println!("{}", "-".repeat(120));
            
            for result in results {
                let timestamp = result["timestamp"].as_str().unwrap_or("N/A");
                let app = result["active_application"].as_str().unwrap_or("Unknown");
                let frame_id = result["frame_id"].as_str().unwrap_or(result["id"].as_str().unwrap_or("N/A"));
                let content = result["text_content"].as_str().unwrap_or("No text");
                
                println!("{:<20} {:<20} {:<30} {:<50}", 
                    &timestamp[..std::cmp::min(20, timestamp.len())],
                    &app[..std::cmp::min(20, app.len())],
                    &frame_id[..std::cmp::min(30, frame_id.len())],
                    &content[..std::cmp::min(50, content.len())]
                );
            }
        }
        _ => {
            error!("Unknown format: {}. Use 'json' or 'table'", format);
        }
    }

    Ok(())
}

async fn show_stats(db_path: Option<PathBuf>) -> Result<()> {
    let db = TranscriptDatabase::new(db_path).await?;
    let visual_db = VisualDataManager::new(db.pool.clone());

    let stats = visual_db.get_stats().await?;
    let app_usage = visual_db.get_application_usage(Some(10)).await?;
    let code_analysis = visual_db.get_code_analysis().await?;

    println!("Video Processing Statistics");
    println!("===========================");
    println!("Total frames: {}", stats.total_frames);
    println!("Total sessions: {}", stats.total_sessions);
    println!("Total storage: {:.2} MB", stats.total_size_bytes as f64 / 1024.0 / 1024.0);
    println!("Compression ratio: {:.1}x", stats.compression_ratio);
    println!("Frames with OCR: {}", stats.frames_with_ocr);
    println!("Frames with analysis: {}", stats.frames_with_analysis);
    println!("Unique applications: {}", stats.unique_applications);
    println!("Storage saved: {:.2} MB", stats.storage_saved_bytes as f64 / 1024.0 / 1024.0);

    if !app_usage.is_empty() {
        println!("\nTop Applications:");
        println!("{:<30} {:<10} {:<20}", "Application", "Frames", "Productivity");
        println!("{}", "-".repeat(60));
        for usage in app_usage.iter().take(10) {
            println!("{:<30} {:<10} {:<20.1}", 
                usage.application, 
                usage.frame_count,
                usage.avg_productivity
            );
        }
    }

    if !code_analysis.is_empty() {
        println!("\nCode Analysis:");
        println!("{:<20} {:<10} {:<15}", "Language", "Snippets", "Avg Complexity");
        println!("{}", "-".repeat(45));
        for analysis in code_analysis.iter().take(10) {
            println!("{:<20} {:<10} {:<15.1}", 
                analysis.programming_language, 
                analysis.snippet_count,
                analysis.avg_complexity
            );
        }
    }

    Ok(())
}

async fn run_cleanup(db_path: Option<PathBuf>, keep_days: i64, dry_run: bool) -> Result<()> {
    let db = TranscriptDatabase::new(db_path).await?;
    
    let cutoff_date = Utc::now() - chrono::Duration::days(keep_days);
    
    info!("Cleaning up data older than {} days (before {})", keep_days, cutoff_date);
    
    if dry_run {
        info!("DRY RUN - no data will be deleted");
    }

    // For now, just show what would be deleted
    let query = VideoQuery {
        end_time: Some(cutoff_date),
        limit: Some(1000),
        ..Default::default()
    };

    let visual_db = VisualDataManager::new(db.pool.clone());
    let old_frames = visual_db.query_frames(&query).await?;
    
    println!("Found {} frames to clean up", old_frames.len());
    
    if !dry_run && !old_frames.is_empty() {
        // TODO: Implement actual cleanup logic
        warn!("Cleanup implementation not yet complete");
    }

    Ok(())
}

async fn scan_for_new_files(
    input_dir: &PathBuf,
    processed_files: &mut std::collections::HashSet<PathBuf>,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut new_files = Vec::new();
    
    if !input_dir.exists() {
        return Ok(new_files);
    }

    let mut dir_reader = tokio::fs::read_dir(input_dir).await?;
    
    while let Some(entry) = dir_reader.next_entry().await? {
        let path = entry.path();
        
        if path.is_file() 
            && path.extension().and_then(|s| s.to_str()) == Some("png")
            && !path.file_name().unwrap_or_default().to_string_lossy().contains("_compressed")
            && !processed_files.contains(&path) {
            
            // Read file data
            match tokio::fs::read(&path).await {
                Ok(data) => {
                    new_files.push((path.clone(), data));
                    processed_files.insert(path);
                }
                Err(e) => {
                    warn!("Failed to read file {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(new_files)
}

async fn create_frame_from_file(
    file_path: &PathBuf,
    session_id: &str,
    image_data: &[u8],
) -> savant_video::VideoFrame {
    let image = image::load_from_memory(image_data).unwrap_or_else(|_| {
        image::DynamicImage::new_rgba8(1, 1)
    });

    savant_video::VideoFrame {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        file_path: file_path.clone(),
        resolution: (image.width(), image.height()),
        file_size: image_data.len() as u64,
        image_hash: format!("{:x}", md5::compute(image_data)),
        metadata: savant_video::FrameMetadata {
            session_id: session_id.to_string(),
            display_id: None,
            active_application: None,
            window_title: None,
            change_detected: true,
            ocr_text: None,
            enhanced_analysis: None,
            detected_applications: Vec::new(),
            activity_classification: None,
            visual_context: None,
        },
    }
}