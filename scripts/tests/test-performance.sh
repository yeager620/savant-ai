#!/bin/bash
# Performance and benchmark tests for Savant AI components
# Tests response times, memory usage, and throughput

set -e

echo "⚡ Performance and Benchmark Tests"
echo "=================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Performance thresholds (milliseconds)
OCR_FAST_THRESHOLD=2000    # 2 seconds for fast mode
OCR_FULL_THRESHOLD=30000   # 30 seconds for full mode
VISION_THRESHOLD=1000      # 1 second for vision analysis
DB_QUERY_THRESHOLD=500     # 500ms for database queries
MCP_THRESHOLD=1000         # 1 second for MCP operations

# Utility functions
time_command() {
    local start_time=$(date +%s%N)
    "$@" &>/dev/null
    local exit_code=$?
    local end_time=$(date +%s%N)
    local duration=$((($end_time - $start_time) / 1000000))  # Convert to milliseconds
    echo $duration
    return $exit_code
}

check_threshold() {
    local duration=$1
    local threshold=$2
    local test_name="$3"
    
    if [ $duration -le $threshold ]; then
        echo -e "${GREEN}✅ $test_name: ${duration}ms (< ${threshold}ms)${NC}"
        return 0
    else
        echo -e "${RED}❌ $test_name: ${duration}ms (> ${threshold}ms)${NC}"
        return 1
    fi
}

# Build all tools in release mode for accurate performance testing
echo -e "${BLUE}Building tools in release mode...${NC}"
cargo build --workspace --release
echo -e "${GREEN}✅ Build completed${NC}"

# Create test data
echo -e "\n${BLUE}Setting up test data...${NC}"

# Create test image for OCR/Vision tests
if ! command -v convert &> /dev/null; then
    echo -e "${YELLOW}⚠️  ImageMagick not available, creating screenshot instead${NC}"
    screencapture -x test_perf_image.png 2>/dev/null || {
        echo -e "${RED}❌ Cannot create test image${NC}"
        exit 1
    }
else
    convert -size 800x600 -background white -fill black -font Arial -pointsize 16 \
            label:"Performance Test Image\nThis is a test image for OCR and vision analysis.\nIt contains multiple lines of text for accurate testing.\nCreated: $(date)" \
            test_perf_image.png
fi

# Create large test image for stress testing
convert test_perf_image.png -resize 200% test_perf_large.png 2>/dev/null || cp test_perf_image.png test_perf_large.png

echo -e "${GREEN}✅ Test data created${NC}"

# Test 1: OCR Performance
echo -e "\n${BLUE}Testing OCR Performance${NC}"
echo "------------------------"

echo "Testing OCR fast mode..."
duration=$(time_command cargo run --package savant-ocr -- extract --input test_perf_image.png --format text --fast)
check_threshold $duration $OCR_FAST_THRESHOLD "OCR Fast Mode"

echo "Testing OCR standard mode..."
duration=$(time_command cargo run --package savant-ocr -- extract --input test_perf_image.png --format json)
check_threshold $duration $OCR_FULL_THRESHOLD "OCR Standard Mode"

echo "Testing OCR with large image..."
duration=$(time_command cargo run --package savant-ocr -- extract --input test_perf_large.png --format text --fast)
echo -e "${BLUE}   OCR Large Image (fast): ${duration}ms${NC}"

# Test 2: Computer Vision Performance
echo -e "\n${BLUE}Testing Computer Vision Performance${NC}"
echo "-----------------------------------"

echo "Testing vision analysis..."
duration=$(time_command cargo run --package savant-vision -- analyze --input test_perf_image.png --detect-apps --format json)
check_threshold $duration $VISION_THRESHOLD "Vision Analysis"

echo "Testing vision with activity classification..."
duration=$(time_command cargo run --package savant-vision -- analyze --input test_perf_image.png --classify-activity)
echo -e "${BLUE}   Vision Activity Classification: ${duration}ms${NC}"

# Test 3: Database Performance
echo -e "\n${BLUE}Testing Database Performance${NC}"
echo "----------------------------"

echo "Testing database list query..."
duration=$(time_command cargo run --package savant-db -- list --limit 10)
check_threshold $duration $DB_QUERY_THRESHOLD "Database List Query"

echo "Testing database stats..."
duration=$(time_command cargo run --package savant-db -- stats)
check_threshold $duration $DB_QUERY_THRESHOLD "Database Stats Query"

echo "Testing database search..."
duration=$(time_command cargo run --package savant-db -- query --text "test" --limit 10)
echo -e "${BLUE}   Database Search Query: ${duration}ms${NC}"

# Test 4: MCP Server Performance
echo -e "\n${BLUE}Testing MCP Server Performance${NC}"
echo "------------------------------"

echo "Testing MCP initialization..."
duration=$(time_command bash -c 'echo '\''{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'\'' | timeout 10s cargo run --package savant-mcp -- --test')
check_threshold $duration $MCP_THRESHOLD "MCP Initialization"

echo "Testing MCP tools listing..."
duration=$(time_command bash -c 'echo '\''{"jsonrpc":"2.0","id":2,"method":"tools/list"}'\'' | timeout 10s cargo run --package savant-mcp -- --test')
check_threshold $duration $MCP_THRESHOLD "MCP Tools Listing"

echo "Testing MCP query..."
duration=$(time_command bash -c 'echo '\''{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query_conversations","arguments":{"query":"test query"}}}'\'' | timeout 15s cargo run --package savant-mcp -- --test')
echo -e "${BLUE}   MCP Query Processing: ${duration}ms${NC}"

# Test 5: Memory Usage Testing
echo -e "\n${BLUE}Testing Memory Usage${NC}"
echo "--------------------"

if command -v /usr/bin/time &> /dev/null; then
    echo "Testing OCR memory usage..."
    /usr/bin/time -f "Memory: %M KB" cargo run --package savant-ocr -- extract --input test_perf_image.png --fast 2>&1 | grep "Memory:" || echo "Memory usage data not available"
    
    echo "Testing vision memory usage..."
    /usr/bin/time -f "Memory: %M KB" cargo run --package savant-vision -- analyze --input test_perf_image.png --detect-apps 2>&1 | grep "Memory:" || echo "Memory usage data not available"
else
    echo -e "${YELLOW}⚠️  /usr/bin/time not available for memory testing${NC}"
fi

# Test 6: Throughput Testing
echo -e "\n${BLUE}Testing Throughput${NC}"
echo "------------------"

echo "Testing OCR throughput (5 iterations)..."
start_time=$(date +%s%N)
for i in {1..5}; do
    cargo run --package savant-ocr -- extract --input test_perf_image.png --fast &>/dev/null
done
end_time=$(date +%s%N)
total_duration=$((($end_time - $start_time) / 1000000))
avg_duration=$((total_duration / 5))
throughput=$((5000 / (total_duration / 1000)))  # Images per second * 1000 for precision
echo -e "${BLUE}   OCR Average: ${avg_duration}ms per image${NC}"
echo -e "${BLUE}   OCR Throughput: $((throughput / 1000)).$((throughput % 1000)) images/second${NC}"

echo "Testing vision throughput (3 iterations)..."
start_time=$(date +%s%N)
for i in {1..3}; do
    cargo run --package savant-vision -- analyze --input test_perf_image.png --detect-apps &>/dev/null
done
end_time=$(date +%s%N)
total_duration=$((($end_time - $start_time) / 1000000))
avg_duration=$((total_duration / 3))
echo -e "${BLUE}   Vision Average: ${avg_duration}ms per image${NC}"

# Test 7: Stress Testing
echo -e "\n${BLUE}Testing Under Load${NC}"
echo "------------------"

echo "Testing concurrent OCR operations..."
start_time=$(date +%s%N)
(
    cargo run --package savant-ocr -- extract --input test_perf_image.png --fast &
    cargo run --package savant-ocr -- extract --input test_perf_image.png --fast &
    cargo run --package savant-ocr -- extract --input test_perf_image.png --fast &
    wait
) &>/dev/null
end_time=$(date +%s%N)
concurrent_duration=$((($end_time - $start_time) / 1000000))
echo -e "${BLUE}   Concurrent OCR (3 parallel): ${concurrent_duration}ms${NC}"

# Test 8: Pipeline Performance
echo -e "\n${BLUE}Testing Pipeline Performance${NC}"
echo "----------------------------"

echo "Testing OCR → jq pipeline..."
start_time=$(date +%s%N)
cargo run --package savant-ocr -- extract --input test_perf_image.png --format json --fast 2>/dev/null | jq '.text' &>/dev/null
end_time=$(date +%s%N)
pipeline_duration=$((($end_time - $start_time) / 1000000))
echo -e "${BLUE}   OCR → jq Pipeline: ${pipeline_duration}ms${NC}"

echo "Testing Vision → jq pipeline..."
start_time=$(date +%s%N)
cargo run --package savant-vision -- analyze --input test_perf_image.png --detect-apps --format json 2>/dev/null | jq '.detected_applications' &>/dev/null
end_time=$(date +%s%N)
vision_pipeline_duration=$((($end_time - $start_time) / 1000000))
echo -e "${BLUE}   Vision → jq Pipeline: ${vision_pipeline_duration}ms${NC}"

# Cleanup
rm -f test_perf_image.png test_perf_large.png

# Performance Summary
echo -e "\n${BLUE}================================${NC}"
echo -e "${BLUE}Performance Test Summary${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "OCR Fast Mode Target: < ${OCR_FAST_THRESHOLD}ms"
echo -e "Vision Analysis Target: < ${VISION_THRESHOLD}ms"
echo -e "Database Query Target: < ${DB_QUERY_THRESHOLD}ms"
echo -e "MCP Operation Target: < ${MCP_THRESHOLD}ms"

echo -e "\n${GREEN}Performance testing completed!${NC}"
echo -e "${BLUE}Check individual results above for detailed timing information.${NC}"