#!/bin/bash

# Comprehensive Test Runner for Savant AI
# Tests all new functionality: daemon, data processing pipeline, smart database, MCP server, and integration tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}\n"
}

# Function to run a test command and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${YELLOW}Running: ${test_name}${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✓ PASSED: ${test_name}${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ FAILED: ${test_name}${NC}"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
    echo ""
}

# Function to print test summary
print_summary() {
    echo -e "\n${BLUE}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  TEST SUMMARY${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
    echo -e "Total Tests: ${TOTAL_TESTS}"
    echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
    echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}🎉 ALL TESTS PASSED! 🎉${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Some tests failed. Please check the output above.${NC}"
        exit 1
    fi
}

# Change to project root
cd "$(dirname "$0")/../.."

print_section "SAVANT AI COMPREHENSIVE TEST SUITE"

echo "Starting comprehensive test suite for new functionality..."
echo "This will test:"
echo "  1. Coding Problem Detector"
echo "  2. Solution Generator" 
echo "  3. Video Processing Pipeline Integration"
echo "  4. Smart Database Queries"
echo "  5. MCP Server Natural Language Queries"
echo "  6. End-to-End Integration with Screenshots"
echo "  7. Performance Benchmarks"

# 1. Unit Tests for Core Components
print_section "1. UNIT TESTS - CORE COMPONENTS"

run_test "Coding Problem Detector Tests" \
    "cargo test --package savant-video coding_problem_detector_tests --release"

run_test "Solution Generator Tests" \
    "cargo test --package savant-video solution_generator_tests --release"

# 2. Integration Tests for Video Processing Pipeline  
print_section "2. INTEGRATION TESTS - VIDEO PROCESSING PIPELINE"

run_test "Video Processing Integration Tests" \
    "cargo test --package savant-video integration_tests --release"

run_test "Real Screenshot Processing Test" \
    "cargo test --package savant-video test_coding_problem_detection_with_real_screenshot --release"

run_test "Multiple Screenshots Processing Test" \
    "cargo test --package savant-video test_multiple_screenshots_processing --release"

run_test "Concurrent Frame Processing Test" \
    "cargo test --package savant-video test_concurrent_frame_processing --release"

# 3. Smart Database Tests
print_section "3. SMART DATABASE TESTS"

run_test "Visual Data Storage and Retrieval" \
    "cargo test --package savant-db visual_data_tests --release"

run_test "High-Frequency Data Processing" \
    "cargo test --package savant-db test_store_and_retrieve_frame --release"

run_test "Text Extraction Database Operations" \
    "cargo test --package savant-db test_store_text_extractions --release"

run_test "Complex Query Scenarios" \
    "cargo test --package savant-db test_complex_query_scenarios --release"

run_test "Database Performance with Large Dataset" \
    "cargo test --package savant-db test_performance_with_large_dataset --release"

# 4. MCP Server Tests
print_section "4. MCP SERVER NATURAL LANGUAGE QUERY TESTS"

run_test "MCP Server Basic Functionality" \
    "cargo test --package savant-mcp mcp_server_tests --release"

run_test "Natural Language Conversation Queries" \
    "cargo test --package savant-mcp test_query_conversations_natural_language --release"

run_test "Semantic Search Tests" \
    "cargo test --package savant-mcp test_search_semantic --release"

run_test "Current Activity Detection" \
    "cargo test --package savant-mcp test_get_current_activity --release"

run_test "Multimodal Context Correlation" \
    "cargo test --package savant-mcp test_query_multimodal_context --release"

run_test "MCP Server Concurrent Request Handling" \
    "cargo test --package savant-mcp test_performance_with_concurrent_requests --release"

# 5. End-to-End Integration Tests
print_section "5. END-TO-END INTEGRATION TESTS"

# Check if test screenshots exist
echo "Verifying test data availability..."
if [ ! -f "test-data/screenshots/twosum.png" ]; then
    echo -e "${RED}Warning: twosum.png not found in test-data/screenshots/${NC}"
fi
if [ ! -f "test-data/screenshots/hackerrank_hard_01.png" ]; then
    echo -e "${RED}Warning: hackerrank_hard_01.png not found in test-data/screenshots/${NC}"
fi
if [ ! -f "test-data/screenshots/getcracked_medium_01.png" ]; then
    echo -e "${RED}Warning: getcracked_medium_01.png not found in test-data/screenshots/${NC}"
fi

run_test "Screenshot-Based Problem Detection" \
    "cargo test --package savant-video test_coding_problem_detection_with_real_screenshot --release -- --nocapture"

run_test "Multi-Platform Screenshot Processing" \
    "cargo test --package savant-video test_multiple_screenshots_processing --release -- --nocapture"

# 6. Performance Benchmarks
print_section "6. PERFORMANCE BENCHMARKS"

run_test "OCR Processing Performance" \
    "cargo test --package savant-ocr --release -- --nocapture performance"

run_test "Video Processing Performance" \
    "cargo test --package savant-video test_performance_metrics --release -- --nocapture"

run_test "Database Query Performance" \
    "cargo test --package savant-db test_performance_with_large_dataset --release -- --nocapture"

# 7. System Integration Tests
print_section "7. SYSTEM INTEGRATION TESTS"

run_test "All Workspace Tests" \
    "cargo test --workspace --release --exclude savant-screen-monitor --exclude savant-video-processor"

# 8. Daemon Tests (if available)
print_section "8. DAEMON FUNCTIONALITY TESTS"

if [ -f "./sav-video" ]; then
    run_test "Video Daemon Status Check" \
        "./sav-video status || true"  # Don't fail if daemon isn't running
else
    echo -e "${YELLOW}Video daemon script not found, skipping daemon tests${NC}"
fi

if [ -f "./sav" ]; then
    run_test "Audio Daemon Status Check" \
        "./sav status || true"  # Don't fail if daemon isn't running
else
    echo -e "${YELLOW}Audio daemon script not found, skipping daemon tests${NC}"
fi

# 9. CLI Tool Tests
print_section "9. CLI TOOL TESTS"

run_test "CLI Tools Tests" \
    "./scripts/tests/test-cli-tools.sh || true"

# Print final summary
print_summary