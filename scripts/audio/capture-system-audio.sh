#!/bin/bash

# System Audio Capture Script
# Usage: ./capture-system-audio.sh [duration_in_seconds] [output_file]

DURATION=${1:-60}  # Default 60 seconds
OUTPUT=${2:-"system_audio_$(date +%Y%m%d_%H%M%S).md"}  # Default with timestamp

echo "🎵 Capturing System Audio"
echo "========================"
echo "Duration: ${DURATION} seconds"
echo "Output: ${OUTPUT}"
echo ""
echo "🔴 Recording started... Play some audio now!"
echo "   (YouTube, Spotify, any audio playing on your system)"
echo ""

# Run the capture command
cargo run --package savant-transcribe -- \
    --duration ${DURATION} \
    --system \
    --output "${OUTPUT}"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ System audio captured and transcribed!"
    echo "📄 Saved to: ${OUTPUT}"
    echo ""
    echo "📖 View the transcript:"
    echo "   cat \"${OUTPUT}\""
    echo ""
    echo "🔍 Search for specific words:"
    echo "   grep -i \"word\" \"${OUTPUT}\""
else
    echo ""
    echo "❌ Capture failed. Make sure:"
    echo "   1. BlackHole is installed (./setup-system-audio.sh)"
    echo "   2. Multi-Output Device is configured"
    echo "   3. Audio is playing during capture"
    echo ""
    echo "🔧 Run setup if needed:"
    echo "   ./setup-system-audio.sh"
    echo ""
    echo "📋 List available devices:"
    echo "   ./audio-devices.sh"
fi