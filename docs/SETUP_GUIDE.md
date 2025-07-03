# Setup Guide

Quick setup for Savant AI. For complete documentation, see [CLAUDE.md](../CLAUDE.md).

## Quick Start

```bash
./setup                      # Automated setup + guided permissions
./verify-permissions         # Check system configuration
./start-daemons             # Start audio + video monitoring
```

## Requirements

- **macOS** (full support) / Linux (limited) / Windows (basic)
- **8GB+ RAM**, **1GB storage**, **Multi-core CPU**
- **Permissions**: Microphone, Screen Recording (macOS)

## Dependencies (Auto-installed)

- Rust/Cargo, Ollama, Tesseract OCR, ImageMagick

## Manual Setup

Only if automated setup fails:

```bash
# Install dependencies
brew install rust ollama tesseract imagemagick

# Install Ollama models
ollama pull llama3.2:3b
ollama pull llama3.2

# Build project
cargo build --release --workspace
```

## macOS Permissions

Grant these in **System Preferences → Security & Privacy → Privacy**:

1. **Screen Recording** (required for video capture)
2. **Microphone** (required for audio capture)
3. **Accessibility** (optional, for advanced features)

Verify: `./verify-permissions`

## Audio Setup

**Basic**: Just microphone capture
**Advanced**: System audio via BlackHole (see [audio-setup.md](guides/audio-setup.md))

## Troubleshooting

- Permission issues: `./scripts/setup/permission-helper.sh`
- Test systems: `./test-systems`
- Check logs: `./monitor-daemons`

For detailed setup instructions, see [CLAUDE.md](../CLAUDE.md).