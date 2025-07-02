# System Audio Setup Guide

This guide walks you through setting up system audio capture for the Savant AI transcription system.

## Overview

To capture and transcribe system audio (music, videos, calls, etc.), you need to route audio through BlackHole while still hearing it through your speakers. This is accomplished using a **Multi-Output Device**.

## Prerequisites

- ✅ BlackHole installed (`brew install blackhole-2ch`)
- ✅ Terminal permissions granted (Microphone + Screen Recording)
- ✅ Savant AI daemons running (`./start-daemons`)

## Step-by-Step Setup

### 1. Open Audio MIDI Setup

```bash
open "/System/Applications/Utilities/Audio MIDI Setup.app"
```

### 2. Create Multi-Output Device

1. **Click the "+" button** in the bottom-left corner
2. **Select "Create Multi-Output Device"**
3. **Name it** "Savant Multi-Output" (optional but recommended)

### 3. Configure the Multi-Output Device

**Check these two devices:**
- ✅ **Built-in Output** (or your preferred speakers/headphones)
- ✅ **BlackHole 2ch**

**Important:** Both boxes must be checked for this to work properly.

### 4. Set as Default Output Device

**Right-click** the "Multi-Output Device" and select **"Use This Device For Sound Output"**

### 5. Verify Setup

You should now see:
- 🔊 **Audio plays through your speakers** (you can hear everything normally)
- 📹 **Audio is captured by BlackHole** (for transcription)

## Testing the Setup

### Quick Test

1. **Play some audio** (YouTube, Spotify, music, etc.)
2. **Wait 30-60 seconds** for the audio daemon to process it
3. **Check recent captures:**
   ```bash
   # Check if new files are being created
   ls -lt /Users/yeager/Documents/savant-ai/data/audio-captures/*.json | head -3
   
   # Check the content of the latest capture
   cat $(ls -t /Users/yeager/Documents/savant-ai/data/audio-captures/*.json | head -1) | jq -r '.text'
   ```

### Expected Results

**Before setup:** You'll see `[no signal]` or `[unclear audio]`  
**After setup:** You'll see actual transcribed words from your audio

### Live Monitoring

Monitor real-time transcription activity:
```bash
./monitor-daemons
```

You should see active processing in the "Recent Audio Logs" section.

## Troubleshooting

### No Audio Through Speakers

**Problem:** You can't hear audio after setup  
**Solution:** Make sure "Built-in Output" is checked in the Multi-Output Device

### Still Getting "[no signal]"

**Problem:** Audio daemon shows no signal  
**Solution:** 
1. Verify Multi-Output Device is set as default output
2. Check that BlackHole 2ch is selected in the device
3. Restart audio daemon: `./sav restart`

### Audio Distortion or Dropouts

**Problem:** Audio quality issues  
**Solution:** 
1. Use "Drift Correction" in Audio MIDI Setup
2. Match sample rates (44.1kHz recommended)
3. Close unnecessary applications

## Advanced Configuration

### Custom Sample Rates

1. **Select BlackHole 2ch** in Audio MIDI Setup
2. **Set Format:** 44100.0 Hz, 2ch-24bit (recommended)
3. **Apply same settings** to Built-in Output

### Privacy Controls

Configure which applications to capture in `~/.config/savant-ai/config.toml`:

```toml
[privacy]
blocked_applications = ["Zoom", "Teams", "FaceTime"]  # These won't be transcribed
recording_schedule = "09:00-17:00"                    # Only capture during work hours
```

## Reverting Changes

### Switch Back to Normal Audio

1. **System Preferences → Sound → Output**
2. **Select "Built-in Output"** (or your preferred device)
3. **Delete Multi-Output Device** in Audio MIDI Setup (optional)

### Automated Revert

```bash
# Switch back to built-in output
sudo defaults write /Library/Preferences/Audio\ Devices "DefaultOutputDevice" -string "Built-in Output"
```

## Command Line Verification

### Check Current Audio Devices

```bash
# List all audio devices
system_profiler SPAudioDataType | grep -E "(Devices:|BlackHole|Built-in|Multi-Output)"

# Check what's currently capturing
./sav status
```

### Manual Audio Test

```bash
# Test direct BlackHole capture (10 seconds)
cargo run --package savant-transcribe -- --duration 10 --device "BlackHole 2ch" --format json --output /tmp/test.json

# Check results
cat /tmp/test.json | jq -r '.text'
```

## Security & Privacy

### What Gets Captured

- ✅ **Music & Videos:** Spotify, YouTube, media players
- ✅ **System Sounds:** Notifications, alerts
- ✅ **Application Audio:** Most apps' audio output
- ❌ **Blocked Apps:** As configured in privacy settings
- ❌ **Microphone:** Only captures output, not input

### Data Storage

- **Location:** `/Users/yeager/Documents/savant-ai/data/audio-captures/`
- **Format:** JSON with timestamps and segments
- **Retention:** Configurable (default: 30 days)
- **Encryption:** Files stored locally, not transmitted

## Integration

### With Savant AI Features

Once system audio is configured:
- 🎯 **Smart Queries:** Ask about content you listened to
- 📊 **Activity Analytics:** Track listening patterns
- 🔍 **Content Search:** Find specific songs, videos, or conversations
- 🤖 **AI Analysis:** Automatic summaries and insights

### CLI Tools

```bash
# Query recent audio content
cargo run --package savant-db -- query --text "music" --limit 10

# Search for specific content
cargo run --package savant-db -- query --speaker "system" --after "2025-07-01"
```

## Frequently Asked Questions

### Q: Will this affect audio quality?
**A:** No, the Multi-Output Device passes audio through without modification.

### Q: Can I use different speakers/headphones?
**A:** Yes, just select your preferred output device instead of "Built-in Output".

### Q: Does this work with Bluetooth devices?
**A:** Yes, replace "Built-in Output" with your Bluetooth device in the Multi-Output Device.

### Q: How much storage does this use?
**A:** Approximately 1-5MB per hour of audio, depending on content complexity.

### Q: Can I pause/resume capture?
**A:** Yes, use `./sav stop` and `./sav start` to control capture.

---

## Support

If you encounter issues:

1. **Check permissions:** `./verify-permissions`
2. **View logs:** `./monitor-daemons`
3. **Test individual components:** `./test-systems`
4. **Reset configuration:** Delete Multi-Output Device and recreate

For additional help, refer to the main documentation or check the troubleshooting section.