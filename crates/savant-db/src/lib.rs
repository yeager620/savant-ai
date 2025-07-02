//! # Savant Database Library
//!
//! Database management for audio transcription data with rich querying capabilities

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::fs::File;
use std::path::PathBuf;
use uuid::Uuid;

pub use savant_stt::{TranscriptionResult, TranscriptionSegment, SessionMetadata, AudioSource};

pub mod speaker_identification;
pub mod semantic_search;
pub mod security;
pub mod natural_query;
pub mod llm_client;

pub use speaker_identification::{Speaker, SpeakerIdentifier, SpeakerMatch, MatchMethod};
pub use semantic_search::{SemanticSearchEngine, SearchResult, ConversationAnalysis, Topic};
pub use security::{QuerySecurityManager, SecurityError, QueryComplexity};
pub use natural_query::{NaturalLanguageQueryParser, QueryIntent, IntentType, QueryResult, QueryProcessor, ConversationContextManager, QueryOptimizer, UserFeedback, LLMQueryResult};
pub use llm_client::{LLMClient, LLMClientFactory, LLMConfig, OllamaClient, OpenAIClient, MockLLMClient};

/// Database connection manager with speaker identification and semantic search
pub struct TranscriptDatabase {
    pub pool: SqlitePool,
    speaker_identifier: Option<SpeakerIdentifier>,
    semantic_engine: Option<SemanticSearchEngine>,
}

/// Conversation record for grouping related segments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub participants: Vec<String>,
    pub context: Option<String>,
    pub segment_count: i64,
    pub total_duration: f64,
}

/// Query builder for complex searches
#[derive(Debug, Default)]
pub struct TranscriptQuery {
    pub conversation_id: Option<String>,
    pub speaker: Option<String>,
    pub audio_source: Option<AudioSource>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub text_contains: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Statistics about conversations
#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationStats {
    pub speaker: String,
    pub conversation_count: i64,
    pub total_duration_seconds: f64,
    pub total_segments: i64,
    pub avg_confidence: f64,
}

impl TranscriptDatabase {
    /// Create new database connection
    pub async fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("savant-ai");
            std::fs::create_dir_all(&path).ok();
            path.push("transcripts.db");
            path
        });

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Create the database file if it doesn't exist
        if !path.exists() {
            File::create(&path)?;
        }
        
        let database_url = format!("sqlite://{}", path.display());
        let pool = SqlitePool::connect(&database_url).await?;
        
        let db = Self { 
            pool: pool.clone(),
            speaker_identifier: Some(SpeakerIdentifier::new(pool.clone())),
            semantic_engine: Some(SemanticSearchEngine::new(pool)),
        };
        db.migrate().await?;
        
        // Engines will be initialized via separate methods
        
        Ok(db)
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
        // Create migration tracking table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(&self.pool).await?;
        
        // Check and run migrations individually
        self.run_migration("001", "../migrations/001_initial.sql").await?;
        self.run_migration("002", "../migrations/002_speaker_identification.sql").await?;
        // Skip complex migrations for now due to SQL parsing issues
        // self.run_migration("003", "../migrations/003_llm_integration.sql").await?;
        // self.run_migration("004", "../migrations/004_database_optimizations.sql").await?;
        
        Ok(())
    }
    
    /// Run a migration only if it hasn't been applied yet
    async fn run_migration(&self, version: &str, file_path: &str) -> Result<()> {
        // Check if migration has already been applied
        let applied = sqlx::query("SELECT version FROM schema_migrations WHERE version = ?")
            .bind(version)
            .fetch_optional(&self.pool)
            .await?;
            
        if applied.is_some() {
            return Ok(()); // Migration already applied
        }
        
        // Run the migration
        self.execute_migration_file(file_path).await?;
        
        // Mark migration as applied
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
            .bind(version)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Execute a migration file by splitting it into individual statements
    async fn execute_migration_file(&self, file_path: &str) -> Result<()> {
        let sql_content = match file_path {
            "../migrations/001_initial.sql" => include_str!("../migrations/001_initial.sql"),
            "../migrations/002_speaker_identification.sql" => include_str!("../migrations/002_speaker_identification.sql"),
            "../migrations/003_llm_integration.sql" => include_str!("../migrations/003_llm_integration.sql"),
            "../migrations/004_database_optimizations.sql" => include_str!("../migrations/004_database_optimizations.sql"),
            _ => return Err(anyhow::anyhow!("Unknown migration file: {}", file_path)),
        };
        
        // Parse SQL statements more carefully
        let statements = self.parse_sql_statements(sql_content);
        
        for statement in statements {
            if !statement.trim().is_empty() {
                if let Err(e) = sqlx::query(&statement).execute(&self.pool).await {
                    // Ignore "already exists" errors for idempotent migrations
                    if let Some(db_err) = e.as_database_error() {
                        let msg = db_err.message();
                        if msg.contains("duplicate column") || 
                           msg.contains("already exists") || 
                           msg.contains("table") && msg.contains("already exists") {
                            continue; // Skip "already exists" errors
                        }
                    }
                    return Err(e.into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse SQL statements from content, handling multi-line statements correctly
    fn parse_sql_statements(&self, content: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_begin_end = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and full-line comments
            if line.is_empty() || line.starts_with("--") {
                continue;
            }
            
            // Remove inline comments (anything after -- in the line)
            let clean_line = if let Some(pos) = line.find("--") {
                line[..pos].trim()
            } else {
                line
            };
            
            // Skip if line became empty after removing comments
            if clean_line.is_empty() {
                continue;
            }
            
            // Add line to current statement
            if !current_statement.is_empty() {
                current_statement.push(' ');
            }
            current_statement.push_str(clean_line);
            
            // Track BEGIN/END blocks for triggers and procedures
            if clean_line.contains("BEGIN") {
                in_begin_end = true;
            }
            if clean_line.contains("END") && in_begin_end {
                in_begin_end = false;
            }
            
            // Look for semicolon at end of cleaned line
            // Only end statement if we're not in a BEGIN/END block
            if clean_line.ends_with(';') && !in_begin_end {
                // Remove the semicolon and add the statement
                current_statement.pop(); // Remove semicolon
                statements.push(current_statement.trim().to_string());
                current_statement.clear();
            }
        }
        
        // Add any remaining statement
        if !current_statement.trim().is_empty() {
            statements.push(current_statement.trim().to_string());
        }
        
        statements
    }

    /// Store a transcription result
    pub async fn store_transcription(
        &self,
        result: &TranscriptionResult,
        conversation_id: Option<String>,
    ) -> Result<String> {
        let conv_id = if let Some(id) = conversation_id {
            id
        } else {
            self.create_conversation(None, None).await?
        };

        // Store each segment
        for segment in &result.segments {
            self.store_segment(&conv_id, result, segment).await?;
        }

        Ok(conv_id)
    }

    /// Create a new conversation
    pub async fn create_conversation(
        &self,
        title: Option<&str>,
        context: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO conversations (id, title, start_time, context) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(title)
        .bind(now)
        .bind(context)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// Store a single transcript segment
    async fn store_segment(
        &self,
        conversation_id: &str,
        result: &TranscriptionResult,
        segment: &TranscriptionSegment,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let metadata = result.session_metadata.as_ref();
        
        let speaker = metadata.and_then(|m| m.speaker.as_ref()).map_or("unknown", |s| s.as_str());
        let audio_source = metadata.map(|m| serde_json::to_string(&m.audio_source).unwrap())
            .unwrap_or_else(|| "\"Unknown\"".to_string());
        let timestamp = metadata.map(|m| m.timestamp).unwrap_or_else(Utc::now);

        sqlx::query(
            r#"INSERT INTO segments 
               (id, conversation_id, timestamp, speaker, audio_source, text, 
                start_time, end_time, confidence, metadata) 
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(conversation_id)
        .bind(timestamp)
        .bind(speaker)
        .bind(&audio_source)
        .bind(&segment.text)
        .bind(segment.start_time)
        .bind(segment.end_time)
        .bind(segment.confidence)
        .bind(serde_json::to_string(&result)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Query segments with flexible filters
    pub async fn query_segments(&self, query: &TranscriptQuery) -> Result<Vec<TranscriptionSegment>> {
        let mut sql = "SELECT metadata FROM segments WHERE 1=1".to_string();
        let mut params: Vec<String> = Vec::new();

        if let Some(conv_id) = &query.conversation_id {
            sql.push_str(" AND conversation_id = ?");
            params.push(conv_id.clone());
        }

        if let Some(speaker) = &query.speaker {
            sql.push_str(" AND speaker = ?");
            params.push(speaker.clone());
        }

        if let Some(text) = &query.text_contains {
            sql.push_str(" AND text LIKE ?");
            params.push(format!("%{}%", text));
        }

        if let Some(start) = &query.start_time {
            sql.push_str(" AND timestamp >= ?");
            params.push(start.to_rfc3339());
        }

        if let Some(end) = &query.end_time {
            sql.push_str(" AND timestamp <= ?");
            params.push(end.to_rfc3339());
        }

        sql.push_str(" ORDER BY timestamp");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = query.offset {
                sql.push_str(&format!(" OFFSET {}", offset));
            }
        }

        let mut query_builder = sqlx::query(&sql);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;
        let mut segments = Vec::new();

        for row in rows {
            let metadata_json: String = row.get("metadata");
            let result: TranscriptionResult = serde_json::from_str(&metadata_json)?;
            segments.extend(result.segments);
        }

        Ok(segments)
    }

    /// Get conversation statistics by speaker
    pub async fn get_speaker_stats(&self) -> Result<Vec<ConversationStats>> {
        let rows = sqlx::query(
            r#"SELECT 
                speaker,
                COUNT(DISTINCT conversation_id) as conversation_count,
                SUM(end_time - start_time) as total_duration,
                COUNT(*) as total_segments,
                AVG(COALESCE(confidence, 0.0)) as avg_confidence
               FROM segments 
               GROUP BY speaker
               ORDER BY total_duration DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(ConversationStats {
                speaker: row.get("speaker"),
                conversation_count: row.get("conversation_count"),
                total_duration_seconds: row.get::<f64, _>("total_duration"),
                total_segments: row.get("total_segments"),
                avg_confidence: row.get("avg_confidence"),
            });
        }

        Ok(stats)
    }

    /// List all conversations
    pub async fn list_conversations(&self, limit: Option<i64>) -> Result<Vec<Conversation>> {
        let sql = if let Some(limit) = limit {
            format!(
                r#"SELECT 
                    c.id, c.title, c.start_time, c.end_time, c.context,
                    COUNT(s.id) as segment_count,
                    SUM(s.end_time - s.start_time) as total_duration,
                    GROUP_CONCAT(DISTINCT s.speaker) as participants
                   FROM conversations c
                   LEFT JOIN segments s ON c.id = s.conversation_id
                   GROUP BY c.id
                   ORDER BY c.start_time DESC
                   LIMIT {}"#, 
                limit
            )
        } else {
            r#"SELECT 
                c.id, c.title, c.start_time, c.end_time, c.context,
                COUNT(s.id) as segment_count,
                SUM(s.end_time - s.start_time) as total_duration,
                GROUP_CONCAT(DISTINCT s.speaker) as participants
               FROM conversations c
               LEFT JOIN segments s ON c.id = s.conversation_id
               GROUP BY c.id
               ORDER BY c.start_time DESC"#.to_string()
        };

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        let mut conversations = Vec::new();

        for row in rows {
            let participants_str: Option<String> = row.get("participants");
            let participants = participants_str
                .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
                .unwrap_or_default();

            conversations.push(Conversation {
                id: row.get("id"),
                title: row.get("title"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                participants,
                context: row.get("context"),
                segment_count: row.get("segment_count"),
                total_duration: row.get::<Option<f64>, _>("total_duration").unwrap_or(0.0),
            });
        }

        Ok(conversations)
    }

    /// Export conversation to JSON for external processing
    pub async fn export_conversation(&self, conversation_id: &str) -> Result<serde_json::Value> {
        let query = TranscriptQuery {
            conversation_id: Some(conversation_id.to_string()),
            ..Default::default()
        };

        let segments = self.query_segments(&query).await?;
        
        Ok(serde_json::json!({
            "conversation_id": conversation_id,
            "exported_at": Utc::now(),
            "segments": segments
        }))
    }

    /// Initialize speaker identification system
    pub async fn init_speaker_identification(&mut self) -> Result<()> {
        if let Some(ref mut identifier) = self.speaker_identifier {
            identifier.load_embeddings().await?;
        }
        Ok(())
    }

    /// Initialize semantic search engine
    pub async fn init_semantic_search(&mut self) -> Result<()> {
        if let Some(ref mut engine) = self.semantic_engine {
            engine.load_embeddings().await?;
        }
        Ok(())
    }

    /// Get speaker identifier (immutable access)
    pub fn speaker_identifier(&self) -> Option<&SpeakerIdentifier> {
        self.speaker_identifier.as_ref()
    }

    /// Get mutable speaker identifier
    pub fn speaker_identifier_mut(&mut self) -> Option<&mut SpeakerIdentifier> {
        self.speaker_identifier.as_mut()
    }

    /// Get semantic search engine (immutable access)
    pub fn semantic_engine(&self) -> Option<&SemanticSearchEngine> {
        self.semantic_engine.as_ref()
    }

    /// Get mutable semantic search engine
    pub fn semantic_engine_mut(&mut self) -> Option<&mut SemanticSearchEngine> {
        self.semantic_engine.as_mut()
    }

    /// Text-based search for conversation segments
    pub async fn text_search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if let Some(engine) = &self.semantic_engine {
            engine.text_search(query, limit).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Semantic search for conversation segments (placeholder)
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        _min_similarity: f32,
    ) -> Result<Vec<SearchResult>> {
        // For now, fall back to text search
        self.text_search(query, limit).await
    }

    /// Analyze conversation and extract insights
    pub async fn analyze_conversation(&self, conversation_id: &str) -> Result<ConversationAnalysis> {
        if let Some(engine) = &self.semantic_engine {
            engine.analyze_conversation(conversation_id).await
        } else {
            Err(anyhow::anyhow!("Semantic engine not initialized"))
        }
    }

    /// List all speakers
    pub async fn list_speakers(&self) -> Result<Vec<Speaker>> {
        if let Some(identifier) = &self.speaker_identifier {
            identifier.list_speakers().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Create new speaker
    pub async fn create_speaker(&mut self, name: Option<String>) -> Result<String> {
        if let Some(identifier) = &mut self.speaker_identifier {
            identifier.create_speaker(name, None).await
        } else {
            Err(anyhow::anyhow!("Speaker identifier not initialized"))
        }
    }

    /// Find potential speaker duplicates
    pub async fn find_speaker_duplicates(&self) -> Result<Vec<(String, String, f32)>> {
        if let Some(identifier) = &self.speaker_identifier {
            identifier.find_potential_duplicates().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Merge two speakers
    pub async fn merge_speakers(&mut self, primary_id: &str, secondary_id: &str) -> Result<()> {
        if let Some(identifier) = &mut self.speaker_identifier {
            identifier.merge_speakers(primary_id, secondary_id).await
        } else {
            Err(anyhow::anyhow!("Speaker identifier not initialized"))
        }
    }

    /// Get conversation topics
    pub async fn get_conversation_topics(&self, conversation_id: &str) -> Result<Vec<Topic>> {
        if let Some(engine) = &self.semantic_engine {
            engine.get_conversation_topics(conversation_id).await
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get a single conversation by ID
    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Option<serde_json::Value>> {
        let row = sqlx::query(
            r#"SELECT c.id, c.title, c.start_time, c.end_time, c.context,
                      COUNT(s.id) as segment_count,
                      SUM(s.end_time - s.start_time) as total_duration,
                      GROUP_CONCAT(DISTINCT s.speaker) as participants
               FROM conversations c
               LEFT JOIN segments s ON c.id = s.conversation_id
               WHERE c.id = ?
               GROUP BY c.id"#
        )
        .bind(conversation_id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let participants_str: Option<String> = row.get("participants");
            let participants = participants_str
                .map(|s| s.split(',').map(|p| p.trim().to_string()).collect::<Vec<_>>())
                .unwrap_or_default();
            
            Ok(Some(serde_json::json!({
                "id": row.get::<String, _>("id"),
                "title": row.get::<Option<String>, _>("title"),
                "start_time": row.get::<chrono::DateTime<chrono::Utc>, _>("start_time"),
                "end_time": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("end_time"),
                "context": row.get::<Option<String>, _>("context"),
                "participants": participants,
                "segment_count": row.get::<i64, _>("segment_count"),
                "total_duration": row.get::<Option<f64>, _>("total_duration").unwrap_or(0.0)
            })))
        } else {
            Ok(None)
        }
    }
    
    /// Get all segments for a conversation
    pub async fn get_conversation_segments(&self, conversation_id: &str) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            r#"SELECT id, speaker, text, processed_text, timestamp, confidence, 
                      start_time, end_time
               FROM segments 
               WHERE conversation_id = ?
               ORDER BY timestamp ASC"#
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;
        
        let segments = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<String, _>("id"),
                    "speaker": row.get::<Option<String>, _>("speaker"),
                    "text": row.get::<String, _>("text"),
                    "processed_text": row.get::<Option<String>, _>("processed_text"),
                    "timestamp": row.get::<chrono::DateTime<chrono::Utc>, _>("timestamp"),
                    "confidence": row.get::<Option<f64>, _>("confidence"),
                    "start_time": row.get::<Option<f64>, _>("start_time"),
                    "end_time": row.get::<Option<f64>, _>("end_time")
                })
            })
            .collect();
        
        Ok(segments)
    }
}