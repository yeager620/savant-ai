#!/bin/bash
# Smart audio mode toggle for quality vs capture

echo "🎵 Smart Audio Mode Toggle"
echo "=========================="

# Check current audio device
current_device=$(system_profiler SPAudioDataType | grep "Default Output Device: Yes" -B 5 | grep ":" | head -1 | cut -d: -f1 | xargs)

if [[ "$current_device" == "Multi-Output Device" ]]; then
    echo "📤 Currently in CAPTURE mode (Multi-Output Device)"
    echo "   → System audio is being captured but quality may be reduced"
    echo ""
    echo "Switch to HIGH QUALITY mode?"
    echo "  ✅ Perfect audio quality for music/videos"
    echo "  ❌ No system audio capture (microphone still works)"
    echo ""
    read -p "Switch to high quality mode? (y/n): " choice
    
    if [[ "$choice" == "y" || "$choice" == "Y" ]]; then
        echo ""
        echo "🔧 Switching to high quality mode..."
        echo "1. Open System Preferences → Sound → Output"
        echo "2. Select 'MacBook Pro Speakers'"
        echo "3. Your audio quality will return to normal"
        echo ""
        echo "💡 Audio daemon will still capture microphone"
        echo "💡 Run this script again to re-enable system audio capture"
    fi
    
else
    echo "🎧 Currently in HIGH QUALITY mode (MacBook Pro Speakers)"
    echo "   → Perfect audio quality, microphone capture active"
    echo ""
    echo "Switch to CAPTURE mode?"
    echo "  ✅ Captures system audio (YouTube, music, etc.)"
    echo "  ⚠️  May reduce audio quality due to sample rate conversion"
    echo ""
    read -p "Switch to capture mode? (y/n): " choice
    
    if [[ "$choice" == "y" || "$choice" == "Y" ]]; then
        echo ""
        echo "🔧 Switching to capture mode..."
        echo "1. Open System Preferences → Sound → Output"
        echo "2. Select 'Multi-Output Device'"
        echo "3. System audio will now be captured"
        echo ""
        echo "💡 If audio quality degrades, run ./fix-audio-quality.sh"
        echo "💡 Run this script again to switch back to high quality"
    fi
fi

echo ""
echo "🛠️  Advanced fixes:"
echo "  ./fix-audio-quality.sh    # Fix sample rate issues"  
echo "  ./verify-permissions      # Check audio permissions"
echo "  ./sav status              # Check daemon status"