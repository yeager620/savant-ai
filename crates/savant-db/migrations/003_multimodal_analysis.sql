-- Multimodal analysis tables for enhanced video/audio integration

-- OCR content with spatial indexing and semantic classification
CREATE TABLE IF NOT EXISTS ocr_content (
    id TEXT PRIMARY KEY,
    video_segment_id TEXT NOT NULL,
    text_content TEXT NOT NULL,
    text_type TEXT NOT NULL, -- UIElement, CodeSnippet, DocumentContent, etc.
    bounding_box JSON NOT NULL, -- {x, y, width, height}
    confidence REAL NOT NULL,
    font_info JSON, -- {size, family, style, weight}
    language TEXT,
    processing_time_ms INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (video_segment_id) REFERENCES video_segments(id)
);

-- Visual analysis results
CREATE TABLE IF NOT EXISTS visual_analysis (
    id TEXT PRIMARY KEY,
    video_segment_id TEXT NOT NULL,
    detected_applications JSON NOT NULL, -- Array of DetectedApp objects
    activity_classification JSON NOT NULL, -- ActivityClassification object
    visual_context JSON NOT NULL, -- VisualContext object
    attention_areas JSON, -- Array of high-attention screen regions
    ui_elements JSON, -- Array of detected UI elements
    processing_time_ms INTEGER NOT NULL,
    confidence_scores JSON NOT NULL, -- Overall analysis confidence metrics
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (video_segment_id) REFERENCES video_segments(id)
);

-- Application context tracking
CREATE TABLE IF NOT EXISTS application_contexts (
    id TEXT PRIMARY KEY,
    video_segment_id TEXT NOT NULL,
    primary_application TEXT,
    application_type TEXT, -- IDE, Browser, VideoConferencing, etc.
    application_details JSON, -- Specific context (browser_context, ide_context, etc.)
    window_state TEXT, -- Focused, Background, Minimized, etc.
    confidence REAL NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (video_segment_id) REFERENCES video_segments(id)
);

-- Multimodal event correlation
CREATE TABLE IF NOT EXISTS event_correlations (
    id TEXT PRIMARY KEY,
    video_segment_id TEXT,
    audio_segment_id TEXT,
    correlation_type TEXT NOT NULL, -- Temporal, Causal, Semantic, SpeakerVisual, etc.
    correlation_strength REAL NOT NULL,
    time_offset_ms INTEGER NOT NULL,
    causal_relationship TEXT, -- VideoTriggersAudio, AudioTriggersVideo, etc.
    supporting_evidence JSON,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (video_segment_id) REFERENCES video_segments(id),
    FOREIGN KEY (audio_segment_id) REFERENCES audio_segments(id)
);

-- Synchronized multimodal contexts
CREATE TABLE IF NOT EXISTS multimodal_contexts (
    id TEXT PRIMARY KEY,
    timestamp DATETIME NOT NULL,
    window_start DATETIME NOT NULL,
    window_end DATETIME NOT NULL,
    video_events JSON NOT NULL, -- Array of video events in window
    audio_events JSON NOT NULL, -- Array of audio events in window
    correlations JSON NOT NULL, -- Array of event correlations
    fused_insights JSON NOT NULL, -- Array of generated insights
    confidence_scores JSON NOT NULL, -- ConfidenceScores object
    sync_quality REAL NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Speaker identification and tracking
CREATE TABLE IF NOT EXISTS speaker_identifications (
    id TEXT PRIMARY KEY,
    audio_segment_id TEXT NOT NULL,
    video_segment_id TEXT, -- May be correlated with visual speaker indicators
    speaker_id TEXT NOT NULL,
    speaker_name TEXT, -- Human-readable name if identified
    confidence REAL NOT NULL,
    voice_characteristics JSON, -- Pitch, rate, quality, accent
    visual_indicators JSON, -- Array of visual speaker cues if available
    correlation_confidence REAL, -- Confidence in audio-visual correlation
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (audio_segment_id) REFERENCES audio_segments(id),
    FOREIGN KEY (video_segment_id) REFERENCES video_segments(id)
);

-- Assistance opportunities and recommendations
CREATE TABLE IF NOT EXISTS assistance_opportunities (
    id TEXT PRIMARY KEY,
    multimodal_context_id TEXT,
    opportunity_type TEXT NOT NULL, -- CodingAssistance, DocumentationHelp, etc.
    description TEXT NOT NULL,
    trigger_context JSON NOT NULL, -- What triggered this opportunity
    suggested_actions JSON NOT NULL, -- Array of recommended actions
    urgency_level TEXT NOT NULL, -- Immediate, High, Medium, Low, Background
    confidence REAL NOT NULL,
    context_window_seconds INTEGER NOT NULL,
    status TEXT DEFAULT 'pending', -- pending, presented, accepted, dismissed
    user_feedback TEXT, -- User response to the opportunity
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (multimodal_context_id) REFERENCES multimodal_contexts(id)
);

-- Activity classification history
CREATE TABLE IF NOT EXISTS activity_classifications (
    id TEXT PRIMARY KEY,
    video_segment_id TEXT NOT NULL,
    primary_activity JSON NOT NULL, -- Activity enum with details
    secondary_activities JSON, -- Array of secondary activities
    context_indicators JSON NOT NULL, -- Array of supporting indicators
    evidence JSON NOT NULL, -- Array of evidence supporting classification
    confidence REAL NOT NULL,
    processing_time_ms INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (video_segment_id) REFERENCES video_segments(id)
);

-- Code analysis and snippets
CREATE TABLE IF NOT EXISTS code_analysis (
    id TEXT PRIMARY KEY,
    video_segment_id TEXT NOT NULL,
    ocr_content_id TEXT,
    programming_language TEXT,
    code_content TEXT NOT NULL,
    complexity_score REAL,
    syntax_elements JSON, -- Array of detected syntax elements
    error_indicators JSON, -- Array of detected errors/warnings
    suggestions JSON, -- Array of improvement suggestions
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (video_segment_id) REFERENCES video_segments(id),
    FOREIGN KEY (ocr_content_id) REFERENCES ocr_content(id)
);

-- Workflow and productivity patterns
CREATE TABLE IF NOT EXISTS workflow_patterns (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    pattern_type TEXT NOT NULL, -- ApplicationSwitching, TaskFocusing, etc.
    pattern_data JSON NOT NULL, -- Specific pattern information
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    frequency_score REAL, -- How often this pattern occurs
    efficiency_score REAL, -- Estimated productivity impact
    suggestions JSON, -- Optimization recommendations
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_ocr_content_video_segment ON ocr_content(video_segment_id);
CREATE INDEX IF NOT EXISTS idx_ocr_content_text_type ON ocr_content(text_type);
CREATE INDEX IF NOT EXISTS idx_ocr_content_confidence ON ocr_content(confidence);
CREATE INDEX IF NOT EXISTS idx_ocr_content_language ON ocr_content(language);

CREATE INDEX IF NOT EXISTS idx_visual_analysis_video_segment ON visual_analysis(video_segment_id);
CREATE INDEX IF NOT EXISTS idx_visual_analysis_timestamp ON visual_analysis(created_at);

CREATE INDEX IF NOT EXISTS idx_application_contexts_video_segment ON application_contexts(video_segment_id);
CREATE INDEX IF NOT EXISTS idx_application_contexts_app_type ON application_contexts(application_type);
CREATE INDEX IF NOT EXISTS idx_application_contexts_primary_app ON application_contexts(primary_application);

CREATE INDEX IF NOT EXISTS idx_event_correlations_video_segment ON event_correlations(video_segment_id);
CREATE INDEX IF NOT EXISTS idx_event_correlations_audio_segment ON event_correlations(audio_segment_id);
CREATE INDEX IF NOT EXISTS idx_event_correlations_type ON event_correlations(correlation_type);
CREATE INDEX IF NOT EXISTS idx_event_correlations_strength ON event_correlations(correlation_strength);

CREATE INDEX IF NOT EXISTS idx_multimodal_contexts_timestamp ON multimodal_contexts(timestamp);
CREATE INDEX IF NOT EXISTS idx_multimodal_contexts_window ON multimodal_contexts(window_start, window_end);
CREATE INDEX IF NOT EXISTS idx_multimodal_contexts_sync_quality ON multimodal_contexts(sync_quality);

CREATE INDEX IF NOT EXISTS idx_speaker_identifications_audio_segment ON speaker_identifications(audio_segment_id);
CREATE INDEX IF NOT EXISTS idx_speaker_identifications_video_segment ON speaker_identifications(video_segment_id);
CREATE INDEX IF NOT EXISTS idx_speaker_identifications_speaker_id ON speaker_identifications(speaker_id);
CREATE INDEX IF NOT EXISTS idx_speaker_identifications_confidence ON speaker_identifications(confidence);

CREATE INDEX IF NOT EXISTS idx_assistance_opportunities_context ON assistance_opportunities(multimodal_context_id);
CREATE INDEX IF NOT EXISTS idx_assistance_opportunities_type ON assistance_opportunities(opportunity_type);
CREATE INDEX IF NOT EXISTS idx_assistance_opportunities_urgency ON assistance_opportunities(urgency_level);
CREATE INDEX IF NOT EXISTS idx_assistance_opportunities_status ON assistance_opportunities(status);

CREATE INDEX IF NOT EXISTS idx_activity_classifications_video_segment ON activity_classifications(video_segment_id);
CREATE INDEX IF NOT EXISTS idx_activity_classifications_confidence ON activity_classifications(confidence);

CREATE INDEX IF NOT EXISTS idx_code_analysis_video_segment ON code_analysis(video_segment_id);
CREATE INDEX IF NOT EXISTS idx_code_analysis_language ON code_analysis(programming_language);
CREATE INDEX IF NOT EXISTS idx_code_analysis_complexity ON code_analysis(complexity_score);

CREATE INDEX IF NOT EXISTS idx_workflow_patterns_session ON workflow_patterns(session_id);
CREATE INDEX IF NOT EXISTS idx_workflow_patterns_type ON workflow_patterns(pattern_type);
CREATE INDEX IF NOT EXISTS idx_workflow_patterns_time ON workflow_patterns(start_time, end_time);

-- Full-text search for OCR content
CREATE VIRTUAL TABLE IF NOT EXISTS ocr_content_fts USING fts5(
    text_content,
    text_type,
    language,
    content='ocr_content',
    content_rowid='rowid'
);

-- Triggers to maintain FTS index
CREATE TRIGGER IF NOT EXISTS ocr_content_ai AFTER INSERT ON ocr_content BEGIN
    INSERT INTO ocr_content_fts(rowid, text_content, text_type, language)
    VALUES (new.rowid, new.text_content, new.text_type, new.language);
END;

CREATE TRIGGER IF NOT EXISTS ocr_content_ad AFTER DELETE ON ocr_content BEGIN
    INSERT INTO ocr_content_fts(ocr_content_fts, rowid, text_content, text_type, language)
    VALUES ('delete', old.rowid, old.text_content, old.text_type, old.language);
END;

CREATE TRIGGER IF NOT EXISTS ocr_content_au AFTER UPDATE ON ocr_content BEGIN
    INSERT INTO ocr_content_fts(ocr_content_fts, rowid, text_content, text_type, language)
    VALUES ('delete', old.rowid, old.text_content, old.text_type, old.language);
    INSERT INTO ocr_content_fts(rowid, text_content, text_type, language)
    VALUES (new.rowid, new.text_content, new.text_type, new.language);
END;