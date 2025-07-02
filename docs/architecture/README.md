# Architecture Documentation

System design and architectural principles for Savant AI.

## Documents

- **[System Overview](overview.md)** - High-level architecture and component relationships
- **[Components](components.md)** - Detailed component specifications
- **[Design Principles](principles.md)** - UNIX philosophy and design decisions

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Audio Input   │    │   Video Input   │    │   User Input    │
│  (Microphone)   │    │ (Screen Capture)│    │ (CLI Commands)  │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          v                      v                      v
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Speech-to-Text  │    │ OCR + Vision    │    │   CLI Tools     │
│   Processing    │    │   Analysis      │    │  (UNIX Style)   │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────┬───────────┘                      │
                     v                                  v
            ┌─────────────────┐                ┌─────────────────┐
            │ Multimodal Sync │                │      MCP        │
            │  & Correlation  │                │     Server      │
            └─────────┬───────┘                └─────────┬───────┘
                      │                                  │
                      v                                  v
            ┌─────────────────────────────────────────────────────┐
            │                SQLite Database                      │
            │           (Conversations + Analytics)               │
            └─────────────────────┬───────────────────────────────┘
                                  │
                                  v
            ┌─────────────────────────────────────────────────────┐
            │              LLM Integration                        │
            │         (Ollama, OpenAI, Anthropic)                │
            └─────────────────────────────────────────────────────┘
```

## Design Principles

### UNIX Philosophy
1. **Do One Thing Well**: Each tool has a single, clear purpose
2. **Work Together**: Tools compose via pipes and standard formats
3. **Universal Interface**: Text streams as universal interface
4. **Build on Others**: Leverage existing tools and libraries

### Modularity
- Independent crates for each major component
- Clear interfaces between components
- Pluggable architecture for extensibility
- Local-first with optional cloud integration

### Privacy & Security
- Local processing by default
- Encrypted storage for sensitive data
- User control over data sharing
- Transparent data handling

### Performance
- Real-time processing for interactive use
- Efficient data structures and algorithms
- Configurable trade-offs between speed and accuracy
- Resource-aware scaling

## Component Interaction

Components communicate through:
- **File System**: Shared database and configuration files
- **JSON-RPC**: MCP server for LLM integration
- **Pipes**: CLI tools compose via standard streams
- **Events**: Async event system for multimodal correlation

## Extension Points

The architecture supports extension through:
- **Custom CLI Tools**: Add new UNIX-style tools
- **MCP Tools**: Extend database query capabilities
- **LLM Providers**: Plugin architecture for new providers
- **Analysis Modules**: Custom OCR, vision, or audio processing
- **Storage Backends**: Alternative database or storage systems