use anyhow::Result;
use clap::{Parser, Subcommand};
use chrono::{DateTime, Utc, Duration};
use savant_sync::{
    MultimodalSyncManager, SyncManagerConfig, VideoEvent, AudioEvent, 
    VideoEventType, AudioEventType, VideoEventMetadata, AudioEventMetadata,
    CorrelationAlgorithm, SyncWindow,
};
use serde_json;
use std::io::{self, BufRead};
use tokio;

#[derive(Parser)]
#[command(name = "savant-sync")]
#[command(about = "Multimodal event synchronization and correlation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Correlate audio and video events from stdin
    Correlate {
        /// Time window size in seconds
        #[arg(short, long, default_value = "30")]
        window_size: u32,
        
        /// Minimum correlation strength
        #[arg(long, default_value = "0.3")]
        min_strength: f32,
        
        /// Maximum time offset in milliseconds
        #[arg(long, default_value = "5000")]
        max_offset: i64,
        
        /// Correlation algorithms to use
        #[arg(long, value_delimiter = ',', default_values = ["temporal", "semantic"])]
        algorithms: Vec<String>,
    },
    
    /// Analyze multimodal context for a time window
    Analyze {
        /// Start timestamp (ISO 8601 format)
        #[arg(short, long)]
        start: String,
        
        /// End timestamp (ISO 8601 format)
        #[arg(short, long)]
        end: String,
        
        /// Output format (json, summary)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    
    /// Test synchronization capabilities
    Test {
        /// Number of test events to generate
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Correlate { 
            window_size, 
            min_strength, 
            max_offset, 
            algorithms 
        } => {
            correlate_events(window_size, min_strength, max_offset, algorithms).await?;
        }
        Commands::Analyze { start, end, format } => {
            analyze_context(start, end, format).await?;
        }
        Commands::Test { count } => {
            test_sync(count).await?;
        }
    }
    
    Ok(())
}

async fn correlate_events(
    window_size: u32,
    min_strength: f32,
    max_offset: i64,
    algorithms: Vec<String>,
) -> Result<()> {
    let correlation_algorithms = algorithms.into_iter()
        .filter_map(|alg| match alg.as_str() {
            "temporal" => Some(CorrelationAlgorithm::TemporalProximity),
            "semantic" => Some(CorrelationAlgorithm::SemanticSimilarity),
            "causal" => Some(CorrelationAlgorithm::CausalInference),
            "pattern" => Some(CorrelationAlgorithm::PatternMatching),
            "statistical" => Some(CorrelationAlgorithm::StatisticalCorrelation),
            _ => None,
        })
        .collect();

    let config = SyncManagerConfig {
        default_window_size_seconds: window_size,
        min_correlation_strength: min_strength,
        max_time_offset_ms: max_offset,
        correlation_algorithms,
        ..Default::default()
    };

    let sync_manager = MultimodalSyncManager::new(config);

    // Read events from stdin
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Try to parse as video or audio event
        if let Ok(video_event) = serde_json::from_str::<VideoEvent>(&line) {
            sync_manager.add_video_event(video_event).await?;
        } else if let Ok(audio_event) = serde_json::from_str::<AudioEvent>(&line) {
            sync_manager.add_audio_event(audio_event).await?;
        } else {
            eprintln!("Warning: Could not parse event: {}", line);
        }
    }

    // Get synchronized context for the current time window
    let now = Utc::now();
    if let Some(context) = sync_manager.get_synchronized_context(now).await? {
        println!("{}", serde_json::to_string_pretty(&context)?);
    } else {
        println!("{{\"message\": \"No events to correlate\"}}");
    }

    Ok(())
}

async fn analyze_context(start: String, end: String, format: String) -> Result<()> {
    let start_time: DateTime<Utc> = start.parse()
        .map_err(|e| anyhow::anyhow!("Invalid start timestamp: {}", e))?;
    let end_time: DateTime<Utc> = end.parse()
        .map_err(|e| anyhow::anyhow!("Invalid end timestamp: {}", e))?;

    let config = SyncManagerConfig::default();
    let sync_manager = MultimodalSyncManager::new(config);

    let contexts = sync_manager.get_context_timeline(start_time, end_time).await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&contexts)?);
        }
        "summary" => {
            println!("=== Multimodal Context Analysis ===");
            println!("Time period: {} to {}", start_time, end_time);
            println!("Windows analyzed: {}", contexts.len());
            
            let total_video_events: usize = contexts.iter().map(|c| c.video_events.len()).sum();
            let total_audio_events: usize = contexts.iter().map(|c| c.audio_events.len()).sum();
            let total_correlations: usize = contexts.iter().map(|c| c.correlations.len()).sum();
            
            println!("Total video events: {}", total_video_events);
            println!("Total audio events: {}", total_audio_events);
            println!("Total correlations: {}", total_correlations);
            
            if !contexts.is_empty() {
                let avg_sync_quality: f32 = contexts.iter()
                    .map(|c| c.confidence_scores.overall_sync_quality)
                    .sum::<f32>() / contexts.len() as f32;
                println!("Average sync quality: {:.2}", avg_sync_quality);
            }
        }
        _ => {
            anyhow::bail!("Unsupported format: {}", format);
        }
    }

    Ok(())
}

async fn test_sync(count: usize) -> Result<()> {
    println!("Testing multimodal synchronization with {} events...", count);
    
    let config = SyncManagerConfig::default();
    let sync_manager = MultimodalSyncManager::new(config);

    let now = Utc::now();
    
    // Generate test events
    for i in 0..count {
        let timestamp = now + Duration::seconds(i as i64 * 5);
        let event_id = format!("test_event_{}", i);
        
        // Create test video event
        let video_event = VideoEvent {
            event_id: format!("video_{}", event_id),
            timestamp,
            event_type: VideoEventType::ApplicationDetected,
            frame_id: Some(format!("frame_{}", i)),
            metadata: VideoEventMetadata {
                application_name: Some("Test App".to_string()),
                activity_type: Some("Testing".to_string()),
                text_content: Some(format!("Test content {}", i)),
                ui_elements: vec!["button".to_string(), "text_field".to_string()],
                change_score: Some(0.8),
            },
            confidence: 0.9,
        };
        
        // Create test audio event slightly offset
        let audio_event = AudioEvent {
            event_id: format!("audio_{}", event_id),
            timestamp: timestamp + Duration::milliseconds(500),
            event_type: AudioEventType::SpeechStarted,
            segment_id: Some(format!("segment_{}", i)),
            metadata: AudioEventMetadata {
                speaker_id: Some("test_speaker".to_string()),
                transcription: Some(format!("Test speech {}", i)),
                audio_source: Some("microphone".to_string()),
                volume_level: Some(0.75),
                audio_quality_score: Some(0.85),
                language: Some("en".to_string()),
            },
            confidence: 0.95,
        };
        
        sync_manager.add_video_event(video_event).await?;
        sync_manager.add_audio_event(audio_event).await?;
    }
    
    // Analyze the test window
    let window = SyncWindow::new(now, 60, 5);
    let context = sync_manager.synchronize_window(window).await?;
    
    println!("Test Results:");
    println!("- Video events: {}", context.video_events.len());
    println!("- Audio events: {}", context.audio_events.len());
    println!("- Correlations: {}", context.correlations.len());
    println!("- Fused insights: {}", context.fused_insights.len());
    println!("- Overall sync quality: {:.2}", context.confidence_scores.overall_sync_quality);
    println!("- Temporal alignment: {:.2}", context.confidence_scores.temporal_alignment);
    
    if !context.correlations.is_empty() {
        println!("\nCorrelation Details:");
        for correlation in &context.correlations {
            println!("  - {} → {} (strength: {:.2}, offset: {}ms)", 
                correlation.video_event_id, 
                correlation.audio_event_id,
                correlation.strength,
                correlation.time_offset_ms);
        }
    }
    
    if !context.fused_insights.is_empty() {
        println!("\nGenerated Insights:");
        for insight in &context.fused_insights {
            println!("  - {}: {} (confidence: {:.2})", 
                insight.insight_type.to_string(),
                insight.description,
                insight.confidence);
        }
    }
    
    Ok(())
}

// Helper trait for InsightType to string conversion
trait InsightTypeExt {
    fn to_string(&self) -> String;
}

impl InsightTypeExt for savant_sync::InsightType {
    fn to_string(&self) -> String {
        match self {
            savant_sync::InsightType::SpeakerIdentification => "Speaker Identification".to_string(),
            savant_sync::InsightType::ActivityTransition => "Activity Transition".to_string(),
            savant_sync::InsightType::ApplicationAudioMapping => "Application Audio Mapping".to_string(),
            savant_sync::InsightType::WorkflowPattern => "Workflow Pattern".to_string(),
            savant_sync::InsightType::ProductivityInsight => "Productivity Insight".to_string(),
            savant_sync::InsightType::CollaborationEvent => "Collaboration Event".to_string(),
            savant_sync::InsightType::LearningOpportunity => "Learning Opportunity".to_string(),
            savant_sync::InsightType::ProblemIndicator => "Problem Indicator".to_string(),
            savant_sync::InsightType::ContextSwitch => "Context Switch".to_string(),
            savant_sync::InsightType::MultitaskingDetected => "Multitasking Detected".to_string(),
        }
    }
}