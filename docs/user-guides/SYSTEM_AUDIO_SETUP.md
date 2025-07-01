# System Audio Capture Setup

This document explains how to set up automated system audio capture that monitors all audio (input/output) at the system level while preserving normal audio usage.

## What This Does

- **Captures ALL system audio**: Everything that plays through speakers/headphones
- **Captures microphone input**: All microphone usage across apps
- **Preserves normal audio**: You still hear audio normally and can use your mic
- **Background operation**: Runs invisibly in the background with single-instance protection
- **Automatic transcription**: Converts all captured audio to searchable text
- **Zero configuration**: Automated setup handles everything

## Quick Setup

### Automated Setup
```bash
./scripts/setup/auto-setup-system-audio.sh
```

This script automatically:
- Installs BlackHole virtual audio device
- Configures multi-output audio routing
- Sets up background capture daemon
- Creates automatic startup service

### Manual Control
```bash
# Check daemon status and recent activity
./sav status

# Start background capture daemon
./sav start

# Stop capture daemon
./sav stop

# Restart daemon
./sav restart

# View live daemon logs
./sav logs

# List all captured transcripts
./sav list

# Search transcripts for specific text
./sav search "meeting"

# Test single-instance protection
./sav test

# Run automated setup
./sav setup
```

## How It Works

### Audio Routing Architecture
```
System Audio → Multi-Output Device → [BlackHole 2ch] → Savant Capture
                                  → [Your Speakers] → You hear normally
                                  
Microphone → Apps (normal usage)
          → BlackHole → Savant Capture
```

### Components Installed

1. **BlackHole 2ch**: Virtual audio loopback device
2. **Multi-Output Device**: Routes audio to both speakers and BlackHole
3. **Capture Daemon**: Background service with PID-based single-instance protection
4. **Control Script**: Management interface for daemon operations

### Single-Instance Protection

The daemon uses PID-based locking to ensure only one instance runs at a time:

1. **PID File Check**: Verifies no existing daemon is running
2. **Process Validation**: Confirms PID corresponds to active process
3. **Automatic Cleanup**: Removes stale PID files from crashed instances
4. **Signal Handling**: Proper cleanup on termination signals

### What Happens with Multiple Instances

When attempting to start a second daemon instance:
- **Direct Script**: Exits with error message and existing PID
- **Control Script**: Shows warning with current daemon PID
- **Testing Mode**: Verifies protection is working correctly

## File Locations

- **Captures**: `~/Documents/savant-ai/data/audio-captures/*.md`
- **Daemon Logs**: `~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log`
- **PID File**: `~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.pid`
- **Daemon Script**: `~/Documents/savant-ai/scripts/audio/savant-audio-daemon.sh`
- **Control Script**: `~/Documents/savant-ai/scripts/audio/savant-audio-control.sh`

## Daemon Management

All commands should be run from the `savant-ai` project directory.

### Primary Interface (Recommended)
```bash
# Check daemon status and recent activity
./sav status

# Start the daemon
./sav start

# Stop the daemon
./sav stop

# Restart the daemon
./sav restart

# Test single-instance protection
./sav test
```

### Direct Daemon Control (Advanced)
```bash
# Start daemon directly (handles single-instance protection)
./scripts/audio/savant-audio-daemon.sh

# Check daemon process
ps aux | grep savant-audio-daemon

# Stop daemon by PID
kill $(cat ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.pid)
```

### Legacy LaunchAgent Control
```bash
# Check if launchd service exists
launchctl list | grep savant

# Control via launchctl (if service is installed)
launchctl load ~/Library/LaunchAgents/com.savant.audio.daemon.plist
launchctl unload ~/Library/LaunchAgents/com.savant.audio.daemon.plist
```

### Monitoring and Logs
```bash
# View live logs
./sav logs

# View recent log entries
tail -20 ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log

# List all captures
./sav list

# Search transcripts
./sav search "meeting"

# Check capture directory
ls -la ~/Documents/savant-ai/data/audio-captures/

# View latest capture
tail ~/Documents/savant-ai/data/audio-captures/$(ls -t ~/Documents/savant-ai/data/audio-captures/ | head -1)
```

### Debugging and Testing
```bash
# Test single-instance protection
./sav test

# Check system audio devices
./scripts/audio/audio-devices.sh

# Test transcription directly
cargo run --package savant-transcribe -- --duration 10 --system --output test.md

# Manual daemon execution (for debugging)
./scripts/audio/savant-audio-daemon.sh
```

## Usage Examples

### Content Search
```bash
# Search for meetings
./sav search "meeting"

# Search for music
./sav search "song\|music\|artist"

# Search for calls
./sav search "zoom\|teams\|call"

# List all captures
./sav list
```

## What Gets Captured

### Audio Sources
- **Music**: Spotify, Apple Music, YouTube, SoundCloud
- **Videos**: Netflix, YouTube, Vimeo, streaming services
- **Calls**: Zoom, Teams, Discord, Slack, FaceTime
- **Games**: All game audio and voice chat
- **System**: Notifications, alerts, sound effects
- **Microphone**: Your voice in calls, dictation, voice commands

### Transcript Format
Each capture creates a timestamped markdown file:
```markdown
# Audio Transcription

**Date:** 2024-01-15 14:30:25 UTC
**Model:** whisper-base.en
**Processing Time:** 1250ms

---

## Full Transcript

[Meeting discussion about project timeline...]

## Timestamped Segments

### Segment 1 (00:00 - 02:30)
"Welcome everyone to today's standup meeting..."

### Segment 2 (02:30 - 05:15)
"Let's review the sprint goals..."
```

## Configuration

### Capture Settings
The daemon captures in 5-minute segments by default. Edit the daemon script to customize:

```bash
# In ~/Documents/savant-ai/scripts/audio/savant-audio-daemon.sh
SEGMENT_DURATION=300  # 5 minutes (change as needed)
CAPTURE_DIR="$SAVANT_DIR/data/audio-captures"  # Output directory
```

### Audio Quality
- **Sample Rate**: 16kHz (optimized for speech recognition)
- **Channels**: Mono (reduces file size, maintains quality)
- **Format**: F32 floating point
- **Compression**: Automatic via Whisper preprocessing

## Privacy and Security

### Data Handling
- **Local Only**: All processing happens on your machine
- **No Cloud**: Audio never leaves your computer
- **Encrypted Storage**: Files stored in your home directory
- **User Control**: Start/stop anytime, delete transcripts anytime

### Permissions Required
- **Microphone Access**: To capture mic input
- **Audio Device Access**: To configure routing
- **Administrator**: For system-level audio configuration

### What's NOT Captured
- Audio from apps that bypass system audio
- DRM-protected content (Netflix, some streaming services)
- Audio when daemon is stopped
- Audio from other user accounts

## Troubleshooting

### Common Issues

#### "BlackHole device not found"
```bash
# Reinstall BlackHole
brew uninstall blackhole-2ch
brew install blackhole-2ch
./scripts/setup/auto-setup-system-audio.sh
```

#### "No audio capture"
```bash
# Check device list
./scripts/audio/audio-devices.sh

# Verify BlackHole is available
./sav status

# Check daemon logs
./sav logs
```

#### "Daemon won't start"
```bash
# Check daemon status
./sav status

# Restart daemon
./sav restart

# Test single-instance protection
./sav test

# Check for conflicting processes
ps aux | grep savant-audio-daemon
```

#### "Multiple instance errors"
```bash
# Check current daemon status
./sav status

# Stop existing daemon first
./sav stop

# Clean up stale PID file if needed
rm -f ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.pid

# Start fresh daemon
./sav start
```

#### "No transcripts generated"
Make sure Whisper model is available:
```bash
# Download Whisper model
mkdir -p models
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin -O models/ggml-base.en.bin
```

### Reset Everything
```bash
# Stop daemon
./sav stop

# Remove launchd service (if installed)
rm -f ~/Library/LaunchAgents/com.savant.audio.daemon.plist

# Clear logs
rm -f ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log
rm -f ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.pid

# Clear captures (optional)
rm -rf ~/Documents/savant-ai/data/audio-captures/

# Uninstall BlackHole
brew uninstall blackhole-2ch

# Re-run setup
./scripts/setup/auto-setup-system-audio.sh
```

## Important Notes

### Audio Routing
- **No Audio Loss**: You'll still hear everything normally
- **Microphone Works**: All apps can use your mic as usual
- **System Sounds**: All notifications and alerts still play
- **Headphone Support**: Works with any audio output device

### Performance Impact
- **CPU**: Minimal (< 5% during active transcription)
- **Memory**: ~100MB for daemon + Whisper model
- **Storage**: ~10MB per hour of audio (compressed transcripts)
- **Battery**: Negligible impact on laptop battery

### Compatibility
- **macOS**: 10.13+ (High Sierra and newer)
- **Audio Devices**: All Core Audio compatible devices
- **Apps**: Works with all macOS applications
- **External**: Bluetooth headphones, USB audio interfaces

## Advanced Usage

### Batch Processing
Process multiple audio files:
```bash
for file in ~/Downloads/*.wav; do
    cargo run --package savant-transcribe -- --file "$file" --output "${file%.wav}.md"
done
```

### Custom Search Scripts
```bash
# Find all mentions of specific people
grep -r -i "john\|sarah\|mike" ~/Documents/savant-ai/data/audio-captures/

# Find calls longer than 30 minutes
find ~/Documents/savant-ai/data/audio-captures/ -name "*.md" -exec grep -l "30:" {} \;

# Extract action items
grep -r -i "todo\|action\|follow.up" ~/Documents/savant-ai/data/audio-captures/
```

### Integration with Other Tools
```bash
# Convert to PDF
pandoc ~/Documents/savant-ai/data/audio-captures/meeting.md -o meeting.pdf

# Upload to cloud (after review)
cp ~/Documents/savant-ai/data/audio-captures/*.md ~/Dropbox/transcripts/

# Search with Spotlight
mdfind "kind:text AND (meeting OR call)"
```

## Support

If you encounter issues:

1. **Check Status**: `./sav status`
2. **View Logs**: `./sav logs`
3. **Test Protection**: `./sav test`
4. **List Devices**: `./scripts/audio/audio-devices.sh`
5. **Restart Daemon**: `./sav restart`

For persistent issues, check the daemon log file:
```bash
tail -f ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log
```