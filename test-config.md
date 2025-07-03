# Test Configuration Guide

## Test Organization

The comprehensive test suite is organized into several categories:

### 1. Unit Tests
- **Location**: `crates/*/tests/`
- **Command**: `cargo test --workspace`
- **Coverage**: Individual components and functions

#### New Functionality Unit Tests:
- `crates/savant-video/tests/coding_problem_detector_tests.rs` - Tests problem detection from OCR/vision
- `crates/savant-video/tests/solution_generator_tests.rs` - Tests LLM solution generation
- `crates/savant-db/tests/visual_data_tests.rs` - Tests smart database operations
- `crates/savant-mcp/tests/mcp_server_tests.rs` - Tests MCP natural language queries

### 2. Integration Tests
- **Location**: `crates/savant-video/tests/integration_tests.rs`
- **Command**: `cargo test --package savant-video integration_tests`
- **Coverage**: Full processing pipeline with real screenshots

### 3. Performance Benchmarks
- **Location**: `crates/savant-video/tests/performance_benchmarks.rs`
- **Command**: `cargo test --package savant-video performance_benchmarks --release -- --nocapture`
- **Coverage**: Processing speed, memory usage, throughput

### 4. End-to-End Tests
- **Script**: `scripts/tests/test-new-functionality.sh`
- **Coverage**: Complete workflow from screenshot to solution

## Quick Test Commands

### Test New Functionality Only
```bash
./scripts/tests/test-new-functionality.sh
```

### Run Comprehensive Test Suite
```bash
./scripts/tests/run-comprehensive-tests.sh
```

### Test Specific Components
```bash
# Just the coding problem detector
cargo test --package savant-video coding_problem_detector_tests

# Just the solution generator
cargo test --package savant-video solution_generator_tests

# Just the database integration
cargo test --package savant-db visual_data_tests

# Just the MCP server
cargo test --package savant-mcp mcp_server_tests
```

### Test with Real Screenshots
```bash
# Process all test screenshots
cargo test --package savant-video test_multiple_screenshots_processing --release -- --nocapture

# Test specific screenshot
cargo test --package savant-video test_coding_problem_detection_with_real_screenshot --release -- --nocapture
```

### Performance Testing
```bash
# Run all performance benchmarks
cargo test --package savant-video performance_benchmarks --release -- --nocapture

# Quick performance check
cargo test --package savant-video benchmark_single_frame_processing --release -- --nocapture
```

## Test Data

### Screenshots
Located in `test-data/screenshots/`:
- `twosum.png` - LeetCode Two Sum problem
- `hackerrank_hard_01.png` - HackerRank hard problem
- `getcracked_medium_01.png` - Medium difficulty coding challenge

### Expected Test Results

#### Problem Detection
- Should detect algorithm challenges in coding platform screenshots
- Should identify problem titles, descriptions, and starter code
- Should classify programming language and platform

#### Solution Generation
- Should generate working code solutions
- Should include explanations and complexity analysis
- Should have confidence scores > 0.5

#### Database Operations
- Should store and retrieve text extractions with positioning
- Should handle high-frequency frame data efficiently
- Should support complex multimodal queries

#### MCP Server
- Should handle natural language queries about screen content
- Should correlate audio and visual events
- Should provide real-time activity detection

## Performance Targets

### Processing Speed
- Single frame processing: < 5 seconds
- OCR text extraction: < 1 second (fast mode)
- Problem detection: < 2 seconds
- Solution generation: < 3 seconds

### Throughput
- Should process > 0.1 frames per second continuously
- Should handle concurrent requests efficiently

### Memory Usage
- Should use < 500MB total for processing pipeline
- Should not leak memory during continuous operation

## Troubleshooting

### Common Issues

1. **Missing Test Screenshots**
   - Ensure files exist in `test-data/screenshots/`
   - Check file permissions and paths

2. **LLM Integration Failures**
   - Verify mock LLM provider is working
   - Check network connectivity for real LLM calls

3. **Database Errors**
   - Ensure SQLite is available
   - Check migration files are present

4. **OCR/Vision Failures**
   - Verify Tesseract is installed
   - Check image file formats and sizes

### Debug Commands

```bash
# Enable debug logging
RUST_LOG=debug cargo test [test_name]

# Run with verbose output
cargo test [test_name] -- --nocapture

# Run single test
cargo test [test_name] --package [package_name]
```

## Continuous Integration

The test suite is designed to run in CI environments:

```bash
# CI-friendly test command
./scripts/tests/run-all-tests.sh --unit-only

# With performance benchmarks
./scripts/tests/run-all-tests.sh --with-performance
```

## Test Coverage

### Current Coverage
- Coding Problem Detection: ✅ Complete
- Solution Generation: ✅ Complete
- Database Integration: ✅ Complete
- MCP Server: ✅ Complete
- End-to-End Workflow: ✅ Complete
- Performance Benchmarks: ✅ Complete

### Future Enhancements
- Add more diverse test screenshots
- Expand language support testing
- Add stress testing for high-frequency processing
- Include security and privacy testing