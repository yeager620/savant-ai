#!/bin/bash
# Interactive Natural Language Query Script for Personal Audio Data
# Allows querying your actual audio transcription data via chatbot

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🎤 Personal Audio Data Query System${NC}"
echo -e "${CYAN}===================================${NC}"

# Configuration
AUDIO_DATA_DIR="/Users/yeager/Documents/savant-ai/data/audio-captures"
DB_PATH="./data/databases/dev/personal-transcripts.db"
DB_CLI="./target/release/savant-db"
MCP_SERVER="./target/release/savant-mcp-server"

# Check dependencies
echo -e "${BLUE}Checking dependencies...${NC}"

# Check if Ollama is running
if ! curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
    echo -e "${RED}❌ Ollama is not running. Please start with: ollama serve${NC}"
    exit 1
fi

# Check if devstral model is available
if ! curl -s http://localhost:11434/api/tags | jq -r '.models[].name' | grep -q "devstral" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  devstral model not found. Trying to pull...${NC}"
    ollama pull devstral || {
        echo -e "${RED}❌ Failed to pull devstral model${NC}"
        exit 1
    }
fi

# Build required tools
echo -e "${BLUE}Building required tools...${NC}"
cargo build --package savant-db --package savant-mcp --release >/dev/null 2>&1

if [ ! -f "$DB_CLI" ] || [ ! -f "$MCP_SERVER" ]; then
    echo -e "${RED}❌ Required binaries not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Dependencies ready${NC}"

# Function to import audio data from markdown files
import_audio_data() {
    echo -e "\n${BLUE}📥 Importing your audio transcription data...${NC}"
    
    if [ ! -d "$AUDIO_DATA_DIR" ]; then
        echo -e "${RED}❌ Audio data directory not found: $AUDIO_DATA_DIR${NC}"
        exit 1
    fi
    
    # Check current database state
    EXISTING_COUNT=$($DB_CLI --db-path "$DB_PATH" list --limit 1 2>/dev/null | grep -c "id:" 2>/dev/null || echo "0")
    
    if [ "$EXISTING_COUNT" -gt 0 ]; then
        echo -e "${YELLOW}📊 Database already contains $EXISTING_COUNT conversation(s)${NC}"
        echo -e "${BLUE}Do you want to reimport data? (y/N):${NC}"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            echo -e "${GREEN}✅ Using existing database${NC}"
            return
        fi
        echo -e "${YELLOW}🗑️  Clearing existing data...${NC}"
        rm -f "$DB_PATH"
    fi
    
    local imported_count=0
    local meaningful_count=0
    
    echo -e "${BLUE}Processing audio files from July 1st, 2025...${NC}"
    
    # Process each .md file in the audio captures directory
    for file in "$AUDIO_DATA_DIR"/system_audio_20250701_*.md; do
        if [ -f "$file" ]; then
            local filename=$(basename "$file")
            local timestamp=$(echo "$filename" | sed -E 's/system_audio_([0-9]{8})_([0-9]{6})\.md/\1T\2/')
            local formatted_time=$(echo "$timestamp" | sed -E 's/([0-9]{4})([0-9]{2})([0-9]{2})T([0-9]{2})([0-9]{2})([0-9]{2})/\1-\2-\3T\4:\5:\6Z/')
            
            # Check if file contains meaningful content by looking at the main text field
            local main_text=$(jq -r '.text // ""' "$file" 2>/dev/null)
            local has_meaningful_content=false
            
            # Check if main text contains actual speech (not just unclear audio or silence)
            if [[ "$main_text" != *"[unclear audio]"* ]] && [[ "$main_text" != *"[ Silence ]"* ]] && [ ${#main_text} -gt 50 ]; then
                has_meaningful_content=true
            elif [[ "$main_text" == *"[unclear audio]"* ]] && [ ${#main_text} -gt 100 ]; then
                # Even if it starts with unclear audio, if there's substantial content, import it
                has_meaningful_content=true
            fi
            
            if [ "$has_meaningful_content" = true ]; then
                echo -e "${CYAN}  📄 Importing: $filename (${formatted_time})${NC}"
                
                # Create a proper session metadata wrapper for the JSON content
                local session_id="audio-session-$(date +%s)-$imported_count"
                
                # Read the JSON content and add session metadata
                jq --arg session_id "$session_id" \
                   --arg timestamp "$formatted_time" \
                   --arg speaker "system_audio" \
                   '. + {
                     "session_metadata": {
                       "session_id": $session_id,
                       "timestamp": $timestamp,
                       "audio_source": "SystemAudio", 
                       "speaker": $speaker,
                       "device_info": "MacBook Pro"
                     }
                   }' "$file" | \
                $DB_CLI --db-path "$DB_PATH" store --title "System Audio Capture - $formatted_time" 2>/dev/null
                
                meaningful_count=$((meaningful_count + 1))
            else
                echo -e "${YELLOW}  ⏭️  Skipping: $filename (no meaningful audio content)${NC}"
            fi
            
            imported_count=$((imported_count + 1))
        fi
    done
    
    echo -e "\n${GREEN}✅ Import complete!${NC}"
    echo -e "${BLUE}📊 Processed: $imported_count files${NC}"
    echo -e "${BLUE}📈 Meaningful content: $meaningful_count conversations${NC}"
    
    # Show database statistics
    echo -e "\n${BLUE}📋 Database Summary:${NC}"
    $DB_CLI --db-path "$DB_PATH" stats 2>/dev/null || echo "Database statistics not available"
}

# Function to query the database using natural language
query_data() {
    local query="$1"
    
    if [ -z "$query" ]; then
        echo -e "${RED}❌ No query provided${NC}"
        return 1
    fi
    
    echo -e "\n${CYAN}🤖 Processing your query: \"$query\"${NC}"
    echo -e "${BLUE}===========================================${NC}"
    
    # Create JSON-RPC request for MCP server
    local mcp_request=$(cat <<EOF
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_conversations",
    "arguments": {
      "query": "$query",
      "session_id": "personal-query-$(date +%s)"
    }
  }
}
EOF
    )
    
    echo -e "${BLUE}🔍 Searching your audio data...${NC}"
    
    # Query via MCP server with Ollama
    local response=$(echo "$mcp_request" | $MCP_SERVER --database "$DB_PATH" --llm-provider ollama --llm-model devstral 2>/dev/null)
    
    # Extract and display the response content
    echo -e "\n${GREEN}🎯 Analysis Results:${NC}"
    echo -e "${CYAN}================${NC}"
    
    echo "$response" | jq -r '.result.content[0].text // .result.response // .result // "No response available"' 2>/dev/null | \
    sed 's/^/  /' | \
    sed "s/\[unclear audio\]/$(echo -e "${YELLOW}[unclear audio]${NC}")/g" | \
    sed "s/\[no signal\]/$(echo -e "${YELLOW}[no signal]${NC}")/g"
    
    echo -e "\n${BLUE}💡 Technical Details:${NC}"
    echo "$response" | jq -r '.result // empty' 2>/dev/null | head -5 | sed 's/^/  /'
}

# Function to show help
show_help() {
    cat <<EOF
${CYAN}🎤 Personal Audio Data Query System${NC}

${BLUE}DESCRIPTION:${NC}
  Query your personal audio transcription data using natural language.
  The system imports your existing .md audio files and allows you to ask
  questions about them using an AI chatbot (via Ollama).

${BLUE}USAGE:${NC}
  $0 [options] [query]

${BLUE}OPTIONS:${NC}
  --import, -i     Import/reimport audio data from markdown files
  --interactive    Start interactive query mode  
  --stats          Show database statistics
  --help, -h       Show this help message

${BLUE}EXAMPLES:${NC}
  # Import your audio data
  $0 --import
  
  # Query your data
  $0 "give me a summary of my audio data from july 1st 2025"
  $0 "what was the longest conversation today?"
  $0 "show me any meaningful conversations from this morning"
  
  # Interactive mode
  $0 --interactive

${BLUE}DATA LOCATION:${NC}
  Audio files: $AUDIO_DATA_DIR
  Database: $DB_PATH

${BLUE}REQUIREMENTS:${NC}
  - Ollama running (ollama serve)
  - llama3.2 model (ollama pull llama3.2)
EOF
}

# Function for interactive mode
interactive_mode() {
    echo -e "\n${CYAN}🔄 Interactive Query Mode${NC}"
    echo -e "${BLUE}Type 'exit' or 'quit' to stop${NC}\n"
    
    while true; do
        echo -e -n "${CYAN}❓ Your question: ${NC}"
        read -r user_query
        
        case "$user_query" in
            "exit"|"quit"|"q")
                echo -e "${GREEN}👋 Goodbye!${NC}"
                break
                ;;
            "")
                echo -e "${YELLOW}Please enter a query${NC}"
                continue
                ;;
            *)
                query_data "$user_query"
                echo -e "\n${BLUE}─────────────────────────────────────────${NC}"
                ;;
        esac
    done
}

# Parse command line arguments
case "${1:-}" in
    --import|-i)
        import_audio_data
        ;;
    --interactive)
        # Check if data exists first
        EXISTING_COUNT=$($DB_CLI --db-path "$DB_PATH" list --limit 1 2>/dev/null | grep -c "id:" 2>/dev/null || echo "0")
        if [ "$EXISTING_COUNT" -eq 0 ]; then
            echo -e "${YELLOW}⚠️  No data found in database. Importing first...${NC}"
            import_audio_data
        fi
        interactive_mode
        ;;
    --stats)
        echo -e "${BLUE}📊 Database Statistics:${NC}"
        $DB_CLI --db-path "$DB_PATH" stats 2>/dev/null || echo "No data available"
        $DB_CLI --db-path "$DB_PATH" list --limit 10 2>/dev/null || echo "No conversations available"
        ;;
    --help|-h|"")
        show_help
        ;;
    *)
        # Direct query mode
        EXISTING_COUNT=$($DB_CLI --db-path "$DB_PATH" list --limit 1 2>/dev/null | grep -c "id:" 2>/dev/null || echo "0")
        if [ "$EXISTING_COUNT" -eq 0 ]; then
            echo -e "${YELLOW}⚠️  No data found in database. Importing first...${NC}"
            import_audio_data
        fi
        query_data "$*"
        ;;
esac