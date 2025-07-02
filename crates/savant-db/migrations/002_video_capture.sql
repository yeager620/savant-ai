-- Video capture tables for screen recording functionality

-- Video segments table (mirrors audio segments structure)
CREATE TABLE IF NOT EXISTS video_segments (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    timestamp DATETIME NOT NULL,
    duration_seconds REAL DEFAULT 0,
    file_path TEXT NOT NULL,
    screen_resolution TEXT,
    change_detected BOOLEAN DEFAULT FALSE,
    metadata TEXT,  -- JSON blob like audio segments
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

-- Video frames table for individual captures
CREATE TABLE IF NOT EXISTS video_frames (
    id TEXT PRIMARY KEY,
    segment_id TEXT REFERENCES video_segments(id),
    timestamp DATETIME NOT NULL,
    image_hash TEXT,
    ocr_text TEXT,
    file_size_bytes INTEGER,
    active_application TEXT,
    window_title TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Video sessions table for capture sessions
CREATE TABLE IF NOT EXISTS video_sessions (
    id TEXT PRIMARY KEY,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    frame_count INTEGER DEFAULT 0,
    total_size_bytes INTEGER DEFAULT 0,
    config TEXT,  -- JSON capture configuration
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Multimodal sessions for audio-video synchronization
CREATE TABLE IF NOT EXISTS multimodal_sessions (
    id TEXT PRIMARY KEY,
    audio_session_id TEXT,
    video_session_id TEXT REFERENCES video_sessions(id),
    sync_offset_seconds INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_video_segments_conversation_id ON video_segments(conversation_id);
CREATE INDEX IF NOT EXISTS idx_video_segments_timestamp ON video_segments(timestamp);
CREATE INDEX IF NOT EXISTS idx_video_frames_segment_id ON video_frames(segment_id);
CREATE INDEX IF NOT EXISTS idx_video_frames_timestamp ON video_frames(timestamp);
CREATE INDEX IF NOT EXISTS idx_video_frames_hash ON video_frames(image_hash);
CREATE INDEX IF NOT EXISTS idx_video_frames_app ON video_frames(active_application);
CREATE INDEX IF NOT EXISTS idx_video_sessions_start_time ON video_sessions(start_time);

-- Full-text search for OCR content
CREATE VIRTUAL TABLE IF NOT EXISTS video_ocr_fts USING fts5(
    ocr_text,
    content='video_frames',
    content_rowid='rowid'
);

-- Triggers to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS video_ocr_fts_insert AFTER INSERT ON video_frames 
WHEN new.ocr_text IS NOT NULL BEGIN
    INSERT INTO video_ocr_fts(rowid, ocr_text) VALUES (new.rowid, new.ocr_text);
END;

CREATE TRIGGER IF NOT EXISTS video_ocr_fts_delete AFTER DELETE ON video_frames 
WHEN old.ocr_text IS NOT NULL BEGIN
    INSERT INTO video_ocr_fts(video_ocr_fts, rowid, ocr_text) VALUES('delete', old.rowid, old.ocr_text);
END;

CREATE TRIGGER IF NOT EXISTS video_ocr_fts_update AFTER UPDATE ON video_frames 
WHEN old.ocr_text IS NOT NULL OR new.ocr_text IS NOT NULL BEGIN
    INSERT INTO video_ocr_fts(video_ocr_fts, rowid, ocr_text) VALUES('delete', old.rowid, old.ocr_text);
    INSERT INTO video_ocr_fts(rowid, ocr_text) VALUES (new.rowid, new.ocr_text);
END;