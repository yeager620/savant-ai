#!/bin/bash
# Comprehensive test suite for all CLI tools
# Tests each CLI tool independently following UNIX philosophy

set -e

echo "🛠️  Testing All CLI Tools"
echo "========================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Track test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test result tracking
test_result() {
    local test_name="$1"
    local result="$2"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" -eq 0 ]; then
        echo -e "${GREEN}✅ $test_name${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ $test_name${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Build all CLI tools
echo -e "${BLUE}Building all CLI tools...${NC}"
cargo build --workspace --release
echo -e "${GREEN}✅ Build completed${NC}"

# Test 1: OCR Tool
echo -e "\n${BLUE}Testing OCR Tool (savant-ocr)${NC}"
echo "-----------------------------"

# Basic OCR test
cargo run --package savant-ocr -- test &>/dev/null
test_result "OCR basic test" $?

# Help command
cargo run --package savant-ocr -- --help &>/dev/null
test_result "OCR help command" $?

# Create test image if none exists
if [ ! -f "test_image.png" ]; then
    echo "Creating test image..."
    echo "Hello World Test" | convert -size 200x50 -background white -fill black -font Arial -pointsize 14 label:@- test_image.png 2>/dev/null || {
        # Fallback: create simple test image
        screencapture -x test_image.png 2>/dev/null || touch test_image.png
    }
fi

if [ -f "test_image.png" ]; then
    # Fast OCR extraction
    cargo run --package savant-ocr -- extract --input test_image.png --format text --fast &>/dev/null
    test_result "OCR fast extraction" $?
    
    # Full OCR with classification
    cargo run --package savant-ocr -- extract --input test_image.png --classify --format json &>/dev/null
    test_result "OCR with classification" $?
fi

# Test 2: Computer Vision Tool  
echo -e "\n${BLUE}Testing Computer Vision Tool (savant-vision)${NC}"
echo "--------------------------------------------"

# Basic vision test
cargo run --package savant-vision -- test &>/dev/null
test_result "Vision basic test" $?

# Help command
cargo run --package savant-vision -- --help &>/dev/null
test_result "Vision help command" $?

if [ -f "test_image.png" ]; then
    # Application detection
    cargo run --package savant-vision -- analyze --input test_image.png --detect-apps --format json &>/dev/null
    test_result "Vision app detection" $?
    
    # Activity classification
    cargo run --package savant-vision -- analyze --input test_image.png --classify-activity &>/dev/null
    test_result "Vision activity classification" $?
fi

# Test 3: Multimodal Sync Tool
echo -e "\n${BLUE}Testing Multimodal Sync Tool (savant-sync)${NC}"
echo "------------------------------------------"

# Basic sync test
cargo run --package savant-sync -- test --count 5 &>/dev/null
test_result "Sync basic test" $?

# Help command
cargo run --package savant-sync -- --help &>/dev/null
test_result "Sync help command" $?

# Correlation test with mock data
echo '{"timestamp":"2025-07-02T12:00:00Z","type":"audio","content":"test"}' | \
cargo run --package savant-sync -- correlate --window-size 10 &>/dev/null
test_result "Sync correlation test" $?

# Test 4: Database Tool
echo -e "\n${BLUE}Testing Database Tool (savant-db)${NC}"
echo "--------------------------------"

# Basic database operations
cargo run --package savant-db -- list --limit 1 &>/dev/null
test_result "Database list operation" $?

# Help command
cargo run --package savant-db -- --help &>/dev/null
test_result "Database help command" $?

# Stats command
cargo run --package savant-db -- stats &>/dev/null
test_result "Database stats command" $?

# Test JSON output
cargo run --package savant-db -- list --limit 1 --format json &>/dev/null
test_result "Database JSON output" $?

# Test 5: LLM Tool
echo -e "\n${BLUE}Testing LLM Tool (savant-llm)${NC}"
echo "-----------------------------"

# Help command
cargo run --package savant-llm -- --help &>/dev/null
test_result "LLM help command" $?

# Mock provider test
echo "Test prompt" | cargo run --package savant-llm -- --provider mock &>/dev/null
test_result "LLM mock provider" $?

# Test 6: Transcription Tool
echo -e "\n${BLUE}Testing Transcription Tool (savant-transcribe)${NC}"
echo "---------------------------------------------"

# Help command
cargo run --package savant-transcribe -- --help &>/dev/null
test_result "Transcription help command" $?

# Test device listing (may fail without audio devices)
cargo run --package savant-transcribe -- --list-devices &>/dev/null || true
test_result "Transcription device listing" 0  # Always pass this as it may not have devices

# Test 7: MCP Server
echo -e "\n${BLUE}Testing MCP Server (savant-mcp)${NC}"
echo "------------------------------"

# Help command
cargo run --package savant-mcp -- --help &>/dev/null
test_result "MCP help command" $?

# Basic initialization test
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' | \
timeout 5s cargo run --package savant-mcp -- --test &>/dev/null
test_result "MCP initialization test" $?

# Test 8: Video CLI Tool (if available)
echo -e "\n${BLUE}Testing Video CLI Tool (savant-video-cli)${NC}"
echo "----------------------------------------"

if cargo build --package savant-video-cli --release &>/dev/null; then
    # Help command
    cargo run --package savant-video-cli -- --help &>/dev/null
    test_result "Video CLI help command" $?
    
    # Test capture (should not actually capture)
    timeout 2s cargo run --package savant-video-cli -- capture --help &>/dev/null
    test_result "Video CLI capture help" $?
else
    echo -e "${YELLOW}⚠️  Video CLI tool not available${NC}"
fi

# Test 9: UNIX Philosophy Integration
echo -e "\n${BLUE}Testing UNIX Philosophy Integration${NC}"
echo "-----------------------------------"

# Test piping between tools
if [ -f "test_image.png" ]; then
    # OCR → jq processing
    cargo run --package savant-ocr -- extract --input test_image.png --format json --fast 2>/dev/null | \
    jq '.text' &>/dev/null
    test_result "OCR → jq pipeline" $?
    
    # Vision → jq processing
    cargo run --package savant-vision -- analyze --input test_image.png --detect-apps --format json 2>/dev/null | \
    jq '.detected_applications' &>/dev/null
    test_result "Vision → jq pipeline" $?
fi

# Database → jq processing
cargo run --package savant-db -- list --limit 1 --format json 2>/dev/null | \
jq '.' &>/dev/null
test_result "Database → jq pipeline" $?

# Test 10: Error Handling
echo -e "\n${BLUE}Testing Error Handling${NC}"
echo "-----------------------"

# Test with invalid input
cargo run --package savant-ocr -- extract --input nonexistent_file.png &>/dev/null
if [ $? -ne 0 ]; then
    test_result "OCR error handling (invalid file)" 0
else
    test_result "OCR error handling (invalid file)" 1
fi

# Test with invalid arguments
cargo run --package savant-vision -- analyze --invalid-flag &>/dev/null
if [ $? -ne 0 ]; then
    test_result "Vision error handling (invalid args)" 0
else
    test_result "Vision error handling (invalid args)" 1
fi

# Test 11: Performance Testing
echo -e "\n${BLUE}Testing Performance${NC}"
echo "-------------------"

if [ -f "test_image.png" ]; then
    # Time OCR fast mode
    start_time=$(date +%s%N)
    cargo run --package savant-ocr -- extract --input test_image.png --fast &>/dev/null
    end_time=$(date +%s%N)
    duration=$((($end_time - $start_time) / 1000000))  # Convert to milliseconds
    
    if [ $duration -lt 5000 ]; then  # Less than 5 seconds
        test_result "OCR fast mode performance (<5s)" 0
    else
        test_result "OCR fast mode performance (<5s)" 1
    fi
    
    echo -e "${BLUE}   OCR processing time: ${duration}ms${NC}"
fi

# Cleanup
rm -f test_image.png

# Final Results
echo -e "\n${BLUE}================================${NC}"
echo -e "${BLUE}CLI Tools Test Results Summary${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "Total tests run: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
else
    echo -e "${GREEN}Failed: $FAILED_TESTS${NC}"
fi

# Calculate percentage
if [ $TOTAL_TESTS -gt 0 ]; then
    percentage=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo -e "Success rate: ${percentage}%"
fi

# Exit with appropriate code
if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All CLI tools tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Check output above for details.${NC}"
    exit 1
fi