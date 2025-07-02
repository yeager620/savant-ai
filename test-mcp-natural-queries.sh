#!/bin/bash
# Test script for natural language querying via MCP server
# Tests the LLM-powered query processing and MCP protocol compliance

set -e

echo "🧪 Testing Natural Language Database Queries via MCP Server"
echo "=========================================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Build the MCP server
echo -e "${BLUE}Building savant-mcp server...${NC}"
cargo build --package savant-mcp --release

MCP_SERVER="./target/release/savant-mcp"

# Check if server binary exists
if [ ! -f "$MCP_SERVER" ]; then
    echo -e "${RED}Error: MCP server binary not found at $MCP_SERVER${NC}"
    echo "Run: cargo build --package savant-mcp --release"
    exit 1
fi

# Test MCP server initialization
echo -e "\n${BLUE}Test 1: MCP Server Initialization${NC}"
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
echo "$INIT_REQUEST" | timeout 10s $MCP_SERVER --test 2>/dev/null | jq '.' || {
    echo -e "${RED}❌ Initialization test failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Initialization successful${NC}"

# Test tools listing
echo -e "\n${BLUE}Test 2: List Available Tools${NC}"
TOOLS_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
echo "$TOOLS_REQUEST" | timeout 10s $MCP_SERVER --test 2>/dev/null | jq '.result.tools[]?.name' || {
    echo -e "${RED}❌ Tools listing failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Tools listing successful${NC}"

# Test natural language queries
echo -e "\n${BLUE}Test 3: Natural Language Conversation Query${NC}"
QUERY_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query_conversations","arguments":{"query":"Find conversations about meetings from last week","session_id":"test-session-123"}}}'
echo "$QUERY_REQUEST" | timeout 15s $MCP_SERVER --test 2>/dev/null | jq '.result' || {
    echo -e "${RED}❌ Natural language query failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Natural language query successful${NC}"

# Test speaker analytics
echo -e "\n${BLUE}Test 4: Speaker Analytics Query${NC}"
ANALYTICS_REQUEST='{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_speaker_analytics","arguments":{"speaker":"john"}}}'
echo "$ANALYTICS_REQUEST" | timeout 15s $MCP_SERVER --test 2>/dev/null | jq '.result' || {
    echo -e "${RED}❌ Speaker analytics failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Speaker analytics successful${NC}"

# Test semantic search
echo -e "\n${BLUE}Test 5: Semantic Content Search${NC}"
SEARCH_REQUEST='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"search_semantic","arguments":{"query":"project planning discussion","limit":10}}}'
echo "$SEARCH_REQUEST" | timeout 15s $MCP_SERVER --test 2>/dev/null | jq '.result' || {
    echo -e "${RED}❌ Semantic search failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Semantic search successful${NC}"

# Test database statistics
echo -e "\n${BLUE}Test 6: Database Statistics${NC}"
STATS_REQUEST='{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"get_database_stats","arguments":{"include_performance":true}}}'
echo "$STATS_REQUEST" | timeout 10s $MCP_SERVER --test 2>/dev/null | jq '.result' || {
    echo -e "${RED}❌ Database stats failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Database statistics successful${NC}"

# Test resources listing
echo -e "\n${BLUE}Test 7: List Resources${NC}"
RESOURCES_REQUEST='{"jsonrpc":"2.0","id":7,"method":"resources/list"}'
echo "$RESOURCES_REQUEST" | timeout 10s $MCP_SERVER --test 2>/dev/null | jq '.result.resources[]?.uri' || {
    echo -e "${RED}❌ Resources listing failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Resources listing successful${NC}"

# Test prompts
echo -e "\n${BLUE}Test 8: List Prompts${NC}"
PROMPTS_REQUEST='{"jsonrpc":"2.0","id":8,"method":"prompts/list"}'
echo "$PROMPTS_REQUEST" | timeout 10s $MCP_SERVER --test 2>/dev/null | jq '.result.prompts[]?.name' || {
    echo -e "${RED}❌ Prompts listing failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Prompts listing successful${NC}"

echo -e "\n${GREEN}🎉 All MCP natural language query tests passed!${NC}"
echo -e "${BLUE}MCP server is ready for LLM integration${NC}"