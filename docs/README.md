# Savant AI Documentation

This directory contains all documentation for the Savant AI project.

## 📁 Documentation Structure

### User Guides (`user-guides/`)
- **[System Audio Setup](user-guides/SYSTEM_AUDIO_SETUP.md)** - Complete guide to setting up audio capture and transcription
- **[Getting Started](user-guides/GETTING_STARTED.md)** - Quick start guide for new users
- **[Audio Management](user-guides/AUDIO_MANAGEMENT.md)** - Managing audio captures and transcriptions

### Development Documentation (`development/`)
- **[UNIX Philosophy Demo](development/UNIX_PHILOSOPHY_DEMO.md)** - Demonstration of UNIX philosophy implementation
- **[UNIX Refactor Plan](development/UNIX_REFACTOR_PLAN.md)** - Planning document for UNIX-style refactoring
- **[Architecture Overview](development/ARCHITECTURE.md)** - System architecture and design patterns
- **[Development Setup](development/DEVELOPMENT.md)** - Setting up development environment

### API Documentation (`api/`)
- **[Audio API](api/AUDIO_API.md)** - Audio capture and transcription APIs
- **[LLM API](api/LLM_API.md)** - Language model integration APIs

## 🚀 Quick Navigation

- **New to Savant AI?** Start with [Getting Started](user-guides/GETTING_STARTED.md)
- **Setting up audio?** See [System Audio Setup](user-guides/SYSTEM_AUDIO_SETUP.md)
- **Developer?** Check [Development Setup](development/DEVELOPMENT.md)
- **Need help?** See [Troubleshooting](user-guides/TROUBLESHOOTING.md)

## 📋 Project Structure Overview

```
savant-ai/
├── docs/                     # All documentation
│   ├── user-guides/         # End-user documentation
│   ├── development/         # Developer documentation
│   └── api/                 # API reference
├── scripts/                 # Organized utility scripts
│   ├── audio/              # Audio capture scripts
│   ├── setup/              # Installation/setup scripts
│   └── utils/              # General utilities
├── data/                   # Application data (not in git)
│   ├── audio-captures/     # Transcribed audio files
│   ├── daemon-logs/        # Daemon log files
│   └── test-captures/      # Test transcription files
├── crates/                 # Rust workspace crates
└── src-tauri/             # Tauri desktop application
```

## 🔧 Common Tasks

### Audio System Management
```bash
# Check audio devices
./scripts/audio/audio-devices.sh

# Start audio daemon
launchctl load ~/Library/LaunchAgents/com.savant.audio.daemon.plist

# Stop audio daemon
launchctl unload ~/Library/LaunchAgents/com.savant.audio.daemon.plist

# View recent captures
ls -la data/audio-captures/

# Check daemon logs
tail -f data/daemon-logs/savant-audio-daemon.log
```

### Development Commands
```bash
# Run full application
cargo tauri dev

# Test audio capture (10 seconds)
cargo run --package savant-transcribe -- --device "BlackHole 2ch" --duration 10 --output data/test-captures/test.md

# Run tests
cargo test --workspace
```

## 📊 Data Management

All application data is stored in the `data/` directory:

- **Audio Captures**: `data/audio-captures/` - Automatic 5-minute transcription segments
- **Daemon Logs**: `data/daemon-logs/` - System logs and debug information  
- **Test Captures**: `data/test-captures/` - Manual test transcriptions

## 🆘 Support

- Check [Troubleshooting Guide](user-guides/TROUBLESHOOTING.md)
- Review [System Audio Setup](user-guides/SYSTEM_AUDIO_SETUP.md) for audio issues
- See [Development Documentation](development/) for technical details