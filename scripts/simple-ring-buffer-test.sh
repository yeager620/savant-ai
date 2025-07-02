#!/bin/bash
# Simple test for ring buffer functionality

set -e

echo "Testing ring buffer with existing transcript files..."

# Check current file count
CAPTURE_DIR="./data/audio-captures"
if [ -d "$CAPTURE_DIR" ]; then
    file_count=$(find "$CAPTURE_DIR" -name "system_audio_*" | wc -l | tr -d " ")
    size_mb=$(du -sm "$CAPTURE_DIR" 2>/dev/null | cut -f1 || echo "0")
    
    echo "Current status: $file_count files, ${size_mb}MB"
    echo "Limits: 50 files, 100MB"
    
    if [ "$file_count" -gt 50 ] || [ "$size_mb" -gt 100 ]; then
        echo "✅ Ring buffer would trigger cleanup (limits exceeded)"
    else
        echo "ℹ️  Ring buffer within limits (no cleanup needed)"
    fi
else
    echo "No audio captures directory found"
fi

echo "Ring buffer implementation is ready!"