#!/bin/bash
# Create a smart Multi-Output Device that adapts to connected speakers/headphones

echo "🎧 Adaptive Audio Setup Creator"
echo "==============================="

echo "This script helps create a Multi-Output Device that automatically"
echo "adapts to your connected speakers or headphones."
echo ""

# Detect current audio output devices
echo "📱 Detecting available audio devices..."
echo ""

# List all output devices
system_profiler SPAudioDataType | grep -E "^\s*[A-Za-z].*:" | grep -v "Input" | while read -r device; do
    device_name=$(echo "$device" | sed 's/:$//')
    sample_rate=$(system_profiler SPAudioDataType | grep -A 10 "$device_name:" | grep "Current SampleRate" | awk '{print $2}')
    echo "🔊 $device_name: ${sample_rate:-Unknown} Hz"
done

echo ""
echo "🛠️  Recommended setup for different scenarios:"
echo ""

echo "Scenario 1: Always match MacBook speakers (48kHz)"
echo "------------------------------------------------"
echo "✅ Best for: Consistent setup, always high quality"
echo "⚙️  BlackHole: 48000 Hz"
echo "⚙️  Multi-Output: 48000 Hz, Built-in Output as Master"
echo ""

echo "Scenario 2: Match external headphones/speakers"
echo "----------------------------------------------"
echo "✅ Best for: Using high-end audio equipment"
echo "⚙️  Check your headphones' native sample rate"
echo "⚙️  Common rates: 44100 Hz (CD quality), 48000 Hz (professional), 96000 Hz (hi-res)"
echo ""

echo "Scenario 3: Adaptive setup (manual switching)"
echo "---------------------------------------------"
echo "✅ Best for: Multiple audio devices"
echo "⚙️  Use ./toggle-audio-mode.sh to switch between devices"
echo "⚙️  Automatically updates Multi-Output Device settings"
echo ""

read -p "Which scenario do you want to set up? (1/2/3): " scenario

case $scenario in
    1)
        echo ""
        echo "🔧 Setting up Scenario 1: Always match MacBook speakers"
        ./auto-match-sample-rates.sh
        ;;
    2)
        echo ""
        echo "🔧 Setting up Scenario 2: Match external audio device"
        echo ""
        read -p "What's the sample rate of your external device? (44100/48000/96000): " ext_rate
        
        if [[ "$ext_rate" =~ ^[0-9]+$ ]]; then
            echo "Setting up for $ext_rate Hz..."
            osascript -e "
            tell application \"Audio MIDI Setup\"
                activate
            end tell
            
            display dialog \"Configure for external device at $ext_rate Hz:
            
1. Select 'BlackHole 2ch' → Set Format to $ext_rate Hz
2. Select 'Multi-Output Device' → Set Format to $ext_rate Hz  
3. In Multi-Output Device, check your external device + BlackHole
4. Set external device as Master Device (right-click)\" buttons {\"OK\"}
            "
        else
            echo "Invalid sample rate. Please run the script again."
        fi
        ;;
    3)
        echo ""
        echo "🔧 Setting up Scenario 3: Adaptive setup"
        echo ""
        echo "✅ Your toggle script is ready: ./toggle-audio-mode.sh"
        echo "✅ Run it anytime to switch between high-quality and capture modes"
        echo ""
        echo "💡 Tip: Create an alias for quick access:"
        echo "   echo 'alias audio-toggle=\"~/Documents/savant-ai/toggle-audio-mode.sh\"' >> ~/.zshrc"
        ;;
    *)
        echo "Invalid choice. Please run the script again."
        ;;
esac

echo ""
echo "🔮 Pro Tips for Always Matching Speaker Quality:"
echo "================================================"
echo ""
echo "1. 🎯 Use SwitchAudioSource for automation:"
echo "   brew install switchaudio-osx"
echo "   SwitchAudioSource -s 'Multi-Output Device'"
echo ""
echo "2. 📱 Create Shortcuts app automation (macOS 12+):"
echo "   - Trigger: Connect/disconnect Bluetooth headphones"
echo "   - Action: Run shell script to update sample rates"
echo ""
echo "3. 🔄 Add to login items:"
echo "   System Preferences → Users & Groups → Login Items"
echo "   Add: auto-match-sample-rates.sh"
echo ""
echo "4. 📊 Monitor changes:"
echo "   ./monitor-daemons shows current audio setup in real-time"