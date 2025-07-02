#!/bin/bash
# Automatically match BlackHole and Multi-Output Device to speaker sample rate

echo "🎵 Auto Sample Rate Matcher"
echo "==========================="

# Function to get sample rate of a device
get_sample_rate() {
    local device_name="$1"
    system_profiler SPAudioDataType | grep -A 10 "$device_name:" | grep "Current SampleRate" | awk '{print $2}'
}

# Function to set sample rate for a device using osascript
set_sample_rate() {
    local device_name="$1"
    local sample_rate="$2"
    
    echo "🔧 Setting $device_name to $sample_rate Hz..."
    
    # This opens Audio MIDI Setup and provides instructions
    # Since macOS doesn't allow direct sample rate changes via command line
    osascript -e "
    tell application \"Audio MIDI Setup\"
        activate
    end tell
    
    display dialog \"Please manually set $device_name to $sample_rate Hz:
    
1. Select '$device_name' in the left sidebar
2. In the Format section, change sample rate to $sample_rate Hz
3. Click OK when done\" buttons {\"OK\"} default button \"OK\"
    "
}

echo "📊 Current audio device sample rates:"
echo "======================================"

# Get current sample rates
builtin_rate=$(get_sample_rate "MacBook Pro Speakers")
blackhole_rate=$(get_sample_rate "BlackHole 2ch")
multioutput_rate=$(get_sample_rate "Multi-Output Device")

echo "MacBook Pro Speakers: ${builtin_rate:-Unknown} Hz"
echo "BlackHole 2ch: ${blackhole_rate:-Unknown} Hz" 
echo "Multi-Output Device: ${multioutput_rate:-Unknown} Hz"
echo ""

# Check if we have a built-in speaker rate to match
if [ -n "$builtin_rate" ] && [ "$builtin_rate" != "Unknown" ]; then
    target_rate="$builtin_rate"
    echo "🎯 Target sample rate: $target_rate Hz (matching MacBook Pro Speakers)"
    echo ""
    
    # Check if BlackHole needs updating
    if [ "$blackhole_rate" != "$target_rate" ]; then
        echo "⚠️  BlackHole 2ch is at $blackhole_rate Hz, should be $target_rate Hz"
        read -p "Fix BlackHole sample rate? (y/n): " fix_blackhole
        if [[ "$fix_blackhole" == "y" || "$fix_blackhole" == "Y" ]]; then
            set_sample_rate "BlackHole 2ch" "$target_rate"
        fi
    else
        echo "✅ BlackHole 2ch already at correct rate ($blackhole_rate Hz)"
    fi
    
    # Check if Multi-Output Device needs updating
    if [ "$multioutput_rate" != "$target_rate" ]; then
        echo "⚠️  Multi-Output Device is at $multioutput_rate Hz, should be $target_rate Hz"
        read -p "Fix Multi-Output Device sample rate? (y/n): " fix_multioutput
        if [[ "$fix_multioutput" == "y" || "$fix_multioutput" == "Y" ]]; then
            set_sample_rate "Multi-Output Device" "$target_rate"
        fi
    else
        echo "✅ Multi-Output Device already at correct rate ($multioutput_rate Hz)"
    fi
    
else
    echo "❌ Could not detect MacBook Pro Speakers sample rate"
    echo "   Defaulting to 48000 Hz (standard high quality)"
    target_rate="48000"
    
    set_sample_rate "BlackHole 2ch" "$target_rate"
    set_sample_rate "Multi-Output Device" "$target_rate"
fi

echo ""
echo "🔄 After making changes, restart your audio daemon:"
echo "   ./sav stop && ./sav start"
echo ""
echo "📝 To make this permanent, save these settings in Audio MIDI Setup"