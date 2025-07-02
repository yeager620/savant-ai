# TODO

## Completed ✓

- ✓ Smart database for long-term screen capture data storage
- ✓ Automatic processing pipeline: .png → comprehensive text extraction with positioning
- ✓ MCP server with natural language queries ("what did I work on?", "how much coding?")
- ✓ Sub-second monitoring (500ms intervals) with change detection
- ✓ Real-time task/question detection for proactive assistance
- ✓ High-frequency optimized database schema with spatial indexing

## In Progress 🔄

### I) Enhanced Multimodal Intelligence
- [ ] Advanced pattern recognition for recurring workflows
- [ ] Predictive assistance based on detected user intent
- [ ] Cross-application context correlation
- [ ] Automatic code problem detection → LLM solution pipeline

### II) Improved GUI  
- [ ] macOS menu bar dropdown with daemon controls
- [ ] App analytics dashboard (storage, RAM, performance metrics)
- [ ] Real-time assistance notifications overlay
- [ ] Activity timeline visualization

### III) Enhanced LLM Integration
- [ ] Add llama3.2 as default model (faster than devstral)
- [ ] Unified multimodal chatbot interface
- [ ] Context-aware model selection (coding vs general tasks)
- [ ] Real-time assistance suggestions UI

## Future Features 🚀

### IV) Distribution & Deployment
- [ ] Containerization with Docker
- [ ] Automated release workflow with GitHub Actions
- [ ] Homebrew formula for easy installation

### V) Cross-Platform Support
- [ ] Windows implementation (Windows API screen capture)
- [ ] Linux support (X11/Wayland capture)
- [ ] Platform-specific permission management

### VI) Advanced Testing & Validation
- [ ] Unit tests for audio/video data pipeline
- [ ] HackerRank problem detection & solution benchmarks
- [ ] Website identification accuracy tests
- [ ] Performance regression testing for high-frequency capture
- [ ] Multimodal correlation accuracy validation

### VII) Performance Optimization
- [ ] GPU acceleration for computer vision
- [ ] Distributed processing for large datasets
- [ ] Advanced compression algorithms
- [ ] Real-time frame deduplication optimization

## Architecture Notes

All new features must maintain:
- UNIX philosophy (composable, single-purpose tools)
- High-frequency monitoring capability (sub-second precision)
- Privacy-first design with explicit consent
- Comprehensive text extraction with precise positioning
- Natural language query support via MCP integration