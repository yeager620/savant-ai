#!/bin/bash

# Video daemon control script - mirrors audio control structure
# Provides start/stop/status/logs commands for video capture daemon

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
DAEMON_SCRIPT="$SCRIPT_DIR/savant-video-daemon.sh"

# Configuration
PID_FILE="$HOME/.config/savant-ai/video-daemon.pid"
LOG_DIR="$HOME/.config/savant-ai/daemon-logs"
CAPTURE_DIR="$HOME/Library/Application Support/savant-ai/video-captures"
LOG_FILE="$LOG_DIR/video-daemon.log"
ERROR_LOG="$LOG_DIR/video-daemon-error.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print usage
usage() {
    echo "Usage: $0 {start|stop|restart|status|logs|list|test}"
    echo ""
    echo "Commands:"
    echo "  start    - Start the video capture daemon"
    echo "  stop     - Stop the video capture daemon"
    echo "  restart  - Restart the video capture daemon"
    echo "  status   - Show daemon status"
    echo "  logs     - Show recent daemon logs"
    echo "  list     - List recent captures"
    echo "  test     - Run a test capture"
    exit 1
}

# Check if daemon is running
is_running() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

# Start daemon
start_daemon() {
    if is_running; then
        echo -e "${YELLOW}Video daemon is already running (PID: $(cat "$PID_FILE"))${NC}"
        return 1
    fi
    
    echo -e "${GREEN}Starting video capture daemon...${NC}"
    
    # Build the project if needed
    if [ ! -f "$PROJECT_ROOT/target/release/savant-video" ] && [ ! -f "$PROJECT_ROOT/target/debug/savant-video" ]; then
        echo "Building savant-video..."
        (cd "$PROJECT_ROOT" && cargo build --package savant-video-cli)
    fi
    
    # Start daemon in background
    nohup "$DAEMON_SCRIPT" > /dev/null 2>&1 &
    
    # Wait for daemon to start
    sleep 2
    
    if is_running; then
        echo -e "${GREEN}Video daemon started successfully (PID: $(cat "$PID_FILE"))${NC}"
        echo -e "Log file: $LOG_FILE"
        return 0
    else
        echo -e "${RED}Failed to start video daemon${NC}"
        if [ -f "$ERROR_LOG" ]; then
            echo "Recent errors:"
            tail -n 5 "$ERROR_LOG"
        fi
        return 1
    fi
}

# Stop daemon
stop_daemon() {
    if ! is_running; then
        echo -e "${YELLOW}Video daemon is not running${NC}"
        return 0
    fi
    
    PID=$(cat "$PID_FILE")
    echo -e "${YELLOW}Stopping video daemon (PID: $PID)...${NC}"
    
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
        echo -e "${YELLOW}Force killing daemon...${NC}"
        kill -KILL "$PID" 2>/dev/null || true
    fi
    
    rm -f "$PID_FILE"
    echo -e "${GREEN}Video daemon stopped${NC}"
}

# Show daemon status
show_status() {
    if is_running; then
        PID=$(cat "$PID_FILE")
        echo -e "${GREEN}Video daemon is running${NC}"
        echo "PID: $PID"
        
        # Show process info
        ps -p "$PID" -o pid,ppid,user,%cpu,%mem,etime,command | tail -n +1
        
        # Show recent captures
        echo ""
        echo "Recent captures:"
        find "$CAPTURE_DIR" -name "*.png" -type f -mtime -1 | wc -l | xargs echo "  Captures in last 24h:"
        
        # Show storage usage
        if [ -d "$CAPTURE_DIR" ]; then
            du -sh "$CAPTURE_DIR" 2>/dev/null | xargs echo "  Storage usage:"
        fi
    else
        echo -e "${RED}Video daemon is not running${NC}"
    fi
}

# Show logs
show_logs() {
    if [ ! -f "$LOG_FILE" ]; then
        echo "No log file found"
        return
    fi
    
    echo "Recent daemon logs:"
    echo "=================="
    tail -n 50 "$LOG_FILE"
    
    if [ -f "$ERROR_LOG" ] && [ -s "$ERROR_LOG" ]; then
        echo ""
        echo "Recent errors:"
        echo "============="
        tail -n 20 "$ERROR_LOG"
    fi
}

# List recent captures
list_captures() {
    echo "Recent video captures:"
    echo "===================="
    
    if [ -d "$CAPTURE_DIR" ]; then
        find "$CAPTURE_DIR" -name "*.png" -type f -mtime -7 | sort -r | head -20 | while read -r file; do
            size=$(du -h "$file" | cut -f1)
            date=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M:%S" "$file" 2>/dev/null || stat -c "%y" "$file" 2>/dev/null | cut -d. -f1)
            echo "$date - $size - $(basename "$file")"
        done
    else
        echo "No captures directory found"
    fi
}

# Run test capture
test_capture() {
    echo -e "${GREEN}Running test capture...${NC}"
    
    # Build if needed
    if [ ! -f "$PROJECT_ROOT/target/release/savant-video" ] && [ ! -f "$PROJECT_ROOT/target/debug/savant-video" ]; then
        echo "Building savant-video..."
        (cd "$PROJECT_ROOT" && cargo build --package savant-video-cli)
    fi
    
    # Find the binary
    CAPTURE_CMD="$PROJECT_ROOT/target/release/savant-video"
    if [ ! -f "$CAPTURE_CMD" ]; then
        CAPTURE_CMD="$PROJECT_ROOT/target/debug/savant-video"
    fi
    
    # Run single capture
    "$CAPTURE_CMD" start --interval 5 --duration 5 --format text
    
    echo -e "${GREEN}Test capture completed${NC}"
}

# Main command handling
case "$1" in
    start)
        start_daemon
        ;;
    stop)
        stop_daemon
        ;;
    restart)
        stop_daemon
        sleep 2
        start_daemon
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    list)
        list_captures
        ;;
    test)
        test_capture
        ;;
    *)
        usage
        ;;
esac