# MCP Client Setup: Connecting Ollama Chatbot to Savant AI Database (Improved Architecture)

This guide explains how to configure an Ollama chatbot as an MCP (Model Context Protocol) client to access the Savant AI database through the enhanced MCP server with LLM-powered query processing.

## Overview

The improved setup involves:
1. **Enhanced MCP Server**: Savant AI's database server with LLM-powered query processing
2. **MCP Client**: Ollama chatbot configured with MCP support
3. **Connection**: Abstracted transport layer (stdio primary, extensible to HTTP/WebSocket)
4. **Context Management**: Session-based conversation context for follow-up queries
5. **Security**: Enhanced validation with timing attack prevention and rate limiting

## Prerequisites

- Ollama installed and running locally
- Savant AI project with MCP server built
- Node.js/Python environment for MCP client implementation

## Server Setup

### 1. Build and Start MCP Server

```bash
# Build the MCP server
cargo build --package savant-mcp --release

# Start the server (stdio mode)
./target/release/savant-mcp --log-level info
```

The enhanced server provides these tools:
- `query_conversations`: LLM-powered natural language database queries with context awareness
- `get_speaker_analytics`: Advanced speaker statistics and interaction patterns
- `search_semantic`: Optimized full-text search with pre-computed indexes
- `get_conversation_context`: Retrieve conversation details with relationship analysis
- `list_speakers`: Get all speakers with interaction matrices
- `export_conversation`: Export conversation data (security-limited)
- `learn_from_feedback`: Query optimization based on user feedback
- `get_query_suggestions`: Smart query suggestions based on successful patterns

### 2. Test Server Connectivity

```bash
# Test with sample request
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/savant-mcp
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {"name": "query_conversations", "description": "Query conversations using natural language"},
      {"name": "get_speaker_analytics", "description": "Get analytics for specific speakers"}
    ]
  }
}
```

## Client Setup Options

### Option A: Use Existing MCP Client Framework

#### Claude Desktop Integration
Add to `~/.config/claude-desktop/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "savant-database": {
      "command": "/path/to/savant-ai/target/release/savant-mcp",
      "args": ["--log-level", "info"]
    }
  }
}
```

#### Continue.dev Integration
Add to `.continue/config.json`:

```json
{
  "models": [...],
  "mcpServers": [
    {
      "name": "savant-database",
      "command": "/path/to/savant-ai/target/release/savant-mcp"
    }
  ]
}
```

### Option B: Custom Ollama MCP Client

#### Python Implementation

Create `ollama_mcp_client.py`:

```python
import json
import subprocess
import asyncio
import uuid
from typing import Dict, Any

class OllamaMCPClient:
    def __init__(self, server_command: str):
        self.server_command = server_command
        self.server_process = None
        self.session_id = str(uuid.uuid4())  # For context management
    
    async def start_server(self):
        self.server_process = await asyncio.create_subprocess_exec(
            *self.server_command.split(),
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )
    
    async def send_request(self, method: str, params: Dict[str, Any]) -> Dict[str, Any]:
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        }
        
        self.server_process.stdin.write(json.dumps(request).encode() + b'\n')
        await self.server_process.stdin.drain()
        
        response_line = await self.server_process.stdout.readline()
        return json.loads(response_line.decode())
    
    async def query_database(self, natural_query: str, session_id: str = None) -> str:
        if session_id is None:
            session_id = self.session_id
            
        response = await self.send_request("tools/call", {
            "name": "query_conversations",
            "arguments": {
                "query": natural_query,
                "session_id": session_id  # For context management
            }
        })
        
        if "result" in response:
            return response["result"]["content"][0]["text"]
        return "Query failed"
    
    async def provide_feedback(self, query: str, feedback: str) -> str:
        """Provide feedback to improve query learning"""
        response = await self.send_request("tools/call", {
            "name": "learn_from_feedback",
            "arguments": {
                "query": query,
                "feedback": feedback
            }
        })
        
        if "result" in response:
            return response["result"]["content"][0]["text"]
        return "Feedback recorded"
    
    async def get_suggestions(self, partial_query: str) -> list:
        """Get query suggestions based on successful patterns"""
        response = await self.send_request("tools/call", {
            "name": "get_query_suggestions",
            "arguments": {"query": partial_query}
        })
        
        if "result" in response:
            return response["result"]["suggestions"]
        return []

# Enhanced Usage with Context and Feedback
async def main():
    import uuid
    import requests
    
    client = OllamaMCPClient("/path/to/savant-ai/target/release/savant-mcp")
    await client.start_server()
    
    session_id = str(uuid.uuid4())
    
    # Initial query
    result = await client.query_database("Show me conversations from yesterday", session_id)
    print(f"Database result: {result}")
    
    # Send to Ollama with enhanced context
    ollama_response = requests.post("http://localhost:11434/api/generate", json={
        "model": "llama3.2",
        "prompt": f"""You are an AI assistant with access to conversation transcript data.
        
        User Query: "Show me conversations from yesterday"
        Database Results: {result}
        
        Provide a helpful, conversational summary of the results. If there are multiple conversations, 
        highlight key participants and topics. If no results, suggest alternative time ranges.
        
        Response:""",
        "stream": False
    })
    
    ollama_answer = ollama_response.json()['response']
    print(f"Ollama response: {ollama_answer}")
    
    # Follow-up query using context
    followup = await client.query_database("Who participated in the longest one?", session_id)
    print(f"Follow-up result: {followup}")
    
    # Provide feedback to improve future queries
    await client.provide_feedback("Show me conversations from yesterday", "Good")

asyncio.run(main())
```

#### Node.js Implementation

Create `ollama-mcp-client.js`:

```javascript
const { spawn } = require('child_process');
const readline = require('readline');
const axios = require('axios');

class OllamaMCPClient {
    constructor(serverCommand) {
        this.serverCommand = serverCommand;
        this.serverProcess = null;
    }
    
    async startServer() {
        this.serverProcess = spawn(this.serverCommand, ['--log-level', 'info']);
        this.rl = readline.createInterface({
            input: this.serverProcess.stdout,
            output: process.stdout,
            terminal: false
        });
    }
    
    async sendRequest(method, params) {
        return new Promise((resolve) => {
            const request = {
                jsonrpc: "2.0",
                id: 1,
                method,
                params
            };
            
            this.serverProcess.stdin.write(JSON.stringify(request) + '\n');
            
            this.rl.once('line', (line) => {
                resolve(JSON.parse(line));
            });
        });
    }
    
    async queryDatabase(naturalQuery) {
        const response = await this.sendRequest("tools/call", {
            name: "query_conversations",
            arguments: { query: naturalQuery }
        });
        
        return response.result?.content?.[0]?.text || "Query failed";
    }
    
    async askOllama(prompt, model = "llama3.2") {
        const response = await axios.post("http://localhost:11434/api/generate", {
            model,
            prompt,
            stream: false
        });
        
        return response.data.response;
    }
}

// Usage
async function main() {
    const client = new OllamaMCPClient("/path/to/savant-ai/target/release/savant-mcp");
    await client.startServer();
    
    // Interactive chat loop
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });
    
    console.log("Ollama MCP Client ready. Ask questions about your transcripts!");
    
    rl.on('line', async (userInput) => {
        try {
            // Query database first
            const dbResult = await client.queryDatabase(userInput);
            
            // Send combined context to Ollama
            const prompt = `User question: "${userInput}"
Database result: ${dbResult}

Provide a natural language answer based on the database results:`;
            
            const answer = await client.askOllama(prompt);
            console.log(`\nAnswer: ${answer}\n`);
        } catch (error) {
            console.error("Error:", error.message);
        }
    });
}

main().catch(console.error);
```

## Usage Workflow

### 1. Start Both Services

```bash
# Terminal 1: Start Ollama
ollama serve

# Terminal 2: Verify Ollama models
ollama list

# Terminal 3: Start MCP client
python ollama_mcp_client.py
# or
node ollama-mcp-client.js
```

### 2. Enhanced Example Interactions

**Initial Query**: "Show me conversations where John talked about the project meeting"

**Enhanced Flow**:
1. Client sends natural language query to enhanced MCP server with session ID
2. MCP server uses LLM-powered query processing to understand intent
3. Security layer validates query and estimates complexity
4. Query processor converts to optimized SQL with context awareness
5. Database returns results using pre-computed indexes
6. Client sends results + context to Ollama for natural language response
7. User receives conversational answer
8. Context manager stores query for follow-up questions

**Sample Enhanced Output**:
```
User: Show me conversations where John talked about the project meeting
MCP Server: [LLM Processing] Intent: find_conversations, Entity: speaker=john, topic=project meeting
Database: Found 3 conversations with John mentioning "project meeting" (confidence: 0.92)
Ollama: I found 3 conversations where John discussed project meetings:

1. **June 28, 2025** - 15-minute discussion about Q3 timeline delays
2. **June 29, 2025** - 8-minute resource allocation review  
3. **June 30, 2025** - 22-minute status update with action items

Would you like me to show you specific details from any of these conversations, or search for related topics?

User: Tell me more about the status update
MCP Server: [Context Aware] Using previous result context, focusing on June 30 conversation
Database: Retrieved detailed segments from conversation ID matching June 30
Ollama: The June 30 status update was the longest discussion (22 minutes). John covered:

• **Timeline Updates**: Moved Phase 2 deadline from July 15 to July 22
• **Resource Changes**: Added two developers to the frontend team
• **Action Items**: 
  - Sarah to finalize API documentation by July 5
  - Mike to review security requirements by July 8
  - Next meeting scheduled for July 7

The conversation had high confidence scores (avg 0.89) and included 3 other participants.

User: That's helpful!
[System automatically records positive feedback for query optimization]
```

## Enhanced Configuration Options

### MCP Server Configuration

Environment variables for the enhanced server:
```bash
# Database path
export SAVANT_DB_PATH=/path/to/transcript.db

# Log level
export RUST_LOG=info

# Enhanced security settings
export SAVANT_MAX_RESULTS=1000
export SAVANT_QUERY_TIMEOUT=30
export SAVANT_RATE_LIMIT_PER_MINUTE=60
export SAVANT_MAX_COMPLEXITY_PER_MINUTE=100

# LLM integration (optional)
export SAVANT_LLM_PROVIDER=ollama  # ollama, openai, anthropic
export SAVANT_LLM_ENDPOINT=http://localhost:11434
export SAVANT_LLM_MODEL=llama3.2

# Context management
export SAVANT_CONTEXT_RETENTION_HOURS=24
export SAVANT_MAX_CONTEXT_QUERIES=10
```

### Ollama Model Selection

Choose appropriate models based on use case:
- **llama3.2:3b** - Fast responses, basic reasoning
- **llama3.2:8b** - Better reasoning, moderate speed
- **mistral:7b** - Good balance of speed and quality
- **codellama:13b** - Better for technical queries

### Advanced Features

#### Tool Chaining
```python
# Chain multiple MCP tools
async def complex_query(client, speaker_name):
    # Get speaker analytics
    analytics = await client.send_request("tools/call", {
        "name": "get_speaker_analytics",
        "arguments": {"speaker": speaker_name}
    })
    
    # Search their recent conversations
    conversations = await client.send_request("tools/call", {
        "name": "query_conversations", 
        "arguments": {"query": f"recent conversations by {speaker_name}"}
    })
    
    # Combine results for Ollama
    context = f"Analytics: {analytics}\nConversations: {conversations}"
    return await client.ask_ollama(f"Summarize {speaker_name}'s recent activity: {context}")
```

#### Custom Prompt Templates
```python
def create_database_prompt(user_query, db_results):
    return f"""You are an AI assistant with access to transcript database results.

User Question: {user_query}

Database Results: {db_results}

Instructions:
- Provide a conversational, helpful response
- Reference specific details from the database when relevant
- If no results found, suggest alternative queries
- Keep responses concise but informative

Response:"""
```

## Troubleshooting

### Common Issues

1. **Server Connection Failed**
   ```bash
   # Check if server binary exists
   ls -la target/release/savant-mcp
   
   # Test server manually
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/release/savant-mcp
   ```

2. **Ollama Not Responding**
   ```bash
   # Check Ollama status
   curl http://localhost:11434/api/tags
   
   # Restart Ollama
   pkill ollama && ollama serve
   ```

3. **Database Access Errors**
   ```bash
   # Check database file permissions
   ls -la ~/.local/share/savant-ai/transcript.db
   
   # Verify database schema
   sqlite3 ~/.local/share/savant-ai/transcript.db ".schema"
   ```

### Debug Mode

Enable detailed logging:
```bash
# Server debug
RUST_LOG=debug ./target/release/savant-mcp

# Client debug (Python)
import logging
logging.basicConfig(level=logging.DEBUG)
```

## Security Considerations

- MCP server runs with database read-only access
- Query validation prevents SQL injection
- Rate limiting on database queries
- Local-only connections (no network exposure)
- User data remains on local machine

## Performance Tips

- Use appropriate Ollama model size for your hardware
- Cache frequent database queries
- Implement connection pooling for high-frequency usage
- Monitor memory usage with large result sets