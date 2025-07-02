use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;
use uuid::Uuid;

use savant_video::{VideoFrame, VideoSession, CompressedFrame, VideoAnalysisResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoQuery {
    pub session_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub active_application: Option<String>,
    pub text_contains: Option<String>,
    pub activity_type: Option<String>,
    pub min_confidence: Option<f32>,
    pub has_code: Option<bool>,
    pub has_opportunities: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoStats {
    pub total_frames: i64,
    pub total_sessions: i64,
    pub total_size_bytes: i64,
    pub compression_ratio: f32,
    pub frames_with_ocr: i64,
    pub frames_with_analysis: i64,
    pub unique_applications: i64,
    pub storage_saved_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationUsage {
    pub application: String,
    pub frame_count: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub avg_productivity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub activity_type: String,
    pub duration_minutes: f32,
    pub frame_count: i64,
    pub productivity_score: f32,
    pub primary_applications: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub programming_language: String,
    pub snippet_count: i64,
    pub avg_complexity: f32,
    pub unique_frames: i64,
    pub first_detected: DateTime<Utc>,
    pub last_detected: DateTime<Utc>,
}

pub struct VisualDataManager {
    pool: SqlitePool,
}

impl VisualDataManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new video session
    pub async fn create_session(&self, config_snapshot: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO video_sessions (id, start_time, config_snapshot) VALUES (?, ?, ?)"
        )
        .bind(&id)
        .bind(now)
        .bind(config_snapshot)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// End a video session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let now = Utc::now();

        // Update session end time and calculate statistics
        sqlx::query(
            r#"UPDATE video_sessions 
               SET end_time = ?, 
                   frame_count = (SELECT COUNT(*) FROM video_frames WHERE session_id = ?),
                   total_size_bytes = (SELECT COALESCE(SUM(file_size_bytes), 0) FROM video_frames WHERE session_id = ?),
                   updated_at = ?
               WHERE id = ?"#
        )
        .bind(now)
        .bind(session_id)
        .bind(session_id)
        .bind(now)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store a video frame
    pub async fn store_frame(&self, frame: &VideoFrame) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO video_frames 
               (id, session_id, timestamp, file_path, resolution_width, resolution_height, 
                file_size_bytes, image_hash, change_detected, active_application, window_title, display_id)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&frame.id)
        .bind(&frame.metadata.session_id)
        .bind(frame.timestamp)
        .bind(frame.file_path.to_string_lossy().to_string())
        .bind(frame.resolution.0 as i64)
        .bind(frame.resolution.1 as i64)
        .bind(frame.file_size as i64)
        .bind(&frame.image_hash)
        .bind(frame.metadata.change_detected)
        .bind(&frame.metadata.active_application)
        .bind(&frame.metadata.window_title)
        .bind(&frame.metadata.display_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store a compressed frame (update existing frame record)
    pub async fn store_compressed_frame(&self, compressed_frame: &CompressedFrame) -> Result<()> {
        // First store the original frame if not already stored
        self.store_frame(&compressed_frame.original_frame).await?;

        // Update with compression information
        sqlx::query(
            r#"UPDATE video_frames 
               SET compressed_path = ?, compressed_size_bytes = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#
        )
        .bind(compressed_frame.compressed_path.to_string_lossy().to_string())
        .bind(compressed_frame.compressed_size_bytes as i64)
        .bind(&compressed_frame.original_frame.id)
        .execute(&self.pool)
        .await?;

        // Store processing result if available
        if let Some(ref analysis) = compressed_frame.processing_result {
            self.store_enhanced_analysis(&compressed_frame.original_frame.id, analysis).await?;
        }

        Ok(())
    }

    /// Store OCR content
    pub async fn store_ocr_content(
        &self,
        frame_id: &str,
        ocr_result: &savant_ocr::OCRResult,
    ) -> Result<()> {
        for block in &ocr_result.text_blocks {
            let id = Uuid::new_v4().to_string();
            let bounding_box = serde_json::to_string(&block.bounding_box)?;

            sqlx::query(
                r#"INSERT INTO video_ocr_content 
                   (id, frame_id, text_content, text_type, bounding_box, confidence, language)
                   VALUES (?, ?, ?, ?, ?, ?, ?)"#
            )
            .bind(&id)
            .bind(frame_id)
            .bind(&block.text)
            .bind(format!("{:?}", block.semantic_type))
            .bind(&bounding_box)
            .bind(block.confidence)
            .bind(&ocr_result.detected_language)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Store vision analysis
    pub async fn store_vision_analysis(
        &self,
        frame_id: &str,
        analysis: &savant_vision::ScreenAnalysis,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let detected_apps = serde_json::to_string(&analysis.app_context.detected_applications)?;
        let activity_classification = serde_json::to_string(&analysis.activity_classification)?;
        let visual_context = serde_json::to_string(&analysis.visual_context)?;
        let ui_elements = serde_json::to_string(&analysis.visual_elements)?;

        let primary_app_type = analysis.app_context.detected_applications
            .first()
            .map(|app| format!("{:?}", app.app_type))
            .unwrap_or_else(|| "Unknown".to_string());

        sqlx::query(
            r#"INSERT INTO video_vision_analysis 
               (id, frame_id, detected_applications, activity_classification, visual_context, 
                ui_elements, primary_app_type, secondary_apps_count)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind(&detected_apps)
        .bind(&activity_classification)
        .bind(&visual_context)
        .bind(&ui_elements)
        .bind(&primary_app_type)
        .bind(analysis.app_context.detected_applications.len().saturating_sub(1) as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store enhanced analysis result
    pub async fn store_enhanced_analysis(
        &self,
        frame_id: &str,
        analysis: &VideoAnalysisResult,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let analysis_json = serde_json::to_string(analysis)?;
        let app_context_json = serde_json::to_string(&analysis.application_context)?;
        let text_summary_json = serde_json::to_string(&analysis.text_summary)?;
        let opportunities_json = serde_json::to_string(&analysis.interaction_opportunities)?;
        let stats_json = serde_json::to_string(&analysis.processing_stats)?;

        sqlx::query(
            r#"INSERT INTO video_enhanced_analysis 
               (id, frame_id, analysis_result, application_context, text_summary, 
                interaction_opportunities, processing_stats, total_processing_time_ms)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind(&analysis_json)
        .bind(&app_context_json)
        .bind(&text_summary_json)
        .bind(&opportunities_json)
        .bind(&stats_json)
        .bind(analysis.processing_stats.total_processing_time_ms as i64)
        .execute(&self.pool)
        .await?;

        // Store individual components for easier querying
        if let Some(ref ocr_result) = analysis.ocr_result {
            self.store_ocr_content(frame_id, ocr_result).await?;
        }

        if let Some(ref screen_analysis) = analysis.screen_analysis {
            self.store_vision_analysis(frame_id, screen_analysis).await?;
        }

        // Store application contexts
        if let Some(ref browser_ctx) = analysis.application_context.browser_context {
            self.store_app_context(frame_id, "browser", &serde_json::to_string(browser_ctx)?).await?;
        }

        if let Some(ref ide_ctx) = analysis.application_context.ide_context {
            self.store_app_context(frame_id, "ide", &serde_json::to_string(ide_ctx)?).await?;
            
            // Store code snippets separately
            for snippet in &ide_ctx.code_snippets {
                self.store_code_snippet(frame_id, snippet).await?;
            }
        }

        if let Some(ref meeting_ctx) = analysis.application_context.meeting_context {
            self.store_app_context(frame_id, "meeting", &serde_json::to_string(meeting_ctx)?).await?;
        }

        if let Some(ref prod_ctx) = analysis.application_context.productivity_context {
            self.store_app_context(frame_id, "productivity", &serde_json::to_string(prod_ctx)?).await?;
        }

        // Store interaction opportunities
        for opportunity in &analysis.interaction_opportunities {
            self.store_interaction_opportunity(frame_id, opportunity).await?;
        }

        Ok(())
    }

    /// Store application context
    async fn store_app_context(&self, frame_id: &str, context_type: &str, context_data: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO video_app_contexts (id, frame_id, context_type, context_data) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(frame_id)
        .bind(context_type)
        .bind(context_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store code snippet
    async fn store_code_snippet(&self, frame_id: &str, snippet: &savant_video::CodeSnippet) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"INSERT INTO video_code_snippets 
               (id, frame_id, programming_language, code_content, complexity_score, context)
               VALUES (?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind(&snippet.language)
        .bind(&snippet.content)
        .bind(snippet.complexity_score)
        .bind("") // Context field, could be enhanced
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store interaction opportunity
    async fn store_interaction_opportunity(
        &self,
        frame_id: &str,
        opportunity: &savant_video::InteractionOpportunity,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"INSERT INTO video_interaction_opportunities 
               (id, frame_id, opportunity_type, description, confidence, suggested_action, context_info, urgency)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind(format!("{:?}", opportunity.opportunity_type))
        .bind(&opportunity.description)
        .bind(opportunity.confidence)
        .bind(&opportunity.suggested_action)
        .bind(&opportunity.context)
        .bind(format!("{:?}", opportunity.urgency))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Query video frames with filters
    pub async fn query_frames(&self, query: &VideoQuery) -> Result<Vec<serde_json::Value>> {
        let mut sql = r#"
            SELECT vf.*, 
                   COUNT(voc.id) as ocr_blocks,
                   COUNT(vio.id) as opportunities,
                   vva.primary_app_type
            FROM video_frames vf
            LEFT JOIN video_ocr_content voc ON vf.id = voc.frame_id
            LEFT JOIN video_interaction_opportunities vio ON vf.id = vio.frame_id
            LEFT JOIN video_vision_analysis vva ON vf.id = vva.frame_id
            WHERE 1=1
        "#.to_string();

        let mut params: Vec<String> = Vec::new();

        if let Some(session_id) = &query.session_id {
            sql.push_str(" AND vf.session_id = ?");
            params.push(session_id.clone());
        }

        if let Some(start_time) = &query.start_time {
            sql.push_str(" AND vf.timestamp >= ?");
            params.push(start_time.to_rfc3339());
        }

        if let Some(end_time) = &query.end_time {
            sql.push_str(" AND vf.timestamp <= ?");
            params.push(end_time.to_rfc3339());
        }

        if let Some(app) = &query.active_application {
            sql.push_str(" AND vf.active_application = ?");
            params.push(app.clone());
        }

        if let Some(text) = &query.text_contains {
            sql.push_str(" AND vf.id IN (SELECT frame_id FROM video_ocr_content WHERE text_content LIKE ?)");
            params.push(format!("%{}%", text));
        }

        if query.has_code == Some(true) {
            sql.push_str(" AND vf.id IN (SELECT frame_id FROM video_code_snippets)");
        }

        if query.has_opportunities == Some(true) {
            sql.push_str(" AND vf.id IN (SELECT frame_id FROM video_interaction_opportunities)");
        }

        sql.push_str(" GROUP BY vf.id ORDER BY vf.timestamp DESC");

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
        let mut frames = Vec::new();

        for row in rows {
            frames.push(serde_json::json!({
                "id": row.get::<String, _>("id"),
                "session_id": row.get::<String, _>("session_id"),
                "timestamp": row.get::<DateTime<Utc>, _>("timestamp"),
                "file_path": row.get::<String, _>("file_path"),
                "compressed_path": row.get::<Option<String>, _>("compressed_path"),
                "resolution_width": row.get::<Option<i64>, _>("resolution_width"),
                "resolution_height": row.get::<Option<i64>, _>("resolution_height"),
                "file_size_bytes": row.get::<Option<i64>, _>("file_size_bytes"),
                "compressed_size_bytes": row.get::<Option<i64>, _>("compressed_size_bytes"),
                "image_hash": row.get::<String, _>("image_hash"),
                "change_detected": row.get::<bool, _>("change_detected"),
                "active_application": row.get::<Option<String>, _>("active_application"),
                "window_title": row.get::<Option<String>, _>("window_title"),
                "primary_app_type": row.get::<Option<String>, _>("primary_app_type"),
                "ocr_blocks": row.get::<i64, _>("ocr_blocks"),
                "opportunities": row.get::<i64, _>("opportunities")
            }));
        }

        Ok(frames)
    }

    /// Get video statistics
    pub async fn get_stats(&self) -> Result<VideoStats> {
        let row = sqlx::query(
            r#"SELECT 
                COUNT(DISTINCT vf.id) as total_frames,
                COUNT(DISTINCT vf.session_id) as total_sessions,
                COALESCE(SUM(vf.file_size_bytes), 0) as total_size_bytes,
                COALESCE(AVG(CAST(vf.file_size_bytes AS FLOAT) / NULLIF(vf.compressed_size_bytes, 0)), 1.0) as compression_ratio,
                COUNT(DISTINCT voc.frame_id) as frames_with_ocr,
                COUNT(DISTINCT vea.frame_id) as frames_with_analysis,
                COUNT(DISTINCT vf.active_application) as unique_applications,
                COALESCE(SUM(vf.file_size_bytes - COALESCE(vf.compressed_size_bytes, vf.file_size_bytes)), 0) as storage_saved_bytes
               FROM video_frames vf
               LEFT JOIN video_ocr_content voc ON vf.id = voc.frame_id
               LEFT JOIN video_enhanced_analysis vea ON vf.id = vea.frame_id"#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(VideoStats {
            total_frames: row.get("total_frames"),
            total_sessions: row.get("total_sessions"),
            total_size_bytes: row.get("total_size_bytes"),
            compression_ratio: row.get("compression_ratio"),
            frames_with_ocr: row.get("frames_with_ocr"),
            frames_with_analysis: row.get("frames_with_analysis"),
            unique_applications: row.get("unique_applications"),
            storage_saved_bytes: row.get("storage_saved_bytes"),
        })
    }

    /// Get application usage statistics
    pub async fn get_application_usage(&self, limit: Option<i64>) -> Result<Vec<ApplicationUsage>> {
        let sql = if let Some(limit) = limit {
            format!(
                r#"SELECT * FROM app_usage_summary 
                   ORDER BY frame_count DESC LIMIT {}"#,
                limit
            )
        } else {
            "SELECT * FROM app_usage_summary ORDER BY frame_count DESC".to_string()
        };

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        let mut usage = Vec::new();

        for row in rows {
            usage.push(ApplicationUsage {
                application: row.get("active_application"),
                frame_count: row.get("frame_count"),
                first_seen: row.get("first_seen"),
                last_seen: row.get("last_seen"),
                avg_productivity: row.get("avg_productivity"),
            });
        }

        Ok(usage)
    }

    /// Get code analysis summary
    pub async fn get_code_analysis(&self) -> Result<Vec<CodeAnalysis>> {
        let rows = sqlx::query("SELECT * FROM code_analysis_summary ORDER BY snippet_count DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut analysis = Vec::new();
        for row in rows {
            analysis.push(CodeAnalysis {
                programming_language: row.get("programming_language"),
                snippet_count: row.get("snippet_count"),
                avg_complexity: row.get("avg_complexity"),
                unique_frames: row.get("unique_frames"),
                first_detected: row.get("first_detected"),
                last_detected: row.get("last_detected"),
            });
        }

        Ok(analysis)
    }

    /// Search OCR content using full-text search
    pub async fn search_text(&self, query: &str, limit: Option<i64>) -> Result<Vec<serde_json::Value>> {
        let limit_clause = limit.map_or_else(|| "".to_string(), |l| format!(" LIMIT {}", l));
        
        let sql = format!(
            r#"SELECT vf.id, vf.timestamp, vf.active_application, voc.text_content, 
                      voc.text_type, voc.confidence
               FROM video_ocr_fts 
               JOIN video_ocr_content voc ON video_ocr_fts.rowid = voc.rowid
               JOIN video_frames vf ON voc.frame_id = vf.id
               WHERE video_ocr_fts MATCH ?
               ORDER BY rank{}"#,
            limit_clause
        );

        let rows = sqlx::query(&sql)
            .bind(query)
            .fetch_all(&self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(serde_json::json!({
                "frame_id": row.get::<String, _>("id"),
                "timestamp": row.get::<DateTime<Utc>, _>("timestamp"),
                "active_application": row.get::<Option<String>, _>("active_application"),
                "text_content": row.get::<String, _>("text_content"),
                "text_type": row.get::<String, _>("text_type"),
                "confidence": row.get::<f64, _>("confidence")
            }));
        }

        Ok(results)
    }

    /// Get interaction opportunities
    pub async fn get_opportunities(
        &self,
        urgency_filter: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut sql = r#"
            SELECT vio.*, vf.timestamp, vf.active_application
            FROM video_interaction_opportunities vio
            JOIN video_frames vf ON vio.frame_id = vf.id
            WHERE 1=1
        "#.to_string();

        let mut params = Vec::new();
        if let Some(urgency) = urgency_filter {
            sql.push_str(" AND vio.urgency = ?");
            params.push(urgency.to_string());
        }

        sql.push_str(" ORDER BY vf.timestamp DESC");

        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut query_builder = sqlx::query(&sql);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;
        let mut opportunities = Vec::new();

        for row in rows {
            opportunities.push(serde_json::json!({
                "id": row.get::<String, _>("id"),
                "frame_id": row.get::<String, _>("frame_id"),
                "timestamp": row.get::<DateTime<Utc>, _>("timestamp"),
                "active_application": row.get::<Option<String>, _>("active_application"),
                "opportunity_type": row.get::<String, _>("opportunity_type"),
                "description": row.get::<String, _>("description"),
                "confidence": row.get::<f64, _>("confidence"),
                "suggested_action": row.get::<String, _>("suggested_action"),
                "context_info": row.get::<String, _>("context_info"),
                "urgency": row.get::<String, _>("urgency")
            }));
        }

        Ok(opportunities)
    }
}