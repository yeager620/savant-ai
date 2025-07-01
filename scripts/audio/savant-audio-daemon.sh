#!/bin/bash

# Savant Audio Daemon - Continuous system audio capture
# This runs in the background and captures all system audio

# Use absolute paths to avoid permission issues
SAVANT_DIR="$HOME/Documents/savant-ai"
CAPTURE_DIR="$SAVANT_DIR/data/audio-captures"
LOG_FILE="$SAVANT_DIR/data/daemon-logs/savant-audio-daemon.log"
SEGMENT_DURATION=300  # 5 minutes per segment

# Create capture directory
mkdir -p "$CAPTURE_DIR"

# Function to log with timestamp
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S'): $1" | tee -a "$LOG_FILE"
}

# Function to check if we can access the savant directory
check_savant_dir() {
    if [ ! -d "$SAVANT_DIR" ]; then
        log_message "Error: Savant directory not found: $SAVANT_DIR"
        return 1
    fi
    return 0
}

# Function to capture audio segment
capture_segment() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local output_file="$CAPTURE_DIR/system_audio_$timestamp.md"
    
    log_message "Starting capture segment: $output_file"
    
    if ! check_savant_dir; then
        log_message "Skipping capture - savant directory not accessible"
        return 1
    fi
    
    cd "$SAVANT_DIR"
    
    cargo run --package savant-transcribe -- \
        --duration $SEGMENT_DURATION \
        --device "BlackHole 2ch" \
        --output "$output_file" 2>>"$LOG_FILE"
    
    if [ $? -eq 0 ]; then
        log_message "Capture completed: $output_file"
    else
        log_message "Capture failed for segment $timestamp"
    fi
}

# Main daemon loop
log_message "Savant Audio Daemon started (PID: $$)"

while true; do
    capture_segment
    sleep 5  # Brief pause between segments
done
