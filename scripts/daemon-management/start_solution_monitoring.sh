#!/bin/bash

# Script to start solution monitoring integrated with screen capture
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Starting Solution Monitoring System...${NC}"

# Check if video daemon is running
if pgrep -f "sav-video" > /dev/null; then
    echo -e "${YELLOW}Video capture daemon is already running${NC}"
else
    echo -e "${GREEN}Starting video capture daemon...${NC}"
    "$PROJECT_ROOT/sav-video" start --interval 500 --enable-ocr --enable-vision --enable-correlation
fi

# Initialize solution processor in the app
echo -e "${GREEN}Initializing solution processor...${NC}"
osascript -e 'tell application "Savant AI" to activate'

# Wait a moment for the app to start
sleep 2

# Send initialization command via the app
# This would normally be done through the app's UI or automatically on startup
echo -e "${GREEN}Solution monitoring is now active${NC}"
echo -e "${YELLOW}The system will:${NC}"
echo "  - Monitor your screen every 500ms"
echo "  - Detect coding problems automatically"
echo "  - Generate solutions in real-time"
echo "  - Display solutions in an overlay window"

echo -e "\n${GREEN}To stop monitoring, run:${NC} $PROJECT_ROOT/sav-video stop"