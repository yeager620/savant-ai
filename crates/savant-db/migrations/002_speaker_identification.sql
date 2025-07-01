-- Enhanced schema for speaker identification and semantic search
-- Migration 002: Speaker Identification System

-- Speakers table for voice biometrics and identification
CREATE TABLE IF NOT EXISTS speakers (
    id TEXT PRIMARY KEY,
    name TEXT,
    display_name TEXT,
    voice_embedding BLOB,  -- 512-dimensional vector for pyannote embeddings
    text_patterns TEXT,     -- JSON array of common phrases/speech patterns
    contact_info TEXT,      -- JSON with email, phone, social profiles
    confidence_threshold REAL DEFAULT 0.75,
    total_conversation_time REAL DEFAULT 0.0,  -- Total seconds of speech
    total_conversations INTEGER DEFAULT 0,
    last_interaction DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Speaker aliases for handling name variations and merging
CREATE TABLE IF NOT EXISTS speaker_aliases (
    id TEXT PRIMARY KEY,
    primary_speaker_id TEXT REFERENCES speakers(id) ON DELETE CASCADE,
    alias_name TEXT NOT NULL,
    alias_embedding BLOB,
    merge_confidence REAL,
    source TEXT,  -- 'manual', 'automatic', 'voice_match'
    merged_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Speaker relationships and interaction analytics
CREATE TABLE IF NOT EXISTS speaker_relationships (
    speaker_a_id TEXT REFERENCES speakers(id) ON DELETE CASCADE,
    speaker_b_id TEXT REFERENCES speakers(id) ON DELETE CASCADE,
    total_conversations INTEGER DEFAULT 0,
    total_duration REAL DEFAULT 0.0,  -- Total seconds together
    last_conversation DATETIME,
    common_topics TEXT,  -- JSON array of frequent discussion topics
    sentiment_trend REAL,  -- Average sentiment in conversations
    interaction_frequency REAL,  -- Conversations per week
    relationship_strength REAL,  -- Calculated metric 0-1
    PRIMARY KEY (speaker_a_id, speaker_b_id),
    CHECK (speaker_a_id < speaker_b_id)  -- Ensure consistent ordering
);

-- Enhanced conversations table with semantic embeddings
ALTER TABLE conversations ADD COLUMN summary TEXT;
ALTER TABLE conversations ADD COLUMN topics TEXT;  -- JSON array of extracted topics
ALTER TABLE conversations ADD COLUMN sentiment_score REAL;
ALTER TABLE conversations ADD COLUMN quality_score REAL;
ALTER TABLE conversations ADD COLUMN participant_count INTEGER DEFAULT 0;
ALTER TABLE conversations ADD COLUMN location TEXT;

-- Enhanced segments table with speaker identification and embeddings
ALTER TABLE segments ADD COLUMN speaker_confidence REAL;
ALTER TABLE segments ADD COLUMN audio_features TEXT;  -- JSON with pitch, volume, etc
ALTER TABLE segments ADD COLUMN semantic_embedding BLOB;  -- 384-dim for semantic search
ALTER TABLE segments ADD COLUMN processed_text TEXT;  -- Cleaned/normalized text
ALTER TABLE segments ADD COLUMN language_code TEXT DEFAULT 'en';
ALTER TABLE segments ADD COLUMN emotion TEXT;  -- Detected emotion if available

-- Speaker voice samples for training/improvement
CREATE TABLE IF NOT EXISTS voice_samples (
    id TEXT PRIMARY KEY,
    speaker_id TEXT REFERENCES speakers(id) ON DELETE CASCADE,
    audio_path TEXT NOT NULL,
    embedding BLOB NOT NULL,
    duration REAL NOT NULL,
    quality_score REAL,
    transcription TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Conversation topics for better organization
CREATE TABLE IF NOT EXISTS topics (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    embedding BLOB,  -- Topic embedding for similarity
    frequency INTEGER DEFAULT 0,  -- How often this topic appears
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Many-to-many relationship between conversations and topics
CREATE TABLE IF NOT EXISTS conversation_topics (
    conversation_id TEXT REFERENCES conversations(id) ON DELETE CASCADE,
    topic_id TEXT REFERENCES topics(id) ON DELETE CASCADE,
    relevance_score REAL,  -- How relevant this topic is to the conversation
    PRIMARY KEY (conversation_id, topic_id)
);

-- Update segments foreign key to reference speakers
-- Note: We'll handle speaker_id migration in application code since SQLite 
-- doesn't support adding foreign key constraints to existing columns

-- Indexes for speaker identification performance
CREATE INDEX IF NOT EXISTS idx_speakers_name ON speakers(name);
CREATE INDEX IF NOT EXISTS idx_speakers_updated ON speakers(updated_at);
CREATE INDEX IF NOT EXISTS idx_speaker_aliases_primary ON speaker_aliases(primary_speaker_id);
CREATE INDEX IF NOT EXISTS idx_speaker_relationships_a ON speaker_relationships(speaker_a_id);
CREATE INDEX IF NOT EXISTS idx_speaker_relationships_b ON speaker_relationships(speaker_b_id);
CREATE INDEX IF NOT EXISTS idx_voice_samples_speaker ON voice_samples(speaker_id);
CREATE INDEX IF NOT EXISTS idx_voice_samples_quality ON voice_samples(quality_score);
CREATE INDEX IF NOT EXISTS idx_segments_speaker_conf ON segments(speaker_confidence);
CREATE INDEX IF NOT EXISTS idx_segments_processed_text ON segments(processed_text);
CREATE INDEX IF NOT EXISTS idx_conversation_topics_conv ON conversation_topics(conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_topics_topic ON conversation_topics(topic_id);

-- Enhanced full-text search including processed text
DROP TABLE IF EXISTS segments_fts;
CREATE VIRTUAL TABLE segments_fts USING fts5(
    original_text,
    processed_text,
    speaker,
    content='segments',
    content_rowid='rowid'
);

-- Updated triggers for FTS
DROP TRIGGER IF EXISTS segments_fts_insert;
DROP TRIGGER IF EXISTS segments_fts_delete;
DROP TRIGGER IF EXISTS segments_fts_update;

CREATE TRIGGER segments_fts_insert AFTER INSERT ON segments BEGIN
    INSERT INTO segments_fts(rowid, original_text, processed_text, speaker) 
    VALUES (new.rowid, new.text, COALESCE(new.processed_text, new.text), new.speaker);
END;

CREATE TRIGGER segments_fts_delete AFTER DELETE ON segments BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, original_text, processed_text, speaker) 
    VALUES('delete', old.rowid, old.text, COALESCE(old.processed_text, old.text), old.speaker);
END;

CREATE TRIGGER segments_fts_update AFTER UPDATE ON segments BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, original_text, processed_text, speaker) 
    VALUES('delete', old.rowid, old.text, COALESCE(old.processed_text, old.text), old.speaker);
    INSERT INTO segments_fts(rowid, original_text, processed_text, speaker) 
    VALUES (new.rowid, new.text, COALESCE(new.processed_text, new.text), new.speaker);
END;

-- Trigger to update speaker statistics
CREATE TRIGGER update_speaker_stats AFTER INSERT ON segments 
WHEN NEW.speaker != 'unknown' BEGIN
    UPDATE speakers SET 
        total_conversation_time = total_conversation_time + (NEW.end_time - NEW.start_time),
        last_interaction = NEW.timestamp,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.speaker;
END;

-- Trigger to update conversation participant count
CREATE TRIGGER update_conversation_participants AFTER INSERT ON segments BEGIN
    UPDATE conversations SET 
        participant_count = (
            SELECT COUNT(DISTINCT speaker) 
            FROM segments 
            WHERE conversation_id = NEW.conversation_id
        )
    WHERE id = NEW.conversation_id;
END;