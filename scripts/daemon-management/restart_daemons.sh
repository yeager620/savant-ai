#!/bin/bash
echo "🔄 Restarting all Savant AI daemons..."

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Stop all daemons first
echo "Stopping existing daemons..."
"$SCRIPT_DIR/stop_all_daemons.sh"

# Wait a moment for clean shutdown
echo "Waiting for clean shutdown..."
sleep 3

# Start all daemons
echo "Starting daemons..."
"$SCRIPT_DIR/start_all_daemons.sh"

echo "🔄 Restart complete!"