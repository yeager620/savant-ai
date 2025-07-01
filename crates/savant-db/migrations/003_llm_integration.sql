-- Enhanced schema for LLM database integration
-- Adds query history, intent classification, and natural language processing support

-- Natural language query metadata
CREATE TABLE IF NOT EXISTS query_history (
    id TEXT PRIMARY KEY,
    natural_query TEXT NOT NULL,
    structured_query TEXT NOT NULL,
    intent_type TEXT NOT NULL,
    entities TEXT, -- JSON object with extracted entities
    execution_time_ms INTEGER,
    result_count INTEGER,
    user_feedback TEXT,
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT,
    timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Query intent classification patterns
CREATE TABLE IF NOT EXISTS query_intents (
    id TEXT PRIMARY KEY,
    intent_name TEXT NOT NULL,
    description TEXT,
    example_queries TEXT, -- JSON array of example queries
    sql_template TEXT,
    required_parameters TEXT, -- JSON array of required parameters
    confidence_threshold REAL DEFAULT 0.7,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- MCP server session management
CREATE TABLE IF NOT EXISTS mcp_sessions (
    id TEXT PRIMARY KEY,
    client_info TEXT, -- JSON with client details
    capabilities TEXT, -- JSON with supported features
    started_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    query_count INTEGER DEFAULT 0,
    active BOOLEAN DEFAULT TRUE
);

-- Enhanced conversation metadata for better LLM querying
ALTER TABLE conversations ADD COLUMN IF NOT EXISTS summary TEXT;
ALTER TABLE conversations ADD COLUMN IF NOT EXISTS topics TEXT; -- JSON array
ALTER TABLE conversations ADD COLUMN IF NOT EXISTS sentiment_score REAL;
ALTER TABLE conversations ADD COLUMN IF NOT EXISTS quality_score REAL;
ALTER TABLE conversations ADD COLUMN IF NOT EXISTS participant_count INTEGER DEFAULT 0;

-- Enhanced segment metadata for semantic search
ALTER TABLE segments ADD COLUMN IF NOT EXISTS processed_text TEXT;
ALTER TABLE segments ADD COLUMN IF NOT EXISTS semantic_embedding BLOB;
ALTER TABLE segments ADD COLUMN IF NOT EXISTS topic_tags TEXT; -- JSON array

-- Indexes for efficient LLM queries
CREATE INDEX IF NOT EXISTS idx_query_history_timestamp ON query_history(timestamp);
CREATE INDEX IF NOT EXISTS idx_query_history_intent ON query_history(intent_type);
CREATE INDEX IF NOT EXISTS idx_conversations_topics ON conversations(topics);
CREATE INDEX IF NOT EXISTS idx_conversations_sentiment ON conversations(sentiment_score);
CREATE INDEX IF NOT EXISTS idx_segments_processed_text ON segments(processed_text);

-- Insert default query intent patterns
INSERT OR IGNORE INTO query_intents (id, intent_name, description, example_queries, sql_template, required_parameters) VALUES
('find_conversations', 'FindConversations', 'Find conversations with specific participants or criteria', 
 '["Find all conversations with John", "Show me talks between Alice and Bob", "List meetings involving Sarah"]',
 'SELECT c.*, GROUP_CONCAT(DISTINCT s.speaker) as participants FROM conversations c LEFT JOIN segments s ON c.id = s.conversation_id WHERE {participant_filter} GROUP BY c.id ORDER BY c.start_time DESC LIMIT {limit}',
 '["speaker", "limit"]'),

('analyze_speaker', 'AnalyzeSpeaker', 'Get analytics and statistics for a specific speaker',
 '["Analyze speaker John", "How much did Alice talk?", "Statistics for Bob"]',
 'SELECT speaker, COUNT(DISTINCT conversation_id) as conversation_count, SUM(end_time - start_time) as total_duration, COUNT(*) as total_segments, AVG(COALESCE(confidence, 0.0)) as avg_confidence FROM segments WHERE speaker = {speaker} GROUP BY speaker',
 '["speaker"]'),

('search_content', 'SearchContent', 'Search for specific content or topics in conversations',
 '["Search for project alpha", "Find mentions of budget", "Conversations about deadlines"]',
 'SELECT s.*, c.title as conversation_title FROM segments s JOIN conversations c ON s.conversation_id = c.id WHERE s.text LIKE {search_term} OR s.processed_text LIKE {search_term} ORDER BY s.timestamp DESC LIMIT {limit}',
 '["search_term", "limit"]'),

('get_statistics', 'GetStatistics', 'Get overall database statistics and summaries',
 '["Show database stats", "How many conversations?", "Total recording time"]',
 'SELECT COUNT(DISTINCT c.id) as total_conversations, COUNT(DISTINCT s.speaker) as unique_speakers, COUNT(s.id) as total_segments, SUM(s.end_time - s.start_time) as total_duration FROM conversations c LEFT JOIN segments s ON c.id = s.conversation_id',
 '[]'),

('export_data', 'ExportData', 'Export conversation data for analysis',
 '["Export conversation 123", "Get data for last week", "Export all conversations with John"]',
 'SELECT * FROM segments WHERE conversation_id = {conversation_id} ORDER BY timestamp',
 '["conversation_id"]');

-- Create full-text search virtual table for enhanced search
CREATE VIRTUAL TABLE IF NOT EXISTS segments_fts USING fts5(
    original_text,
    processed_text,
    speaker,
    conversation_title,
    content='',
    contentless_delete=1
);

-- Populate FTS table with existing data
INSERT INTO segments_fts(original_text, processed_text, speaker, conversation_title)
SELECT 
    s.text,
    COALESCE(s.processed_text, s.text),
    s.speaker,
    COALESCE(c.title, 'Untitled Conversation')
FROM segments s
LEFT JOIN conversations c ON s.conversation_id = c.id
WHERE NOT EXISTS (SELECT 1 FROM segments_fts WHERE rowid = s.rowid);

-- Triggers to maintain FTS table
CREATE TRIGGER IF NOT EXISTS segments_fts_insert AFTER INSERT ON segments BEGIN
    INSERT INTO segments_fts(rowid, original_text, processed_text, speaker, conversation_title)
    SELECT 
        NEW.rowid,
        NEW.text,
        COALESCE(NEW.processed_text, NEW.text),
        NEW.speaker,
        COALESCE(c.title, 'Untitled Conversation')
    FROM conversations c 
    WHERE c.id = NEW.conversation_id;
END;

CREATE TRIGGER IF NOT EXISTS segments_fts_delete AFTER DELETE ON segments BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, original_text, processed_text, speaker, conversation_title)
    VALUES('delete', OLD.rowid, OLD.text, OLD.processed_text, OLD.speaker, '');
END;

CREATE TRIGGER IF NOT EXISTS segments_fts_update AFTER UPDATE ON segments BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, original_text, processed_text, speaker, conversation_title)
    VALUES('delete', OLD.rowid, OLD.text, OLD.processed_text, OLD.speaker, '');
    
    INSERT INTO segments_fts(rowid, original_text, processed_text, speaker, conversation_title)
    SELECT 
        NEW.rowid,
        NEW.text,
        COALESCE(NEW.processed_text, NEW.text),
        NEW.speaker,
        COALESCE(c.title, 'Untitled Conversation')
    FROM conversations c 
    WHERE c.id = NEW.conversation_id;
END;