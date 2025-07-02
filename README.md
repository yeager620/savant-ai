# Savant AI

Invisible AI assistant with multimodal intelligence: real-time audio transcription, intelligent screen capture with OCR and computer vision, and composable UNIX CLI tools. Features stealth screen capture, context-aware analysis, and seamless LLM integration.

*macOS only*

## Features

- **Multimodal Intelligence**: Audio-video correlation with context understanding
- **Screen Capture + OCR**: Stealth screen capture with intelligent text extraction (0.9s fast mode)
- **Computer Vision**: Application detection, activity classification, UI analysis
- **Audio Pipeline**: Real-time speech-to-text with speaker separation + Whisper processing
- **Smart Database**: SQLite with multimodal analytics + MCP server for LLM integration  
- **Natural Language Queries**: Plain English database queries via chat interface
- **UNIX CLI Tools**: Composable command-line utilities following data pipeline principles
- **LLM Integration**: Multi-provider support (Ollama, OpenAI, DeepSeek, Anthropic)
- **Privacy First**: Stealth mode, app blocking, explicit consent requirements

## Quick Start

```bash
# Install dependencies
brew install ollama tesseract imagemagick
ollama pull devstral && ollama serve

# Clone and setup
git clone <repo> && cd savant-ai

# Start all multimodal systems
./start-daemons                   # Audio + Video + AI analysis

# Monitor system status
./monitor-daemons                 # Real-time dashboard

# Test everything is working
./test-systems                    # Comprehensive test suite

# Run main application
cargo tauri dev
```

## **Integrated Daemon Management**

### **One-Command Operations**
```bash
# Convenience commands (recommended)
./start-daemons                 # Start audio + video + multimodal analysis
./stop-daemons                  # Stop all daemons gracefully
./monitor-daemons               # Real-time monitoring dashboard  
./test-systems                  # Test all components

# Full script paths
./scripts/daemon-management/start_all_daemons.sh
./scripts/daemon-management/stop_all_daemons.sh
./scripts/daemon-management/restart_daemons.sh
./scripts/daemon-management/monitor_daemons.sh
./scripts/daemon-management/test_all_systems.sh
```

### **What Each Script Does**

#### **start_all_daemons.sh**
- Checks dependencies (ollama, tesseract, imagemagick)
- Installs devstral model if missing
- Starts Ollama server
- Launches audio daemon
- Launches video daemon with OCR + vision + correlation
- Shows status verification

#### **monitor_daemons.sh**  
- Real-time daemon status monitoring
- System resource usage
- Recent log entries from both daemons
- Auto-refreshes every 5 seconds

#### **test_all_systems.sh**
- Verifies all dependencies installed
- Tests Ollama connectivity
- Tests each component (OCR, vision, sync)
- Tests with sample images if available
- Database connectivity check

## CLI Tools

### Quick Start Examples
```bash
# OCR text extraction (fast mode for real-time)
cargo run --package savant-ocr -- extract --input screenshot.png --format text --fast

# Computer vision analysis
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --classify-activity

# Video capture with multimodal analysis
./sav-video start --enable-ocr --enable-vision

# Multimodal synchronization
cargo run --package savant-sync -- test --count 10

# LLM inference
echo "prompt" | savant-llm --model devstral | jq '.content'
```

### Complete CLI Reference

#### OCR Text Extraction (savant-ocr)
```bash
# Basic usage with performance options
cargo run --package savant-ocr -- extract --input screenshot.png --fast          # 0.9s processing
cargo run --package savant-ocr -- extract --input screenshot.png --classify --analyze  # Full analysis (28s)

# Output formats
cargo run --package savant-ocr -- extract --input screenshot.png --format text
cargo run --package savant-ocr -- extract --input screenshot.png --format structured --classify

# Multi-language support
cargo run --package savant-ocr -- extract --input screenshot.png --languages "eng,spa" --confidence 0.7

# Testing
cargo run --package savant-ocr -- test                           # Built-in test
cargo run --package savant-ocr -- test --input custom_image.png  # Test with your image
```

#### Computer Vision (savant-vision)
```bash
# Application and activity detection
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --classify-activity

# UI analysis with different output formats
cargo run --package savant-vision -- analyze --input screenshot.png --detect-ui --format detailed
cargo run --package savant-vision -- analyze --input screenshot.png --benchmark --format summary

# Testing
cargo run --package savant-vision -- test
```

#### Multimodal Synchronization (savant-sync)
```bash
# Event correlation
cargo run --package savant-sync -- correlate --window-size 30 --algorithms temporal,semantic

# Time window analysis
cargo run --package savant-sync -- analyze --start "2025-01-01T10:00:00Z" --end "2025-01-01T11:00:00Z"
cargo run --package savant-sync -- analyze --start "2025-01-01T10:00:00Z" --end "2025-01-01T11:00:00Z" --format summary

# Testing with sample data
cargo run --package savant-sync -- test --count 20
```

#### Video Capture System (sav-video)
```bash
# Individual daemon management
./sav-video start --interval 30 --enable-ocr --enable-vision --enable-correlation
./sav-video status
./sav-video logs
./sav-video stop
./sav-video test

# Integrated management (recommended)
./start-daemons                  # Start both audio and video daemons
./monitor-daemons                # Monitor all systems in real-time
./stop-daemons                   # Stop all daemons
```

#### Database Management (savant-db)
```bash
# Query operations
savant-db --db-path ~/.config/savant-ai/transcripts.db list --limit 10
savant-db --db-path ./data.db query --speaker "john" --text "meeting"
savant-db --db-path ./data.db stats
```

#### Pipeline Examples (UNIX Philosophy)
```bash
# OCR → LLM analysis pipeline
cargo run --package savant-ocr -- extract --input screenshot.png --format text --fast | \
  cargo run --package savant-llm -- --model devstral | jq -r '.content'

# Vision analysis with filtering
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --format json | \
  jq '.detected_applications[] | select(.confidence > 0.8)'

# Database integration
cargo run --package savant-ocr -- extract --input screenshot.png --format json | \
  savant-db store --type "screenshot_analysis"
```

#### Image Preprocessing (for better OCR performance)
```bash
# Resize large images before OCR (recommended for >2M pixels)
sips -Z 1400 screenshot.png --out screenshot_small.png           # macOS native tool
convert screenshot.png -resize 1400x screenshot_small.png        # ImageMagick (better quality)

# Then process the resized image
cargo run --package savant-ocr -- extract --input screenshot_small.png --fast
```

#### Testing & Diagnostics
```bash
# Component tests
cargo test --package savant-ocr
cargo test --package savant-vision
cargo test --package savant-sync
cargo test --workspace

# Integration tests
./scripts/tests/test-mcp-natural-queries.sh    # MCP + LLM integration
./scripts/tests/test-database-sql.sh           # Database functionality

# Performance testing with logging
RUST_LOG=debug cargo run --package savant-ocr -- extract --input large_image.png --fast
```

## Architecture

### Desktop App
- **Frontend**: Leptos WASM (taskbar UI)
- **Backend**: Tauri (commands, tray, stealth)

### UNIX Tools
- `savant-ocr` - OCR text extraction and classification (0.9s fast mode)
- `savant-vision` - Computer vision analysis and app detection
- `savant-sync` - Multimodal event correlation and synchronization
- `savant-video-cli` - Screen capture with multimodal analysis
- `savant-transcribe` - Audio → text with speaker ID
- `savant-db` - Database management + MCP server
- `savant-llm` - LLM inference engine
- `savant-mcp` - Model Context Protocol server

### Data Flow
```
# Multimodal Intelligence Pipeline
Screen Capture → OCR + Vision Analysis → Database → Context Correlation
Microphone → Audio Capture → Whisper STT → Speaker Detection → Sync Engine
Multimodal Data → MCP Server → External LLMs → Proactive Assistance

# Legacy Audio Pipeline
System Audio → Audio Capture → Speaker Detection → Analytics → Natural Language Queries
```

## MCP Integration

Smart database server exposing conversation data to LLMs via JSON-RPC 2.0:

```bash
# Start MCP server
savant-mcp --llm-provider ollama

# Query conversations naturally
curl -X POST stdin <<< '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"query_conversations","arguments":{"query":"Find meetings about project alpha"}}}'
```

**Tools**: `query_conversations`, `get_speaker_analytics`, `search_semantic`, `query_multimodal_context`, `get_current_activity`, `find_assistance_opportunities`
**Resources**: `conversations://list`, `speakers://list`, `activities://list`, `applications://list`, `multimodal_contexts://list`, `schema://database`

## Development

```bash
cargo tauri dev              # Full app development
cargo test --workspace       # Run all tests
cargo build --release        # Production build

# Individual components
trunk serve                   # Frontend only
cargo run --package savant-db -- --help
```

## Configuration

`~/.config/savant-ai/config.toml`:
- AI provider settings + API keys
- Audio transcription preferences  
- Browser monitoring configuration
- Stealth mode options

## Script Organization

All management scripts are organized in `scripts/daemon-management/` with convenience wrappers in the project root:

```
scripts/
├── daemon-management/          # Integrated system management
│   ├── start_all_daemons.sh   # Complete startup sequence
│   ├── stop_all_daemons.sh    # Graceful shutdown
│   ├── restart_daemons.sh     # Clean restart
│   ├── monitor_daemons.sh     # Real-time dashboard
│   └── test_all_systems.sh    # Comprehensive testing
├── audio/                     # Audio daemon scripts
├── video/                     # Video daemon scripts
├── tests/                     # Integration tests
└── setup/                     # Installation scripts

# Convenience commands (project root)
start-daemons, stop-daemons, monitor-daemons, test-systems
```

## Platform Requirements

- **macOS**: Screen Recording + Accessibility API + microphone permissions
- **Tesseract OCR**: Text extraction engine (`brew install tesseract`)
- **Ollama**: Local LLM runtime (`ollama serve`)
- **ImageMagick**: Optional, for better image resizing (`brew install imagemagick`)
- **Dependencies**: Built into Cargo workspace

## Performance Optimization

**OCR Processing:**
- Fast mode: 0.9s per image (real-time suitable)
- Standard mode: 28s per image (high accuracy)
- Automatic image resizing for large screenshots
- Built-in integer overflow protection

**Image Preprocessing:**
```bash
# Resize large images before OCR (recommended)
sips -Z 1400 screenshot.png --out screenshot_small.png

# Or with ImageMagick for better quality
convert screenshot.png -resize 1400x screenshot_small.png
```

## UNIX Philosophy

Each tool does one thing well:
- **Single Purpose**: Focused, testable components
- **Text Streams**: JSON I/O for data exchange
- **Composability**: Tools pipe together naturally
- **Independence**: Can run without main application