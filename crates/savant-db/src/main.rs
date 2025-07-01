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
    /// Semantic search across all conversations
    Search {
        /// Search query text
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Minimum similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        threshold: f32,
        /// Speaker filter
        #[arg(long)]
        speaker: Option<String>,
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
    /// Analyze conversation and extract insights
    Analyze {
        /// Conversation ID to analyze
        conversation_id: String,
    },
    /// Speaker management commands
    Speaker {
        #[command(subcommand)]
        command: SpeakerCommands,
    },
    /// Topic management commands
    Topic {
        #[command(subcommand)]
        command: TopicCommands,
    },
}

#[derive(Subcommand)]
enum SpeakerCommands {
    /// List all speakers
    List,
    /// Create a new speaker
    Create {
        /// Speaker name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Show speaker details and statistics
    Show {
        /// Speaker ID
        speaker_id: String,
    },
    /// Find potential duplicate speakers
    Duplicates,
    /// Merge two speakers
    Merge {
        /// Primary speaker ID (keep this one)
        primary: String,
        /// Secondary speaker ID (merge into primary)
        secondary: String,
    },
}

#[derive(Subcommand)]
enum TopicCommands {
    /// List topics for a conversation
    List {
        /// Conversation ID
        conversation_id: String,
    },
    /// Extract topics from conversation
    Extract {
        /// Conversation ID
        conversation_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut db = TranscriptDatabase::new(cli.db_path).await?;
    
    // Initialize enhanced features
    db.init_speaker_identification().await?;
    db.init_semantic_search().await?;

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

        Commands::Search { query, limit, threshold, speaker } => {
            let results = db.semantic_search(&query, limit, threshold).await?;
            
            if results.is_empty() {
                println!("No results found for query: \"{}\"", query);
            } else {
                println!("Found {} results for query: \"{}\"", results.len(), query);
                println!("{}", "─".repeat(80));
                
                for result in results {
                    if let Some(ref speaker_filter) = speaker {
                        if result.speaker_id.as_ref().map_or(true, |s| s != speaker_filter) {
                            continue;
                        }
                    }
                    
                    println!("Similarity: {:.3} | Speaker: {} | Time: {}", 
                             result.similarity_score,
                             result.speaker_id.unwrap_or_else(|| "Unknown".to_string()),
                             result.timestamp.format("%Y-%m-%d %H:%M:%S"));
                    println!("📝 Text: {}", result.text);
                    
                    if let Some(context) = result.context_before {
                        println!("Context: {}", context);
                    }
                    if let Some(context) = result.context_after {
                        println!("Context: {}", context);
                    }
                    
                    println!("{}", "─".repeat(80));
                }
            }
        }

        Commands::Analyze { conversation_id } => {
            let analysis = db.analyze_conversation(&conversation_id).await?;
            
            println!("Conversation Analysis: {}", conversation_id);
            println!("{}", "═".repeat(60));
            println!("📝 Summary: {}", analysis.summary);
            println!("Duration: {:.1} seconds", analysis.duration);
            println!("Participants: {}", analysis.participant_count);
            println!("Quality Score: {:.2}", analysis.quality_score);
            println!("Sentiment: {:.2} ({})", 
                     analysis.sentiment_score,
                     if analysis.sentiment_score > 0.1 { "Positive" } 
                     else if analysis.sentiment_score < -0.1 { "Negative" } 
                     else { "Neutral" });
            
            if !analysis.topics.is_empty() {
                println!("🏷️  Topics: {}", analysis.topics.join(", "));
            }
            
            if !analysis.key_phrases.is_empty() {
                println!("🔑 Key Phrases: {}", analysis.key_phrases.join(", "));
            }
        }

        Commands::Speaker { command } => {
            match command {
                SpeakerCommands::List => {
                    let speakers = db.list_speakers().await?;
                    
                    if speakers.is_empty() {
                        println!("No speakers found in database");
                    } else {
                        println!("{:<36} {:<20} {:<15} {:<10} {:<15}", 
                                 "ID", "Name", "Conversations", "Time", "Last Seen");
                        println!("{}", "─".repeat(100));
                        
                        for speaker in speakers {
                            let name = speaker.display_name.or(speaker.name).unwrap_or_else(|| "Unknown".to_string());
                            let last_seen = speaker.last_interaction
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_else(|| "Never".to_string());
                            
                            println!("{:<36} {:<20} {:<15} {:<10.1}s {:<15}", 
                                     speaker.id,
                                     name,
                                     speaker.total_conversations,
                                     speaker.total_conversation_time,
                                     last_seen);
                        }
                    }
                }
                
                SpeakerCommands::Create { name } => {
                    let speaker_id = db.create_speaker(name).await?;
                    println!("Created speaker: {}", speaker_id);
                }
                
                SpeakerCommands::Show { speaker_id } => {
                    let speakers = db.list_speakers().await?;
                    if let Some(speaker) = speakers.iter().find(|s| s.id == speaker_id) {
                        println!("Speaker Details");
                        println!("{}", "═".repeat(40));
                        println!("ID: {}", speaker.id);
                        println!("Name: {}", speaker.display_name.as_ref().or(speaker.name.as_ref()).unwrap_or(&"Unknown".to_string()));
                        println!("Total Conversations: {}", speaker.total_conversations);
                        println!("Total Time: {:.1} seconds", speaker.total_conversation_time);
                        println!("Confidence Threshold: {:.2}", speaker.confidence_threshold);
                        if let Some(last) = speaker.last_interaction {
                            println!("Last Interaction: {}", last.format("%Y-%m-%d %H:%M:%S"));
                        }
                        println!("Created: {}", speaker.created_at.format("%Y-%m-%d %H:%M:%S"));
                    } else {
                        println!("Speaker not found: {}", speaker_id);
                    }
                }
                
                SpeakerCommands::Duplicates => {
                    let duplicates = db.find_speaker_duplicates().await?;
                    
                    if duplicates.is_empty() {
                        println!("No potential duplicates found");
                    } else {
                        println!("Potential Speaker Duplicates");
                        println!("{}", "═".repeat(60));
                        
                        for (speaker_a, speaker_b, similarity) in duplicates {
                            println!("Similarity: {:.3}", similarity);
                            println!("  Speaker A: {}", speaker_a);
                            println!("  Speaker B: {}", speaker_b);
                            println!("{}", "─".repeat(40));
                        }
                    }
                }
                
                SpeakerCommands::Merge { primary, secondary } => {
                    db.merge_speakers(&primary, &secondary).await?;
                    println!("Merged speaker {} into {}", secondary, primary);
                }
            }
        }

        Commands::Topic { command } => {
            match command {
                TopicCommands::List { conversation_id } => {
                    let topics = db.get_conversation_topics(&conversation_id).await?;
                    
                    if topics.is_empty() {
                        println!("No topics found for conversation: {}", conversation_id);
                    } else {
                        println!("Topics for conversation: {}", conversation_id);
                        println!("{}", "═".repeat(50));
                        
                        for topic in topics {
                            println!("• {} (frequency: {})", topic.name, topic.frequency);
                            if let Some(desc) = topic.description {
                                println!("  {}", desc);
                            }
                        }
                    }
                }
                
                TopicCommands::Extract { conversation_id } => {
                    if let Some(engine) = db.semantic_engine() {
                        let topics = engine.extract_topics(&conversation_id).await?;
                        
                        if topics.is_empty() {
                            println!("No topics extracted for conversation: {}", conversation_id);
                        } else {
                            println!("Extracted topics for conversation: {}", conversation_id);
                            println!("{}", "═".repeat(50));
                            
                            for topic in topics {
                                println!("• {}", topic);
                            }
                        }
                    } else {
                        println!("Semantic engine not available for topic extraction");
                    }
                }
            }
        }
    }

    Ok(())
}