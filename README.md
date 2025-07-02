# Savant AI

Invisible AI assistant with real-time audio transcription, smart database, and UNIX philosophy CLI tools. Operates as seamless sidebar with browser monitoring and stealth capabilities.

*macOS only*

## Features

- **Audio Pipeline**: Real-time speech-to-text with speaker separation + Whisper processing
- **Smart Database**: SQLite with analytics + MCP server for LLM integration  
- **Natural Language Queries**: Plain English database queries via chat interface
- **UNIX CLI Tools**: Composable command-line utilities following data pipeline principles
- **LLM Integration**: Multi-provider support (Ollama, OpenAI, DeepSeek, Anthropic)
- **Browser Monitoring**: Accessibility API content detection
- **Stealth Mode**: Hidden from screenshots with system tray operation

## Quick Start

```bash
# Dependencies
brew install ollama
ollama pull devstral && ollama serve

# Run application
git clone <repo> && cd savant-ai
cargo tauri dev

# Test CLI tools
./test-mcp-natural-queries.sh  # MCP + LLM integration
./test-database-sql.sh         # Direct database access
```

## CLI Tools

```bash
# Audio transcription
savant-transcribe --speaker "user" --duration 10

# Database queries  
savant-db list --limit 10
savant-db query --speaker "john" --text "meeting"

# LLM inference
echo "prompt" | savant-llm --model devstral | jq '.content'

# MCP server for external LLMs
savant-mcp --llm-provider ollama
```

## Architecture

### Desktop App
- **Frontend**: Leptos WASM (taskbar UI)
- **Backend**: Tauri (commands, tray, stealth)

### UNIX Tools
- `savant-transcribe` - Audio → text with speaker ID
- `savant-db` - Database management + MCP server
- `savant-llm` - LLM inference engine
- `savant-mcp` - Model Context Protocol server

### Data Flow
```
Microphone → Audio Capture → Whisper STT → Database → MCP Server → External LLMs
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

**Tools**: `query_conversations`, `get_speaker_analytics`, `search_semantic`, `get_database_stats`
**Resources**: `conversations://list`, `speakers://list`, `schema://database`

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

## Platform Requirements

- **macOS**: Accessibility API + microphone permissions
- **Ollama**: Local LLM runtime (`ollama serve`)
- **Dependencies**: Built into Cargo workspace

## UNIX Philosophy

Each tool does one thing well:
- **Single Purpose**: Focused, testable components
- **Text Streams**: JSON I/O for data exchange
- **Composability**: Tools pipe together naturally
- **Independence**: Can run without main application