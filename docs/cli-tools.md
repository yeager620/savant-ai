# CLI Tools Guide

Savant AI provides a suite of UNIX philosophy CLI tools for composable audio transcription, LLM inference, and database management.

## Design Principles

- **Single Purpose**: Each tool does one thing well
- **Composability**: Tools pipe together seamlessly
- **JSON I/O**: Structured data for integration
- **Independence**: Tools work without the main application
- **Error Handling**: Errors to stderr, data to stdout

## savant-transcribe

Real-time audio transcription with speaker separation and post-processing.

### Basic Usage

```bash
# Record microphone for 10 seconds with English language
savant-transcribe --language en --duration 10

# Capture system audio (computer output)
savant-transcribe --system --duration 30

# Specify speaker for identification
savant-transcribe --speaker "john_doe" --duration 60
```

### Advanced Options

```bash
# Session-based recording for conversation grouping
savant-transcribe --session-id "meeting-123" --speaker "user"

# Custom Whisper model
savant-transcribe --model "models/ggml-large.bin" --language en

# Different output formats
savant-transcribe --format json --output transcript.json
savant-transcribe --format markdown --output transcript.md

# Device selection
savant-transcribe --device "USB Microphone" --duration 30
```

### JSON Output Format

```json
{
  "text": "Hello, this is a test transcription",
  "language": "en",
  "segments": [
    {
      "text": "Hello, this is a test transcription",
      "start_time": 0.0,
      "end_time": 3.2,
      "confidence": 0.95,
      "words": null
    }
  ],
  "processing_time_ms": 1250,
  "model_used": "models/ggml-base.en.bin",
  "session_metadata": {
    "session_id": "uuid-generated-or-provided",
    "timestamp": "2025-07-01T17:26:25Z",
    "audio_source": "Microphone",
    "speaker": "user_name",
    "device_info": "savant-transcribe-0.1.0"
  }
}
```

### Post-Processing Features

The transcription tool automatically handles common speech-to-text issues:

- **Silence Detection**: Replaces repetitive "you" patterns with `[unclear audio]`
- **Noise Filtering**: Converts isolated filler words to `[no signal]`
- **Speaker Separation**: Distinguishes microphone vs system audio sources
- **Language Forcing**: `--language en` prevents auto-detection issues

## savant-db

Database management for conversation history and transcript analytics.

### Storage Operations

```bash
# Store transcription from stdin
echo '{"text":"hello","segments":[...]}' | savant-db store --title "Test Conversation"

# Store with conversation context
savant-transcribe --speaker "user" | savant-db store --title "Daily Standup" --context "Team meeting"

# Create new conversation manually
savant-db create --title "Interview Session" --context "Technical interview with candidate"
```

### Querying & Search

```bash
# Query by speaker
savant-db query --speaker "john_doe" --limit 50

# Search content with full-text search
savant-db query --text "project alpha discussion"

# Time-based filtering
savant-db query --start "2025-07-01T00:00:00Z" --end "2025-07-01T23:59:59Z"

# Combined filters with pagination
savant-db query --speaker "user" --text "meeting" --limit 20 --offset 40
```

### Analytics & Statistics

```bash
# Speaker conversation statistics
savant-db stats

# List recent conversations
savant-db list --limit 20

# Export conversation for external analysis
savant-db export conversation-id-123 --output analysis.json
```

### Database Schema

```sql
-- Conversations table
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    title TEXT,
    start_time DATETIME,
    end_time DATETIME,
    context TEXT,
    participants TEXT  -- JSON array
);

-- Transcript segments table
CREATE TABLE segments (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    timestamp DATETIME,
    speaker TEXT,
    audio_source TEXT,  -- "Microphone", "SystemAudio", etc.
    text TEXT,
    start_time REAL,    -- Relative time within segment
    end_time REAL,
    confidence REAL,
    metadata TEXT       -- Full JSON blob
);
```

## savant-llm

Standalone LLM inference tool for various AI providers.

### Basic Usage

```bash
# Direct prompt with default model
savant-llm "What is Rust programming language?"

# Specify model and provider
savant-llm "Explain quantum computing" --model devstral --provider ollama

# Read from stdin (UNIX philosophy)
echo "Summarize this text: Lorem ipsum..." | savant-llm
```

### Provider Configuration

```bash
# Ollama (local inference)
savant-llm "Test prompt" --provider ollama --model devstral

# OpenAI
savant-llm "Test prompt" --provider openai --auth "sk-..." --model gpt-4

# DeepSeek (cost-effective)
savant-llm "Test prompt" --provider deepseek --auth "sk-..." 

# Anthropic
savant-llm "Test prompt" --provider anthropic --auth "sk-..." --model claude-3
```

### Advanced Features

```bash
# JSON input/output for composability
echo '{"prompt": "Hello", "model": "devstral"}' | savant-llm | jq '.content'

# Batch processing (one prompt per line)
cat prompts.txt | savant-llm batch

# Model listing and connection testing
savant-llm models --provider ollama
savant-llm test --provider openai --auth "sk-..."
```

## Pipeline Examples

### Complete Audio → Database Pipeline

```bash
# Record 30 seconds, identify speaker, store in database
savant-transcribe --speaker "user" --duration 30 | \
savant-db store --title "Voice Note $(date +%Y%m%d-%H%M)"

# System audio capture for call recording
savant-transcribe --system --speaker "call_audio" --duration 1800 | \
savant-db store --title "Client Call - $(date)" --context "Sales discussion"
```

### LLM Integration

```bash
# Transcribe audio and get AI summary
savant-transcribe --duration 60 | \
jq -r '.text' | \
savant-llm "Summarize this transcription: "

# Query database and get AI analysis
savant-db query --speaker "user" --limit 10 | \
jq -r '.[].text' | \
savant-llm "Analyze the sentiment of these conversations"
```

### Data Processing Workflows

```bash
# Export all conversations from last week
savant-db query --start "$(date -d '7 days ago' -Iseconds)" | \
jq '.[] | {speaker: .speaker, text: .text}' > weekly_transcripts.json

# Find all mentions of specific topics
savant-db query --text "project alpha" | \
jq -r '.[] | "\(.timestamp): \(.speaker) - \(.text)"' > project_mentions.txt

# Speaker statistics for time tracking
savant-db stats | \
jq '.[] | "\(.speaker): \(.total_duration_seconds/3600) hours"'
```

### Integration with External Tools

```bash
# Convert to different formats
savant-db export conv-123 | jq -r '.segments[].text' | pandoc -o transcript.pdf

# Send to external APIs
savant-transcribe --duration 30 | \
curl -X POST -H "Content-Type: application/json" \
     -d @- https://api.example.com/analyze

# Backup to cloud storage
savant-db query --start "2025-01-01" | \
gzip | aws s3 cp - s3://backup-bucket/transcripts-$(date +%Y%m).json.gz
```

## Error Handling

### Common Issues

```bash
# Model not found
savant-transcribe --model "nonexistent.bin"
# Error: Model file not found: nonexistent.bin

# Database connection issues
savant-db list
# Error: unable to open database file

# Audio device issues
savant-transcribe --device "Unknown Mic"
# Error: Audio device not found: Unknown Mic
```

### Debugging

```bash
# Verbose output
RUST_LOG=debug savant-transcribe --duration 5

# Test individual components
savant-llm test --provider ollama
savant-db create --title "Test" && echo "Database working"

# Check audio devices
savant-transcribe --list-devices
```

## Performance Optimization

### Audio Processing

```bash
# Use smaller models for speed
savant-transcribe --model "models/ggml-tiny.en.bin" --duration 60

# Batch processing for efficiency
find recordings/ -name "*.wav" | \
xargs -P 4 -I {} savant-transcribe --input {}
```

### Database Operations

```bash
# Limit results for large datasets
savant-db query --limit 100 --offset 0

# Use specific indexes
savant-db query --speaker "user" --start "2025-07-01T00:00:00Z"
```

## Configuration

### Environment Variables

```bash
# Default language for transcription
export SAVANT_DEFAULT_LANGUAGE="en"

# Database path
export SAVANT_DB_PATH="/custom/path/transcripts.db"

# Ollama endpoint
export OLLAMA_HOST="http://localhost:11434"
```

### CLI Aliases

```bash
# Convenient aliases
alias st="savant-transcribe"
alias sdb="savant-db"
alias sllm="savant-llm"

# Common workflows
alias record="st --speaker user --duration 60"
alias query-recent="sdb query --start $(date -d '1 day ago' -Iseconds)"
```