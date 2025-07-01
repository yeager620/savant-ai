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

Advanced database management with speaker identification, semantic search, and conversation analytics.

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
# Basic queries by speaker and time
savant-db query --speaker "john_doe" --limit 50
savant-db query --start "2025-07-01T00:00:00Z" --end "2025-07-01T23:59:59Z"

# Advanced semantic search
savant-db search "project deadline discussion" --limit 10 --threshold 0.7
savant-db search "budget concerns" --speaker "manager" --limit 5

# Combined filters with pagination
savant-db query --speaker "user" --text "meeting" --limit 20 --offset 40
```

### Speaker Management

```bash
# List all speakers with statistics
savant-db speaker list

# Create new speaker profile
savant-db speaker create --name "John Doe"

# Show detailed speaker information
savant-db speaker show speaker-uuid-here

# Find potential duplicate speakers
savant-db speaker duplicates

# Merge two speaker profiles
savant-db speaker merge primary-speaker-id secondary-speaker-id
```

### Analytics & Insights

```bash
# Overall database statistics
savant-db stats

# Analyze specific conversation
savant-db analyze conversation-uuid-here

# Topic management
savant-db topic list conversation-uuid-here
savant-db topic extract conversation-uuid-here

# List recent conversations
savant-db list --limit 20

# Export conversation for external analysis
savant-db export conversation-id-123 --output analysis.json
```

### Enhanced Database Schema

```sql
-- Speaker profiles with analytics
CREATE TABLE speakers (
    id TEXT PRIMARY KEY,
    name TEXT,
    voice_embedding BLOB,               -- Voice biometrics
    total_conversation_time REAL,
    total_conversations INTEGER,
    last_interaction TIMESTAMPTZ
);

-- Time-partitioned conversations
CREATE TABLE conversations (
    id UUID PRIMARY KEY,
    start_time TIMESTAMPTZ NOT NULL,   -- Partition key
    participant_ids UUID[],             -- Array of speaker IDs
    summary TEXT,
    topics JSONB,                       -- Auto-extracted topics
    sentiment_score REAL,
    quality_score REAL
);

-- Speaker relationship analytics
CREATE TABLE speaker_relationships (
    speaker_a_id TEXT,
    speaker_b_id TEXT,
    total_conversations INTEGER,
    total_duration REAL,
    relationship_strength REAL,
    common_topics TEXT,                 -- JSON array
    PRIMARY KEY (speaker_a_id, speaker_b_id)
);

-- Enhanced transcript segments
CREATE TABLE conversation_segments (
    id UUID PRIMARY KEY,
    conversation_id UUID REFERENCES conversations(id),
    speaker_id UUID REFERENCES speakers(id),
    original_text TEXT,
    processed_text TEXT,                -- Cleaned/normalized
    semantic_embedding BLOB,            -- 384-dim for similarity search
    timestamp TIMESTAMPTZ NOT NULL
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

# Continuous background capture with automatic speaker identification
./sav start  # Daemon handles continuous processing
```

### Speaker Intelligence Workflows

```bash
# Find all conversations with specific person
savant-db query --speaker "john_doe" --limit 50

# Analyze relationship between two people
savant-db speaker show john_doe  # Shows interaction statistics

# Merge duplicate speaker profiles
savant-db speaker duplicates     # Find potential duplicates
savant-db speaker merge primary-id secondary-id

# Track conversation patterns over time
savant-db analyze timeline --speaker "john_doe" --timeframe weekly
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

# Semantic search with AI-powered insights
savant-db search "project concerns" --limit 5 | \
jq -r '.[].text' | \
savant-llm "What are the main issues mentioned in these conversations?"
```

### Advanced Analytics Workflows

```bash
# Speaker relationship analysis
savant-db speaker list | jq -r '.[].id' | while read speaker; do
  echo "=== $speaker ==="
  savant-db analyze relationships --speaker "$speaker"
done

# Topic extraction and analysis
savant-db list --limit 20 | jq -r '.[].id' | while read conv_id; do
  savant-db topic extract "$conv_id"
  savant-db topic list "$conv_id"
done

# Export speaker interaction matrix
savant-db export-relationships --format csv > speaker_matrix.csv

# Conversation quality analysis
savant-db query --limit 100 | \
jq '.[] | select(.quality_score < 0.8) | .id' | \
while read conv_id; do
  echo "Low quality conversation: $conv_id"
  savant-db analyze "$conv_id"
done
```

### Data Processing Workflows

```bash
# Export all conversations from last week with speaker attribution
savant-db query --start "$(date -d '7 days ago' -Iseconds)" | \
jq '.[] | {speaker: .speaker_name, text: .text, timestamp: .timestamp}' > weekly_transcripts.json

# Find semantic matches across conversations
savant-db search "project alpha" --limit 20 | \
jq -r '.[] | "\(.timestamp): \(.speaker_name) - \(.text)"' > project_mentions.txt

# Speaker time tracking and analytics
savant-db speaker list | \
jq '.[] | "\(.name // .id): \(.total_conversation_time/3600 | round) hours, \(.total_conversations) conversations"'

# Generate conversation network graph data
savant-db export-relationships --format json | \
jq '.relationships[] | {source: .speaker_a_name, target: .speaker_b_name, weight: .relationship_strength}' > network.json
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