#!/bin/bash
# Personal Audio Data Query System
# Ask questions about your audio transcription data using natural language

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${CYAN}🎤 Personal Audio Query System${NC}"
echo -e "${CYAN}=============================${NC}"

# Build tools
echo -e "${BLUE}🔧 Building database and MCP tools...${NC}"
cargo build --package savant-db --package savant-mcp --release >/dev/null 2>&1

# Check Ollama
if ! curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Starting Ollama is required${NC}"
    echo -e "Run: ${BLUE}ollama serve${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Tools ready${NC}"

# Configuration
DB_PATH="./data/databases/dev/personal-audio.db"
AUDIO_DIR="/Users/yeager/Documents/savant-ai/data/audio-captures"

# Create a sample database with your actual audio content
setup_database() {
    echo -e "${BLUE}📥 Setting up your audio database...${NC}"
    
    # Remove existing database
    rm -f "$DB_PATH"
    
    # Find files with meaningful content
    local imported=0
    for file in "$AUDIO_DIR"/system_audio_20250701_*.md; do
        if [ -f "$file" ]; then
            # Extract main text
            local content=$(jq -r '.text // ""' "$file" 2>/dev/null || echo "")
            
            # Skip files with minimal content
            if [ ${#content} -lt 200 ]; then
                continue
            fi
            
            # Extract timestamp from filename  
            local timestamp=$(basename "$file" | sed -E 's/system_audio_([0-9]{4})([0-9]{2})([0-9]{2})_([0-9]{2})([0-9]{2})([0-9]{2})\.md/\1-\2-\3T\4:\5:\6Z/')
            
            # Summarize content for better querying
            local summary=$(echo "$content" | head -c 500)
            
            # Create database entry
            cat <<EOF | ./target/release/savant-db --db-path "$DB_PATH" store --title "Audio Capture $timestamp" >/dev/null 2>&1
{
  "text": $(echo "$content" | jq -R),
  "language": "en", 
  "segments": [
    {
      "text": $(echo "$summary" | jq -R),
      "start_time": 0.0,
      "end_time": 300.0,
      "confidence": 0.85,
      "words": null
    }
  ],
  "processing_time_ms": 100,
  "model_used": "whisper",
  "session_metadata": {
    "session_id": "audio-$imported",
    "timestamp": "$timestamp", 
    "audio_source": "SystemAudio",
    "speaker": "system_audio",
    "device_info": "MacBook Pro"
  }
}
EOF
            
            echo -e "${CYAN}  📄 Imported: $(basename "$file") → $timestamp${NC}"
            imported=$((imported + 1))
            
            # Limit imports for demo
            if [ $imported -ge 10 ]; then
                break
            fi
        fi
    done
    
    echo -e "${GREEN}✅ Imported $imported audio files${NC}"
    
    # Show what we imported
    echo -e "\n${BLUE}📊 Your Audio Database:${NC}"
    ./target/release/savant-db --db-path "$DB_PATH" list --limit 5
}

# Query function
ask_question() {
    local question="$1"
    echo -e "\n${CYAN}❓ \"$question\"${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
    
    # Create MCP query
    local mcp_request=$(cat <<EOF
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_conversations",
    "arguments": {
      "query": "$question",
      "session_id": "user-query-$(date +%s)"
    }
  }
}
EOF
    )
    
    echo -e "${BLUE}🤖 Analyzing your audio data with Ollama...${NC}"
    
    # Query via MCP server
    local result=$(echo "$mcp_request" | timeout 45s ./target/release/savant-mcp-server --database "$DB_PATH" --llm-provider ollama --llm-model devstral 2>/dev/null | head -1)
    
    if [ -n "$result" ]; then
        echo -e "\n${GREEN}🎯 AI Analysis:${NC}"
        # Extract the response content
        echo "$result" | jq -r '.result.content[0].text // .result // "No analysis available"' 2>/dev/null | sed 's/^/  /'
        
        echo -e "\n${BLUE}📋 Technical Details:${NC}"
        echo "$result" | jq -r '.result' 2>/dev/null | head -10 | sed 's/^/  /'
    else
        echo -e "\n${YELLOW}⚠️  No response received. Try a different question.${NC}"
    fi
}

# Main interface
case "${1:-help}" in
    setup|--setup)
        setup_database
        ;;
    stats|--stats)
        echo -e "${BLUE}📊 Database Statistics:${NC}"
        ./target/release/savant-db --db-path "$DB_PATH" stats
        ./target/release/savant-db --db-path "$DB_PATH" list --limit 5
        ;;
    help|--help|-h)
        cat <<EOF
${CYAN}🎤 Personal Audio Query System${NC}

${BLUE}DESCRIPTION:${NC}
  Query your personal audio transcription data using natural language.
  Uses Ollama AI to understand and answer questions about your audio content.

${BLUE}SETUP:${NC}
  $0 setup                 # Import your audio files to database

${BLUE}USAGE:${NC}
  $0 "your question"       # Ask about your audio data

${BLUE}EXAMPLES:${NC}
  $0 setup
  $0 "give me a summary of my audio data from july 1st 2025"
  $0 "what topics were discussed in my recordings?"
  $0 "what was the main subject of my audio captures?"
  $0 "summarize the technical content in my audio"

${BLUE}REQUIREMENTS:${NC}
  - Ollama running: ollama serve
  - Model: ollama pull llama3.2
  - Audio files in: $AUDIO_DIR

${BLUE}DATABASE:${NC}
  Location: $DB_PATH
EOF
        ;;
    *)
        # Check if database exists
        if [ ! -f "$DB_PATH" ] || [ $(stat -f%z "$DB_PATH" 2>/dev/null || echo 0) -lt 1000 ]; then
            echo -e "${YELLOW}📊 No audio database found. Setting up first...${NC}"
            setup_database
            echo ""
        fi
        
        # Answer the question
        ask_question "$*"
        ;;
esac

echo -e "\n${BLUE}💡 Tip: Run '$0 --help' for more options${NC}"