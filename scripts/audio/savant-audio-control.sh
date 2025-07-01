#!/bin/bash

# Savant Audio Control Script
# Easy controls for the system audio capture daemon

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${GREEN}✅ $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }

# Helper functions for daemon management
is_daemon_running() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE" 2>/dev/null)
        if [ -n "$pid" ] && ps -p "$pid" > /dev/null 2>&1; then
            return 0  # Running
        fi
    fi
    return 1  # Not running
}

get_daemon_pid() {
    if [ -f "$PID_FILE" ]; then
        cat "$PID_FILE" 2>/dev/null
    fi
}

is_daemon_running_launchd() {
    sudo launchctl list | grep -q "com.savant.audio.daemon" 2>/dev/null
}

# Updated paths to match the daemon script
SAVANT_DIR="$HOME/Documents/savant-ai"
DAEMON_SCRIPT="$SAVANT_DIR/scripts/audio/savant-audio-daemon.sh"
DAEMON_PLIST="/Library/LaunchAgents/com.savant.audio.daemon.plist"
CAPTURE_DIR="$SAVANT_DIR/data/audio-captures"
LOG_FILE="$SAVANT_DIR/data/daemon-logs/savant-audio-daemon.log"
PID_FILE="$SAVANT_DIR/data/daemon-logs/savant-audio-daemon.pid"

show_status() {
    echo "🎵 Savant Audio System Status"
    echo "============================="
    echo ""
    
    # Check if daemon is running (PID-based)
    if is_daemon_running; then
        local pid=$(get_daemon_pid)
        print_status "Audio capture daemon is RUNNING (PID: $pid)"
        
        # Show recent activity
        if [[ -f "$LOG_FILE" ]]; then
            echo ""
            print_info "Recent activity (last 5 lines):"
            tail -5 "$LOG_FILE" | sed 's/^/  /'
        fi
    elif is_daemon_running_launchd; then
        print_status "Audio capture daemon is RUNNING (via launchd)"
        print_info "Use 'launchctl' commands to manage this instance"
    else
        print_warning "Audio capture daemon is STOPPED"
    fi
    
    echo ""
    
    # Check captures directory
    if [[ -d "$CAPTURE_DIR" ]]; then
        local count=$(ls -1 "$CAPTURE_DIR"/*.md 2>/dev/null | wc -l)
        print_info "Captured files: $count transcripts in $CAPTURE_DIR"
        
        if [[ $count -gt 0 ]]; then
            echo ""
            print_info "Recent captures:"
            ls -lt "$CAPTURE_DIR"/*.md 2>/dev/null | head -3 | while read line; do
                echo "  $line"
            done
        fi
    else
        print_info "No captures directory found"
    fi
    
    echo ""
    
    # Check BlackHole availability
    if cargo run --package savant-audio --bin list-devices 2>/dev/null | grep -q -i "blackhole"; then
        print_status "BlackHole device available"
    else
        print_error "BlackHole device not found"
        echo "  Run: ./auto-setup-system-audio.sh"
    fi
}

start_daemon() {
    echo "🚀 Starting audio capture daemon..."
    
    # Check if already running
    if is_daemon_running; then
        local pid=$(get_daemon_pid)
        print_warning "Daemon is already running (PID: $pid)"
        return
    fi
    
    if is_daemon_running_launchd; then
        print_warning "Daemon is already running via launchd"
        return
    fi
    
    # Check if daemon script exists
    if [[ ! -f "$DAEMON_SCRIPT" ]]; then
        print_error "Daemon script not found: $DAEMON_SCRIPT"
        return 1
    fi
    
    # Start daemon in background
    print_info "Starting daemon script in background..."
    nohup "$DAEMON_SCRIPT" > /dev/null 2>&1 &
    local daemon_pid=$!
    
    # Give it a moment to start
    sleep 2
    
    # Verify it started successfully
    if is_daemon_running; then
        local actual_pid=$(get_daemon_pid)
        print_status "Audio capture daemon started successfully! (PID: $actual_pid)"
        print_info "Monitor with: tail -f $LOG_FILE"
        print_info "Stop with: $0 stop"
    else
        print_error "Failed to start daemon"
        print_info "Check logs: cat $LOG_FILE"
        return 1
    fi
}

stop_daemon() {
    echo "🛑 Stopping audio capture daemon..."
    
    local stopped_something=false
    
    # Stop PID-based daemon
    if is_daemon_running; then
        local pid=$(get_daemon_pid)
        print_info "Stopping PID-based daemon (PID: $pid)..."
        
        # Try graceful shutdown first
        kill -TERM "$pid" 2>/dev/null
        sleep 2
        
        # Check if it stopped
        if ! is_daemon_running; then
            print_status "Audio capture daemon stopped gracefully"
            stopped_something=true
        else
            print_warning "Graceful shutdown failed, using force..."
            kill -KILL "$pid" 2>/dev/null
            sleep 1
            
            if ! is_daemon_running; then
                print_status "Audio capture daemon stopped (forced)"
                stopped_something=true
            else
                print_error "Failed to stop daemon"
            fi
        fi
        
        # Clean up stale PID file
        rm -f "$PID_FILE"
    fi
    
    # Stop launchd-based daemon
    if is_daemon_running_launchd; then
        print_info "Stopping launchd-based daemon..."
        sudo launchctl unload "$DAEMON_PLIST" 2>/dev/null
        sleep 1
        
        if ! is_daemon_running_launchd; then
            print_status "Launchd daemon stopped"
            stopped_something=true
        else
            print_error "Failed to stop launchd daemon"
        fi
    fi
    
    if ! $stopped_something; then
        print_warning "No running daemon found to stop"
    fi
}

restart_daemon() {
    echo "🔄 Restarting audio capture daemon..."
    stop_daemon
    sleep 1
    start_daemon
}

view_logs() {
    if [[ -f "$LOG_FILE" ]]; then
        echo "📋 Live audio capture logs (Ctrl+C to exit):"
        echo "=============================================="
        tail -f "$LOG_FILE"
    else
        print_error "Log file not found: $LOG_FILE"
    fi
}

list_captures() {
    if [[ -d "$CAPTURE_DIR" ]] && [[ $(ls -1 "$CAPTURE_DIR"/*.md 2>/dev/null | wc -l) -gt 0 ]]; then
        echo "📁 Audio capture transcripts:"
        echo "============================="
        ls -lth "$CAPTURE_DIR"/*.md | while read line; do
            echo "  $line"
        done
        echo ""
        print_info "View a transcript: cat \"$CAPTURE_DIR/filename.md\""
        print_info "Search transcripts: grep -r \"search term\" \"$CAPTURE_DIR\""
    else
        print_warning "No captures found in $CAPTURE_DIR"
    fi
}

search_captures() {
    if [[ -z "$1" ]]; then
        echo "Usage: $0 search \"search term\""
        return 1
    fi
    
    if [[ ! -d "$CAPTURE_DIR" ]]; then
        print_error "Captures directory not found"
        return 1
    fi
    
    echo "🔍 Searching for: \"$1\""
    echo "======================"
    
    grep -r -i -n --color=always "$1" "$CAPTURE_DIR"/*.md 2>/dev/null | while IFS=: read file line content; do
        local filename=$(basename "$file")
        echo "📄 $filename (line $line):"
        echo "   $content"
        echo ""
    done || print_warning "No matches found for \"$1\""
}

test_multiple_instances() {
    echo "🧪 Testing multiple instance protection..."
    echo "========================================"
    
    if is_daemon_running; then
        local pid=$(get_daemon_pid)
        print_info "Daemon is currently running (PID: $pid)"
        print_info "Attempting to start a second instance..."
        
        # Try to start the daemon script directly to test protection
        "$DAEMON_SCRIPT" &
        local test_pid=$!
        sleep 2
        
        # Check if the test process is still running (it shouldn't be)
        if ps -p "$test_pid" > /dev/null 2>&1; then
            print_error "Multiple instance protection FAILED - second instance is running!"
            kill "$test_pid" 2>/dev/null
        else
            print_status "Multiple instance protection WORKING - second instance was rejected"
        fi
    else
        print_warning "No daemon currently running. Start one first with: $0 start"
    fi
}

show_help() {
    echo "🎵 Savant Audio Control"
    echo "======================"
    echo ""
    echo "Usage: $0 <command> [arguments]"
    echo ""
    echo "Commands:"
    echo "  status     - Show daemon status and recent activity"
    echo "  start      - Start the audio capture daemon"
    echo "  stop       - Stop the audio capture daemon"
    echo "  restart    - Restart the audio capture daemon"
    echo "  logs       - View live daemon logs"
    echo "  list       - List all captured transcripts"
    echo "  search     - Search transcripts for text"
    echo "  test       - Test multiple instance protection"
    echo "  setup      - Run the automated setup"
    echo "  help       - Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 status                    # Check if daemon is running"
    echo "  $0 start                     # Start background capture"
    echo "  $0 search \"meeting\"          # Find mentions of 'meeting'"
    echo "  $0 logs                      # Watch live capture activity"
    echo ""
    echo "Files:"
    echo "  Captures: $CAPTURE_DIR/"
    echo "  Logs:     $LOG_FILE"
    echo ""
}

# Main command handling
case "$1" in
    "status"|"")
        show_status
        ;;
    "start")
        start_daemon
        ;;
    "stop")
        stop_daemon
        ;;
    "restart")
        restart_daemon
        ;;
    "logs")
        view_logs
        ;;
    "list")
        list_captures
        ;;
    "search")
        search_captures "$2"
        ;;
    "test")
        test_multiple_instances
        ;;
    "setup")
        ./auto-setup-system-audio.sh
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac