# Savant AI - UNIX Philosophy Components

This directory contains the modular, UNIX-philosophy components of Savant AI. Each crate is designed to "do one thing well" and can be used standalone or composed with other tools.

## **Available Components**

### **savant-core** - Shared Types and Utilities
Common data structures and utilities used across all Savant AI components.

**Purpose**: Provide consistent data types for inter-component communication
**Dependencies**: Minimal (serde, chrono, uuid)
**Usage**: Library only - not a standalone tool

```rust
use savant_core::{ChatMessage, LlmRequest, LlmResponse};
```

### **savant-llm** - LLM Inference CLI Tool
Standalone command-line tool for Large Language Model inference.

**Purpose**: Handle AI model communication (Ollama, OpenAI, DeepSeek, Anthropic)  
**Input**: JSON or plain text via stdin/args
**Output**: JSON responses to stdout

```bash
# Install
cargo install --path crates/savant-llm

# Basic usage
echo "What is Rust?" | savant-llm
savant-llm --prompt "Hello" --model devstral

# With other tools  
echo "Explain UNIX" | savant-llm | jq -r '.content' | wc -w
```

**CLI Reference:**
```
savant-llm [OPTIONS] [COMMAND]

Commands:
  query   Process a single prompt
  batch   Process batch of prompts from stdin (one per line)
  models  List available models
  test    Test connection to provider

Options:
  -p, --prompt <PROMPT>        Prompt text (alternative to stdin)
  -m, --model <MODEL>          Model name [default: devstral]
  -t, --temperature <TEMP>     Generation temperature [default: 0.7]
      --max-tokens <TOKENS>    Maximum tokens [default: 4096]
  -s, --stream                 Enable streaming output
  -P, --provider <PROVIDER>    Provider [default: ollama]
      --auth <AUTH>            API key or URL
```

## **Planned Components** 🔄

### **savant-browser** - Browser Content Extraction
Extract and analyze content from browser tabs.

```bash
# Future usage:
savant-browser --scan-chrome              # List all Chrome tabs
savant-browser --get-active-tab          # Get current tab content  
savant-browser --detect-questions        # Find questions in content
savant-browser --monitor                 # Watch for tab changes
```

### **savant-chat** - Chat History Management
Manage conversation history with persistent storage.

```bash
# Future usage:
echo '{"user":"Hello","assistant":"Hi!"}' | savant-chat --save
savant-chat --load                       # Load conversation history
savant-chat --export --format json      # Export chat data
savant-chat --clear                      # Clear history
```

### **savant-config** - Configuration Management
Manage application settings and preferences.

```bash
# Future usage:
savant-config --get llm.model           # Get configuration value
savant-config --set llm.model codellama # Set configuration value
savant-config --export                  # Export all settings
savant-config --import settings.json    # Import settings
```

### **savant-stealth** - Window Stealth Operations
Handle OS-level window hiding and transparency.

```bash
# Future usage:
savant-stealth --hide-from-screenshots  # Enable stealth mode
savant-stealth --set-transparency 0.8   # Set window transparency
savant-stealth --always-on-top         # Keep window on top
```

## **Design Principles**

### **1. Single Responsibility**
Each tool does exactly one thing:
- `savant-llm`: LLM inference only
- `savant-browser`: Browser content only  
- `savant-chat`: Chat history only
- etc.

### **2. Composability**
Tools work together via standard UNIX pipes:
```bash
savant-browser --detect-questions | \
savant-llm --batch | \
savant-chat --save-responses
```

### **3. Standard Interfaces**
- **Input**: JSON via stdin or CLI arguments
- **Output**: JSON to stdout  
- **Errors**: Human-readable to stderr
- **Exit codes**: 0 for success, non-zero for errors

### **4. Testability**
Each component can be tested in complete isolation:
```bash
cargo test --package savant-llm
cargo test --package savant-browser
```

## **JSON Schemas**

### **LlmRequest**
```json
{
  "prompt": "What is Rust?",
  "model": "devstral", 
  "provider": {"Ollama": {"url": "http://localhost:11434"}},
  "options": {
    "temperature": 0.7,
    "max_tokens": 4096,
    "stream": false
  },
  "context": null
}
```

### **LlmResponse**  
```json
{
  "content": "Rust is a systems programming language...",
  "model": "devstral",
  "provider": "ollama", 
  "tokens_used": null,
  "processing_time_ms": 1400,
  "finished": true
}
```

### **ChatMessage**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Hello, how are you?",
  "is_user": true,
  "timestamp": "2025-07-01T02:52:16.848567Z",
  "metadata": null
}
```

## **Development Workflow**

### **Building Individual Components**
```bash
# Build just the LLM tool
cargo build --package savant-llm

# Build all components
cargo build --workspace
```

### **Testing Individual Components**
```bash
# Test just one component
cargo test --package savant-llm

# Test all components
cargo test --workspace
```

### **Installing CLI Tools**
```bash
# Install individual tool
cargo install --path crates/savant-llm

# Install all CLI tools (when ready)
cargo install --path crates/savant-llm
cargo install --path crates/savant-browser  
cargo install --path crates/savant-chat
```

## **Integration Examples**

### **Simple AI Assistant**
```bash
#!/bin/bash
# basic_assistant.sh
while read -r question; do
    echo "$question" | savant-llm | jq -r '.content'
done
```

### **Browser Question Monitor**
```bash
#!/bin/bash  
# auto_answer.sh (when browser component is ready)
savant-browser --monitor | \
while read -r tab_change; do
    echo "$tab_change" | \
    savant-browser --detect-questions | \
    savant-llm --batch
done
```

### **Chat Analysis Pipeline**
```bash
#!/bin/bash
# analyze_conversations.sh (when chat component is ready)
savant-chat --export | \
jq '.messages[] | select(.is_user == false) | .content' | \
savant-llm --prompt "Summarize these AI responses" | \
jq -r '.content'
```

This modular architecture enables you to:
- Test components in isolation
- Build custom workflows
- Replace individual components
- Scale and maintain code more easily
- Follow true UNIX philosophy principles