#!/bin/bash

# Savant AI Daemon Monitor Script
# Provides real-time monitoring of audio and video daemons

echo "🔍 Savant AI Daemon Monitor"
echo "Press Ctrl+C to exit"
echo "=========================="

# Function to get status with color coding
get_status() {
    local daemon_name=$1
    local status_cmd=$2
    
    if $status_cmd &>/dev/null; then
        echo "🟢 $daemon_name: Running"
    else
        echo "🔴 $daemon_name: Stopped"
    fi
}

# Get the project root directory (two levels up from this script)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Function to display system resources
show_resources() {
    echo ""
    echo "📊 System Resources:"
    echo "CPU: $(top -l 1 | grep "CPU usage" | awk '{print $3}' | sed 's/%//')"
    echo "Memory: $(memory_pressure | grep "System-wide memory free percentage" | awk '{print $5}' | sed 's/%//')% free"
    echo "Disk: $(df -h . | tail -1 | awk '{print $5}') used"
}

# Function to show recent log entries
show_recent_logs() {
    echo ""
    echo "📝 Recent Audio Logs (last 3 lines):"
    "$PROJECT_ROOT/sav" logs 2>/dev/null | tail -3 | sed 's/^/  /' || echo "  No audio logs available"
    
    echo ""
    echo "📝 Recent Video Logs (last 3 lines):"
    "$PROJECT_ROOT/sav-video" logs 2>/dev/null | tail -3 | sed 's/^/  /' || echo "  No video logs available"
}

# Function to check Ollama status
check_ollama() {
    if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
        echo "🟢 Ollama Server: Running"
    else
        echo "🔴 Ollama Server: Not responding"
    fi
}

# Main monitoring loop
while true; do
    clear
    echo "🔍 Savant AI Daemon Monitor - $(date '+%H:%M:%S')"
    echo "Press Ctrl+C to exit"
    echo "=========================="
    
    # Check daemon status
    get_status "Audio Daemon " "$PROJECT_ROOT/sav status"
    get_status "Video Daemon " "$PROJECT_ROOT/sav-video status"
    check_ollama
    
    # Show system resources
    show_resources
    
    # Show recent logs
    show_recent_logs
    
    echo ""
    echo "🔄 Refreshing in 5 seconds..."
    sleep 5
done