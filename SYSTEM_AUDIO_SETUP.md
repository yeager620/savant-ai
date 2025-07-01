# System Audio Capture Setup

This document explains how to set up automated system audio capture that "eavesdrops" on all audio (input/output) at the system level while allowing normal audio usage.

## 🎯 What This Does

- **Captures ALL system audio**: Everything that plays through speakers/headphones
- **Captures microphone input**: All microphone usage across apps
- **Preserves normal audio**: You still hear audio normally and can use your mic
- **Background operation**: Runs invisibly in the background
- **Automatic transcription**: Converts all captured audio to searchable text
- **Zero configuration**: Automated setup handles everything

## 🚀 Quick Setup (Automated)

### Option 1: One-Command Setup
```bash
./auto-setup-system-audio.sh
```

This script automatically:
- Installs BlackHole virtual audio device
- Configures multi-output audio routing
- Sets up background capture daemon
- Creates automatic startup service

### Option 2: Manual Control
```bash
# Check status
./savant-audio-control.sh status

# Run setup
./savant-audio-control.sh setup

# Start background capture
./savant-audio-control.sh start

# Stop capture
./savant-audio-control.sh stop

# View live activity
./savant-audio-control.sh logs

# Search transcripts
./savant-audio-control.sh search "meeting"
```

## 🔧 How It Works

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
3. **Capture Daemon**: Background service that records and transcribes
4. **LaunchAgent**: Automatic startup service

## 📁 File Locations

- **Captures**: `~/savant-audio-captures/*.md`
- **Logs**: `~/savant-audio-daemon.log`
- **Daemon Script**: `~/savant-audio-daemon.sh`
- **Service**: `/Library/LaunchAgents/com.savant.audio.daemon.plist`

## 🎮 Usage Examples

### Start Background Capture
```bash
./savant-audio-control.sh start
```

### Monitor Activity
```bash
./savant-audio-control.sh logs
```

### Find Specific Content
```bash
# Search for meetings
./savant-audio-control.sh search "meeting"

# Search for music
./savant-audio-control.sh search "song\|music\|artist"

# Search case-insensitive
./savant-audio-control.sh search "zoom\|teams\|call"
```

### List All Captures
```bash
./savant-audio-control.sh list
```

## 🔍 What Gets Captured

### Audio Sources
- 🎵 **Music**: Spotify, Apple Music, YouTube, SoundCloud
- 🎬 **Videos**: Netflix, YouTube, Vimeo, streaming services
- 💬 **Calls**: Zoom, Teams, Discord, Slack, FaceTime
- 🎮 **Games**: All game audio and voice chat
- 🔔 **System**: Notifications, alerts, sound effects
- 🎙️ **Microphone**: Your voice in calls, dictation, voice commands

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

## ⚙️ Configuration

### Capture Settings
The daemon captures in 5-minute segments by default. Edit `~/savant-audio-daemon.sh` to customize:

```bash
SEGMENT_DURATION=300  # 5 minutes (change as needed)
CAPTURE_DIR="$HOME/savant-audio-captures"  # Output directory
```

### Audio Quality
- **Sample Rate**: 16kHz (optimized for speech recognition)
- **Channels**: Mono (reduces file size, maintains quality)
- **Format**: F32 floating point
- **Compression**: Automatic via Whisper preprocessing

## 🔒 Privacy & Security

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

## 🛠️ Troubleshooting

### Common Issues

#### "BlackHole device not found"
```bash
# Reinstall BlackHole
brew uninstall blackhole-2ch
brew install blackhole-2ch
./auto-setup-system-audio.sh
```

#### "No audio capture"
```bash
# Check device list
./audio-devices.sh

# Verify BlackHole is available
./savant-audio-control.sh status

# Check daemon logs
./savant-audio-control.sh logs
```

#### "Daemon won't start"
```bash
# Check permissions
sudo launchctl list | grep savant

# Restart daemon
./savant-audio-control.sh restart

# Manual daemon start
sudo launchctl load /Library/LaunchAgents/com.savant.audio.daemon.plist
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
./savant-audio-control.sh stop

# Remove service
sudo rm /Library/LaunchAgents/com.savant.audio.daemon.plist

# Remove daemon script
rm ~/savant-audio-daemon.sh

# Uninstall BlackHole
brew uninstall blackhole-2ch

# Re-run setup
./auto-setup-system-audio.sh
```

## 🚨 Important Notes

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

## 📊 Advanced Usage

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
grep -r -i "john\|sarah\|mike" ~/savant-audio-captures/

# Find calls longer than 30 minutes
find ~/savant-audio-captures/ -name "*.md" -exec grep -l "30:" {} \;

# Extract action items
grep -r -i "todo\|action\|follow.up" ~/savant-audio-captures/
```

### Integration with Other Tools
```bash
# Convert to PDF
pandoc ~/savant-audio-captures/meeting.md -o meeting.pdf

# Upload to cloud (after review)
cp ~/savant-audio-captures/*.md ~/Dropbox/transcripts/

# Search with Spotlight
mdfind "kind:text AND (meeting OR call)"
```

## 🆘 Support

If you encounter issues:

1. **Check Status**: `./savant-audio-control.sh status`
2. **View Logs**: `./savant-audio-control.sh logs`
3. **List Devices**: `./audio-devices.sh`
4. **Restart Daemon**: `./savant-audio-control.sh restart`

For persistent issues, check the daemon log file:
```bash
tail -f ~/savant-audio-daemon.log
```

## 🔮 Future Enhancements

Planned features:
- Real-time transcription display
- Speaker identification
- Automatic meeting summaries
- Calendar integration
- Voice command triggers
- Multi-language support
- Custom vocabulary training