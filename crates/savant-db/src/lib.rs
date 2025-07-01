//! # Savant Database Library
//!
//! Database management for audio transcription data with rich querying capabilities

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;
use uuid::Uuid;

pub use savant_stt::{TranscriptionResult, TranscriptionSegment, SessionMetadata, AudioSource};

/// Database connection manager
pub struct TranscriptDatabase {
    pool: SqlitePool,
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

        let database_url = format!("sqlite:{}", path.display());
        let pool = SqlitePool::connect(&database_url).await?;
        
        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
        sqlx::query(include_str!("../migrations/001_initial.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
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
}