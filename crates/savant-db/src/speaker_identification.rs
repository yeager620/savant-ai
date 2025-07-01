//! Speaker identification and voice biometric matching
//! 
//! This module provides a simplified speaker identification system that works with
//! existing transcription data and can be enhanced with ML models later.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ndarray::{Array1};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use uuid::Uuid;

/// Speaker profile with voice biometrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub id: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub confidence_threshold: f32,
    pub total_conversation_time: f32,
    pub total_conversations: i64,
    pub last_interaction: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Voice embedding for speaker identification
#[derive(Debug, Clone)]
pub struct VoiceEmbedding {
    pub vector: Array1<f32>,
    pub speaker_id: String,
    pub confidence: f32,
}

/// Result of speaker matching attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerMatch {
    pub speaker_id: String,
    pub confidence: f32,
    pub method: MatchMethod,
    pub is_new_speaker: bool,
}

/// Method used for speaker identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchMethod {
    VoiceEmbedding,
    TextPatterns,
    Manual,
    Unknown,
}

/// Speaker identification system
pub struct SpeakerIdentifier {
    pool: SqlitePool,
    embedding_cache: HashMap<String, VoiceEmbedding>,
    confidence_threshold: f32,
}

impl SpeakerIdentifier {
    /// Create new speaker identifier
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            embedding_cache: HashMap::new(),
            confidence_threshold: 0.75,
        }
    }

    /// Load all known speaker embeddings into cache
    pub async fn load_embeddings(&mut self) -> Result<()> {
        let rows = sqlx::query(
            "SELECT id, voice_embedding, confidence_threshold FROM speakers WHERE voice_embedding IS NOT NULL"
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let speaker_id: String = row.get("id");
            let embedding_blob: Vec<u8> = row.get("voice_embedding");
            let threshold: f32 = row.get("confidence_threshold");

            // Deserialize embedding vector (assuming 512 dimensions)
            if embedding_blob.len() == 512 * 4 {
                let embedding_data: Vec<f32> = embedding_blob
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                let embedding = VoiceEmbedding {
                    vector: Array1::from_vec(embedding_data),
                    speaker_id: speaker_id.clone(),
                    confidence: threshold,
                };

                self.embedding_cache.insert(speaker_id, embedding);
            }
        }

        Ok(())
    }

    /// Identify speaker from voice embedding (placeholder implementation)
    pub fn identify_speaker(&self, _embedding: &Array1<f32>) -> Option<SpeakerMatch> {
        // For now, this is a placeholder that returns None
        // In a full implementation, this would compare voice embeddings
        // using cosine similarity against known speaker profiles
        None
    }

    /// Identify speaker from text patterns (simplified approach)
    pub fn identify_speaker_by_text(&self, text: &str) -> Option<SpeakerMatch> {
        // Simple pattern-based speaker identification
        // This is a fallback when voice embeddings aren't available
        
        // Check for common speech patterns or phrases
        if text.to_lowercase().contains("hey siri") || text.to_lowercase().contains("ok google") {
            return Some(SpeakerMatch {
                speaker_id: "user".to_string(),
                confidence: 0.8,
                method: MatchMethod::TextPatterns,
                is_new_speaker: false,
            });
        }

        // For system audio (might contain specific app names, etc.)
        if text.to_lowercase().contains("notification") || text.to_lowercase().contains("alert") {
            return Some(SpeakerMatch {
                speaker_id: "system".to_string(),
                confidence: 0.7,
                method: MatchMethod::TextPatterns,
                is_new_speaker: false,
            });
        }

        None
    }

    /// Create new speaker profile
    pub async fn create_speaker(
        &mut self,
        name: Option<String>,
        embedding: Option<Array1<f32>>,
    ) -> Result<String> {
        let speaker_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let embedding_blob = if let Some(emb) = &embedding {
            Some(serialize_embedding(emb))
        } else {
            None
        };

        sqlx::query(
            r#"INSERT INTO speakers 
               (id, name, display_name, voice_embedding, confidence_threshold, 
                total_conversation_time, total_conversations, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&speaker_id)
        .bind(&name)
        .bind(&name) // Use name as display_name initially
        .bind(&embedding_blob)
        .bind(self.confidence_threshold)
        .bind(0.0) // initial conversation time
        .bind(0i64) // initial conversation count
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Update cache if we have an embedding
        if let Some(embedding) = embedding {
            let voice_embedding = VoiceEmbedding {
                vector: embedding,
                speaker_id: speaker_id.clone(),
                confidence: self.confidence_threshold,
            };
            self.embedding_cache.insert(speaker_id.clone(), voice_embedding);
        }

        Ok(speaker_id)
    }

    /// Update speaker embedding with new sample
    pub async fn update_speaker_embedding(
        &mut self,
        speaker_id: &str,
        new_embedding: &Array1<f32>,
    ) -> Result<()> {
        // Get current embedding if it exists
        let current_embedding = self.get_speaker_embedding(speaker_id).await?;

        let updated_embedding = if let Some(current) = current_embedding {
            // Weighted average: 70% existing, 30% new
            &current * 0.7 + new_embedding * 0.3
        } else {
            new_embedding.clone()
        };

        let embedding_blob = serialize_embedding(&updated_embedding);

        sqlx::query(
            "UPDATE speakers SET voice_embedding = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&embedding_blob)
        .bind(Utc::now())
        .bind(speaker_id)
        .execute(&self.pool)
        .await?;

        // Update cache
        let voice_embedding = VoiceEmbedding {
            vector: updated_embedding,
            speaker_id: speaker_id.to_string(),
            confidence: self.confidence_threshold,
        };
        self.embedding_cache.insert(speaker_id.to_string(), voice_embedding);

        Ok(())
    }

    /// Get speaker embedding from database
    async fn get_speaker_embedding(&self, speaker_id: &str) -> Result<Option<Array1<f32>>> {
        let row = sqlx::query(
            "SELECT voice_embedding FROM speakers WHERE id = ?"
        )
        .bind(speaker_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            if let Some(embedding_blob) = row.get::<Option<Vec<u8>>, _>("voice_embedding") {
                return Ok(Some(deserialize_embedding(&embedding_blob)?));
            }
        }

        Ok(None)
    }

    /// Get all speakers
    pub async fn list_speakers(&self) -> Result<Vec<Speaker>> {
        let rows = sqlx::query(
            r#"SELECT id, name, display_name, confidence_threshold, 
                      total_conversation_time, total_conversations, 
                      last_interaction, created_at, updated_at
               FROM speakers 
               ORDER BY total_conversation_time DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut speakers = Vec::new();
        for row in rows {
            speakers.push(Speaker {
                id: row.get("id"),
                name: row.get("name"),
                display_name: row.get("display_name"),
                confidence_threshold: row.get("confidence_threshold"),
                total_conversation_time: row.get("total_conversation_time"),
                total_conversations: row.get("total_conversations"),
                last_interaction: row.get("last_interaction"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(speakers)
    }

    /// Merge two speakers (combine their profiles)
    pub async fn merge_speakers(&mut self, primary_id: &str, secondary_id: &str) -> Result<()> {
        // Move all segments from secondary to primary
        sqlx::query(
            "UPDATE segments SET speaker = ? WHERE speaker = ?"
        )
        .bind(primary_id)
        .bind(secondary_id)
        .execute(&self.pool)
        .await?;

        // Combine statistics
        sqlx::query(
            r#"UPDATE speakers SET 
                total_conversation_time = total_conversation_time + (
                    SELECT total_conversation_time FROM speakers WHERE id = ?
                ),
                total_conversations = total_conversations + (
                    SELECT total_conversations FROM speakers WHERE id = ?
                ),
                updated_at = ?
               WHERE id = ?"#
        )
        .bind(secondary_id)
        .bind(secondary_id)
        .bind(Utc::now())
        .bind(primary_id)
        .execute(&self.pool)
        .await?;

        // Create alias record
        sqlx::query(
            r#"INSERT INTO speaker_aliases 
               (id, primary_speaker_id, alias_name, merge_confidence, source)
               VALUES (?, ?, ?, ?, ?)"#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(primary_id)
        .bind(format!("merged_speaker_{}", secondary_id))
        .bind(1.0)
        .bind("manual")
        .execute(&self.pool)
        .await?;

        // Delete secondary speaker
        sqlx::query("DELETE FROM speakers WHERE id = ?")
            .bind(secondary_id)
            .execute(&self.pool)
            .await?;

        // Remove from cache
        self.embedding_cache.remove(secondary_id);

        Ok(())
    }

    /// Find potential speaker duplicates based on similarity
    pub async fn find_potential_duplicates(&self) -> Result<Vec<(String, String, f32)>> {
        let mut duplicates = Vec::new();
        let embeddings: Vec<_> = self.embedding_cache.values().collect();

        for i in 0..embeddings.len() {
            for j in (i + 1)..embeddings.len() {
                let similarity = cosine_similarity(&embeddings[i].vector, &embeddings[j].vector);
                
                // Flag as potential duplicate if similarity > 0.9
                if similarity > 0.9 {
                    duplicates.push((
                        embeddings[i].speaker_id.clone(),
                        embeddings[j].speaker_id.clone(),
                        similarity,
                    ));
                }
            }
        }

        Ok(duplicates)
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
fn deserialize_embedding(blob: &[u8]) -> Result<Array1<f32>> {
    if blob.len() % 4 != 0 {
        return Err(anyhow!("Invalid embedding blob length"));
    }

    let embedding_data: Vec<f32> = blob
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(Array1::from_vec(embedding_data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let b = Array1::from_vec(vec![0.0, 1.0, 0.0]);
        let c = Array1::from_vec(vec![1.0, 0.0, 0.0]);

        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
        assert!((cosine_similarity(&a, &c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_serialization() {
        let original = Array1::from_vec(vec![1.0, 2.5, -3.7, 0.0]);
        let serialized = serialize_embedding(&original);
        let deserialized = deserialize_embedding(&serialized).unwrap();

        for (orig, deser) in original.iter().zip(deserialized.iter()) {
            assert!((orig - deser).abs() < 1e-6);
        }
    }
}