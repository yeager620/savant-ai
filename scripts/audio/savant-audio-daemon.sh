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
MAX_BUFFER_SIZE_MB=100  # Maximum size of transcript buffer in MB
MAX_BUFFER_FILES=50     # Maximum number of transcript files to keep

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

# Function to manage ring buffer - keeps transcript storage within limits
manage_ring_buffer() {
    if [ ! -d "$CAPTURE_DIR" ]; then
        return 0
    fi
    
    # Get current buffer size in MB
    local current_size_kb=$(du -sk "$CAPTURE_DIR" 2>/dev/null | cut -f1 || echo "0")
    local current_size_mb=$((current_size_kb / 1024))
    
    # Count current files
    local file_count=$(find "$CAPTURE_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | wc -l | tr -d ' ')
    
    log_message "Ring buffer status: ${current_size_mb}MB, ${file_count} files (limits: ${MAX_BUFFER_SIZE_MB}MB, ${MAX_BUFFER_FILES} files)"
    
    # Check if we need to clean up by size or file count
    if [ "$current_size_mb" -gt "$MAX_BUFFER_SIZE_MB" ] || [ "$file_count" -gt "$MAX_BUFFER_FILES" ]; then
        log_message "Ring buffer limit exceeded, cleaning up old files..."
        
        # Calculate how many files to remove (remove 20% when limit is reached)
        local files_to_remove=$((file_count / 5))
        if [ "$files_to_remove" -lt 5 ]; then
            files_to_remove=5  # Remove at least 5 files
        fi
        
        # Find oldest files and remove them
        local removed_count=0
        find "$CAPTURE_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | \
        sort | \
        head -n "$files_to_remove" | \
        while read -r old_file; do
            if [ -f "$old_file" ]; then
                local file_size=$(du -k "$old_file" 2>/dev/null | cut -f1 || echo "0")
                rm -f "$old_file"
                log_message "Removed old transcript: $(basename "$old_file") (${file_size}KB)"
                removed_count=$((removed_count + 1))
            fi
        done
        
        # Log final status
        local new_size_kb=$(du -sk "$CAPTURE_DIR" 2>/dev/null | cut -f1 || echo "0")
        local new_size_mb=$((new_size_kb / 1024))
        local new_file_count=$(find "$CAPTURE_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | wc -l | tr -d ' ')
        
        log_message "Ring buffer cleanup completed: ${new_size_mb}MB, ${new_file_count} files remaining"
    fi
}

# Function to capture audio segment
capture_segment() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local output_file="$CAPTURE_DIR/system_audio_$timestamp.json"
    
    log_message "Starting capture segment: $output_file"
    
    if ! check_savant_dir; then
        log_message "Skipping capture - savant directory not accessible"
        return 1
    fi
    
    # Check ring buffer size and cleanup if needed
    manage_ring_buffer
    
    cd "$SAVANT_DIR"
    
    cargo run --package savant-transcribe -- \
        --duration $SEGMENT_DURATION \
        --device "BlackHole 2ch" \
        --format json \
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
