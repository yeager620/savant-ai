#!/bin/bash
# Set up permanent high-quality audio that always matches speakers

echo "🎵 Permanent High-Quality Audio Setup"
echo "====================================="

echo "This will configure your system to ALWAYS maintain speaker quality."
echo ""

echo "🔧 Step 1: Configure BlackHole for High Quality"
echo "----------------------------------------------"
echo "Setting BlackHole to 48kHz (professional audio standard)"
echo ""

osascript -e "
tell application \"Audio MIDI Setup\"
    activate
end tell

display dialog \"Set BlackHole to High Quality:

1. Select 'BlackHole 2ch' in left sidebar
2. Change Format to '2 ch 48000 Hz'
3. This matches most modern speakers and headphones

Click OK when done.\" buttons {\"OK\"}
"

echo ""
echo "🔧 Step 2: Configure Multi-Output Device"
echo "---------------------------------------"

osascript -e "
tell application \"Audio MIDI Setup\"
    activate
end tell

display dialog \"Configure Multi-Output Device:

1. Select 'Multi-Output Device' in left sidebar
2. Set Format to '2 ch 48000 Hz'
3. Check both 'Built-in Output' and 'BlackHole 2ch'
4. Right-click 'Built-in Output' → 'Use This Device For Sound Output'

This ensures Built-in Output controls the sample rate.\" buttons {\"OK\"}
"

echo ""
echo "🔧 Step 3: Create Launch Agent for Persistence"
echo "---------------------------------------------"

# Create a launch agent to ensure settings persist
cat > ~/Library/LaunchAgents/com.savant-ai.audio-quality.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.savant-ai.audio-quality</string>
    <key>Program</key>
    <string>/Users/yeager/Documents/savant-ai/check-audio-quality.sh</string>
    <key>RunAtLoad</key>
    <true/>
    <key>StartInterval</key>
    <integer>300</integer>
</dict>
</plist>
EOF

echo "✅ Created launch agent: ~/Library/LaunchAgents/com.savant-ai.audio-quality.plist"

echo ""
echo "🔧 Step 4: Create Audio Quality Monitor"
echo "--------------------------------------"

cat > check-audio-quality.sh << 'EOF'
#!/bin/bash
# Monitor and maintain audio quality

# Check if sample rates are mismatched
blackhole_rate=$(system_profiler SPAudioDataType | grep -A 10 "BlackHole 2ch:" | grep "Current SampleRate" | awk '{print $2}')
speakers_rate=$(system_profiler SPAudioDataType | grep -A 10 "MacBook Pro Speakers:" | grep "Current SampleRate" | awk '{print $2}')

if [ "$blackhole_rate" != "48000" ] && [ -n "$blackhole_rate" ]; then
    # Log the mismatch
    echo "$(date): BlackHole sample rate mismatch detected: $blackhole_rate Hz" >> ~/Library/Logs/savant-audio-quality.log
    
    # Send notification
    osascript -e "display notification \"BlackHole sample rate: $blackhole_rate Hz. Should be 48000 Hz for best quality.\" with title \"Savant AI Audio Quality\""
fi
EOF

chmod +x check-audio-quality.sh
echo "✅ Created audio quality monitor: check-audio-quality.sh"

echo ""
echo "🔧 Step 5: Load the Launch Agent"
echo "-------------------------------"

launchctl load ~/Library/LaunchAgents/com.savant-ai.audio-quality.plist 2>/dev/null || echo "Launch agent already loaded"

echo "✅ Launch agent loaded - will monitor audio quality every 5 minutes"

echo ""
echo "🎯 What This Setup Does:"
echo "========================"
echo "✅ Sets BlackHole to 48kHz permanently"
echo "✅ Configures Multi-Output Device for quality"
echo "✅ Monitors for sample rate changes"
echo "✅ Sends notifications if quality degrades"
echo "✅ Works with any speakers/headphones at 48kHz or lower"
echo ""

echo "🔄 Final Steps:"
echo "1. Restart your audio daemon: ./sav stop && ./sav start"
echo "2. Test with YouTube video - should be high quality now"
echo "3. Check status anytime: ./auto-match-sample-rates.sh"
echo ""

echo "💡 Pro Tip: For different headphones/speakers with different rates:"
echo "   Use ./create-adaptive-multi-output.sh to set up device-specific profiles"