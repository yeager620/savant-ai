# Quick Start

Get Savant AI running in minutes.

## Prerequisites

- macOS (primary support) or Linux
- 8GB+ RAM recommended
- Terminal access

## Installation

```bash
# Clone and setup
git clone <repository-url>
cd savant-ai
./setup

# Verify installation
./verify-permissions
./test-systems
```

## First Run

```bash
# Start all systems
./start-daemons

# Monitor status
./monitor-daemons

# Test functionality
./sav test                         # Audio transcription
./sav-video test                   # Screen capture

# Stop when done
./stop-daemons
```

## Next Steps

- **[Complete Setup Guide](../SETUP_GUIDE.md)** - Full configuration options
- **[Permissions Guide](../guides/permissions.md)** - macOS permissions setup
- **[CLI Tools Reference](../reference/cli-tools.md)** - Command line usage
- **[Audio Setup](../guides/audio-setup.md)** - System audio capture

## Common Issues

**Permissions Error**: Run `./verify-permissions` and follow instructions
**Ollama Not Found**: Install with `brew install ollama && ollama serve`
**No Audio**: Check microphone permissions in System Preferences

For detailed troubleshooting, see the [Setup Guide](../SETUP_GUIDE.md#troubleshooting).