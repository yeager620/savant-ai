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

DAEMON_PLIST="/Library/LaunchAgents/com.savant.audio.daemon.plist"
CAPTURE_DIR="$HOME/savant-audio-captures"
LOG_FILE="$HOME/savant-audio-daemon.log"

show_status() {
    echo "🎵 Savant Audio System Status"
    echo "============================="
    echo ""
    
    # Check if daemon is running
    if sudo launchctl list | grep -q "com.savant.audio.daemon"; then
        print_status "Audio capture daemon is RUNNING"
        
        # Show recent activity
        if [[ -f "$LOG_FILE" ]]; then
            echo ""
            print_info "Recent activity (last 5 lines):"
            tail -5 "$LOG_FILE" | sed 's/^/  /'
        fi
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
    
    if sudo launchctl list | grep -q "com.savant.audio.daemon"; then
        print_warning "Daemon is already running"
        return
    fi
    
    if [[ ! -f "$DAEMON_PLIST" ]]; then
        print_error "Daemon not installed. Run setup first:"
        echo "  ./auto-setup-system-audio.sh"
        return 1
    fi
    
    sudo launchctl load "$DAEMON_PLIST"
    sleep 2
    
    if sudo launchctl list | grep -q "com.savant.audio.daemon"; then
        print_status "Audio capture daemon started successfully!"
        print_info "Monitor with: tail -f $LOG_FILE"
    else
        print_error "Failed to start daemon"
    fi
}

stop_daemon() {
    echo "🛑 Stopping audio capture daemon..."
    
    if ! sudo launchctl list | grep -q "com.savant.audio.daemon"; then
        print_warning "Daemon is not running"
        return
    fi
    
    sudo launchctl unload "$DAEMON_PLIST"
    sleep 2
    
    if ! sudo launchctl list | grep -q "com.savant.audio.daemon"; then
        print_status "Audio capture daemon stopped"
    else
        print_error "Failed to stop daemon"
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