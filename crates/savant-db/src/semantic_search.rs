//! Semantic search and text-based conversation analysis
//!
//! This module provides text-based analysis and search capabilities that can be enhanced
//! with semantic embeddings later.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ndarray::{Array1};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use uuid::Uuid;

/// Semantic embedding for text segments
#[derive(Debug, Clone)]
pub struct SemanticEmbedding {
    pub id: String,
    pub text: String,
    pub embedding: Array1<f32>,
    pub timestamp: DateTime<Utc>,
    pub speaker_id: Option<String>,
    pub conversation_id: String,
}

/// Conversation topic with semantic representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub frequency: i64,
    pub created_at: DateTime<Utc>,
}

/// Semantic search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub segment_id: String,
    pub conversation_id: String,
    pub speaker_id: Option<String>,
    pub text: String,
    pub similarity_score: f32,
    pub timestamp: DateTime<Utc>,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
}

/// Conversation analysis with extracted insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationAnalysis {
    pub conversation_id: String,
    pub summary: String,
    pub topics: Vec<String>,
    pub sentiment_score: f32,
    pub key_phrases: Vec<String>,
    pub duration: f32,
    pub participant_count: i32,
    pub quality_score: f32,
}

/// Semantic search engine for conversations
pub struct SemanticSearchEngine {
    pool: SqlitePool,
    embedding_cache: HashMap<String, Array1<f32>>,
    embedding_dimension: usize,
}

impl SemanticSearchEngine {
    /// Create new semantic search engine
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            embedding_cache: HashMap::new(),
            embedding_dimension: 384, // MiniLM default
        }
    }

    /// Load embeddings into memory for fast similarity search
    pub async fn load_embeddings(&mut self) -> Result<()> {
        let rows = sqlx::query(
            "SELECT id, semantic_embedding FROM segments WHERE semantic_embedding IS NOT NULL"
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let segment_id: String = row.get("id");
            if let Some(embedding_blob) = row.get::<Option<Vec<u8>>, _>("semantic_embedding") {
                if let Ok(embedding) = deserialize_embedding(&embedding_blob, self.embedding_dimension) {
                    self.embedding_cache.insert(segment_id, embedding);
                }
            }
        }

        Ok(())
    }

    /// Store semantic embedding for a text segment
    pub async fn store_embedding(
        &mut self,
        segment_id: &str,
        text: &str,
        embedding: &Array1<f32>,
    ) -> Result<()> {
        let embedding_blob = serialize_embedding(embedding);

        sqlx::query(
            "UPDATE segments SET semantic_embedding = ?, processed_text = ? WHERE id = ?"
        )
        .bind(&embedding_blob)
        .bind(text)
        .bind(segment_id)
        .execute(&self.pool)
        .await?;

        // Update cache
        self.embedding_cache.insert(segment_id.to_string(), embedding.clone());

        Ok(())
    }

    /// Text-based search for conversation segments (simplified implementation)
    pub async fn text_search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Simple text search using SQL LIKE and full-text search
        let rows = sqlx::query(
            r#"SELECT s.id, s.conversation_id, s.speaker, s.text, s.timestamp,
                      s.start_time, s.end_time
               FROM segments s
               WHERE s.text LIKE ?
               ORDER BY s.timestamp DESC
               LIMIT ?"#
        )
        .bind(format!("%{}%", query))
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let conversation_id: String = row.get("conversation_id");
            let start_time: f32 = row.get("start_time");
            let end_time: f32 = row.get("end_time");

            // Get context around the match
            let (context_before, context_after) = self.get_context(&conversation_id, start_time, end_time).await?;

            results.push(SearchResult {
                segment_id: row.get("id"),
                conversation_id,
                speaker_id: row.get("speaker"),
                text: row.get("text"),
                similarity_score: 1.0, // Placeholder for text match
                timestamp: row.get("timestamp"),
                context_before,
                context_after,
            });
        }

        Ok(results)
    }

    /// Enhanced semantic search (placeholder for future ML implementation)
    pub async fn semantic_search(
        &self,
        _query_embedding: &Array1<f32>,
        _limit: usize,
        _min_similarity: f32,
    ) -> Result<Vec<SearchResult>> {
        // For now, this returns empty results as a placeholder
        // In a full implementation, this would use sentence transformers
        // to generate embeddings and perform cosine similarity search
        Ok(Vec::new())
    }

    /// Get full search result with context
    async fn get_search_result(
        &self,
        segment_id: &str,
        similarity_score: f32,
    ) -> Result<Option<SearchResult>> {
        let row = sqlx::query(
            r#"SELECT s.id, s.conversation_id, s.speaker, s.text, s.timestamp,
                      s.start_time, s.end_time
               FROM segments s
               WHERE s.id = ?"#
        )
        .bind(segment_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let conversation_id: String = row.get("conversation_id");
            let start_time: f32 = row.get("start_time");
            let end_time: f32 = row.get("end_time");

            // Get context (previous and next segments)
            let (context_before, context_after) = self.get_context(&conversation_id, start_time, end_time).await?;

            Ok(Some(SearchResult {
                segment_id: row.get("id"),
                conversation_id,
                speaker_id: row.get("speaker"),
                text: row.get("text"),
                similarity_score,
                timestamp: row.get("timestamp"),
                context_before,
                context_after,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get conversation context around a segment
    async fn get_context(
        &self,
        conversation_id: &str,
        start_time: f32,
        end_time: f32,
    ) -> Result<(Option<String>, Option<String>)> {
        // Get previous segment
        let context_before = sqlx::query(
            r#"SELECT text FROM segments 
               WHERE conversation_id = ? AND end_time <= ? 
               ORDER BY end_time DESC LIMIT 1"#
        )
        .bind(conversation_id)
        .bind(start_time)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| row.get::<String, _>("text"));

        // Get next segment
        let context_after = sqlx::query(
            r#"SELECT text FROM segments 
               WHERE conversation_id = ? AND start_time >= ? 
               ORDER BY start_time ASC LIMIT 1"#
        )
        .bind(conversation_id)
        .bind(end_time)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| row.get::<String, _>("text"));

        Ok((context_before, context_after))
    }

    /// Extract and store topics for a conversation
    pub async fn extract_topics(&self, conversation_id: &str) -> Result<Vec<String>> {
        // Get all text from conversation
        let rows = sqlx::query(
            "SELECT text FROM segments WHERE conversation_id = ? ORDER BY start_time"
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        let full_text: String = rows
            .into_iter()
            .map(|row| row.get::<String, _>("text"))
            .collect::<Vec<_>>()
            .join(" ");

        // Simple keyword-based topic extraction (can be enhanced with ML models)
        let topics = self.simple_topic_extraction(&full_text);

        // Store topics in database
        for topic in &topics {
            self.store_topic(conversation_id, topic).await?;
        }

        Ok(topics)
    }

    /// Simple topic extraction using keyword frequency
    fn simple_topic_extraction(&self, text: &str) -> Vec<String> {
        let mut topics = Vec::new();
        
        // Common topic indicators
        let topic_keywords = [
            ("work", vec!["work", "job", "project", "meeting", "deadline", "task"]),
            ("family", vec!["family", "kids", "children", "parent", "mom", "dad"]),
            ("health", vec!["health", "doctor", "medical", "hospital", "sick", "medicine"]),
            ("technology", vec!["computer", "software", "app", "digital", "tech", "code"]),
            ("travel", vec!["travel", "trip", "vacation", "flight", "hotel", "visit"]),
            ("food", vec!["food", "restaurant", "cooking", "recipe", "eat", "dinner"]),
            ("money", vec!["money", "budget", "cost", "expensive", "cheap", "buy"]),
            ("education", vec!["school", "learn", "study", "education", "course", "student"]),
        ];

        let text_lower = text.to_lowercase();
        
        for (topic_name, keywords) in topic_keywords.iter() {
            let mut count = 0;
            for keyword in keywords {
                count += text_lower.matches(keyword).count();
            }
            
            // If topic has significant presence (>= 3 mentions), include it
            if count >= 3 {
                topics.push(topic_name.to_string());
            }
        }

        topics
    }

    /// Store topic association with conversation
    async fn store_topic(&self, conversation_id: &str, topic_name: &str) -> Result<()> {
        // Get or create topic
        let topic_id = self.get_or_create_topic(topic_name).await?;

        // Create conversation-topic association
        sqlx::query(
            r#"INSERT OR IGNORE INTO conversation_topics 
               (conversation_id, topic_id, relevance_score) 
               VALUES (?, ?, ?)"#
        )
        .bind(conversation_id)
        .bind(&topic_id)
        .bind(1.0) // Default relevance
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get existing topic or create new one
    async fn get_or_create_topic(&self, topic_name: &str) -> Result<String> {
        // Check if topic exists
        if let Some(row) = sqlx::query("SELECT id FROM topics WHERE name = ?")
            .bind(topic_name)
            .fetch_optional(&self.pool)
            .await?
        {
            return Ok(row.get("id"));
        }

        // Create new topic
        let topic_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO topics (id, name, frequency) VALUES (?, ?, ?)"
        )
        .bind(&topic_id)
        .bind(topic_name)
        .bind(1i64)
        .execute(&self.pool)
        .await?;

        Ok(topic_id)
    }

    /// Analyze conversation and extract insights
    pub async fn analyze_conversation(&self, conversation_id: &str) -> Result<ConversationAnalysis> {
        // Get conversation segments
        let segments = sqlx::query(
            r#"SELECT text, speaker, start_time, end_time, confidence 
               FROM segments 
               WHERE conversation_id = ? 
               ORDER BY start_time"#
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        if segments.is_empty() {
            return Err(anyhow!("No segments found for conversation"));
        }

        // Extract full text and calculate metrics
        let full_text: String = segments
            .iter()
            .map(|row| row.get::<String, _>("text"))
            .collect::<Vec<_>>()
            .join(" ");

        let duration = segments
            .iter()
            .map(|row| row.get::<f32, _>("end_time") - row.get::<f32, _>("start_time"))
            .sum();

        let participant_count = segments
            .iter()
            .map(|row| row.get::<String, _>("speaker"))
            .collect::<std::collections::HashSet<_>>()
            .len() as i32;

        let avg_confidence = segments
            .iter()
            .filter_map(|row| row.get::<Option<f32>, _>("confidence"))
            .sum::<f32>() / segments.len() as f32;

        // Generate summary (first few sentences)
        let summary = self.generate_summary(&full_text);

        // Extract topics
        let topics = self.extract_topics(conversation_id).await?;

        // Simple sentiment analysis (can be enhanced)
        let sentiment_score = self.analyze_sentiment(&full_text);

        // Extract key phrases
        let key_phrases = self.extract_key_phrases(&full_text);

        Ok(ConversationAnalysis {
            conversation_id: conversation_id.to_string(),
            summary,
            topics,
            sentiment_score,
            key_phrases,
            duration,
            participant_count,
            quality_score: avg_confidence,
        })
    }

    /// Generate conversation summary
    fn generate_summary(&self, text: &str) -> String {
        let sentences: Vec<&str> = text.split('.').take(3).collect();
        sentences.join(". ").trim().to_string()
    }

    /// Simple sentiment analysis
    fn analyze_sentiment(&self, text: &str) -> f32 {
        let positive_words = ["good", "great", "excellent", "happy", "love", "amazing", "wonderful"];
        let negative_words = ["bad", "terrible", "awful", "hate", "horrible", "disappointing"];

        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter().map(|w| text_lower.matches(w).count()).sum::<usize>();
        let negative_count = negative_words.iter().map(|w| text_lower.matches(w).count()).sum::<usize>();

        if positive_count + negative_count == 0 {
            0.0 // neutral
        } else {
            (positive_count as f32 - negative_count as f32) / (positive_count + negative_count) as f32
        }
    }

    /// Extract key phrases using simple frequency analysis
    fn extract_key_phrases(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split_whitespace()
            .filter(|w| w.len() > 3) // Filter short words
            .collect();

        let mut word_counts: HashMap<&str, usize> = HashMap::new();
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }

        let mut frequent_words: Vec<_> = word_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2) // Minimum frequency
            .collect();

        frequent_words.sort_by(|a, b| b.1.cmp(&a.1));
        frequent_words
            .into_iter()
            .take(10)
            .map(|(word, _)| word.to_string())
            .collect()
    }

    /// Get conversation topics
    pub async fn get_conversation_topics(&self, conversation_id: &str) -> Result<Vec<Topic>> {
        let rows = sqlx::query(
            r#"SELECT t.id, t.name, t.description, t.frequency, t.created_at,
                      ct.relevance_score
               FROM topics t
               JOIN conversation_topics ct ON t.id = ct.topic_id
               WHERE ct.conversation_id = ?
               ORDER BY ct.relevance_score DESC"#
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        let mut topics = Vec::new();
        for row in rows {
            topics.push(Topic {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                frequency: row.get("frequency"),
                created_at: row.get("created_at"),
            });
        }

        Ok(topics)
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    let dot_product = a.dot(b);
    let norm_a = a.mapv(|x| x * x).sum().sqrt();
    let norm_b = b.mapv(|x| x * x).sum().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Serialize embedding vector to binary format
fn serialize_embedding(embedding: &Array1<f32>) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|&x| x.to_le_bytes())
        .collect()
}

/// Deserialize embedding vector from binary format
fn deserialize_embedding(blob: &[u8], expected_dim: usize) -> Result<Array1<f32>> {
    if blob.len() != expected_dim * 4 {
        return Err(anyhow!("Invalid embedding blob length: expected {}, got {}", expected_dim * 4, blob.len()));
    }

    let embedding_data: Vec<f32> = blob
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(Array1::from_vec(embedding_data))
}