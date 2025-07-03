#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
echo "Starting Savant AI multimodal daemons..."

# Check dependencies
if ! command -v ollama &> /dev/null; then
    echo "Error: ollama not installed. Run: brew install ollama"
    exit 1
fi

if ! command -v tesseract &> /dev/null; then
    echo "Error: tesseract not installed. Run: brew install tesseract"
    exit 1
fi

# Check if required models are available
echo "Checking Ollama models..."
if ! ollama list | grep -q "llama3.2"; then
    echo "Installing llama3.2 model (this may take a few minutes)..."
    ollama pull llama3.2
fi

# Start Ollama if not running
if ! pgrep -f "ollama serve" > /dev/null; then
    echo "Starting Ollama server..."
    ollama serve &
    sleep 3
    echo "Waiting for Ollama to initialize..."
fi

# Verify Ollama is responsive
echo "Verifying Ollama connection..."
timeout 10 bash -c 'until curl -s http://localhost:11434/api/tags > /dev/null; do sleep 1; done' || {
    echo "Warning: Ollama server may not be fully ready"
}

# Start audio daemon
echo "Starting audio daemon..."
if "$SCRIPT_DIR/../../sav" start; then
    echo "✓ Audio daemon started successfully"
else
    echo "⚠ Audio daemon failed to start (may already be running)"
fi

# Wait a moment for audio daemon to initialize
sleep 2

# Start enhanced video daemon with all features
echo "Starting video daemon with multimodal analysis..."
if "$SCRIPT_DIR/../../sav-video" start --interval 30 --enable-ocr --enable-vision --enable-correlation; then
    echo "✓ Video daemon started successfully"
else
    echo "⚠ Video daemon failed to start (may already be running)"
fi

echo ""
echo "=== Daemon Status ==="
"$SCRIPT_DIR/../../sav" status 2>/dev/null || echo "Audio daemon: Unknown status"
echo "---"
"$SCRIPT_DIR/../../sav-video" status 2>/dev/null || echo "Video daemon: Unknown status"

echo ""
echo "🚀 All daemons startup complete!"
echo ""
echo "📊 Monitor with:"
echo "  ./monitor-daemons (from project root)"
echo "  ./scripts/daemon-management/monitor_daemons.sh"
echo ""
echo "🔍 Check logs with:"
echo "  \"$SCRIPT_DIR/../../sav\" logs"
echo "  \"$SCRIPT_DIR/../../sav-video\" logs"
echo ""
echo "🛑 Stop all with:"
echo "  ./stop-daemons (from project root)"
echo "  ./scripts/daemon-management/stop_all_daemons.sh"