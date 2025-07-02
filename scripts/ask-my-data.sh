#!/bin/bash
# Simple Natural Language Query Script for Personal Audio Data
# Query your actual audio transcription data via chatbot

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}🎤 Ask My Audio Data${NC}"
echo -e "${CYAN}===================${NC}"

# Config
AUDIO_DIR="/Users/yeager/Documents/savant-ai/data/audio-captures"
DB_PATH="$HOME/.config/savant-ai/my-audio-data.db"
DB_CLI="./target/release/savant-db"
MCP_SERVER="./target/release/savant-mcp-server"

# Build tools
echo -e "${BLUE}Building tools...${NC}"
cargo build --package savant-db --package savant-mcp --release >/dev/null 2>&1

# Check dependencies
if ! curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
    echo -e "${RED}❌ Ollama not running. Run: ollama serve${NC}"
    exit 1
fi

# Import data if database is empty
import_data() {
    echo -e "${BLUE}📥 Importing your July 1st audio data...${NC}"
    
    local count=0
    for file in "$AUDIO_DIR"/system_audio_20250701_*.md; do
        if [ -f "$file" ]; then
            # Get main text content
            local main_text=$(jq -r '.text // ""' "$file" 2>/dev/null || echo "")
            
            # Skip if no meaningful content (< 100 chars or only silence/unclear)
            if [ ${#main_text} -lt 100 ] || [[ "$main_text" == *"[unclear audio]"* && ${#main_text} -lt 200 ]]; then
                continue
            fi
            
            # Extract timestamp from filename
            local timestamp=$(basename "$file" | sed -E 's/system_audio_([0-9]{4})([0-9]{2})([0-9]{2})_([0-9]{2})([0-9]{2})([0-9]{2})\.md/\1-\2-\3T\4:\5:\6Z/')
            
            # Create simplified version for import
            local simplified=$(cat <<EOF
{
  "text": $(echo "$main_text" | jq -R),
  "language": "en",
  "segments": [
    {
      "text": $(echo "$main_text" | cut -c1-200 | jq -R),
      "start_time": 0.0,
      "end_time": 300.0,
      "confidence": 0.8,
      "words": null
    }
  ],
  "processing_time_ms": 100,
  "model_used": "whisper",
  "session_metadata": {
    "session_id": "audio-session-$count",
    "timestamp": "$timestamp",
    "audio_source": "SystemAudio",
    "speaker": "system_audio",
    "device_info": "MacBook Pro"
  }
}
EOF
            )
            
            echo -e "${CYAN}  📄 $(basename "$file") → $timestamp${NC}"
            echo "$simplified" | $DB_CLI --db-path "$DB_PATH" store --title "Audio Capture $timestamp" >/dev/null 2>&1
            count=$((count + 1))
        fi
    done
    
    echo -e "${GREEN}✅ Imported $count meaningful audio files${NC}"
}

# Query function
query_data() {
    local query="$1"
    echo -e "\n${CYAN}🤖 \"$query\"${NC}"
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    
    # Create MCP request
    local request=$(cat <<EOF
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_conversations",
    "arguments": {
      "query": "$query",
      "session_id": "user-query-$(date +%s)"
    }
  }
}
EOF
    )
    
    echo -e "${BLUE}🔍 Analyzing your audio data...${NC}"
    
    # Query via MCP
    local response=$(echo "$request" | timeout 30s $MCP_SERVER --database "$DB_PATH" --llm-provider ollama --llm-model devstral 2>/dev/null | head -1)
    
    # Extract meaningful response
    if [ -n "$response" ]; then
        echo -e "\n${GREEN}📋 Results:${NC}"
        echo "$response" | jq -r '.result.content[0].text // .result // "No specific results found"' 2>/dev/null | sed 's/^/  /'
    else
        echo -e "\n${YELLOW}⚠️  No response received. Your audio data might not contain relevant information.${NC}"
    fi
}

# Check if data exists
DB_SIZE=$(du -k "$DB_PATH" 2>/dev/null | cut -f1 2>/dev/null || echo "0")
if [ "$DB_SIZE" -lt 10 ]; then
    echo -e "${YELLOW}📊 No data found. Importing your audio files...${NC}"
    import_data
fi

# Handle arguments
case "${1:-}" in
    --reimport)
        rm -f "$DB_PATH"
        import_data
        ;;
    --stats)
        echo -e "${BLUE}📊 Your Audio Data:${NC}"
        $DB_CLI --db-path "$DB_PATH" list --limit 5 2>/dev/null || echo "No data"
        ;;
    --interactive|-i)
        while true; do
            echo -e -n "\n${CYAN}❓ Ask about your audio data (or 'quit'): ${NC}"
            read -r user_query
            case "$user_query" in
                quit|exit|q) echo -e "${GREEN}👋 Goodbye!${NC}"; break ;;
                "") continue ;;
                *) query_data "$user_query" ;;
            esac
        done
        ;;
    --help|-h|"")
        cat <<EOF
${CYAN}🎤 Ask My Audio Data${NC}

${BLUE}USAGE:${NC}
  $0 "your question about audio data"
  $0 --interactive     # Interactive mode
  $0 --reimport        # Reimport audio files
  $0 --stats           # Show data summary

${BLUE}EXAMPLES:${NC}
  $0 "summarize my audio from july 1st 2025"
  $0 "what topics were discussed?"
  $0 "what was the longest conversation about?"
  $0 --interactive

${BLUE}REQUIREMENTS:${NC}
  - Ollama running: ollama serve
  - Model available: ollama pull devstral
EOF
        ;;
    *)
        query_data "$*"
        ;;
esac