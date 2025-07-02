#!/bin/bash
echo "Stopping all Savant AI daemons..."

# Stop video daemon first (it may depend on audio)
echo "Stopping video daemon..."
if ../../sav-video stop; then
    echo "✓ Video daemon stopped"
else
    echo "⚠ Video daemon stop failed or not running"
fi

# Stop audio daemon
echo "Stopping audio daemon..."
if ../../sav stop; then
    echo "✓ Audio daemon stopped"
else
    echo "⚠ Audio daemon stop failed or not running"
fi

# Optional: Stop Ollama server (uncomment if desired)
# echo "Stopping Ollama server..."
# pkill -f "ollama serve" && echo "✓ Ollama server stopped" || echo "⚠ Ollama server not running"

echo ""
echo "=== Final Status Check ==="
../../sav status 2>/dev/null || echo "Audio daemon: Stopped"
echo "---"
../../sav-video status 2>/dev/null || echo "Video daemon: Stopped"

echo ""
echo "🛑 All daemons stopped successfully!"