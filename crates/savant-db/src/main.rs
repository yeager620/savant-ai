//! # Savant Database CLI
//!
//! Command-line interface for managing transcription database

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use savant_db::{TranscriptDatabase, TranscriptQuery};
use serde_json;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "savant-db", about = "Manage Savant AI transcription database")]
struct Cli {
    /// Database file path (defaults to ~/.config/savant-ai/transcripts.db)
    #[arg(long)]
    db_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store transcription from JSON input (stdin or file)
    Store {
        /// Input file (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<PathBuf>,
        /// Conversation ID to add to (creates new if not provided)
        #[arg(short, long)]
        conversation: Option<String>,
        /// Title for new conversation
        #[arg(short, long)]
        title: Option<String>,
    },
    /// Query transcription segments
    Query {
        /// Conversation ID filter
        #[arg(long)]
        conversation: Option<String>,
        /// Speaker filter
        #[arg(long)]
        speaker: Option<String>,
        /// Search text content
        #[arg(long)]
        text: Option<String>,
        /// Start time filter (RFC3339 format)
        #[arg(long)]
        start: Option<String>,
        /// End time filter (RFC3339 format)
        #[arg(long)]
        end: Option<String>,
        /// Limit results
        #[arg(long, default_value = "50")]
        limit: i64,
        /// Offset for pagination
        #[arg(long, default_value = "0")]
        offset: i64,
    },
    /// List conversations
    List {
        /// Limit number of conversations
        #[arg(long, default_value = "20")]
        limit: i64,
    },
    /// Show conversation statistics by speaker
    Stats,
    /// Export conversation to JSON
    Export {
        /// Conversation ID to export
        conversation_id: String,
        /// Output file (prints to stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Create a new conversation
    Create {
        /// Conversation title
        #[arg(short, long)]
        title: Option<String>,
        /// Conversation context/description
        #[arg(short, long)]
        context: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = TranscriptDatabase::new(cli.db_path).await?;

    match cli.command {
        Commands::Store { input, conversation, title } => {
            let input_reader: Box<dyn BufRead> = if let Some(path) = input {
                Box::new(BufReader::new(std::fs::File::open(path)?))
            } else {
                Box::new(io::stdin().lock())
            };

            for line in input_reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                let result: savant_stt::TranscriptionResult = serde_json::from_str(&line)?;
                
                let conv_id = if let Some(id) = &conversation {
                    id.clone()
                } else {
                    db.create_conversation(title.as_deref(), None).await?
                };

                let stored_id = db.store_transcription(&result, Some(conv_id)).await?;
                println!("Stored transcription in conversation: {}", stored_id);
            }
        }

        Commands::Query { conversation, speaker, text, start, end, limit, offset } => {
            let start_time = if let Some(s) = start {
                Some(DateTime::parse_from_rfc3339(&s)?.with_timezone(&Utc))
            } else {
                None
            };

            let end_time = if let Some(e) = end {
                Some(DateTime::parse_from_rfc3339(&e)?.with_timezone(&Utc))
            } else {
                None
            };

            let query = TranscriptQuery {
                conversation_id: conversation,
                speaker,
                text_contains: text,
                start_time,
                end_time,
                limit: Some(limit),
                offset: Some(offset),
                ..Default::default()
            };

            let segments = db.query_segments(&query).await?;
            
            for segment in segments {
                println!("{}", serde_json::to_string_pretty(&segment)?);
            }
        }

        Commands::List { limit } => {
            let conversations = db.list_conversations(Some(limit)).await?;
            
            println!("{:<36} {:<20} {:<20} {:<10} {:<10}", 
                     "ID", "Title", "Start Time", "Segments", "Duration");
            println!("{}", "─".repeat(100));
            
            for conv in conversations {
                let title = conv.title.unwrap_or_else(|| "Untitled".to_string());
                let title = if title.len() > 18 { 
                    format!("{}...", &title[..15]) 
                } else { 
                    title 
                };
                
                println!("{:<36} {:<20} {:<20} {:<10} {:<10.1}s", 
                         conv.id,
                         title,
                         conv.start_time.format("%Y-%m-%d %H:%M"),
                         conv.segment_count,
                         conv.total_duration);
            }
        }

        Commands::Stats => {
            let stats = db.get_speaker_stats().await?;
            
            println!("{:<20} {:<15} {:<15} {:<10} {:<10}", 
                     "Speaker", "Conversations", "Total Time", "Segments", "Avg Conf");
            println!("{}", "─".repeat(80));
            
            for stat in stats {
                println!("{:<20} {:<15} {:<15.1}s {:<10} {:<10.2}", 
                         stat.speaker,
                         stat.conversation_count,
                         stat.total_duration_seconds,
                         stat.total_segments,
                         stat.avg_confidence);
            }
        }

        Commands::Export { conversation_id, output } => {
            let export_data = db.export_conversation(&conversation_id).await?;
            let json_output = serde_json::to_string_pretty(&export_data)?;
            
            if let Some(path) = output {
                std::fs::write(path, json_output)?;
                println!("Exported conversation {} to file", conversation_id);
            } else {
                println!("{}", json_output);
            }
        }

        Commands::Create { title, context } => {
            let conversation_id = db.create_conversation(title.as_deref(), context.as_deref()).await?;
            println!("Created conversation: {}", conversation_id);
        }
    }

    Ok(())
}