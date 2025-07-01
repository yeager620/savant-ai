#!/bin/bash

# Fixed Savant Audio Daemon - Continuous system audio capture

# Use absolute paths to avoid permission issues
CAPTURE_DIR="$HOME/savant-audio-captures"
SEGMENT_DURATION=300  # 5 minutes per segment
LOG_FILE="$HOME/savant-audio-daemon.log"
SAVANT_DIR="$HOME/Documents/savant-ai"

# Create capture directory
mkdir -p "$CAPTURE_DIR"

# Function to log with timestamp
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S'): $1" >> "$LOG_FILE"
}

# Function to capture audio segment
capture_segment() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local output_file="$CAPTURE_DIR/system_audio_$timestamp.md"
    
    log_message "Starting capture segment: $output_file"
    
    # Change to the correct directory
    cd "$SAVANT_DIR" || {
        log_message "Failed to change to directory: $SAVANT_DIR"
        return 1
    }
    
    # Run capture with timeout and proper error handling
    timeout $SEGMENT_DURATION cargo run --package savant-transcribe -- \
        --duration $SEGMENT_DURATION \
        --system \
        --output "$output_file" >> "$LOG_FILE" 2>&1
    
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        log_message "Capture completed successfully: $output_file"
    elif [ $exit_code -eq 124 ]; then
        log_message "Capture timed out (expected): $output_file"
    else
        log_message "Capture failed with exit code $exit_code for segment $timestamp"
    fi
}

# Main daemon loop
log_message "Savant Audio Daemon started (PID: $$) in directory: $(pwd)"
log_message "Capture directory: $CAPTURE_DIR"
log_message "Log file: $LOG_FILE"

# Verify dependencies
if ! command -v cargo &> /dev/null; then
    log_message "ERROR: cargo not found in PATH"
    exit 1
fi

if [ ! -d "$SAVANT_DIR" ]; then
    log_message "ERROR: Savant directory not found: $SAVANT_DIR"
    exit 1
fi

# Check if BlackHole is available
cd "$SAVANT_DIR"
if ! cargo run --package savant-audio --bin list-devices 2>/dev/null | grep -i "blackhole"; then
    log_message "WARNING: BlackHole device not detected"
fi

log_message "Starting capture loop..."

while true; do
    capture_segment
    sleep 5  # Brief pause between segments
done