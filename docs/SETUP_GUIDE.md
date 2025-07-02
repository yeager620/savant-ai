# Savant AI Setup Guide

Complete setup and configuration guide for the Savant AI multimodal assistant.

## Quick Start

For new installations, run the automated setup:

```bash
./setup                      # Complete automated setup + guided permissions
./verify-permissions         # Verify system configuration
./start-daemons             # Start all systems
./monitor-daemons           # Real-time monitoring dashboard
```

## System Requirements

### Platform Support
- **macOS**: Full support with enhanced features
- **Linux**: Core functionality (limited multimodal features)
- **Windows**: Basic functionality only

### Hardware Requirements
- **Memory**: 8GB+ recommended for optimal performance
- **Storage**: ~1GB for dependencies + models
- **CPU**: Multi-core recommended for real-time multimodal analysis
- **Network**: Required for LLM provider setup (Ollama, OpenAI, etc.)

### Dependencies

**Core Dependencies** (Auto-installed by `./setup`):
- **Rust/Cargo**: Development environment
- **Ollama**: Local LLM runtime (`ollama serve`)
- **Tesseract**: OCR engine with language packs
- **ImageMagick**: Image preprocessing (optional, improves quality)

**macOS Specific**:
- **Xcode Command Line Tools**: Required for compilation
- **System Permissions**: Microphone, Screen Recording, Accessibility

## Installation Methods

### Method 1: Automated Setup (Recommended)

**For new users:**
```bash
# Clone repository
git clone <repository-url>
cd savant-ai

# Run automated setup
./setup

# Verify installation
./verify-permissions
./test-systems
```

The automated setup will:
1. Install all dependencies via Homebrew
2. Download and configure Ollama models
3. Build all Rust components
4. Guide you through macOS permissions setup
5. Test all systems and provide diagnostics

### Method 2: Manual Installation

**Step 1: Dependencies**
```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install core dependencies
brew install rust ollama tesseract imagemagick

# Install Ollama models
ollama pull llama3.2:3b    # Fast, lightweight model
ollama pull devstral       # Development-focused model
```

**Step 2: Build Project**
```bash
# Build all components
cargo build --release --workspace

# Verify build
cargo test --workspace
```

**Step 3: Manual Permissions Setup** (see [Permissions Section](#permissions-setup))

## Permissions Setup (macOS)

### Required Permissions Checklist

#### 1. Screen Recording Permission ⭐ **CRITICAL**
**Required for**: Video capture, OCR, computer vision analysis

**Setup**:
1. **System Preferences** → **Security & Privacy** → **Privacy** → **Screen Recording**
2. Click **lock icon** and enter password
3. Enable for your terminal application:
   - **Terminal** (default macOS terminal)
   - **iTerm2**, **Warp**, **VS Code** (if using alternative terminals)

**Verify**:
```bash
./sav-video test
./verify-permissions
```

#### 2. Microphone Permission ⭐ **CRITICAL**
**Required for**: Audio capture, speech-to-text, multimodal correlation

**Setup**:
1. **System Preferences** → **Security & Privacy** → **Privacy** → **Microphone**
2. Enable for your terminal application and any IDEs

**Verify**:
```bash
./sav test
./sav start && ./sav logs
```

#### 3. Accessibility Permission (Recommended)
**Required for**: Advanced UI detection, window management

**Setup**:
1. **System Preferences** → **Security & Privacy** → **Privacy** → **Accessibility**
2. Enable for your terminal application

#### 4. Full Disk Access (Optional)
**Required for**: Advanced file monitoring, system-wide analysis

### Automated Permissions Verification

```bash
./verify-permissions              # Check all permissions
./scripts/setup/permission-helper.sh  # Advanced troubleshooting
```

## Audio System Configuration

### Basic Audio Setup

**Microphone Only** (simplest):
```bash
# Test microphone capture
./sav start
./sav test
./sav logs  # Verify transcription working
```

### System Audio Capture (Advanced)

For capturing system audio (speakers, video calls, music), you need a virtual audio device.

#### Option 1: Multi-Output Device (Recommended)

**Advantages**: 
- Capture system audio while still hearing it
- No additional software required
- Works with all applications

**Setup Steps**:

1. **Open Audio MIDI Setup**:
   - Press `Cmd + Space`, type "Audio MIDI Setup"
   - Or go to Applications → Utilities → Audio MIDI Setup

2. **Install BlackHole** (virtual audio device):
   ```bash
   # Download and install
   open https://github.com/ExistentialAudio/BlackHole/releases
   # Install the .pkg file (BlackHole 2ch recommended)
   ```

3. **Create Multi-Output Device**:
   - In Audio MIDI Setup, click the **"+"** button → **Create Multi-Output Device**
   - Name it: **"Built-in + BlackHole"**
   - Check both boxes:
     - ✅ **Built-in Output** (so you can hear audio)
     - ✅ **BlackHole 2ch** (for Savant AI to capture)
   - Set **Built-in Output** as Master Device (right-click)

4. **Set as Default Output**:
   - **System Preferences** → **Sound** → **Output**
   - Select **"Built-in + BlackHole"**

5. **Configure Savant AI**:
   ```bash
   # Test system audio capture
   ./scripts/audio/audio-devices.sh    # List available devices
   ./sav start --device "BlackHole 2ch"
   ```

**Verify Setup**:
```bash
# Play some audio (music, video) and run:
./sav test
./sav logs  # Should show transcribed audio from system
```

#### Option 2: BlackHole Only (Alternative)

If you prefer to route audio manually:

```bash
# Install BlackHole
brew install --cask blackhole-2ch

# Set BlackHole as output in System Preferences → Sound
# Configure Savant AI to capture from BlackHole
./sav start --device "BlackHole 2ch"
```

**Note**: With this method, you won't hear system audio unless you manually route it.

## Development Setup

### Build Configurations

**Development build** (faster compilation):
```bash
cargo tauri dev              # Full app with hot reload
trunk serve                  # Frontend only (localhost:1420)
```

**Production build**:
```bash
cargo tauri build --release  # Optimized release build
```

### Testing Environment

```bash
# Run all tests
cargo test --workspace

# Test individual components  
cargo test --package savant-ocr
cargo test --package savant-vision
cargo test --package savant-sync

# Integration tests
./scripts/tests/test-mcp-natural-queries.sh    # MCP + LLM integration
./scripts/tests/test-database-sql.sh           # Database functionality
./test-systems                                 # Complete system test
```

## Configuration

### Configuration Files

**Main config**: `~/.config/savant-ai/config.toml`

```toml
[audio]
device = "BlackHole 2ch"           # Audio input device
language = "en"                    # Primary language
speaker_detection = true           # Enable speaker identification

[video_capture]
interval_seconds = 30              # Capture frequency
stealth_mode = true               # Invisible capture
enable_ocr = true                 # Text extraction
enable_vision_analysis = true     # App detection
enable_multimodal_correlation = true

[ocr]
engines = ["tesseract"]
languages = ["eng", "spa", "fra"]
min_confidence = 0.5
enable_text_classification = true
preprocessing_enabled = true

[vision]
enable_app_detection = true
enable_activity_classification = true
enable_ui_analysis = true
pattern_matching_threshold = 0.6

[multimodal]
correlation_window_seconds = 30
min_correlation_strength = 0.3
enable_predictive_sync = true
max_events_per_window = 100

[privacy]
enabled = true
blocked_applications = ["Zoom", "Google Meet", "Teams", "Slack"]
recording_schedule = "09:00-17:00"
notification_interval = 1800
require_explicit_consent = true

[storage]
base_path = "~/Library/Application Support/savant-ai"
max_storage_gb = 20
retention_days = 30
enable_compression = true

[llm]
provider = "ollama"               # ollama, openai, anthropic, deepseek
endpoint = "http://localhost:11434"
model = "llama3.2:3b"
timeout_seconds = 30

[mcp]
enabled = true
port = 3000
max_results = 1000
query_timeout = 30
rate_limit_per_minute = 60
max_complexity_per_minute = 100
context_retention_hours = 24
max_context_queries = 10
```

### Environment Variables

```bash
# Database configuration
export SAVANT_DB_PATH=~/.config/savant-ai/transcripts.db

# Logging
export RUST_LOG=info              # debug, info, warn, error

# LLM Configuration
export SAVANT_LLM_PROVIDER=ollama
export SAVANT_LLM_ENDPOINT=http://localhost:11434
export SAVANT_LLM_MODEL=llama3.2:3b

# Security settings
export SAVANT_MAX_RESULTS=1000
export SAVANT_QUERY_TIMEOUT=30
export SAVANT_RATE_LIMIT_PER_MINUTE=60

# Feature flags
export SAVANT_ENABLE_OCR=true
export SAVANT_ENABLE_VISION=true
export SAVANT_ENABLE_MULTIMODAL=true
```

## Daemon Management

### Quick Commands

```bash
# Start all systems
./start-daemons                    # Start audio + video + multimodal analysis

# Monitor systems
./monitor-daemons                  # Real-time monitoring dashboard

# Test systems
./test-systems                     # Comprehensive system test

# Stop systems
./stop-daemons                     # Stop all daemons cleanly
```

### Individual Daemon Control

```bash
# Audio daemon
./sav start|stop|status|logs|test

# Video daemon  
./sav-video start|stop|status|logs|test
```

### Monitoring and Logs

```bash
# Real-time monitoring
./monitor-daemons                  # Interactive dashboard

# View logs
./sav logs                         # Audio daemon logs
./sav-video logs                   # Video daemon logs
tail -f ~/.config/savant-ai/daemon-logs/*.log  # All logs

# System status
./sav status && ./sav-video status
```

## MCP Server Setup

The Model Context Protocol (MCP) server enables LLM integration for natural language database queries.

### Basic MCP Setup

```bash
# Build MCP server
cargo build --package savant-mcp --release

# Start MCP server (stdio mode)
./target/release/savant-mcp --log-level info

# Test MCP server
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/release/savant-mcp
```

### LLM Provider Integration

#### Ollama Integration (Local)

```bash
# Start Ollama server
ollama serve

# Install models
ollama pull llama3.2:3b          # Fast, lightweight
ollama pull llama3.2:8b          # Better reasoning
ollama pull devstral             # Development-focused

# Configure Savant AI
export SAVANT_LLM_PROVIDER=ollama
export SAVANT_LLM_ENDPOINT=http://localhost:11434
export SAVANT_LLM_MODEL=llama3.2:3b
```

#### Cloud Provider Setup

**OpenAI**:
```bash
export SAVANT_LLM_PROVIDER=openai
export OPENAI_API_KEY=your_api_key
export SAVANT_LLM_MODEL=gpt-4
```

**Anthropic**:
```bash
export SAVANT_LLM_PROVIDER=anthropic
export ANTHROPIC_API_KEY=your_api_key
export SAVANT_LLM_MODEL=claude-3-sonnet
```

**DeepSeek**:
```bash
export SAVANT_LLM_PROVIDER=deepseek
export DEEPSEEK_API_KEY=your_api_key
export SAVANT_LLM_MODEL=deepseek-chat
```

### MCP Client Examples

#### Claude Desktop Integration
Add to `~/.config/claude-desktop/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "savant-database": {
      "command": "/path/to/savant-ai/target/release/savant-mcp",
      "args": ["--log-level", "info"]
    }
  }
}
```

#### Continue.dev Integration
Add to `.continue/config.json`:

```json
{
  "models": [...],
  "mcpServers": [
    {
      "name": "savant-database",
      "command": "/path/to/savant-ai/target/release/savant-mcp"
    }
  ]
}
```

## Usage Examples

### Quick Start Workflow

```bash
# 1. Start everything
./start-daemons

# 2. Verify systems are working
./monitor-daemons
# Check that both audio and video daemons show "Running"

# 3. Test functionality
./sav test                         # Test audio transcription
./sav-video test                   # Test screen capture

# 4. Use CLI tools
cargo run --package savant-db -- list --limit 10
cargo run --package savant-ocr -- extract --input screenshot.png --fast
cargo run --package savant-llm -- "What were the main topics in recent conversations?"

# 5. Stop when done
./stop-daemons
```

### UNIX Pipeline Examples

```bash
# OCR → LLM analysis
cargo run --package savant-ocr -- extract --input screenshot.png --format text --fast | \
  cargo run --package savant-llm -- --model devstral | jq -r '.content'

# Vision analysis with filtering
cargo run --package savant-vision -- analyze --input screenshot.png --detect-apps --format json | \
  jq '.detected_applications[] | select(.confidence > 0.8)'

# Database integration
cargo run --package savant-ocr -- extract --input screenshot.png --format json | \
  cargo run --package savant-db -- store --type "screenshot_analysis"
```

## Troubleshooting

### Common Issues

#### "Operation not permitted" Errors
**Cause**: Missing Screen Recording or Microphone permissions
**Fix**: 
1. Grant permissions in System Preferences → Security & Privacy
2. Restart terminal application
3. Run `./verify-permissions`

#### "No audio devices found"
**Cause**: Audio drivers or permissions
**Fix**:
```bash
./scripts/audio/audio-devices.sh   # List available devices
./verify-permissions               # Check microphone permissions
# Check Audio MIDI Setup app for device configuration
```

#### "Screen capture failed"  
**Cause**: Screen Recording permission not granted
**Fix**:
```bash
./verify-permissions
# Enable Screen Recording in System Preferences → Security & Privacy
# May need to restart terminal/IDE after granting permission
```

#### "Ollama not responding"
**Cause**: Ollama server not running or firewall blocking
**Fix**:
```bash
ollama serve                       # Start Ollama server
curl http://localhost:11434/api/tags  # Test connection
./verify-permissions               # Check firewall settings
```

#### OCR "Unknown format" errors
**Cause**: Image format or Tesseract configuration
**Fix**:
```bash
brew reinstall tesseract
tesseract --list-langs             # Verify language packs installed
# Try preprocessing image:
sips -Z 1400 input.png --out output.png
```

#### Poor OCR Performance
**Fix**:
```bash
# Resize large images before processing
sips -Z 1400 input.png --out output.png

# Use fast mode for real-time processing
cargo run --package savant-ocr -- extract --input image.png --fast

# Check image quality and contrast
```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# Enable debug logging
export RUST_LOG=debug

# Run specific component in debug mode
RUST_LOG=debug ./sav start
RUST_LOG=debug cargo run --package savant-ocr -- extract --input screenshot.png

# View system logs
./monitor-daemons                  # Interactive monitoring
./sav logs && ./sav-video logs     # Daemon logs
```

### Performance Optimization

#### OCR Performance
- Use `--fast` flag for real-time processing (30x speedup)
- Resize images >2MP before processing
- Enable preprocessing for better accuracy vs speed

#### Memory Usage
- Monitor with `./monitor-daemons`
- Adjust retention settings in config
- Close unnecessary applications during heavy processing

#### Storage Management
- Configure retention policies in `config.toml`
- Use compression for long-term storage
- Regular cleanup: `./scripts/cleanup-databases.sh`

## Security Considerations

### Data Privacy
- All processing happens locally by default
- No data sent to cloud unless explicitly configured
- Audio/video data stored locally with encryption
- MCP server has read-only database access

### Network Security
- MCP server runs local-only by default
- Query validation prevents SQL injection
- Rate limiting on database queries
- User data remains on local machine

### File Permissions
- Database files have restricted permissions
- Log files in user directory only
- No sensitive data in process names or command lines

## Advanced Configuration

### Custom Model Integration

```bash
# Add custom Ollama model
ollama pull custom-model:latest
export SAVANT_LLM_MODEL=custom-model:latest

# Use different models for different tasks
export SAVANT_OCR_MODEL=vision-specialized-model
export SAVANT_TRANSCRIPTION_MODEL=audio-specialized-model
```

### Multi-Language Support

```toml
[ocr]
languages = ["eng", "spa", "fra", "deu", "jpn"]
default_language = "eng"
auto_detect_language = true

[audio] 
languages = ["en", "es", "fr", "de", "ja"]
default_language = "en"
auto_detect_language = true
```

### Enterprise Features

```toml
[enterprise]
sso_enabled = false
audit_logging = true
compliance_mode = "strict"    # strict, normal, permissive
data_retention_policy = "gdpr"

[monitoring]
metrics_enabled = true
health_check_interval = 30
performance_logging = true
```

## Support and Resources

### Getting Help

```bash
# Check system status
./verify-permissions
./test-systems

# View documentation
./docs/                           # Complete documentation
./docs/api/CLI_REFERENCE.md       # CLI tools reference

# Get component help
cargo run --package savant-ocr -- --help
cargo run --package savant-vision -- --help
./sav --help
./sav-video --help
```

### Community Resources
- GitHub Issues: Report bugs and feature requests
- Discussions: Community support and tips
- Wiki: Extended documentation and examples

### Professional Support
- Enterprise support available
- Custom integration services
- Training and onboarding