# UNIX Philosophy Refactoring Plan

## Current Architecture Problems

### Anti-UNIX Patterns Identified
1. **Monolithic Design**: Single Tauri app doing chat, browser monitoring, UI, and system management
2. **No Isolation**: Components can't be tested independently
3. **No Composability**: Can't pipe output between components
4. **Mixed Concerns**: Business logic tightly coupled with GUI

## Proposed UNIX-Style Architecture

### Core Principle: "Do One Thing Well"

Split the monolithic app into focused, single-purpose tools:

```
savant-ai/
├── crates/
│   ├── savant-llm/           # CLI: LLM inference only
│   ├── savant-browser/       # CLI: Browser content extraction only  
│   ├── savant-chat/          # CLI: Chat history management only
│   ├── savant-config/        # CLI: Configuration management only
│   ├── savant-stealth/       # CLI: Window stealth operations only
│   └── savant-ui/            # GUI: Orchestrates other tools via CLI
└── bin/                      # Standalone executables
    ├── savant-llm
    ├── savant-browser  
    ├── savant-chat
    ├── savant-config
    └── savant-stealth
```

### Component Design

#### 1. `savant-llm` - LLM Inference Tool
**Purpose**: Handle AI model communication only
**Input**: JSON prompt via stdin or args
**Output**: JSON response to stdout
**Usage**:
```bash
echo '{"prompt":"Hello","model":"devstral"}' | savant-llm
# {"response":"Hi there!","tokens":15,"duration_ms":234}

savant-llm --prompt "What is Rust?" --model devstral --stream
# Streams response line by line
```

#### 2. `savant-browser` - Browser Content Extractor  
**Purpose**: Extract text from browser tabs only
**Input**: Browser name or process ID
**Output**: JSON with tab contents
**Usage**:
```bash
savant-browser --scan-chrome
# {"tabs":[{"title":"Stack Overflow","content":"How to...","active":true}]}

savant-browser --detect-questions | jq '.prompts'
# ["How do I implement async Rust?", "What is the best web framework?"]
```

#### 3. `savant-chat` - Chat History Manager
**Purpose**: Store and retrieve conversation history only  
**Input**: JSON messages via stdin
**Output**: JSON chat history
**Usage**:
```bash
echo '{"user":"Hello","assistant":"Hi!"}' | savant-chat --save
savant-chat --load | jq '.messages[-5:]'  # Last 5 messages
savant-chat --clear
```

#### 4. `savant-config` - Configuration Manager
**Purpose**: Manage app settings only
**Input**: Config keys/values
**Output**: Configuration JSON
**Usage**:
```bash
savant-config --get llm.model
# devstral

savant-config --set llm.model codellama
savant-config --export | savant-config --import  # Backup/restore
```

#### 5. `savant-stealth` - Window Stealth Operations
**Purpose**: Handle OS-level window hiding only
**Input**: Window operations
**Output**: Success/failure status
**Usage**:
```bash
savant-stealth --hide-from-screenshots
savant-stealth --set-transparency 0.8
savant-stealth --enable-always-on-top
```

### Benefits of This Architecture

#### 1. **Testability**
```bash
# Test LLM independently
echo '{"prompt":"test"}' | savant-llm > output.json
assert_eq "$(jq '.response' output.json)" "test response"

# Test browser monitoring independently  
savant-browser --mock-data | savant-browser --detect-questions
```

#### 2. **Composability** 
```bash
# Chain tools together
savant-browser --detect-questions | \
savant-llm --batch | \
savant-chat --save-responses

# Custom workflows
savant-config --get llm.model | xargs -I {} savant-llm --model {}
```

#### 3. **Flexibility**
```bash
# Use different frontends
savant-browser | python custom_analyzer.py | savant-llm
savant-chat --load | rust-analyzer-bot | savant-llm --stream

# Easy debugging
RUST_LOG=debug savant-llm --prompt "test" 2> debug.log
```

#### 4. **Easy Expansion**
```bash
# Add new tools easily
savant-voice --speech-to-text | savant-llm | savant-voice --text-to-speech
savant-screen --ocr | savant-llm | savant-notify --desktop
```

## Implementation Strategy

### Phase 1: Extract Core Libraries
1. Create `crates/savant-core/` with shared types
2. Extract business logic from Tauri commands into libraries
3. Add comprehensive unit tests for each library

### Phase 2: Create CLI Tools
1. Build standalone CLI binaries for each component
2. Implement JSON stdin/stdout interfaces
3. Add CLI argument parsing and help documentation

### Phase 3: Refactor GUI
1. Convert Tauri app to orchestration layer
2. Replace direct function calls with CLI subprocess calls
3. Add error handling for CLI tool failures

### Phase 4: Testing & Documentation
1. Add integration tests using CLI tools
2. Create examples showing tool composition
3. Document CLI interfaces and data formats

## Data Flow Example

### Before (Monolithic)
```
User Input → Tauri Command → Mixed Business Logic → GUI Update
```

### After (UNIX Style)
```
User Input → savant-ui → savant-browser → stdout JSON →
savant-llm → stdout JSON → savant-chat → savant-ui → GUI Update
```

### CLI Composition Example
```bash
#!/bin/bash
# automated_qa.sh - Monitor browser and auto-answer questions

savant-browser --monitor | \
while read -r tab_data; do
    echo "$tab_data" | \
    savant-browser --detect-questions | \
    savant-llm --batch --model devstral | \
    savant-chat --save-qa-pair
done
```

This approach transforms your monolithic app into a composable toolkit following classic UNIX design principles.