#!/bin/bash

# Video capture daemon - mirrors audio daemon structure
# Captures screenshots at regular intervals with stealth mode

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

# Configuration
PID_FILE="$HOME/.config/savant-ai/video-daemon.pid"
LOG_DIR="$HOME/.config/savant-ai/daemon-logs"
CAPTURE_DIR="$HOME/.config/savant-ai/video-captures"
LOG_FILE="$LOG_DIR/video-daemon.log"
ERROR_LOG="$LOG_DIR/video-daemon-error.log"

# Default settings
DEFAULT_INTERVAL=30  # seconds between captures
DEFAULT_QUALITY="medium"
DEFAULT_STEALTH="true"

# Parse arguments
INTERVAL="${1:-$DEFAULT_INTERVAL}"
QUALITY="${2:-$DEFAULT_QUALITY}"
STEALTH="${3:-$DEFAULT_STEALTH}"

# Create directories
mkdir -p "$LOG_DIR" "$CAPTURE_DIR"
mkdir -p "$(dirname "$PID_FILE")"

# Logging functions
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

error_log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1" | tee -a "$ERROR_LOG" >&2
}

# Check if daemon is already running
check_running() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        else
            rm -f "$PID_FILE"
        fi
    fi
    return 1
}

# Stop any existing daemon
stop_daemon() {
    if check_running; then
        PID=$(cat "$PID_FILE")
        log "Stopping existing video daemon (PID: $PID)"
        kill -TERM "$PID" 2>/dev/null || true
        
        # Wait for process to exit
        for i in {1..10}; do
            if ! ps -p "$PID" > /dev/null 2>&1; then
                break
            fi
            sleep 1
        done
        
        # Force kill if still running
        if ps -p "$PID" > /dev/null 2>&1; then
            kill -KILL "$PID" 2>/dev/null || true
        fi
        
        rm -f "$PID_FILE"
    fi
}

# Signal handlers
cleanup() {
    log "Video daemon shutting down..."
    rm -f "$PID_FILE"
    exit 0
}

trap cleanup SIGTERM SIGINT

# Main daemon function
run_daemon() {
    # Save PID
    echo $$ > "$PID_FILE"
    
    log "Video daemon started (PID: $$)"
    log "Interval: ${INTERVAL}s, Quality: $QUALITY, Stealth: $STEALTH"
    
    # Build capture command
    CAPTURE_CMD="$PROJECT_ROOT/target/release/savant-video"
    if [ ! -f "$CAPTURE_CMD" ]; then
        CAPTURE_CMD="$PROJECT_ROOT/target/debug/savant-video"
        if [ ! -f "$CAPTURE_CMD" ]; then
            error_log "savant-video binary not found. Run 'cargo build' first."
            exit 1
        fi
    fi
    
    # Add stealth flag if needed
    STEALTH_FLAG=""
    if [ "$STEALTH" != "true" ]; then
        STEALTH_FLAG="--no-stealth"
    fi
    
    # Run continuous capture
    while true; do
        # Check if we should continue
        if [ ! -f "$PID_FILE" ] || [ "$(cat "$PID_FILE")" != "$$" ]; then
            log "PID file removed or changed, exiting"
            break
        fi
        
        # Run capture with timeout to prevent hanging
        timeout 300 "$CAPTURE_CMD" start \
            --interval "$INTERVAL" \
            --duration "$INTERVAL" \
            --format json \
            $STEALTH_FLAG \
            2>> "$ERROR_LOG" | while IFS= read -r line; do
                # Log captured frames
                if [[ "$line" == *"frame"* ]]; then
                    echo "$line" >> "$LOG_FILE"
                fi
            done
        
        # Brief pause between capture sessions
        sleep 1
    done
    
    cleanup
}

# Ensure single instance
if check_running; then
    error_log "Video daemon is already running (PID: $(cat "$PID_FILE"))"
    exit 1
fi

# Stop any existing daemon
stop_daemon

# Start daemon
log "Starting video capture daemon..."
run_daemon