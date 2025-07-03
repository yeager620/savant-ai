#!/bin/bash

# Test Runner for New Functionality Only
# Focuses specifically on the new coding problem detection and solution generation features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Change to project root
cd "$(dirname "$0")/../.."

echo -e "${BLUE}🧪 Testing New Functionality: Coding Problem Detection & Solution Generation${NC}\n"

# Check test screenshots
echo "📸 Checking test screenshots..."
for screenshot in "twosum.png" "hackerrank_hard_01.png" "getcracked_medium_01.png"; do
    if [ -f "test-data/screenshots/$screenshot" ]; then
        echo -e "${GREEN}✓${NC} Found: $screenshot"
    else
        echo -e "${RED}✗${NC} Missing: $screenshot"
    fi
done
echo ""

# 1. Test Coding Problem Detector
echo -e "${YELLOW}1. Testing Coding Problem Detector...${NC}"
if cargo test --package savant-video coding_problem_detector_tests --release; then
    echo -e "${GREEN}✓ Coding Problem Detector Tests Passed${NC}\n"
else
    echo -e "${RED}✗ Coding Problem Detector Tests Failed${NC}\n"
    exit 1
fi

# 2. Test Solution Generator
echo -e "${YELLOW}2. Testing Solution Generator...${NC}"
if cargo test --package savant-video solution_generator_tests --release; then
    echo -e "${GREEN}✓ Solution Generator Tests Passed${NC}\n"
else
    echo -e "${RED}✗ Solution Generator Tests Failed${NC}\n"
    exit 1
fi

# 3. Test Integration Pipeline
echo -e "${YELLOW}3. Testing Integrated Processing Pipeline...${NC}"
if cargo test --package savant-video integration_tests --release; then
    echo -e "${GREEN}✓ Integration Tests Passed${NC}\n"
else
    echo -e "${RED}✗ Integration Tests Failed${NC}\n"
    exit 1
fi

# 4. Test with Real Screenshots
echo -e "${YELLOW}4. Testing with Real Screenshots...${NC}"
if cargo test --package savant-video test_multiple_screenshots_processing --release -- --nocapture; then
    echo -e "${GREEN}✓ Screenshot Processing Tests Passed${NC}\n"
else
    echo -e "${RED}✗ Screenshot Processing Tests Failed${NC}\n"
    exit 1
fi

# 5. Test Database Integration
echo -e "${YELLOW}5. Testing Smart Database Integration...${NC}"
if cargo test --package savant-db visual_data_tests --release; then
    echo -e "${GREEN}✓ Database Integration Tests Passed${NC}\n"
else
    echo -e "${RED}✗ Database Integration Tests Failed${NC}\n"
    exit 1
fi

# 6. Test MCP Server
echo -e "${YELLOW}6. Testing MCP Server Natural Language Queries...${NC}"
if cargo test --package savant-mcp mcp_server_tests --release; then
    echo -e "${GREEN}✓ MCP Server Tests Passed${NC}\n"
else
    echo -e "${RED}✗ MCP Server Tests Failed${NC}\n"
    exit 1
fi

echo -e "${GREEN}🎉 All New Functionality Tests Passed! 🎉${NC}"
echo ""
echo "The following components are working correctly:"
echo "  ✅ Coding Problem Detection from Screenshots"
echo "  ✅ LLM-based Solution Generation"
echo "  ✅ Integrated Processing Pipeline"
echo "  ✅ Smart Database Storage and Queries" 
echo "  ✅ MCP Server Natural Language Interface"
echo "  ✅ Real-time Screenshot Analysis"
echo ""
echo "Ready for production use!"