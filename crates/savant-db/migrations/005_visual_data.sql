-- Migration 005: Visual Data Storage Schema
-- Adds tables for storing and querying visual/video data with multimodal analysis

-- Video sessions table - groups related video captures
CREATE TABLE IF NOT EXISTS video_sessions (
    id TEXT PRIMARY KEY,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    frame_count INTEGER DEFAULT 0,
    total_size_bytes INTEGER DEFAULT 0,
    config_snapshot TEXT, -- JSON snapshot of capture config
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Video frames table - individual captured frames
CREATE TABLE IF NOT EXISTS video_frames (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    file_path TEXT NOT NULL,
    compressed_path TEXT, -- Path to compressed version if available
    resolution_width INTEGER,
    resolution_height INTEGER,
    file_size_bytes INTEGER,
    compressed_size_bytes INTEGER,
    image_hash TEXT, -- MD5 hash for deduplication
    change_detected BOOLEAN DEFAULT FALSE,
    active_application TEXT,
    window_title TEXT,
    display_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES video_sessions(id) ON DELETE CASCADE
);

-- OCR content extracted from video frames
CREATE TABLE IF NOT EXISTS video_ocr_content (
    id TEXT PRIMARY KEY,
    frame_id TEXT NOT NULL,
    text_content TEXT NOT NULL,
    text_type TEXT, -- UIElement, CodeSnippet, DocumentContent, ChatMessage, etc.
    bounding_box TEXT, -- JSON: {x, y, width, height}
    confidence REAL,
    language TEXT DEFAULT 'eng',
    processing_time_ms INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (frame_id) REFERENCES video_frames(id) ON DELETE CASCADE
);

-- Computer vision analysis results
CREATE TABLE IF NOT EXISTS video_vision_analysis (
    id TEXT PRIMARY KEY,
    frame_id TEXT NOT NULL,
    detected_applications TEXT, -- JSON array of detected apps
    activity_classification TEXT, -- JSON object with activity details
    visual_context TEXT, -- JSON object with visual elements
    ui_elements TEXT, -- JSON array of detected UI elements
    primary_app_type TEXT, -- Browser, IDE, VideoConferencing, etc.
    secondary_apps_count INTEGER DEFAULT 0,
    processing_time_ms INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (frame_id) REFERENCES video_frames(id) ON DELETE CASCADE
);

-- Enhanced video analysis (comprehensive analysis results)
CREATE TABLE IF NOT EXISTS video_enhanced_analysis (
    id TEXT PRIMARY KEY,
    frame_id TEXT NOT NULL,
    analysis_result TEXT NOT NULL, -- Full JSON of VideoAnalysisResult
    application_context TEXT, -- JSON of ApplicationContext
    text_summary TEXT, -- JSON of TextSummary
    interaction_opportunities TEXT, -- JSON array of opportunities
    processing_stats TEXT, -- JSON of ProcessingStats
    total_processing_time_ms INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (frame_id) REFERENCES video_frames(id) ON DELETE CASCADE
);

-- Application context details (extracted for easier querying)
CREATE TABLE IF NOT EXISTS video_app_contexts (
    id TEXT PRIMARY KEY,
    frame_id TEXT NOT NULL,
    context_type TEXT NOT NULL, -- browser, ide, meeting, productivity
    context_data TEXT NOT NULL, -- JSON specific to context type
    confidence REAL DEFAULT 0.0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (frame_id) REFERENCES video_frames(id) ON DELETE CASCADE
);

-- Code snippets detected in video frames
CREATE TABLE IF NOT EXISTS video_code_snippets (
    id TEXT PRIMARY KEY,
    frame_id TEXT NOT NULL,
    programming_language TEXT,
    code_content TEXT NOT NULL,
    line_range_start INTEGER,
    line_range_end INTEGER,
    complexity_score REAL DEFAULT 0.0,
    context TEXT, -- IDE context, file extension, etc.
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (frame_id) REFERENCES video_frames(id) ON DELETE CASCADE
);

-- Interaction opportunities detected
CREATE TABLE IF NOT EXISTS video_interaction_opportunities (
    id TEXT PRIMARY KEY,
    frame_id TEXT NOT NULL,
    opportunity_type TEXT NOT NULL, -- CodingAssistance, DocumentationHelp, etc.
    description TEXT NOT NULL,
    confidence REAL NOT NULL,
    suggested_action TEXT,
    context_info TEXT,
    urgency TEXT, -- Low, Medium, High, Critical
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (frame_id) REFERENCES video_frames(id) ON DELETE CASCADE
);

-- Multimodal event correlations between audio and video
CREATE TABLE IF NOT EXISTS multimodal_correlations (
    id TEXT PRIMARY KEY,
    video_frame_id TEXT,
    audio_segment_id TEXT,
    correlation_type TEXT NOT NULL, -- temporal, semantic, causal, pattern, statistical
    correlation_strength REAL NOT NULL,
    time_offset_ms INTEGER, -- Time difference between events
    correlation_data TEXT, -- JSON with correlation details
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (video_frame_id) REFERENCES video_frames(id) ON DELETE CASCADE,
    FOREIGN KEY (audio_segment_id) REFERENCES segments(id) ON DELETE CASCADE
);

-- Synchronized multimodal contexts (fused insights)
CREATE TABLE IF NOT EXISTS multimodal_contexts (
    id TEXT PRIMARY KEY,
    timestamp DATETIME NOT NULL,
    time_window_start DATETIME NOT NULL,
    time_window_end DATETIME NOT NULL,
    video_events TEXT, -- JSON array of video events in window
    audio_events TEXT, -- JSON array of audio events in window
    correlations TEXT, -- JSON array of correlations found
    fused_insights TEXT, -- JSON object with combined analysis
    confidence_scores TEXT, -- JSON object with confidence metrics
    context_summary TEXT, -- Human-readable summary
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Activity timeline (high-level activity classification over time)
CREATE TABLE IF NOT EXISTS activity_timeline (
    id TEXT PRIMARY KEY,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    activity_type TEXT NOT NULL, -- coding, meeting, browsing, writing, etc.
    activity_details TEXT, -- JSON with specific details
    primary_application TEXT,
    productivity_score REAL, -- 0.0 to 1.0
    confidence REAL NOT NULL,
    frame_count INTEGER DEFAULT 0,
    audio_segment_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Processing statistics and performance metrics
CREATE TABLE IF NOT EXISTS video_processing_stats (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    frames_processed INTEGER DEFAULT 0,
    frames_compressed INTEGER DEFAULT 0,
    frames_analyzed INTEGER DEFAULT 0,
    total_processing_time_ms INTEGER DEFAULT 0,
    compression_ratio REAL DEFAULT 1.0,
    storage_saved_bytes INTEGER DEFAULT 0,
    avg_ocr_time_ms REAL,
    avg_vision_time_ms REAL,
    avg_analysis_time_ms REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES video_sessions(id) ON DELETE CASCADE
);

-- Indexes for performance optimization

-- Frame lookup indexes
CREATE INDEX IF NOT EXISTS idx_video_frames_session_timestamp ON video_frames(session_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_video_frames_timestamp ON video_frames(timestamp);
CREATE INDEX IF NOT EXISTS idx_video_frames_hash ON video_frames(image_hash);
CREATE INDEX IF NOT EXISTS idx_video_frames_app ON video_frames(active_application);

-- OCR content search indexes
CREATE INDEX IF NOT EXISTS idx_video_ocr_frame_id ON video_ocr_content(frame_id);
CREATE INDEX IF NOT EXISTS idx_video_ocr_text_type ON video_ocr_content(text_type);
CREATE INDEX IF NOT EXISTS idx_video_ocr_language ON video_ocr_content(language);

-- Vision analysis indexes
CREATE INDEX IF NOT EXISTS idx_video_vision_frame_id ON video_vision_analysis(frame_id);
CREATE INDEX IF NOT EXISTS idx_video_vision_app_type ON video_vision_analysis(primary_app_type);

-- Application context indexes
CREATE INDEX IF NOT EXISTS idx_video_app_contexts_frame_id ON video_app_contexts(frame_id);
CREATE INDEX IF NOT EXISTS idx_video_app_contexts_type ON video_app_contexts(context_type);

-- Code snippet indexes
CREATE INDEX IF NOT EXISTS idx_video_code_frame_id ON video_code_snippets(frame_id);
CREATE INDEX IF NOT EXISTS idx_video_code_language ON video_code_snippets(programming_language);

-- Interaction opportunity indexes
CREATE INDEX IF NOT EXISTS idx_video_opportunities_frame_id ON video_interaction_opportunities(frame_id);
CREATE INDEX IF NOT EXISTS idx_video_opportunities_type ON video_interaction_opportunities(opportunity_type);
CREATE INDEX IF NOT EXISTS idx_video_opportunities_urgency ON video_interaction_opportunities(urgency);

-- Multimodal correlation indexes
CREATE INDEX IF NOT EXISTS idx_multimodal_corr_video ON multimodal_correlations(video_frame_id);
CREATE INDEX IF NOT EXISTS idx_multimodal_corr_audio ON multimodal_correlations(audio_segment_id);
CREATE INDEX IF NOT EXISTS idx_multimodal_corr_type ON multimodal_correlations(correlation_type);
CREATE INDEX IF NOT EXISTS idx_multimodal_corr_strength ON multimodal_correlations(correlation_strength);

-- Multimodal context indexes
CREATE INDEX IF NOT EXISTS idx_multimodal_contexts_timestamp ON multimodal_contexts(timestamp);
CREATE INDEX IF NOT EXISTS idx_multimodal_contexts_window ON multimodal_contexts(time_window_start, time_window_end);

-- Activity timeline indexes
CREATE INDEX IF NOT EXISTS idx_activity_timeline_time ON activity_timeline(start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_activity_timeline_type ON activity_timeline(activity_type);
CREATE INDEX IF NOT EXISTS idx_activity_timeline_app ON activity_timeline(primary_application);

-- Full-text search on text content (if SQLite supports FTS)
CREATE VIRTUAL TABLE IF NOT EXISTS video_ocr_fts USING fts5(
    text_content,
    content='video_ocr_content',
    content_rowid='rowid'
);

-- Triggers to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS video_ocr_fts_insert AFTER INSERT ON video_ocr_content BEGIN
    INSERT INTO video_ocr_fts(rowid, text_content) VALUES (NEW.rowid, NEW.text_content);
END;

CREATE TRIGGER IF NOT EXISTS video_ocr_fts_delete AFTER DELETE ON video_ocr_content BEGIN
    DELETE FROM video_ocr_fts WHERE rowid = OLD.rowid;
END;

CREATE TRIGGER IF NOT EXISTS video_ocr_fts_update AFTER UPDATE ON video_ocr_content BEGIN
    DELETE FROM video_ocr_fts WHERE rowid = OLD.rowid;
    INSERT INTO video_ocr_fts(rowid, text_content) VALUES (NEW.rowid, NEW.text_content);
END;

-- Views for common queries

-- Recent activity view
CREATE VIEW IF NOT EXISTS recent_activity AS
SELECT 
    vf.timestamp,
    vf.active_application,
    vf.window_title,
    at.activity_type,
    at.productivity_score,
    COUNT(voc.id) as text_blocks,
    GROUP_CONCAT(DISTINCT voc.text_type) as content_types
FROM video_frames vf
LEFT JOIN activity_timeline at ON vf.timestamp BETWEEN at.start_time AND COALESCE(at.end_time, vf.timestamp)
LEFT JOIN video_ocr_content voc ON vf.id = voc.frame_id
WHERE vf.timestamp >= datetime('now', '-24 hours')
GROUP BY vf.id
ORDER BY vf.timestamp DESC;

-- Application usage summary view
CREATE VIEW IF NOT EXISTS app_usage_summary AS
SELECT 
    active_application,
    COUNT(*) as frame_count,
    MIN(timestamp) as first_seen,
    MAX(timestamp) as last_seen,
    AVG(CASE WHEN at.productivity_score IS NOT NULL THEN at.productivity_score ELSE 0.5 END) as avg_productivity
FROM video_frames vf
LEFT JOIN activity_timeline at ON vf.timestamp BETWEEN at.start_time AND COALESCE(at.end_time, vf.timestamp)
WHERE active_application IS NOT NULL
GROUP BY active_application
ORDER BY frame_count DESC;

-- Code analysis summary view
CREATE VIEW IF NOT EXISTS code_analysis_summary AS
SELECT 
    programming_language,
    COUNT(*) as snippet_count,
    AVG(complexity_score) as avg_complexity,
    COUNT(DISTINCT frame_id) as unique_frames,
    MIN(created_at) as first_detected,
    MAX(created_at) as last_detected
FROM video_code_snippets
WHERE programming_language IS NOT NULL
GROUP BY programming_language
ORDER BY snippet_count DESC;