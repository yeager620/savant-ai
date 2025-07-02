# Test Scripts

Comprehensive test suite for Savant AI components following UNIX philosophy principles.

## Test Categories

### Database Integration Tests
- **`test-database-sql.sh`** - Direct SQL database queries and CLI tool testing
- **`test-mcp-natural-queries.sh`** - MCP server natural language query integration
- **`test-chatbot-integration.sh`** - 🤖 Complete end-to-end chatbot integration with real LLM responses

## Running Tests

```bash
# Run all database tests
./scripts/tests/test-database-sql.sh

# Run MCP server tests  
./scripts/tests/test-mcp-natural-queries.sh

# Run complete chatbot integration test (requires Ollama)
./scripts/tests/test-chatbot-integration.sh

# Make scripts executable if needed
chmod +x scripts/tests/*.sh
```

## Test Philosophy

Tests follow UNIX philosophy:
- **Single Purpose**: Each test script focuses on one component
- **Composability**: Tests can be run independently or together
- **Clear Output**: Structured output with clear pass/fail indicators
- **Exit Codes**: Proper exit codes for automation integration

## Test Coverage

- ✅ Database connection and initialization
- ✅ SQL query functionality 
- ✅ MCP server JSON-RPC 2.0 protocol
- ✅ Natural language query processing
- ✅ CLI tool integration
- ✅ UNIX workflow compositions
- ✅ Error handling and edge cases
- ✅ **NEW**: End-to-end chatbot integration with real LLM responses

## Detailed Test Descriptions

### `test-chatbot-integration.sh` - Complete LLM Integration Test 🤖

This comprehensive test demonstrates the complete workflow from audio transcription data to intelligent chatbot responses:

**Prerequisites:**
```bash
# Ensure Ollama is running
ollama serve

# Ensure devstral model is available
ollama pull devstral
```

**Test Flow:**
1. **Data Population**: Creates realistic meeting transcription data (3 conversations)
   - Project planning meeting with timeline discussions
   - Daily standup with progress updates
   - Client feedback session with improvement suggestions

2. **LLM Integration**: Tests actual queries through MCP server with Ollama
   - "What meetings have we had recently?" → Meeting summaries
   - "What has Sarah been working on?" → Individual contributor analysis
   - "What's the MCP project status?" → Technical project tracking
   - "Who are most active participants?" → Speaker analytics
   - Complex productivity analysis with recommendations

3. **Response Validation**: Verifies LLM generates intelligent responses based on actual data

**Sample Output:**
```
Query: "What meetings have we had recently and what were the main topics?"

Response: "Based on your transcripts, you've had 3 recent meetings:

1. Project Alpha Planning Meeting (2 days ago)
   - Participants: John Doe, Sarah Smith, Mike Johnson  
   - Key topics: Timeline planning, resource allocation, MCP integration as Q2 priority
   
2. Daily Standup - Backend Team (yesterday)
   - Progress: Authentication module completed, database migrations done
   - Current work: API rate limiting, semantic search, MCP server testing
   
3. Client Feedback Session (today)
   - Positive: Real-time transcription working excellently
   - Improvement area: Natural language queries need to be less technical
   
The team appears focused on the MCP integration project as the top priority."
```

**What This Test Demonstrates:**
- ✅ Real LLM querying (not just protocol testing)
- ✅ Contextual understanding of conversation data
- ✅ Multi-conversation analysis and synthesis
- ✅ Speaker-specific insights and tracking
- ✅ Technical project status understanding
- ✅ Business intelligence from meeting data

This test validates that the system can serve as an intelligent assistant that understands your actual conversation history and provides meaningful insights.

## Adding New Tests

Follow the established patterns:
1. Clear test descriptions with numbered steps
2. Proper error handling and exit codes
3. Informative output with color coding
4. Test isolation (no dependencies between tests)