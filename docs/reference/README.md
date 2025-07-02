# Reference Documentation

Detailed technical reference for all Savant AI components.

## Command Line Interface

- **[CLI Tools](cli-tools.md)** - Complete command reference for all CLI utilities
- **[Configuration](configuration.md)** - Configuration file formats and environment variables

## Core Components

- **[Audio Transcription](audio-transcription.md)** - Speech-to-text processing and speaker identification
- **[Database System](database.md)** - SQLite schema, queries, and data management

## API Reference

### CLI Tool APIs
All tools follow UNIX conventions:
- JSON input/output for structured data
- Exit codes: 0 (success), 1 (error), 2 (invalid args)
- Errors to stderr, data to stdout
- Composable via pipes

### Data Formats

**JSON Output** (default for programmatic use):
```json
{
  "text": "transcribed content",
  "confidence": 0.95,
  "timestamp": "2025-07-02T12:00:00Z",
  "metadata": {...}
}
```

**Text Output** (for human reading):
```
transcribed content here
```

**Structured Output** (human-readable detailed):
```
Transcription Result:
  Text: transcribed content
  Confidence: 95%
  Language: English
```

## Performance Characteristics

- **OCR Fast Mode**: 0.9s per image (real-time suitable)
- **OCR Standard**: 28s per image (high accuracy)
- **Audio Transcription**: <5s latency for speaker ID
- **Video Capture**: Configurable intervals (1-300s)
- **Database Queries**: <100ms for typical operations

## Configuration Reference

See individual component documentation for detailed configuration options:
- Audio: Device selection, language settings, speaker detection
- Video: Capture intervals, privacy controls, analysis options
- OCR: Engine selection, language packs, confidence thresholds
- Vision: Application detection, activity classification
- Database: Storage paths, retention policies, backup settings