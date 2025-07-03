# CLI Tools Reference

Savant AI provides composable UNIX-style CLI tools for multimodal AI processing. Each tool follows the principle of "do one thing well" and can be combined via pipes.

## Design Principles

- **Single Purpose**: Each tool has one clear responsibility
- **Composability**: Output of one tool can be input to another
- **JSON I/O**: Structured data exchange between tools
- **Independence**: Tools work without the main GUI application
- **Standard Streams**: Data to stdout, errors to stderr

## Core CLI Tools

### LLM Inference (`savant-llm`)

Direct LLM inference without GUI dependencies.

```bash
# Basic inference
cargo run --bin savant-llm -- "prompt" --model devstral
echo "prompt" | savant-llm | jq '.content'

# With specific model
cargo run --package savant-llm -- "Analyze this text" --model devstral
```

### Audio Transcription (`savant-transcribe`)

Real-time audio capture and speech-to-text processing.

```bash
# Basic transcription
cargo run --package savant-transcribe -- --language en --duration 10

# With speaker identification
cargo run --package savant-transcribe -- --speaker "user" --output transcript.json

# System audio capture (requires BlackHole setup)
cargo run --package savant-transcribe -- --duration 300 --device 'BlackHole 2ch' --format json

# Continuous transcription
cargo run --package savant-transcribe -- --continuous --language en
```

### Video Capture (`savant-video-cli`)

Screen capture with multimodal analysis capabilities.

```bash
# Basic capture
cargo run --package savant-video-cli -- start --interval 30 --duration 3600

# With OCR and vision analysis
cargo run --package savant-video-cli -- start --interval 30 --enable-ocr --enable-vision --enable-correlation

# Single screenshot with analysis
cargo run --package savant-video-cli -- capture --stealth --format png
```

### OCR Text Extraction (`savant-ocr`)

Text extraction from images with semantic classification.

```bash
# Fast extraction (optimized for real-time)
cargo run --package savant-ocr -- extract --input screenshot.png --format text --fast

# Full analysis with classification
cargo run --package savant-ocr -- extract --input screenshot.png --classify --analyze --format json

# Specify language and confidence
cargo run --package savant-ocr -- extract --input screenshot.png --languages "eng,spa" --confidence 0.7

# Test OCR capabilities
cargo run --package savant-ocr -- test --engine tesseract
```

**Performance optimization:**
```bash
# Resize large images first (recommended for >2MP)
sips -Z 1400 input.png --out output.png
cargo run --package savant-ocr -- extract --input output.png --fast
```

### Computer Vision (`savant-vision`)

Visual analysis for application detection and activity classification.

```bash
# Comprehensive analysis
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --classify-activity --detect-ui

# Application detection only
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --format json

# Activity classification
cargo run --package savant-vision -- analyze --input screenshot.png --classify-activity

# Performance benchmark
cargo run --package savant-vision -- benchmark --input screenshot.png --iterations 10
```

### Multimodal Synchronization (`savant-sync`)

Event correlation and timeline management for audio-video data.

```bash
# Correlate events
cargo run --package savant-sync -- correlate --window-size 30 --min-strength 0.3

# Analyze time window
cargo run --package savant-sync -- analyze --start "2025-01-01T10:00:00Z" --end "2025-01-01T11:00:00Z"

# Test correlation algorithms
cargo run --package savant-sync -- test --count 20
```

### Database Management (`savant-db`)

SQLite database operations with natural language queries.

```bash
# List recent conversations
cargo run --package savant-db -- list --limit 10

# Query by speaker
cargo run --package savant-db -- query --speaker "user" --text "meeting"

# Database statistics
cargo run --package savant-db -- stats

# Backup database
cargo run --package savant-db -- backup --output backup.db
```

### MCP Server (`savant-mcp`)

Model Context Protocol server for LLM integration.

```bash
# Start MCP server
cargo run --package savant-mcp -- --llm-provider ollama

# Test mode
cargo run --package savant-mcp -- --test

# Test requests via stdin
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | savant-mcp-server --test
```

## Daemon Management Scripts

### Quick Commands

```bash
# Start all daemons
./start-daemons

# Stop all daemons
./stop-daemons

# Monitor real-time status
./monitor-daemons

# Test all systems
./test-systems

# Check permissions
./verify-permissions
```

### Individual Daemon Control

```bash
# Audio daemon
./sav start|stop|status|logs|test

# Video daemon
./sav-video start|stop|status|logs|list|test
```

## UNIX Philosophy Examples

### Pipeline Processing

```bash
# OCR → LLM analysis
cargo run --package savant-ocr -- extract --input screenshot.png --format text --fast | \
  cargo run --package savant-llm -- --model llama3.2 | jq -r '.content'

# Vision analysis with filtering
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --format json | \
  jq '.detected_applications[] | select(.confidence > 0.8)'

# Database integration
cargo run --package savant-ocr -- extract --input screenshot.png --format json | \
  cargo run --package savant-db -- store --type "screenshot_analysis"
```

### Multi-step Analysis

```bash
# Sequential processing
cargo run --package savant-ocr -- extract --input screenshot.png --classify --format json > ocr.json
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --format json > vision.json

# Combine results
jq -s '.[0] + .[1]' ocr.json vision.json > combined_analysis.json
```

## Output Formats

### JSON (Default)
Structured data for programmatic processing:
```json
{
  "text": "transcribed content",
  "confidence": 0.95,
  "timestamp": "2025-07-02T12:00:00Z",
  "metadata": {...}
}
```

### Text
Plain text for human reading or simple pipelines:
```
transcribed content here
```

### Structured
Human-readable structured output:
```
Transcription Result:
  Text: transcribed content
  Confidence: 95%
  Duration: 10.5s
  Language: English
```

## Error Handling

All tools follow UNIX conventions:
- **Exit Code 0**: Success
- **Exit Code 1**: General error
- **Exit Code 2**: Invalid arguments
- **Errors to stderr**: Error messages don't pollute data pipelines
- **Data to stdout**: Clean output for piping

## Performance Tips

- Use `--fast` flag for real-time OCR processing (30x speedup)
- Resize large images before processing
- Enable verbose logging with `RUST_LOG=debug` for troubleshooting
- Use JSON output for programmatic processing
- Use text output for simple scripts and human reading

## Integration Examples

### With External Tools

```bash
# Integration with standard UNIX tools
cargo run --package savant-transcribe -- --duration 10 | \
  jq -r '.text' | \
  wc -w  # Count words in transcription

# Integration with other analysis tools
cargo run --package savant-vision -- analyze --input screenshot.png --format json | \
  jq '.detected_applications[].name' | \
  sort | uniq -c  # Count unique applications
```

### Configuration

Most tools respect configuration in `~/.config/savant-ai/config.toml`:

```toml
[ocr]
engines = ["tesseract"]
languages = ["eng", "spa"]
min_confidence = 0.5

[vision]
enable_app_detection = true
enable_activity_classification = true

[privacy]
blocked_applications = ["Zoom", "Teams"]
recording_schedule = "09:00-17:00"
```

See `docs/user-guides/CONFIGURATION.md` for complete configuration reference.