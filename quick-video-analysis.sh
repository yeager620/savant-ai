#!/bin/bash
# Quick video analysis commands

echo "🚀 Quick Video Analysis"
echo "======================="

CAPTURE_DIR="$HOME/Library/Application Support/savant-ai/video-captures"

# Get most recent screenshot
latest_screenshot=$(find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -1 | cut -d' ' -f2-)

if [ -z "$latest_screenshot" ]; then
    echo "❌ No screenshots found in $CAPTURE_DIR"
    echo "Make sure the video daemon is running: ./sav-video start"
    exit 1
fi

echo "📸 Most recent screenshot: $(basename "$latest_screenshot")"
echo "📅 Captured: $(stat -f "%Sm" -t "%Y-%m-%d %H:%M:%S" "$latest_screenshot")"
echo "💾 Size: $(ls -lh "$latest_screenshot" | awk '{print $5}')"
echo ""

echo "🔍 Quick OCR Analysis:"
echo "====================="
cargo run --package savant-ocr -- extract --input "$latest_screenshot" --format text --fast

echo ""
echo "👁️  Quick Computer Vision:"
echo "=========================="
cargo run --package savant-vision -- analyze --input "$latest_screenshot" --detect-apps --format summary

echo ""
echo "📊 Storage Summary:"
echo "=================="
total_images=$(find "$CAPTURE_DIR" -name "*.png" -type f | wc -l | tr -d ' ')
total_size=$(du -sh "$CAPTURE_DIR" | cut -f1)
echo "Total screenshots: $total_images"
echo "Total storage: $total_size"

echo ""
echo "🎯 Analysis Options:"
echo "==================="
echo "./analyze-video-data.sh      # Full interactive analysis"
echo "open '$CAPTURE_DIR'           # Browse in Finder"
echo "./sav-video list             # List recent captures"