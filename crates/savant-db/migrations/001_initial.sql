-- Initial database schema for Savant AI transcription storage

-- Conversations table for grouping related transcript segments
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    title TEXT,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    context TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Transcript segments table for individual pieces of transcribed audio
CREATE TABLE IF NOT EXISTS segments (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    speaker TEXT NOT NULL,
    audio_source TEXT NOT NULL,  -- JSON serialized AudioSource enum
    text TEXT NOT NULL,
    start_time REAL NOT NULL,    -- Relative time within the audio segment
    end_time REAL NOT NULL,      -- Relative time within the audio segment
    confidence REAL,             -- Optional confidence score
    metadata TEXT,               -- JSON blob with full TranscriptionResult
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

-- Participants table for speaker management (future use)
CREATE TABLE IF NOT EXISTS participants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    voice_profile TEXT,  -- For future speaker recognition
    metadata TEXT,       -- JSON blob for additional data
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_segments_conversation_id ON segments(conversation_id);
CREATE INDEX IF NOT EXISTS idx_segments_timestamp ON segments(timestamp);
CREATE INDEX IF NOT EXISTS idx_segments_speaker ON segments(speaker);
CREATE INDEX IF NOT EXISTS idx_segments_audio_source ON segments(audio_source);
CREATE INDEX IF NOT EXISTS idx_segments_text_fts ON segments(text);
CREATE INDEX IF NOT EXISTS idx_conversations_start_time ON conversations(start_time);

-- Full-text search for transcript content
CREATE VIRTUAL TABLE IF NOT EXISTS segments_fts USING fts5(
    text,
    content='segments',
    content_rowid='rowid'
);

-- Triggers to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS segments_fts_insert AFTER INSERT ON segments BEGIN
    INSERT INTO segments_fts(rowid, text) VALUES (new.rowid, new.text);
END;

CREATE TRIGGER IF NOT EXISTS segments_fts_delete AFTER DELETE ON segments BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, text) VALUES('delete', old.rowid, old.text);
END;

CREATE TRIGGER IF NOT EXISTS segments_fts_update AFTER UPDATE ON segments BEGIN
    INSERT INTO segments_fts(segments_fts, rowid, text) VALUES('delete', old.rowid, old.text);
    INSERT INTO segments_fts(rowid, text) VALUES (new.rowid, new.text);
END;