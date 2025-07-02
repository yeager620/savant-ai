# Comprehensive Test Suite

Complete test coverage for all Savant AI components following UNIX philosophy principles.

## Quick Start

```bash
# Run all tests (recommended)
./scripts/tests/run-all-tests.sh

# Run specific test categories
./scripts/tests/run-all-tests.sh --unit-only
./scripts/tests/run-all-tests.sh --integration-only
./scripts/tests/run-all-tests.sh --with-performance

# Make scripts executable if needed
chmod +x scripts/tests/*.sh
```

## Test Categories

### 🔧 Unit Tests
**Script**: Built into Rust workspace  
**Command**: `cargo test --workspace`  
**Coverage**: Individual component functionality, data structures, algorithms

### 🛠️ CLI Tools Tests
**Script**: `test-cli-tools.sh`  
**Coverage**: All CLI tools functionality, UNIX philosophy compliance, error handling
- OCR tool (savant-ocr)
- Computer Vision tool (savant-vision)
- Multimodal Sync tool (savant-sync)
- Database tool (savant-db)
- LLM tool (savant-llm)
- Transcription tool (savant-transcribe)
- MCP server (savant-mcp)
- Pipeline integration and error handling

### 🔗 Integration Tests
**Scripts**: `test-database-sql.sh`, `test-mcp-natural-queries.sh`, `test-chatbot-integration.sh`  
**Coverage**: Component interaction and end-to-end workflows

#### Database Integration (`test-database-sql.sh`)
- Direct SQL database queries and CLI tool integration
- UNIX philosophy workflows and command composition
- JSON output piping and data export functionality

#### MCP Integration (`test-mcp-natural-queries.sh`)
- Natural language query processing via MCP server
- JSON-RPC 2.0 protocol compliance
- LLM-powered query understanding and response generation

#### Chatbot Integration (`test-chatbot-integration.sh`)
- Complete end-to-end LLM integration with real responses
- Multi-conversation analysis and synthesis
- Speaker-specific insights and business intelligence

### 🖥️ System Tests
**Script**: `test_all_systems.sh`  
**Coverage**: Full system validation and component health checks
- Dependency verification (Ollama, Tesseract, ImageMagick)
- Daemon status monitoring and service health
- Individual component testing with real data
- Permission and configuration validation

### ⚡ Performance Tests
**Script**: `test-performance.sh`  
**Coverage**: Response times, memory usage, and throughput testing
- OCR performance (fast mode <2s, standard mode <30s)
- Computer Vision analysis (<1s)
- Database query performance (<500ms)
- MCP server response times (<1s)
- Memory usage profiling and throughput testing

## Comprehensive Test Runner

The `run-all-tests.sh` script provides a unified test runner with detailed reporting:

```bash
# Full test suite
./scripts/tests/run-all-tests.sh

# Test categories
./scripts/tests/run-all-tests.sh --unit-only
./scripts/tests/run-all-tests.sh --integration-only
./scripts/tests/run-all-tests.sh --system-only
./scripts/tests/run-all-tests.sh --performance-only

# Include performance tests (optional by default)
./scripts/tests/run-all-tests.sh --with-performance
```

### Test Execution Order
1. **Prerequisites Check**: Verify required tools (jq, cargo, etc.)
2. **Build Phase**: Compile all components in release mode
3. **Unit Tests**: Rust workspace tests (`cargo test --workspace`)
4. **CLI Tests**: Individual tool testing and UNIX workflow validation
5. **Integration Tests**: Database, MCP, and chatbot integration
6. **System Tests**: Full system health and daemon status
7. **Performance Tests**: Benchmarking and optimization validation (optional)

### Output Format
- Color-coded results with detailed timing information
- Category-wise pass/fail tracking with error reporting
- Performance metrics and threshold validation
- Comprehensive summary with troubleshooting hints

## Test Philosophy

All tests follow UNIX philosophy principles:

### Single Purpose
Each test script focuses on one specific component or integration area.

### Composability
Tests can be run independently or together without dependencies.

### Clear Output
Structured output with clear pass/fail indicators and detailed error information.

### Exit Codes
Proper exit codes for automation integration:
- `0`: All tests passed
- `1`: One or more tests failed
- `2`: Invalid arguments or configuration issues

### Error Handling
Comprehensive error handling with informative failure messages and troubleshooting hints.

## Test Coverage Matrix

| Component | Unit Tests | CLI Tests | Integration | System | Performance |
|-----------|------------|-----------|-------------|---------|-------------|
| savant-ocr | ✅ | ✅ | ✅ | ✅ | ✅ |
| savant-vision | ✅ | ✅ | ✅ | ✅ | ✅ |
| savant-sync | ✅ | ✅ | ✅ | ✅ | ✅ |
| savant-db | ✅ | ✅ | ✅ | ✅ | ✅ |
| savant-llm | ✅ | ✅ | ✅ | ✅ | ❌ |
| savant-transcribe | ✅ | ✅ | ❌ | ✅ | ❌ |
| savant-mcp | ✅ | ✅ | ✅ | ❌ | ✅ |
| Audio Daemon | ✅ | ❌ | ❌ | ✅ | ❌ |
| Video Daemon | ✅ | ❌ | ❌ | ✅ | ❌ |
| UNIX Pipelines | ❌ | ✅ | ✅ | ❌ | ✅ |

## Performance Benchmarks

### OCR Processing
- **Fast Mode**: <2 seconds (real-time suitable)
- **Standard Mode**: <30 seconds (high accuracy)
- **Large Images**: Automatic optimization and fallback

### Computer Vision
- **App Detection**: <1 second per frame
- **Activity Classification**: Real-time processing
- **UI Analysis**: <1 second depending on complexity

### Database Operations
- **Simple Queries**: <500ms
- **Complex Searches**: <2 seconds
- **Statistics**: <500ms

### MCP Server
- **Initialization**: <1 second
- **Query Processing**: <1 second for simple queries
- **Natural Language**: Variable based on LLM provider

## Adding New Tests

### Test Script Guidelines
1. **Naming**: Use descriptive names following `test-<component>.sh` pattern
2. **Structure**: Follow established error handling and output formatting
3. **Independence**: Tests should not depend on other tests
4. **Cleanup**: Always clean up temporary files and test data
5. **Documentation**: Include clear test descriptions and expected behavior

### Example Test Structure
```bash
#!/bin/bash
set -e

echo "🧪 Testing [Component Name]"
echo "=========================="

# Colors and utilities
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Test implementation
test_result() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $1${NC}"
    else
        echo -e "${RED}❌ $1${NC}"
        exit 1
    fi
}

# Test cases
echo "Testing basic functionality..."
# Test implementation here
test_result "Basic functionality"

echo -e "\n${GREEN}🎉 All tests passed!${NC}"
```

### Integration with Main Test Runner
New test scripts are automatically discovered by the main test runner if they follow the naming convention and are placed in the `scripts/tests/` directory.

## Troubleshooting

### Common Issues

**Permission Errors**: Run `./verify-permissions` to check macOS permissions
**Build Failures**: Ensure Rust toolchain is up to date with `rustup update`
**Missing Dependencies**: Run `./setup` to install required tools
**Test Timeouts**: Check system resources and close unnecessary applications

### Debug Mode
Enable verbose logging for detailed test execution:
```bash
RUST_LOG=debug ./scripts/tests/run-all-tests.sh
```

### Individual Component Testing
Test specific components in isolation:
```bash
cargo test --package savant-ocr
cargo run --package savant-ocr -- test
./scripts/tests/test-cli-tools.sh  # Will test only OCR if others fail
```

## Continuous Integration

The test suite is designed for CI/CD integration:

```yaml
# Example GitHub Actions workflow
- name: Run Comprehensive Tests
  run: |
    ./scripts/tests/run-all-tests.sh --with-performance
    
- name: Run Quick Tests (PR validation)
  run: |
    ./scripts/tests/run-all-tests.sh --unit-only
    ./scripts/tests/run-all-tests.sh --integration-only
```

## Performance Regression Testing

Run performance tests regularly to detect regressions:
```bash
# Run before changes
./scripts/tests/test-performance.sh > baseline_performance.txt

# Run after changes  
./scripts/tests/test-performance.sh > new_performance.txt

# Compare results
diff baseline_performance.txt new_performance.txt
```