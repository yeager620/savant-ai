-- Database Optimizations for Conversation-Specific Queries
-- Migration 004: Enhanced indexes and materialized views for better performance

-- Enhanced full-text search with conversation-specific configuration
-- Note: SQLite doesn't support custom text search configurations like PostgreSQL
-- But we can optimize FTS5 tables and add specialized indexes

-- Add trigram indexes for fuzzy search (requires FTS5)
CREATE TABLE IF NOT EXISTS conversation_segments_trigrams (
    id INTEGER PRIMARY KEY,
    segment_id TEXT NOT NULL,
    trigram TEXT NOT NULL,
    FOREIGN KEY (segment_id) REFERENCES segments(id)
);

-- Create specialized indexes for conversation queries
CREATE INDEX IF NOT EXISTS idx_segments_speaker_timestamp ON segments(speaker, timestamp)
    WHERE speaker IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_segments_conversation_speaker ON segments(conversation_id, speaker)
    WHERE speaker IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_conversations_date_participant ON conversations(
    DATE(start_time), 
    SUBSTR(context, 1, 100)  -- Index first 100 chars of context for participant info
) WHERE start_time IS NOT NULL;

-- Partial index for active conversations
CREATE INDEX IF NOT EXISTS idx_conversations_active ON conversations(start_time, id)
    WHERE end_time IS NULL OR end_time > datetime('now', '-30 days');

-- Index for conversation duration analysis
CREATE INDEX IF NOT EXISTS idx_segments_duration ON segments(
    conversation_id, 
    (end_time - start_time)
) WHERE end_time IS NOT NULL AND start_time IS NOT NULL;

-- Materialized view for speaker interaction matrix
-- SQLite doesn't have materialized views, so we'll create a table that gets refreshed
CREATE TABLE IF NOT EXISTS speaker_interaction_matrix (
    speaker_a TEXT NOT NULL,
    speaker_b TEXT NOT NULL,
    interaction_count INTEGER NOT NULL DEFAULT 0,
    total_duration REAL NOT NULL DEFAULT 0.0,
    first_interaction TIMESTAMPTZ,
    last_interaction TIMESTAMPTZ,
    conversation_ids TEXT, -- JSON array of conversation IDs
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (speaker_a, speaker_b)
);

-- Trigger to ensure speaker_a <= speaker_b for consistency
CREATE TRIGGER IF NOT EXISTS speaker_interaction_matrix_order
    BEFORE INSERT ON speaker_interaction_matrix
    FOR EACH ROW
    WHEN NEW.speaker_a > NEW.speaker_b
BEGIN
    UPDATE speaker_interaction_matrix SET
        speaker_a = NEW.speaker_b,
        speaker_b = NEW.speaker_a
    WHERE rowid = NEW.rowid;
END;

-- Function-like view for computing speaker interactions (refreshed periodically)
CREATE VIEW IF NOT EXISTS speaker_interactions_live AS
WITH interaction_pairs AS (
    SELECT DISTINCT
        CASE 
            WHEN s1.speaker <= s2.speaker THEN s1.speaker 
            ELSE s2.speaker 
        END as speaker_a,
        CASE 
            WHEN s1.speaker <= s2.speaker THEN s2.speaker 
            ELSE s1.speaker 
        END as speaker_b,
        c.id as conversation_id,
        c.start_time,
        SUM(s1.end_time - s1.start_time) + SUM(s2.end_time - s2.start_time) as duration
    FROM segments s1
    JOIN segments s2 ON s1.conversation_id = s2.conversation_id
    JOIN conversations c ON c.id = s1.conversation_id
    WHERE s1.speaker IS NOT NULL 
      AND s2.speaker IS NOT NULL 
      AND s1.speaker != s2.speaker
      AND s1.id < s2.id  -- Avoid duplicates
    GROUP BY s1.speaker, s2.speaker, c.id
)
SELECT 
    speaker_a, 
    speaker_b,
    COUNT(DISTINCT conversation_id) as interaction_count,
    SUM(duration) as total_duration,
    MIN(start_time) as first_interaction,
    MAX(start_time) as last_interaction,
    GROUP_CONCAT(conversation_id) as conversation_ids
FROM interaction_pairs
GROUP BY speaker_a, speaker_b;

-- Speaker analytics view for faster queries
CREATE VIEW IF NOT EXISTS speaker_analytics AS
SELECT 
    s.speaker,
    COUNT(DISTINCT s.conversation_id) as conversation_count,
    COUNT(s.id) as total_segments,
    SUM(s.end_time - s.start_time) as total_duration,
    AVG(s.end_time - s.start_time) as avg_segment_duration,
    AVG(COALESCE(s.confidence, 0.0)) as avg_confidence,
    MIN(s.timestamp) as first_appearance,
    MAX(s.timestamp) as last_appearance,
    COUNT(DISTINCT DATE(s.timestamp)) as active_days
FROM segments s
WHERE s.speaker IS NOT NULL 
  AND s.speaker != ''
  AND s.end_time IS NOT NULL 
  AND s.start_time IS NOT NULL
GROUP BY s.speaker;

-- Conversation summary view with pre-computed statistics
CREATE VIEW IF NOT EXISTS conversation_summaries AS
SELECT 
    c.id,
    c.title,
    c.start_time,
    c.end_time,
    c.context,
    COUNT(s.id) as segment_count,
    COUNT(DISTINCT s.speaker) as unique_speakers,
    SUM(s.end_time - s.start_time) as total_duration,
    AVG(s.end_time - s.start_time) as avg_segment_duration,
    GROUP_CONCAT(DISTINCT s.speaker) as participants,
    MIN(s.timestamp) as first_segment,
    MAX(s.timestamp) as last_segment,
    AVG(COALESCE(s.confidence, 0.0)) as avg_confidence
FROM conversations c
LEFT JOIN segments s ON c.id = s.conversation_id
GROUP BY c.id, c.title, c.start_time, c.end_time, c.context;

-- Optimized search view for content queries
CREATE VIEW IF NOT EXISTS searchable_content AS
SELECT 
    s.id as segment_id,
    s.conversation_id,
    s.speaker,
    s.timestamp,
    s.text,
    s.processed_text,
    s.confidence,
    c.title as conversation_title,
    c.start_time as conversation_start,
    -- Create searchable text combining content and context
    COALESCE(s.text, '') || ' ' || 
    COALESCE(s.processed_text, '') || ' ' || 
    COALESCE(c.title, '') || ' ' || 
    COALESCE(s.speaker, '') as searchable_text
FROM segments s
JOIN conversations c ON s.conversation_id = c.id
WHERE s.text IS NOT NULL AND s.text != '';

-- Create covering indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_searchable_content_speaker_time ON segments(
    speaker, 
    DATE(timestamp),
    conversation_id
) WHERE speaker IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_searchable_content_conversation_time ON segments(
    conversation_id,
    timestamp,
    speaker
);

-- Index for confidence-based filtering
CREATE INDEX IF NOT EXISTS idx_segments_confidence ON segments(confidence, timestamp)
    WHERE confidence IS NOT NULL;

-- Composite index for complex queries
CREATE INDEX IF NOT EXISTS idx_segments_composite ON segments(
    conversation_id,
    speaker,
    timestamp,
    confidence
) WHERE speaker IS NOT NULL AND confidence IS NOT NULL;

-- Add index to FTS table for better performance
-- Rebuild FTS index with better configuration if it exists
DROP TABLE IF EXISTS segments_fts_temp;
CREATE VIRTUAL TABLE segments_fts_temp USING fts5(
    text, 
    processed_text, 
    speaker, 
    conversation_id UNINDEXED,
    timestamp UNINDEXED,
    content='segments',
    content_rowid='id',
    prefix='2 3 4',  -- Enable prefix matching for 2, 3, 4 character prefixes
    tokenize='porter unicode61'  -- Better tokenization
);

-- Populate the new FTS table
INSERT INTO segments_fts_temp(rowid, text, processed_text, speaker, conversation_id, timestamp)
SELECT id, text, processed_text, speaker, conversation_id, timestamp 
FROM segments 
WHERE text IS NOT NULL AND text != '';

-- Replace old FTS table if it exists
DROP TABLE IF EXISTS segments_fts;
ALTER TABLE segments_fts_temp RENAME TO segments_fts;

-- Create triggers to maintain FTS index
CREATE TRIGGER IF NOT EXISTS segments_fts_insert AFTER INSERT ON segments 
BEGIN
    INSERT INTO segments_fts(rowid, text, processed_text, speaker, conversation_id, timestamp)
    VALUES (NEW.id, NEW.text, NEW.processed_text, NEW.speaker, NEW.conversation_id, NEW.timestamp);
END;

CREATE TRIGGER IF NOT EXISTS segments_fts_delete AFTER DELETE ON segments 
BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, text, processed_text, speaker, conversation_id, timestamp)
    VALUES ('delete', OLD.id, OLD.text, OLD.processed_text, OLD.speaker, OLD.conversation_id, OLD.timestamp);
END;

CREATE TRIGGER IF NOT EXISTS segments_fts_update AFTER UPDATE ON segments 
BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, text, processed_text, speaker, conversation_id, timestamp)
    VALUES ('delete', OLD.id, OLD.text, OLD.processed_text, OLD.speaker, OLD.conversation_id, OLD.timestamp);
    INSERT INTO segments_fts(rowid, text, processed_text, speaker, conversation_id, timestamp)
    VALUES (NEW.id, NEW.text, NEW.processed_text, NEW.speaker, NEW.conversation_id, NEW.timestamp);
END;

-- Performance monitoring table
CREATE TABLE IF NOT EXISTS query_performance (
    id TEXT PRIMARY KEY,
    query_type TEXT NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    result_count INTEGER NOT NULL,
    query_complexity TEXT, -- 'low', 'medium', 'high'
    timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    sql_query TEXT,
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_query_performance_type_time ON query_performance(
    query_type, 
    timestamp
);

CREATE INDEX IF NOT EXISTS idx_query_performance_execution_time ON query_performance(
    execution_time_ms, 
    timestamp
) WHERE execution_time_ms > 1000; -- Index slow queries

-- Cache table for expensive queries (optional)
CREATE TABLE IF NOT EXISTS query_cache (
    cache_key TEXT PRIMARY KEY,
    query_hash TEXT NOT NULL,
    result_data TEXT NOT NULL, -- JSON
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ,
    hit_count INTEGER DEFAULT 0,
    last_accessed TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_query_cache_expires ON query_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_query_cache_hash ON query_cache(query_hash);

-- Cleanup trigger for query cache
CREATE TRIGGER IF NOT EXISTS query_cache_cleanup 
    AFTER INSERT ON query_cache
    WHEN (SELECT COUNT(*) FROM query_cache) > 1000
BEGIN
    DELETE FROM query_cache 
    WHERE expires_at < datetime('now') 
       OR cache_key IN (
           SELECT cache_key FROM query_cache 
           ORDER BY last_accessed ASC 
           LIMIT 100
       );
END;

-- Update performance statistics view
CREATE VIEW IF NOT EXISTS database_performance_stats AS
SELECT 
    'conversations' as table_name,
    COUNT(*) as row_count,
    AVG(LENGTH(title)) as avg_title_length,
    COUNT(*) FILTER (WHERE end_time IS NULL) as active_conversations
FROM conversations
UNION ALL
SELECT 
    'segments' as table_name,
    COUNT(*) as row_count,
    AVG(LENGTH(text)) as avg_text_length,
    COUNT(DISTINCT speaker) as unique_speakers
FROM segments
UNION ALL
SELECT 
    'query_performance' as table_name,
    COUNT(*) as row_count,
    AVG(execution_time_ms) as avg_execution_time,
    COUNT(*) FILTER (WHERE success = FALSE) as failed_queries
FROM query_performance;

-- Analyze command to update SQLite statistics
-- Note: This would typically be run after the migration
-- ANALYZE;