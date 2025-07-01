#!/bin/bash

# Savant Audio Daemon - Continuous system audio capture
# This runs in the background and captures all system audio

# Use absolute paths to avoid permission issues
SAVANT_DIR="$HOME/Documents/savant-ai"
CAPTURE_DIR="$SAVANT_DIR/data/audio-captures"
LOG_FILE="$SAVANT_DIR/data/daemon-logs/savant-audio-daemon.log"
LOCK_FILE="/tmp/savant_audio_daemon.lock"
PID_FILE="$SAVANT_DIR/data/daemon-logs/savant-audio-daemon.pid"
SEGMENT_DURATION=300  # 5 minutes per segment

# Create necessary directories
mkdir -p "$CAPTURE_DIR"
mkdir -p "$(dirname "$LOG_FILE")"
mkdir -p "$(dirname "$PID_FILE")"

# Function to log with timestamp
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S'): $1" | tee -a "$LOG_FILE"
}

# Function to cleanup on exit
cleanup_on_exit() {
    log_message "Savant Audio Daemon stopping (PID: $$)"
    rm -f "$PID_FILE"
    exit 0
}

# Function to check for existing instances and enforce single instance
ensure_single_instance() {
    # Check for existing PID file
    if [ -f "$PID_FILE" ]; then
        local old_pid=$(cat "$PID_FILE" 2>/dev/null)
        if [ -n "$old_pid" ] && ps -p "$old_pid" > /dev/null 2>&1; then
            log_message "Another instance of Savant Audio Daemon is already running (PID: $old_pid). Exiting."
            echo "Error: Daemon already running (PID: $old_pid)"
            echo "Use 'savant-audio-control.sh stop' to stop the existing daemon first."
            exit 1
        else
            log_message "Stale PID file found (PID: $old_pid not running). Removing: $PID_FILE"
            rm -f "$PID_FILE"
        fi
    fi
    
    # Write current PID to file
    echo $$ > "$PID_FILE"
    
    # Set up signal handlers for clean shutdown
    trap cleanup_on_exit EXIT INT TERM
    
    log_message "Savant Audio Daemon started (PID: $$) and acquired lock."
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

# Ensure only one instance runs
ensure_single_instance

# Main daemon loop
while true; do
    capture_segment
    sleep 5  # Brief pause between segments
done
