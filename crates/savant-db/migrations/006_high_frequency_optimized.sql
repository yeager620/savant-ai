-- Migration 006: High-Frequency Data Optimization
-- Optimized schema for sub-second sampling rates with intelligent storage and indexing

-- High-frequency frame captures (main table with minimal data for speed)
CREATE TABLE IF NOT EXISTS hf_video_frames (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_ms INTEGER NOT NULL, -- Unix timestamp in milliseconds for precision
    session_id TEXT NOT NULL,
    frame_hash TEXT NOT NULL, -- SHA-256 hash for deduplication
    change_score REAL DEFAULT 0.0, -- Quantified change from previous frame (0.0-1.0)
    file_path TEXT,
    compressed_path TEXT,
    width INTEGER,
    height INTEGER,
    file_size_bytes INTEGER,
    compressed_size_bytes INTEGER,
    processing_flags INTEGER DEFAULT 0, -- Bitfield: 1=has_ocr, 2=has_vision, 4=has_tasks, 8=has_questions
    active_app_id INTEGER, -- FK to applications table
    created_at INTEGER DEFAULT (unixepoch() * 1000) -- Millisecond precision
);

-- Optimized applications table with caching
CREATE TABLE IF NOT EXISTS hf_applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL UNIQUE,
    app_type TEXT NOT NULL, -- IDE, Browser, etc.
    bundle_id TEXT, -- macOS bundle identifier
    process_name TEXT,
    icon_hash TEXT,
    last_seen INTEGER, -- Timestamp
    usage_count INTEGER DEFAULT 1,
    INDEX idx_app_name (app_name),
    INDEX idx_app_type (app_type)
);

-- Word-level text data with spatial indexing
CREATE TABLE IF NOT EXISTS hf_text_words (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    word_text TEXT NOT NULL,
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    confidence REAL NOT NULL,
    font_size INTEGER,
    is_bold BOOLEAN DEFAULT FALSE,
    semantic_type INTEGER NOT NULL, -- Enum: 0=ui, 1=code, 2=document, 3=chat, 4=error, etc.
    line_id INTEGER, -- Groups words into lines
    region_id INTEGER, -- Groups into screen regions
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Line-level text groupings
CREATE TABLE IF NOT EXISTS hf_text_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    line_text TEXT NOT NULL,
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    word_count INTEGER NOT NULL,
    avg_confidence REAL NOT NULL,
    text_alignment INTEGER DEFAULT 0, -- 0=left, 1=center, 2=right, 3=justified
    is_heading BOOLEAN DEFAULT FALSE,
    font_size INTEGER,
    semantic_type INTEGER NOT NULL,
    region_id INTEGER,
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Screen regions for spatial organization
CREATE TABLE IF NOT EXISTS hf_screen_regions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    region_type INTEGER NOT NULL, -- 0=menubar, 1=sidebar, 2=main, 3=statusbar, etc.
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    z_order INTEGER DEFAULT 1,
    is_scrollable BOOLEAN DEFAULT FALSE,
    scroll_x REAL DEFAULT 0,
    scroll_y REAL DEFAULT 0,
    text_density REAL DEFAULT 0.0, -- Words per square pixel
    interaction_elements_count INTEGER DEFAULT 0,
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Detected tasks with real-time analysis
CREATE TABLE IF NOT EXISTS hf_detected_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    task_type INTEGER NOT NULL, -- 0=debugging, 1=coding, 2=research, etc.
    description TEXT NOT NULL,
    confidence REAL NOT NULL,
    complexity_level INTEGER DEFAULT 2, -- 0=trivial, 1=simple, 2=moderate, 3=complex, 4=expert
    estimated_duration_minutes INTEGER,
    bounding_box TEXT, -- JSON: {x, y, width, height}
    context_data TEXT, -- JSON with task context
    suggested_assistance TEXT, -- JSON array of suggestions
    status INTEGER DEFAULT 0, -- 0=detected, 1=in_progress, 2=completed, 3=abandoned
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Detected questions requiring assistance
CREATE TABLE IF NOT EXISTS hf_detected_questions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    question_type INTEGER NOT NULL, -- 0=error_explanation, 1=how_to, 2=conceptual, etc.
    question_text TEXT NOT NULL,
    confidence REAL NOT NULL,
    urgency_level INTEGER DEFAULT 1, -- 0=low, 1=medium, 2=high, 3=critical
    bounding_box TEXT, -- JSON: {x, y, width, height}
    context_data TEXT, -- JSON with question context
    suggested_answers TEXT, -- JSON array of suggested answers
    answer_provided BOOLEAN DEFAULT FALSE,
    answer_helpful BOOLEAN,
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Real-time assistance opportunities
CREATE TABLE IF NOT EXISTS hf_assistance_opportunities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    opportunity_type INTEGER NOT NULL, -- 0=error_prevention, 1=optimization, 2=learning, etc.
    description TEXT NOT NULL,
    confidence REAL NOT NULL,
    priority_level INTEGER DEFAULT 1, -- 0=low, 1=medium, 2=high, 3=critical
    suggested_action TEXT NOT NULL,
    trigger_context TEXT, -- What triggered this opportunity
    expires_at INTEGER, -- Timestamp when opportunity expires
    status INTEGER DEFAULT 0, -- 0=active, 1=presented, 2=accepted, 3=dismissed, 4=expired
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Context changes and transitions
CREATE TABLE IF NOT EXISTS hf_context_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    change_type INTEGER NOT NULL, -- 0=app_switch, 1=task_transition, 2=focus_shift, etc.
    from_context TEXT,
    to_context TEXT,
    confidence REAL NOT NULL,
    related_actions TEXT, -- JSON array
    impact_score REAL DEFAULT 0.0, -- How significant this change is
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- User intent signals
CREATE TABLE IF NOT EXISTS hf_intent_signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    frame_id INTEGER NOT NULL,
    signal_type INTEGER NOT NULL, -- 0=seeking_help, 1=frustrated, 2=confused, etc.
    strength REAL NOT NULL, -- Signal strength 0.0-1.0
    evidence TEXT, -- JSON array of evidence
    confidence REAL NOT NULL,
    pattern_match TEXT, -- What pattern triggered this signal
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Activity timeline with intelligent aggregation
CREATE TABLE IF NOT EXISTS hf_activity_timeline (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_timestamp_ms INTEGER NOT NULL,
    end_timestamp_ms INTEGER,
    activity_type INTEGER NOT NULL, -- 0=coding, 1=meeting, 2=browsing, etc.
    primary_app_id INTEGER,
    activity_details TEXT, -- JSON with specific details
    productivity_score REAL, -- 0.0-1.0
    focus_score REAL, -- 0.0-1.0 how focused the user was
    frame_count INTEGER DEFAULT 0,
    task_count INTEGER DEFAULT 0,
    question_count INTEGER DEFAULT 0,
    assistance_count INTEGER DEFAULT 0,
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    FOREIGN KEY (primary_app_id) REFERENCES hf_applications(id)
);

-- Frame deduplication and change detection
CREATE TABLE IF NOT EXISTS hf_frame_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    current_frame_id INTEGER NOT NULL,
    previous_frame_id INTEGER,
    change_score REAL NOT NULL, -- 0.0-1.0
    changed_regions TEXT, -- JSON array of changed region IDs
    pixel_diff_percentage REAL,
    text_diff_percentage REAL,
    ui_diff_percentage REAL,
    significant_change BOOLEAN DEFAULT FALSE, -- Above threshold
    change_summary TEXT, -- Brief description of what changed
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    FOREIGN KEY (current_frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE,
    FOREIGN KEY (previous_frame_id) REFERENCES hf_video_frames(id) ON DELETE CASCADE
);

-- Performance and storage optimization tables

-- Aggregated statistics for efficient queries
CREATE TABLE IF NOT EXISTS hf_daily_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date_key TEXT NOT NULL, -- YYYY-MM-DD
    total_frames INTEGER DEFAULT 0,
    total_words INTEGER DEFAULT 0,
    total_tasks INTEGER DEFAULT 0,
    total_questions INTEGER DEFAULT 0,
    total_assistance INTEGER DEFAULT 0,
    unique_applications INTEGER DEFAULT 0,
    productive_minutes INTEGER DEFAULT 0,
    focus_minutes INTEGER DEFAULT 0,
    top_activities TEXT, -- JSON array of top activities
    top_applications TEXT, -- JSON array of top applications
    created_at INTEGER DEFAULT (unixepoch() * 1000),
    UNIQUE(date_key)
);

-- Hot storage for recent data (last 24 hours)
CREATE TABLE IF NOT EXISTS hf_recent_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cache_key TEXT NOT NULL UNIQUE,
    cache_data TEXT NOT NULL, -- JSON data
    expires_at INTEGER NOT NULL,
    created_at INTEGER DEFAULT (unixepoch() * 1000)
);

-- INDEXES FOR HIGH-PERFORMANCE QUERIES

-- Primary timestamp-based indexes (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_hf_frames_timestamp ON hf_video_frames(timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_hf_frames_session_time ON hf_video_frames(session_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_hf_frames_hash ON hf_video_frames(frame_hash);
CREATE INDEX IF NOT EXISTS idx_hf_frames_app_time ON hf_video_frames(active_app_id, timestamp_ms);

-- Change detection indexes
CREATE INDEX IF NOT EXISTS idx_hf_frames_change_score ON hf_video_frames(change_score) WHERE change_score > 0.05;
CREATE INDEX IF NOT EXISTS idx_hf_changes_current ON hf_frame_changes(current_frame_id);
CREATE INDEX IF NOT EXISTS idx_hf_changes_significant ON hf_frame_changes(significant_change, change_score) WHERE significant_change = TRUE;

-- Text search indexes (spatial and content)
CREATE INDEX IF NOT EXISTS idx_hf_words_frame_id ON hf_text_words(frame_id);
CREATE INDEX IF NOT EXISTS idx_hf_words_semantic ON hf_text_words(semantic_type, confidence);
CREATE INDEX IF NOT EXISTS idx_hf_words_spatial ON hf_text_words(x, y, width, height);
CREATE INDEX IF NOT EXISTS idx_hf_words_text ON hf_text_words(word_text) WHERE length(word_text) > 2;

-- Line-level indexes
CREATE INDEX IF NOT EXISTS idx_hf_lines_frame_id ON hf_text_lines(frame_id);
CREATE INDEX IF NOT EXISTS idx_hf_lines_semantic ON hf_text_lines(semantic_type, avg_confidence);
CREATE INDEX IF NOT EXISTS idx_hf_lines_heading ON hf_text_lines(is_heading) WHERE is_heading = TRUE;

-- Region-based indexes
CREATE INDEX IF NOT EXISTS idx_hf_regions_frame_type ON hf_screen_regions(frame_id, region_type);
CREATE INDEX IF NOT EXISTS idx_hf_regions_spatial ON hf_screen_regions(x, y, width, height);
CREATE INDEX IF NOT EXISTS idx_hf_regions_density ON hf_screen_regions(text_density) WHERE text_density > 0.1;

-- Task and question indexes
CREATE INDEX IF NOT EXISTS idx_hf_tasks_frame_type ON hf_detected_tasks(frame_id, task_type);
CREATE INDEX IF NOT EXISTS idx_hf_tasks_status_time ON hf_detected_tasks(status, created_at);
CREATE INDEX IF NOT EXISTS idx_hf_tasks_confidence ON hf_detected_tasks(confidence) WHERE confidence > 0.7;

CREATE INDEX IF NOT EXISTS idx_hf_questions_frame_type ON hf_detected_questions(frame_id, question_type);
CREATE INDEX IF NOT EXISTS idx_hf_questions_urgency ON hf_detected_questions(urgency_level, created_at);
CREATE INDEX IF NOT EXISTS idx_hf_questions_unanswered ON hf_detected_questions(answer_provided) WHERE answer_provided = FALSE;

-- Assistance indexes
CREATE INDEX IF NOT EXISTS idx_hf_assistance_priority ON hf_assistance_opportunities(priority_level, status, created_at);
CREATE INDEX IF NOT EXISTS idx_hf_assistance_active ON hf_assistance_opportunities(status, expires_at) WHERE status = 0;
CREATE INDEX IF NOT EXISTS idx_hf_assistance_type ON hf_assistance_opportunities(opportunity_type, confidence);

-- Context and intent indexes
CREATE INDEX IF NOT EXISTS idx_hf_context_type_time ON hf_context_changes(change_type, created_at);
CREATE INDEX IF NOT EXISTS idx_hf_intent_type_strength ON hf_intent_signals(signal_type, strength, created_at);

-- Activity timeline indexes
CREATE INDEX IF NOT EXISTS idx_hf_timeline_time_range ON hf_activity_timeline(start_timestamp_ms, end_timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_hf_timeline_activity_app ON hf_activity_timeline(activity_type, primary_app_id);
CREATE INDEX IF NOT EXISTS idx_hf_timeline_productivity ON hf_activity_timeline(productivity_score) WHERE productivity_score > 0.7;

-- Application usage indexes
CREATE INDEX IF NOT EXISTS idx_hf_apps_usage ON hf_applications(usage_count, last_seen);
CREATE INDEX IF NOT EXISTS idx_hf_apps_type_usage ON hf_applications(app_type, usage_count);

-- Daily stats indexes
CREATE INDEX IF NOT EXISTS idx_hf_daily_stats_date ON hf_daily_stats(date_key);

-- Cache indexes
CREATE INDEX IF NOT EXISTS idx_hf_cache_expires ON hf_recent_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_hf_cache_key ON hf_recent_cache(cache_key);

-- FULL-TEXT SEARCH TABLES

-- High-performance full-text search on words
CREATE VIRTUAL TABLE IF NOT EXISTS hf_text_fts USING fts5(
    word_text,
    frame_id UNINDEXED,
    semantic_type UNINDEXED,
    confidence UNINDEXED,
    content='hf_text_words',
    content_rowid='id'
);

-- Full-text search on lines for phrase searches
CREATE VIRTUAL TABLE IF NOT EXISTS hf_lines_fts USING fts5(
    line_text,
    frame_id UNINDEXED,
    semantic_type UNINDEXED,
    avg_confidence UNINDEXED,
    content='hf_text_lines',
    content_rowid='id'
);

-- FTS triggers to keep search tables in sync
CREATE TRIGGER IF NOT EXISTS hf_words_fts_insert AFTER INSERT ON hf_text_words BEGIN
    INSERT INTO hf_text_fts(rowid, word_text, frame_id, semantic_type, confidence) 
    VALUES (NEW.id, NEW.word_text, NEW.frame_id, NEW.semantic_type, NEW.confidence);
END;

CREATE TRIGGER IF NOT EXISTS hf_words_fts_delete AFTER DELETE ON hf_text_words BEGIN
    DELETE FROM hf_text_fts WHERE rowid = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS hf_words_fts_update AFTER UPDATE ON hf_text_words BEGIN
    DELETE FROM hf_text_fts WHERE rowid = OLD.id;
    INSERT INTO hf_text_fts(rowid, word_text, frame_id, semantic_type, confidence) 
    VALUES (NEW.id, NEW.word_text, NEW.frame_id, NEW.semantic_type, NEW.confidence);
END;

CREATE TRIGGER IF NOT EXISTS hf_lines_fts_insert AFTER INSERT ON hf_text_lines BEGIN
    INSERT INTO hf_lines_fts(rowid, line_text, frame_id, semantic_type, avg_confidence) 
    VALUES (NEW.id, NEW.line_text, NEW.frame_id, NEW.semantic_type, NEW.avg_confidence);
END;

CREATE TRIGGER IF NOT EXISTS hf_lines_fts_delete AFTER DELETE ON hf_text_lines BEGIN
    DELETE FROM hf_lines_fts WHERE rowid = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS hf_lines_fts_update AFTER UPDATE ON hf_text_lines BEGIN
    DELETE FROM hf_lines_fts WHERE rowid = OLD.id;
    INSERT INTO hf_lines_fts(rowid, line_text, frame_id, semantic_type, avg_confidence) 
    VALUES (NEW.id, NEW.line_text, NEW.frame_id, NEW.semantic_type, NEW.avg_confidence);
END;

-- VIEWS FOR COMMON QUERIES

-- Recent high-value activity (last hour with significant events)
CREATE VIEW IF NOT EXISTS hf_recent_activity AS
SELECT 
    f.id AS frame_id,
    datetime(f.timestamp_ms / 1000, 'unixepoch') AS timestamp,
    a.app_name,
    a.app_type,
    COUNT(DISTINCT t.id) AS task_count,
    COUNT(DISTINCT q.id) AS question_count,
    COUNT(DISTINCT o.id) AS opportunity_count,
    MAX(t.confidence) AS max_task_confidence,
    MAX(q.urgency_level) AS max_question_urgency,
    f.change_score
FROM hf_video_frames f
LEFT JOIN hf_applications a ON f.active_app_id = a.id
LEFT JOIN hf_detected_tasks t ON f.id = t.frame_id AND t.confidence > 0.6
LEFT JOIN hf_detected_questions q ON f.id = q.frame_id AND q.confidence > 0.6
LEFT JOIN hf_assistance_opportunities o ON f.id = o.frame_id AND o.status = 0
WHERE f.timestamp_ms > (unixepoch() * 1000) - 3600000 -- Last hour
  AND (f.change_score > 0.1 OR t.id IS NOT NULL OR q.id IS NOT NULL OR o.id IS NOT NULL)
GROUP BY f.id
ORDER BY f.timestamp_ms DESC;

-- Text density hotspots (regions with high text concentration)
CREATE VIEW IF NOT EXISTS hf_text_hotspots AS
SELECT 
    r.frame_id,
    r.region_type,
    r.x, r.y, r.width, r.height,
    r.text_density,
    COUNT(w.id) AS word_count,
    COUNT(DISTINCT w.semantic_type) AS semantic_variety,
    AVG(w.confidence) AS avg_confidence,
    GROUP_CONCAT(DISTINCT w.word_text, ' ') AS sample_text
FROM hf_screen_regions r
LEFT JOIN hf_text_words w ON r.frame_id = w.frame_id 
    AND w.x >= r.x AND w.x <= r.x + r.width
    AND w.y >= r.y AND w.y <= r.y + r.height
WHERE r.text_density > 0.2
GROUP BY r.id
ORDER BY r.text_density DESC;

-- Application productivity summary
CREATE VIEW IF NOT EXISTS hf_app_productivity AS
SELECT 
    a.app_name,
    a.app_type,
    COUNT(DISTINCT f.id) AS total_frames,
    COUNT(DISTINCT CASE WHEN t.id IS NOT NULL THEN f.id END) AS frames_with_tasks,
    COUNT(DISTINCT CASE WHEN q.id IS NOT NULL THEN f.id END) AS frames_with_questions,
    AVG(CASE WHEN at.productivity_score IS NOT NULL THEN at.productivity_score ELSE 0.5 END) AS avg_productivity,
    SUM(CASE WHEN at.end_timestamp_ms IS NOT NULL 
             THEN (at.end_timestamp_ms - at.start_timestamp_ms) / 60000.0 
             ELSE 0.5 END) AS total_minutes,
    MAX(f.timestamp_ms) AS last_used
FROM hf_applications a
LEFT JOIN hf_video_frames f ON a.id = f.active_app_id
LEFT JOIN hf_detected_tasks t ON f.id = t.frame_id
LEFT JOIN hf_detected_questions q ON f.id = q.frame_id
LEFT JOIN hf_activity_timeline at ON a.id = at.primary_app_id
WHERE a.usage_count > 5
GROUP BY a.id
ORDER BY total_minutes DESC;

-- Task completion funnel
CREATE VIEW IF NOT EXISTS hf_task_funnel AS
SELECT 
    task_type,
    COUNT(*) AS total_detected,
    COUNT(CASE WHEN status >= 1 THEN 1 END) AS started,
    COUNT(CASE WHEN status = 2 THEN 1 END) AS completed,
    COUNT(CASE WHEN status = 3 THEN 1 END) AS abandoned,
    AVG(confidence) AS avg_confidence,
    AVG(complexity_level) AS avg_complexity
FROM hf_detected_tasks
WHERE created_at > (unixepoch() * 1000) - 86400000 -- Last 24 hours
GROUP BY task_type
ORDER BY total_detected DESC;

-- Real-time assistance dashboard
CREATE VIEW IF NOT EXISTS hf_assistance_dashboard AS
SELECT 
    o.id,
    o.opportunity_type,
    o.description,
    o.priority_level,
    o.confidence,
    datetime(o.created_at / 1000, 'unixepoch') AS created_at,
    datetime(o.expires_at / 1000, 'unixepoch') AS expires_at,
    a.app_name,
    f.timestamp_ms AS frame_timestamp,
    CASE 
        WHEN o.expires_at < unixepoch() * 1000 THEN 'expired'
        WHEN o.status = 0 THEN 'active'
        WHEN o.status = 1 THEN 'presented'
        WHEN o.status = 2 THEN 'accepted'
        WHEN o.status = 3 THEN 'dismissed'
        ELSE 'unknown'
    END AS status_text
FROM hf_assistance_opportunities o
JOIN hf_video_frames f ON o.frame_id = f.id
LEFT JOIN hf_applications a ON f.active_app_id = a.id
WHERE o.status IN (0, 1) -- Active or presented
  AND (o.expires_at IS NULL OR o.expires_at > unixepoch() * 1000)
ORDER BY o.priority_level DESC, o.confidence DESC, o.created_at DESC;

-- PRAGMAS for performance optimization
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000; -- 64MB cache
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456; -- 256MB memory map