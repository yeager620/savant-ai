# Development Documentation

Documentation for developers working on Savant AI.

## Getting Started

### Development Environment
```bash
# Clone and setup development environment
git clone <repository-url>
cd savant-ai
./setup

# Development build with hot reload
cargo tauri dev

# Run tests
cargo test --workspace
```

### Project Structure
```
savant-ai/
├── src/                    # Leptos frontend (Tauri app)
├── src-tauri/             # Tauri backend (Rust)
├── crates/                # UNIX CLI tools (workspace)
├── scripts/               # Automation and management scripts
├── docs/                  # Documentation
└── data/                  # Runtime data and databases
```

## Documentation

### Design and Architecture
- **[Implementation Summary](implementation-summary.md)** - Current implementation status
- **[UNIX Philosophy](unix-philosophy.md)** - Design principles and demonstrations
- **[Deprecations](deprecations.md)** - Deprecated features and migration paths

### Implementation Details
- **[Ring Buffer](implementation/ring-buffer.md)** - Audio buffer implementation
- **[Database Pipeline](implementation/database-pipeline.md)** - Data processing pipeline
- **[LLM Integration](implementation/llm-integration.md)** - Language model integration

### Development Processes
- **[Testing Strategy](testing.md)** - Test organization and best practices
- **[Contributing Guidelines](contributing.md)** - Code style and contribution process

## Development Workflow

### Building Components
```bash
# Build specific component
cargo build --package savant-ocr

# Build all CLI tools
cargo build --workspace --release

# Build Tauri app
cargo tauri build
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Test specific component
cargo test --package savant-audio

# Integration tests
./scripts/tests/test-mcp-natural-queries.sh
./scripts/tests/test-database-sql.sh
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace --all-targets

# Check for security issues
cargo audit
```

## Contributing

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` for linting
- Write tests for new functionality
- Document public APIs with rustdoc

### Architecture Guidelines
- Follow UNIX philosophy for CLI tools
- Maintain clear separation between components
- Use structured logging for debugging
- Handle errors gracefully with user-friendly messages

### Testing Requirements
- Unit tests for core functionality
- Integration tests for CLI tools
- End-to-end tests for complete workflows
- Performance tests for critical paths

### Documentation
- Update relevant documentation for changes
- Include usage examples for new features
- Document breaking changes in deprecations.md
- Keep CLI help text current

## Release Process

### Versioning
- Follow semantic versioning (SemVer)
- Tag releases in git
- Update CHANGELOG.md for each release

### Testing Before Release
```bash
# Comprehensive test suite
./test-systems
./scripts/tests/test-database-sql.sh
./scripts/tests/test-mcp-natural-queries.sh

# Performance regression tests
cargo test --release --workspace
```

### Distribution
```bash
# Build release artifacts
cargo tauri build --release

# Package for distribution
./scripts/setup/package-distribution.sh
```

## Debugging

### Logging
```bash
# Enable debug logging
export RUST_LOG=debug

# Component-specific logging
export RUST_LOG=savant_audio=debug,savant_video=trace
```

### Common Debug Scenarios
- **Permission Issues**: Check `./verify-permissions` output
- **Audio Problems**: Monitor with `./sav logs`
- **Video Capture**: Monitor with `./sav-video logs`
- **Database Issues**: Check database file permissions and SQLite logs
- **MCP Integration**: Enable JSON-RPC debug logging

### Performance Profiling
```bash
# Profile specific operations
cargo build --release
perf record ./target/release/savant-ocr extract --input large_image.png
perf report
```

## IDE Setup

### VS Code
Recommended extensions:
- rust-analyzer (Rust language support)
- Tauri (Tauri app development)
- Better TOML (configuration files)

### Configuration
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```