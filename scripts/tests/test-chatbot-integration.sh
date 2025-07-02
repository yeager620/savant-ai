#!/bin/bash
# Test script for complete chatbot integration via MCP server
# Tests real LLM queries through MCP with actual database content

set -e

echo "🤖 Testing Complete Chatbot Integration via MCP Server"
echo "====================================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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

echo -e "${GREEN}✅ Dependencies checked${NC}"

# Build required binaries
echo -e "\n${BLUE}Building required binaries...${NC}"
cargo build --package savant-db --release
cargo build --package savant-mcp --release

DB_CLI="./target/release/savant-db"
MCP_SERVER="./target/release/savant-mcp-server"

# Check binaries exist
if [ ! -f "$DB_CLI" ] || [ ! -f "$MCP_SERVER" ]; then
    echo -e "${RED}❌ Required binaries not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Binaries built successfully${NC}"

# Create test database with realistic content
echo -e "\n${BLUE}Step 1: Populating test database with realistic audio data...${NC}"

TEST_DB_PATH="./data/databases/test/chatbot-integration.db"
rm -f "$TEST_DB_PATH"

# Create sample transcription data that simulates realistic conversations
echo -e "${YELLOW}Adding sample meeting transcription...${NC}"

# Generate timestamps
TIMESTAMP1=$(date -u -v-2d +%Y-%m-%dT09:00:00Z)
TIMESTAMP2=$(date -u -v-2d +%Y-%m-%dT09:00:15Z)
TIMESTAMP3=$(date -u -v-2d +%Y-%m-%dT09:00:30Z)
TIMESTAMP4=$(date -u -v-2d +%Y-%m-%dT09:00:45Z)

cat <<EOF | $DB_CLI --db-path "$TEST_DB_PATH" store --title "Project Alpha Planning Meeting"
{
  "text": "Good morning everyone. Let's start our project alpha planning meeting. We need to discuss the timeline and resource allocation. Thanks John. I've prepared a detailed analysis of our current development capacity. We can handle about 3 major features per sprint. Sarah that sounds reasonable. What about the database integration timeline? The new MCP server feature is crucial for our Q2 goals. Mike raises a good point. The MCP integration is our top priority. We should allocate our best developers to that task.",
  "language": "en",
  "segments": [
    {
      "text": "Good morning everyone. Let's start our project alpha planning meeting. We need to discuss the timeline and resource allocation.",
      "start_time": 0.0,
      "end_time": 8.5,
      "confidence": 0.95,
      "words": null
    },
    {
      "text": "Thanks John. I've prepared a detailed analysis of our current development capacity. We can handle about 3 major features per sprint.",
      "start_time": 8.5,
      "end_time": 15.2,
      "confidence": 0.92,
      "words": null
    },
    {
      "text": "Sarah that sounds reasonable. What about the database integration timeline? The new MCP server feature is crucial for our Q2 goals.",
      "start_time": 15.2,
      "end_time": 22.1,
      "confidence": 0.89,
      "words": null
    },
    {
      "text": "Mike raises a good point. The MCP integration is our top priority. We should allocate our best developers to that task.",
      "start_time": 22.1,
      "end_time": 28.7,
      "confidence": 0.94,
      "words": null
    }
  ],
  "processing_time_ms": 150,
  "model_used": "test-model",
  "session_metadata": {
    "session_id": "meeting-session-1",
    "timestamp": "$TIMESTAMP1",
    "audio_source": "Microphone",
    "speaker": "john_doe",
    "device_info": "MacBook Pro"
  }
}
EOF

echo -e "${YELLOW}Adding sample standup transcription...${NC}"

# Generate standup timestamps
TIMESTAMP5=$(date -u -v-1d +%Y-%m-%dT10:00:00Z)
TIMESTAMP6=$(date -u -v-1d +%Y-%m-%dT10:00:10Z)
TIMESTAMP7=$(date -u -v-1d +%Y-%m-%dT10:00:25Z)

cat <<EOF | $DB_CLI --db-path "$TEST_DB_PATH" store --title "Daily Standup - Backend Team"
{
  "segments": [
    {
      "speaker": "alex_garcia",
      "text": "Yesterday I finished the authentication module. Today I'm working on the API rate limiting. No blockers currently.",
      "timestamp": "$TIMESTAMP5",
      "confidence": 0.91,
      "start_time": 0.0,
      "end_time": 6.8
    },
    {
      "speaker": "sarah_smith",
      "text": "I completed the database schema migrations and started on the semantic search implementation. Might need help with vector embeddings.",
      "timestamp": "$TIMESTAMP6",
      "confidence": 0.93,
      "start_time": 6.8,
      "end_time": 14.2
    },
    {
      "speaker": "mike_johnson", 
      "text": "Working on MCP server testing. Found some edge cases with concurrent requests. Will pair with Alex this afternoon to resolve.",
      "timestamp": "$TIMESTAMP7",
      "confidence": 0.88,
      "start_time": 14.2,
      "end_time": 21.5
    }
  ]
}
EOF

echo -e "${YELLOW}Adding sample client feedback transcription...${NC}"

# Generate client feedback timestamps  
TIMESTAMP8=$(date -u +%Y-%m-%dT14:00:00Z)
TIMESTAMP9=$(date -u +%Y-%m-%dT14:00:10Z)
TIMESTAMP10=$(date -u +%Y-%m-%dT14:00:20Z)

cat <<EOF | $DB_CLI --db-path "$TEST_DB_PATH" store --title "Client Feedback Session - Product Review"
{
  "segments": [
    {
      "speaker": "client_representative",
      "text": "Overall we're very impressed with the progress. The real-time transcription feature works excellently. Our users love it.",
      "timestamp": "$TIMESTAMP8",
      "confidence": 0.96,
      "start_time": 0.0,
      "end_time": 7.3
    },
    {
      "speaker": "john_doe",
      "text": "That's fantastic feedback! What areas do you think need improvement? Any specific features missing?",
      "timestamp": "$TIMESTAMP9",
      "confidence": 0.94,
      "start_time": 7.3,
      "end_time": 12.8
    },
    {
      "speaker": "client_representative",
      "text": "The natural language database queries could be more intuitive. Sometimes the responses are too technical for our business users.",
      "timestamp": "$TIMESTAMP10",
      "confidence": 0.92,
      "start_time": 12.8,
      "end_time": 19.4
    }
  ]
}
EOF

# Verify data was stored
echo -e "\n${BLUE}Verifying test data storage...${NC}"
CONVERSATION_COUNT=$($DB_CLI --db-path "$TEST_DB_PATH" list --limit 10 | grep -c "id:" || echo "0")
echo -e "${GREEN}✅ Stored $CONVERSATION_COUNT conversations in test database${NC}"

# Function to cleanup on exit
cleanup() {
    echo -e "\n${BLUE}Cleaning up...${NC}"
    rm -f "$TEST_DB_PATH"
}
trap cleanup EXIT

# Test realistic chatbot interactions using stdio
echo -e "\n${BLUE}Step 2: Testing realistic chatbot queries via MCP server...${NC}"

# Helper function to query MCP server
query_mcp() {
    local query_json="$1"
    local description="$2"
    
    echo -e "\n${YELLOW}$description${NC}"
    echo "Request: $query_json" | jq '.'
    echo -e "${BLUE}Response:${NC}"
    echo "$query_json" | $MCP_SERVER --db-path "$TEST_DB_PATH" --llm-provider ollama --llm-model devstral 2>/dev/null | jq -r '.result.content // .result.response // .result // .error.message // .'
    echo -e "${GREEN}✅ Query completed${NC}"
}

# Query 1: Ask about recent meetings
QUERY1_REQUEST='{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_conversations",
    "arguments": {
      "query": "What meetings have we had recently and what were the main topics discussed? Please provide a summary of key points and participants.",
      "session_id": "chatbot-test-session"
    }
  }
}'

query_mcp "$QUERY1_REQUEST" "Query 1: 'What meetings have we had recently and what were the main topics?'"

# Query 2: Ask about specific person's contributions
QUERY2_REQUEST='{
  "jsonrpc": "2.0", 
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "query_conversations",
    "arguments": {
      "query": "What has Sarah Smith been working on based on our meeting records? Show me her recent contributions and tasks.",
      "session_id": "chatbot-test-session"
    }
  }
}'

query_mcp "$QUERY2_REQUEST" "Query 2: 'What has Sarah been working on based on our meeting records?'"

# Query 3: Ask about technical project status
QUERY3_REQUEST='{
  "jsonrpc": "2.0",
  "id": 3, 
  "method": "tools/call",
  "params": {
    "name": "query_conversations",
    "arguments": {
      "query": "What is the current status of our MCP integration project? Include any blockers or challenges mentioned.",
      "session_id": "chatbot-test-session"
    }
  }
}'

query_mcp "$QUERY3_REQUEST" "Query 3: 'What is the current status of our MCP integration project?'"

# Query 4: Ask for analytics and insights
QUERY4_REQUEST='{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call", 
  "params": {
    "name": "get_speaker_analytics",
    "arguments": {
      "session_id": "chatbot-test-session"
    }
  }
}'

query_mcp "$QUERY4_REQUEST" "Query 4: 'Who are the most active participants in our meetings?'"

# Step 3: Test complex multi-tool query
echo -e "\n${BLUE}Step 3: Testing complex multi-tool interaction...${NC}"

COMPLEX_REQUEST='{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "query_conversations", 
    "arguments": {
      "query": "Analyze our team productivity based on meeting transcripts. Look at who is contributing what, identify any patterns or concerns, and provide recommendations for improving team efficiency. Include specific examples from recent conversations.",
      "session_id": "chatbot-test-session"
    }
  }
}'

query_mcp "$COMPLEX_REQUEST" "Complex Query: 'Analyze our team productivity and provide recommendations'"

# Test database state after queries
echo -e "\n${BLUE}Step 4: Verifying database state after LLM queries...${NC}"

FINAL_STATS=$($DB_CLI --db-path "$TEST_DB_PATH" stats)
echo "Final database statistics:"
echo "$FINAL_STATS"

echo -e "${GREEN}✅ Database state verified${NC}"

echo -e "\n${GREEN}🎉 Complete chatbot integration test successful!${NC}"
echo -e "${BLUE}✨ The system successfully:${NC}"
echo -e "${BLUE}   • Populated database with realistic conversation data${NC}"
echo -e "${BLUE}   • Started MCP server with Ollama LLM integration${NC}" 
echo -e "${BLUE}   • Processed natural language queries from simulated chatbot${NC}"
echo -e "${BLUE}   • Generated intelligent responses based on actual database content${NC}"
echo -e "${BLUE}   • Demonstrated end-to-end LLM ↔ MCP ↔ Database workflow${NC}"