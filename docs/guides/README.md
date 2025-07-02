# User Guides

Task-oriented guides for specific functionality.

## Setup and Configuration

- **[Permissions Setup](permissions.md)** - macOS system permissions configuration
- **[Audio Setup](audio-setup.md)** - System audio capture with BlackHole/Multi-Output
- **[MCP Integration](mcp-integration.md)** - Model Context Protocol setup for LLM integration

## Usage Guides

### Audio Processing
- Real-time transcription and speaker identification
- System audio capture configuration
- Multi-language support setup

### Video Analysis
- Screen capture with OCR and computer vision
- Privacy controls and blocked applications
- Multimodal event correlation

### Database Queries
- Natural language database queries via MCP
- CLI tools for data analysis
- Export and backup procedures

### LLM Integration
- Ollama local model setup
- Cloud provider configuration (OpenAI, Anthropic, DeepSeek)
- Custom prompt templates and workflows

## Troubleshooting

Each guide includes specific troubleshooting sections. For general issues:

1. Run `./verify-permissions` to check system configuration
2. Check daemon logs with `./monitor-daemons` 
3. Test individual components with `./test-systems`

## Need Help?

- Check [CLI Reference](../reference/cli-tools.md) for command syntax
- See [Architecture Overview](../architecture/overview.md) for system design
- Review [Setup Guide](../SETUP_GUIDE.md) for comprehensive configuration