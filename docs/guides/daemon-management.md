# Daemon Management Guide

Complete guide for managing Savant AI daemons and services.

## Overview

Savant AI runs multiple daemons for real-time multimodal processing:

- **Audio Daemon**: Microphone capture and speech-to-text processing
- **Video Daemon**: Screen capture with OCR and computer vision analysis
- **Ollama Server**: Local LLM processing for natural language queries
- **Multimodal Sync**: Cross-modal event correlation and intelligence

## Quick Start

### Unified Daemon Manager (Recommended)

```bash
# Start all daemons
./savant-daemon start

# Check status
./savant-daemon status

# Monitor in real-time
./savant-daemon monitor

# Stop all daemons
./savant-daemon stop
```

### Convenience Wrappers (Alternative)

```bash
# Start all daemons
./start-daemons

# Monitor status
./monitor-daemons

# Stop all daemons
./stop-daemons
```

## Unified Daemon Manager

The `savant-daemon` script provides a single interface for all daemon operations:

### Basic Commands

```bash
# Core daemon operations
./savant-daemon start           # Start all daemons
./savant-daemon stop            # Stop all daemons gracefully
./savant-daemon restart         # Restart all daemons
./savant-daemon status          # Show daemon status
./savant-daemon monitor         # Real-time monitoring dashboard
./savant-daemon test            # Test all systems
./savant-daemon logs            # Show recent logs
./savant-daemon health          # Quick health check

# Individual daemon control
./savant-daemon start-audio     # Start only audio daemon
./savant-daemon stop-audio      # Stop only audio daemon
./savant-daemon start-video     # Start only video daemon
./savant-daemon stop-video      # Stop only video daemon
```

### Options

```bash
# Verbose output for debugging
./savant-daemon start --verbose
./savant-daemon logs --verbose

# Help information
./savant-daemon --help
```

## Daemon Components

### Audio Daemon

**Purpose**: Real-time audio capture and speech-to-text processing
**Script**: `./sav`
**Configuration**: `~/.config/savant-ai/config.toml`

```bash
# Individual control
./sav start                     # Start audio daemon
./sav stop                      # Stop audio daemon
./sav status                    # Check status
./sav logs                      # View logs
./sav test                      # Test audio functionality
```

**Features**:
- Microphone and system audio capture
- Real-time speech-to-text transcription
- Speaker identification and analytics
- Multi-language support

### Video Daemon

**Purpose**: Screen capture with OCR and computer vision analysis
**Script**: `./sav-video`
**Configuration**: `~/.config/savant-ai/config.toml`

```bash
# Individual control
./sav-video start               # Start video daemon
./sav-video stop                # Stop video daemon
./sav-video status              # Check status
./sav-video logs                # View logs
./sav-video test                # Test video functionality
```

**Features**:
- Stealth screen capture (every 30 seconds by default)
- OCR text extraction with semantic classification
- Computer vision for app detection and activity classification
- Privacy controls and blocked application filtering

### Ollama Server

**Purpose**: Local LLM processing for natural language queries
**Command**: `ollama serve`
**Port**: 11434 (default)

```bash
# Manual control (usually handled automatically)
ollama serve                    # Start Ollama server
curl http://localhost:11434/api/tags  # Test connection
ollama list                     # Show installed models
ollama pull devstral            # Install required model
```

## Monitoring and Status

### Real-time Monitoring

The monitoring dashboard provides live status updates:

```bash
./savant-daemon monitor
```

**Dashboard Features**:
- Live daemon status with color coding
- System resource usage (CPU, memory, disk)
- Recent log entries from all daemons
- Auto-refresh every 5 seconds
- Press Ctrl+C to exit

### Status Checks

```bash
# Comprehensive status
./savant-daemon status

# Quick health check
./savant-daemon health

# Verbose status with system resources
./savant-daemon status --verbose
```

### Log Analysis

```bash
# Recent logs from all daemons
./savant-daemon logs

# Detailed logs
./savant-daemon logs --verbose

# Individual daemon logs
./sav logs
./sav-video logs

# Direct log file access
tail -f ~/.config/savant-ai/daemon-logs/video-daemon.log
tail -f ./data/daemon-logs/savant-audio-daemon.log
```

## Configuration

### Global Configuration

Main configuration file: `~/.config/savant-ai/config.toml`

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

[privacy]
enabled = true
blocked_applications = ["Zoom", "Google Meet", "Teams", "Slack"]
recording_schedule = "09:00-17:00"
require_explicit_consent = true
```

### Environment Variables

```bash
# Logging level
export RUST_LOG=info              # debug, info, warn, error

# LLM Configuration
export SAVANT_LLM_PROVIDER=ollama
export SAVANT_LLM_ENDPOINT=http://localhost:11434
export SAVANT_LLM_MODEL=devstral

# Database path
export SAVANT_DB_PATH=~/.config/savant-ai/transcripts.db
```

## Advanced Operations

### Startup Sequence

When running `./savant-daemon start`, the following happens:

1. **Dependency Check**: Verify Ollama, Tesseract, ImageMagick installation
2. **Model Installation**: Download required LLM models if missing
3. **Ollama Server**: Start LLM server and verify connectivity
4. **Audio Daemon**: Start microphone capture and transcription
5. **Video Daemon**: Start screen capture with multimodal analysis
6. **Health Check**: Verify all components are functioning
7. **Status Report**: Display final system status

### Graceful Shutdown

When running `./savant-daemon stop`, the following happens:

1. **Video Daemon**: Stop screen capture and analysis (dependency order)
2. **Audio Daemon**: Stop audio capture and transcription
3. **Cleanup**: Ensure all processes are properly terminated
4. **Verification**: Confirm all daemons have stopped

### Custom Startup Options

```bash
# Start with specific video capture interval
./sav-video start --interval 60

# Start with enhanced multimodal features
./sav-video start --enable-ocr --enable-vision --enable-correlation

# Start audio with specific device
./sav start --device "BlackHole 2ch"
```

## Troubleshooting

### Common Issues

#### Daemons Won't Start

**Check Dependencies**:
```bash
./savant-daemon health           # Quick dependency check
brew install ollama tesseract imagemagick  # Install missing dependencies
```

**Check Permissions**:
```bash
./verify-permissions            # Verify macOS permissions
```

#### Ollama Server Issues

**Connection Problems**:
```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Restart Ollama
pkill ollama && ollama serve

# Check for port conflicts
lsof -i :11434
```

**Model Issues**:
```bash
# List installed models
ollama list

# Install required model
ollama pull devstral

# Test model functionality
echo "Test prompt" | ollama run devstral
```

#### Permission Errors

**macOS Permissions Required**:
- **Screen Recording**: System Preferences → Security & Privacy → Privacy → Screen Recording
- **Microphone**: System Preferences → Security & Privacy → Privacy → Microphone
- **Accessibility**: System Preferences → Security & Privacy → Privacy → Accessibility (optional)

**Verification**:
```bash
./verify-permissions            # Check all permissions
./savant-daemon test            # Test system functionality
```

#### Performance Issues

**High CPU Usage**:
```bash
# Check system resources
./savant-daemon status --verbose

# Reduce video capture frequency
./sav-video stop
./sav-video start --interval 60  # Capture every 60 seconds instead of 30
```

**Memory Issues**:
```bash
# Monitor memory usage
./savant-daemon monitor

# Check for memory leaks
ps aux | grep -E "(sav|ollama)"
```

### Debug Mode

Enable verbose logging for detailed troubleshooting:

```bash
# Global debug logging
export RUST_LOG=debug

# Component-specific debugging
export RUST_LOG=savant_audio=debug,savant_video=trace

# Run with debug output
./savant-daemon start --verbose
```

### Log File Locations

```bash
# Audio daemon logs
./data/daemon-logs/savant-audio-daemon.log
./data/daemon-logs/savant-audio-daemon.err

# Video daemon logs
~/.config/savant-ai/daemon-logs/video-daemon.log

# System logs (macOS)
/var/log/system.log
```

## Integration with Main Application

### Development Workflow

```bash
# Start daemons first
./savant-daemon start

# Verify everything is running
./savant-daemon status

# Run main application
cargo tauri dev

# Monitor while developing
./savant-daemon monitor
```

### Production Deployment

```bash
# Build release version
cargo tauri build --release

# Start daemons for production
./savant-daemon start

# Run application
./target/release/savant-ai
```

## Automation and Scripting

### Scheduled Operations

```bash
# Start daemons on system boot (launchd example)
# Create: ~/Library/LaunchAgents/com.savant-ai.daemons.plist

# Daily restart for maintenance
0 6 * * * /path/to/savant-ai/savant-daemon restart

# Health monitoring
*/15 * * * * /path/to/savant-ai/savant-daemon health | logger
```

### Custom Scripts

```bash
# Custom monitoring script
#!/bin/bash
if ! ./savant-daemon health | grep -q "✓"; then
    echo "Savant AI health check failed" | mail -s "Alert" admin@example.com
    ./savant-daemon restart
fi
```

## Performance Optimization

### Resource Usage

**Typical Resource Consumption**:
- **CPU**: 10-20% during active processing
- **Memory**: ~200MB baseline + 50MB per active analysis stream
- **Storage**: 5-20MB per hour of captured data
- **Network**: Minimal (local LLM processing)

### Optimization Settings

```toml
# Performance-optimized configuration
[video_capture]
interval_seconds = 60             # Reduce capture frequency
enable_ocr = true                # Keep essential features
enable_vision_analysis = false   # Disable if not needed
enable_multimodal_correlation = false

[ocr]
preprocessing_enabled = false     # Faster processing
min_confidence = 0.6             # Higher threshold

[privacy]
blocked_applications = ["Zoom", "Teams", "Slack"]  # Skip resource-heavy apps
```

### Hardware Recommendations

- **CPU**: Multi-core processor for parallel processing
- **Memory**: 8GB+ RAM recommended
- **Storage**: SSD for faster database operations
- **GPU**: Optional, can accelerate computer vision tasks

## Security Considerations

### Data Privacy

- All processing happens locally by default
- No data sent to cloud services unless explicitly configured
- Audio and video data stored locally with encryption
- Database files have restricted permissions

### Network Security

- MCP server runs local-only by default
- No external network access required for core functionality
- Ollama server bound to localhost only
- Query validation prevents injection attacks

### Access Control

```bash
# Verify file permissions
ls -la ~/.config/savant-ai/
ls -la ./data/daemon-logs/

# Secure configuration
chmod 600 ~/.config/savant-ai/config.toml
chmod 700 ~/.config/savant-ai/daemon-logs/
```