use anyhow::Result;
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::analyzer::{EnhancedVideoAnalyzer, VideoAnalysisResult};
use crate::config::CaptureConfig;
use crate::{FrameMetadata, VideoFrame};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub frames_processed: u64,
    pub frames_compressed: u64,
    pub frames_analyzed: u64,
    pub total_processing_time_ms: u64,
    pub compression_ratio: f32,
    pub storage_saved_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedFrame {
    pub original_frame: VideoFrame,
    pub compressed_path: PathBuf,
    pub compression_ratio: f32,
    pub original_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub processing_result: Option<VideoAnalysisResult>,
}

#[derive(Debug)]
pub enum ProcessingCommand {
    ProcessFrame { frame: VideoFrame, image_data: Vec<u8> },
    Stop,
}

#[derive(Debug)]
pub enum ProcessingEvent {
    FrameProcessed(CompressedFrame),
    ProcessingComplete,
    Error(String),
}

pub struct VideoProcessor {
    config: CaptureConfig,
    analyzer: Option<EnhancedVideoAnalyzer>,
    stats: ProcessingStats,
    frame_counter: u64,
    processing_queue: HashMap<String, (VideoFrame, Vec<u8>)>,
}

impl VideoProcessor {
    pub fn new(config: CaptureConfig) -> Result<Self> {
        let analyzer = if config.enable_processing {
            Some(EnhancedVideoAnalyzer::new()?)
        } else {
            None
        };

        Ok(Self {
            config,
            analyzer,
            stats: ProcessingStats {
                frames_processed: 0,
                frames_compressed: 0,
                frames_analyzed: 0,
                total_processing_time_ms: 0,
                compression_ratio: 0.0,
                storage_saved_bytes: 0,
            },
            frame_counter: 0,
            processing_queue: HashMap::new(),
        })
    }

    pub async fn start_processing(
        &mut self,
        mut receiver: mpsc::Receiver<ProcessingCommand>,
        sender: mpsc::Sender<ProcessingEvent>,
    ) -> Result<()> {
        info!("Starting video processing pipeline");

        while let Some(command) = receiver.recv().await {
            match command {
                ProcessingCommand::ProcessFrame { frame, image_data } => {
                    if let Err(e) = self.process_frame(frame, image_data, &sender).await {
                        error!("Failed to process frame: {}", e);
                        let _ = sender.send(ProcessingEvent::Error(e.to_string())).await;
                    }
                }
                ProcessingCommand::Stop => {
                    info!("Stopping video processing pipeline");
                    let _ = sender.send(ProcessingEvent::ProcessingComplete).await;
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_frame(
        &mut self,
        frame: VideoFrame,
        image_data: Vec<u8>,
        sender: &mpsc::Sender<ProcessingEvent>,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        self.frame_counter += 1;

        debug!("Processing frame {} ({})", self.frame_counter, frame.id);

        // Load image for processing
        let image = image::load_from_memory(&image_data)?;
        
        // Compress image if auto_compress is enabled
        let compressed_frame = if self.config.auto_compress {
            self.compress_frame(&frame, &image, &image_data).await?
        } else {
            CompressedFrame {
                original_frame: frame.clone(),
                compressed_path: frame.file_path.clone(),
                compression_ratio: 1.0,
                original_size_bytes: image_data.len() as u64,
                compressed_size_bytes: image_data.len() as u64,
                processing_result: None,
            }
        };

        // Perform analysis if enabled and it's time to process
        let mut final_frame = compressed_frame;
        if self.config.enable_processing 
            && self.frame_counter % self.config.processing_interval as u64 == 0
            && self.analyzer.is_some() {
            
            debug!("Analyzing frame {} with OCR and vision", self.frame_counter);
            final_frame.processing_result = self.analyze_frame(&image, &frame.metadata).await?;
            self.stats.frames_analyzed += 1;
        }

        // Update statistics
        let processing_time = start_time.elapsed().as_millis() as u64;
        self.stats.frames_processed += 1;
        self.stats.total_processing_time_ms += processing_time;

        if self.config.auto_compress {
            self.stats.frames_compressed += 1;
            self.stats.storage_saved_bytes += final_frame.original_size_bytes - final_frame.compressed_size_bytes;
            
            // Update compression ratio (running average)
            let new_ratio = final_frame.compression_ratio;
            self.stats.compression_ratio = if self.stats.frames_compressed == 1 {
                new_ratio
            } else {
                (self.stats.compression_ratio * (self.stats.frames_compressed - 1) as f32 + new_ratio) 
                    / self.stats.frames_compressed as f32
            };
        }

        // Send processed frame
        let _ = sender.send(ProcessingEvent::FrameProcessed(final_frame)).await;

        debug!(
            "Frame {} processed in {}ms (compression: {:.1}x)",
            self.frame_counter, 
            processing_time,
            if self.config.auto_compress { final_frame.compression_ratio } else { 1.0 }
        );

        Ok(())
    }

    async fn compress_frame(
        &self,
        frame: &VideoFrame,
        image: &DynamicImage,
        original_data: &[u8],
    ) -> Result<CompressedFrame> {
        let original_size = original_data.len() as u64;
        
        // Resize if max_resolution is set
        let processed_image = if let Some((max_width, max_height)) = self.config.max_resolution {
            let (width, height) = (image.width(), image.height());
            
            if width > max_width || height > max_height {
                let ratio = f32::min(
                    max_width as f32 / width as f32,
                    max_height as f32 / height as f32,
                );
                let new_width = (width as f32 * ratio) as u32;
                let new_height = (height as f32 * ratio) as u32;
                
                debug!("Resizing frame from {}x{} to {}x{}", width, height, new_width, new_height);
                image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
            } else {
                image.clone()
            }
        } else {
            image.clone()
        };

        // Create compressed filename
        let original_path = &frame.file_path;
        let compressed_path = if let Some(parent) = original_path.parent() {
            let stem = original_path.file_stem().unwrap_or_default();
            let extension = original_path.extension().unwrap_or_default();
            parent.join(format!("{}_compressed.{}", 
                stem.to_string_lossy(), 
                extension.to_string_lossy()))
        } else {
            original_path.with_extension("compressed.png")
        };

        // Save compressed image
        let jpeg_quality = self.config.quality.jpeg_quality();
        let mut compressed_data = Vec::new();
        
        // Use JPEG for better compression
        processed_image.write_to(
            &mut std::io::Cursor::new(&mut compressed_data),
            ImageFormat::Jpeg
        )?;

        // Write to file
        tokio::fs::write(&compressed_path, &compressed_data).await?;
        
        let compressed_size = compressed_data.len() as u64;
        let compression_ratio = original_size as f32 / compressed_size as f32;

        debug!(
            "Compressed frame: {} -> {} bytes (ratio: {:.1}x)",
            original_size, compressed_size, compression_ratio
        );

        Ok(CompressedFrame {
            original_frame: frame.clone(),
            compressed_path,
            compression_ratio,
            original_size_bytes: original_size,
            compressed_size_bytes: compressed_size,
            processing_result: None,
        })
    }

    async fn analyze_frame(
        &self,
        image: &DynamicImage,
        metadata: &FrameMetadata,
    ) -> Result<Option<VideoAnalysisResult>> {
        if let Some(analyzer) = &self.analyzer {
            match analyzer.analyze_frame(image, metadata).await {
                Ok(result) => {
                    debug!(
                        "Frame analysis completed in {}ms: {} text blocks, {} apps detected",
                        result.processing_stats.total_processing_time_ms,
                        result.text_summary.total_text_blocks,
                        result.application_context.secondary_applications.len() + 
                            if result.application_context.primary_application.is_some() { 1 } else { 0 }
                    );
                    Ok(Some(result))
                }
                Err(e) => {
                    warn!("Frame analysis failed: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_stats(&self) -> &ProcessingStats {
        &self.stats
    }

    pub fn get_frame_counter(&self) -> u64 {
        self.frame_counter
    }
}

/// Utility function to create a processing pipeline
pub fn create_processing_pipeline(
    config: CaptureConfig,
) -> Result<(
    mpsc::Sender<ProcessingCommand>,
    mpsc::Receiver<ProcessingEvent>,
    tokio::task::JoinHandle<Result<()>>,
)> {
    let (cmd_sender, cmd_receiver) = mpsc::channel::<ProcessingCommand>(100);
    let (event_sender, event_receiver) = mpsc::channel::<ProcessingEvent>(100);

    let mut processor = VideoProcessor::new(config)?;
    
    let handle = tokio::spawn(async move {
        processor.start_processing(cmd_receiver, event_sender).await
    });

    Ok((cmd_sender, event_receiver, handle))
}

/// Batch processing function for existing PNG files
pub async fn batch_process_existing_files(
    input_dir: impl AsRef<Path>,
    config: CaptureConfig,
    progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<Vec<CompressedFrame>> {
    let input_path = input_dir.as_ref();
    info!("Starting batch processing of directory: {}", input_path.display());

    // Find all PNG files
    let mut png_files = Vec::new();
    let mut dir_reader = tokio::fs::read_dir(input_path).await?;
    
    while let Some(entry) = dir_reader.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
            // Skip already compressed files
            if !path.file_name().unwrap_or_default().to_string_lossy().contains("_compressed") {
                png_files.push(path);
            }
        }
    }

    png_files.sort();
    info!("Found {} PNG files to process", png_files.len());

    let mut processor = VideoProcessor::new(config)?;
    let mut results = Vec::new();

    for (i, png_path) in png_files.iter().enumerate() {
        if let Some(callback) = &progress_callback {
            callback(i + 1, png_files.len());
        }

        // Load image data
        let image_data = tokio::fs::read(png_path).await?;
        let image = image::load_from_memory(&image_data)?;

        // Create minimal frame metadata
        let frame = VideoFrame {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            file_path: png_path.clone(),
            resolution: (image.width(), image.height()),
            file_size: image_data.len() as u64,
            image_hash: format!("{:x}", md5::compute(&image_data)),
            metadata: FrameMetadata {
                session_id: "batch_processing".to_string(),
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
        };

        // Process frame
        let compressed_frame = processor.compress_frame(&frame, &image, &image_data).await?;
        
        // Optionally analyze frame
        let mut final_frame = compressed_frame;
        if config.enable_processing && (i + 1) % config.processing_interval as usize == 0 {
            final_frame.processing_result = processor.analyze_frame(&image, &frame.metadata).await?;
        }

        results.push(final_frame);

        debug!("Processed file {}/{}: {}", i + 1, png_files.len(), png_path.display());
    }

    info!(
        "Batch processing complete: {} files processed, {:.1}MB saved",
        results.len(),
        processor.get_stats().storage_saved_bytes as f32 / 1024.0 / 1024.0
    );

    Ok(results)
}