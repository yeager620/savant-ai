#!/bin/bash
# Comprehensive test runner for all Savant AI tests
# Runs all test categories in logical order with detailed reporting

set -e

echo "🧪 Savant AI Comprehensive Test Suite"
echo "======================================"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Test categories
UNIT_TESTS=0
INTEGRATION_TESTS=0
SYSTEM_TESTS=0
PERFORMANCE_TESTS=0
FAILED_CATEGORIES=0

# Test result tracking
run_test_category() {
    local category_name="$1"
    local test_script="$2"
    local category_var="$3"
    
    echo -e "\n${PURPLE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${PURPLE}║ $category_name${NC}"
    echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
    
    if [ -f "$test_script" ]; then
        if bash "$test_script"; then
            echo -e "${GREEN}✅ $category_name PASSED${NC}"
            eval "$category_var=1"
        else
            echo -e "${RED}❌ $category_name FAILED${NC}"
            FAILED_CATEGORIES=$((FAILED_CATEGORIES + 1))
        fi
    else
        echo -e "${YELLOW}⚠️  $category_name script not found: $test_script${NC}"
    fi
}

# Parse command line arguments
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_SYSTEM=true
RUN_PERFORMANCE=false  # Optional by default
RUN_CLI=true

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit-only)
            RUN_INTEGRATION=false
            RUN_SYSTEM=false
            RUN_PERFORMANCE=false
            RUN_CLI=false
            shift
            ;;
        --integration-only)
            RUN_UNIT=false
            RUN_SYSTEM=false
            RUN_PERFORMANCE=false
            RUN_CLI=false
            shift
            ;;
        --system-only)
            RUN_UNIT=false
            RUN_INTEGRATION=false
            RUN_PERFORMANCE=false
            RUN_CLI=false
            shift
            ;;
        --with-performance)
            RUN_PERFORMANCE=true
            shift
            ;;
        --performance-only)
            RUN_UNIT=false
            RUN_INTEGRATION=false
            RUN_SYSTEM=false
            RUN_CLI=false
            RUN_PERFORMANCE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --unit-only        Run only unit tests"
            echo "  --integration-only Run only integration tests"
            echo "  --system-only      Run only system tests"
            echo "  --performance-only Run only performance tests"
            echo "  --with-performance Include performance tests"
            echo "  --help            Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for available options"
            exit 1
            ;;
    esac
done

# Display test configuration
echo -e "${BLUE}Test Configuration:${NC}"
echo -e "Unit Tests: $([ "$RUN_UNIT" = true ] && echo "✓" || echo "✗")"
echo -e "Integration Tests: $([ "$RUN_INTEGRATION" = true ] && echo "✓" || echo "✗")"
echo -e "System Tests: $([ "$RUN_SYSTEM" = true ] && echo "✓" || echo "✗")"
echo -e "CLI Tests: $([ "$RUN_CLI" = true ] && echo "✓" || echo "✗")"
echo -e "Performance Tests: $([ "$RUN_PERFORMANCE" = true ] && echo "✓" || echo "✗")"

# Ensure we have the necessary tools
echo -e "\n${BLUE}Checking Prerequisites...${NC}"

# Check for required tools
if ! command -v jq &> /dev/null; then
    echo -e "${RED}❌ jq not found. Install with: brew install jq${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ cargo not found. Rust installation required${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites satisfied${NC}"

# Build everything first
echo -e "\n${BLUE}Building all components...${NC}"
cargo build --workspace --release
echo -e "${GREEN}✅ Build completed${NC}"

START_TIME=$(date +%s)

# 1. Unit Tests (Rust native tests)
if [ "$RUN_UNIT" = true ]; then
    run_test_category "UNIT TESTS" "" ""
    echo -e "${BLUE}Running Rust unit tests...${NC}"
    if cargo test --workspace; then
        echo -e "${GREEN}✅ Unit tests PASSED${NC}"
        UNIT_TESTS=1
    else
        echo -e "${RED}❌ Unit tests FAILED${NC}"
        FAILED_CATEGORIES=$((FAILED_CATEGORIES + 1))
    fi
fi

# 2. CLI Tools Tests
if [ "$RUN_CLI" = true ]; then
    run_test_category "CLI TOOLS TESTS" "scripts/tests/test-cli-tools.sh" "CLI_TESTS"
fi

# 3. Integration Tests
if [ "$RUN_INTEGRATION" = true ]; then
    run_test_category "DATABASE INTEGRATION TESTS" "scripts/tests/test-database-sql.sh" "DB_INTEGRATION"
    run_test_category "MCP INTEGRATION TESTS" "scripts/tests/test-mcp-natural-queries.sh" "MCP_INTEGRATION"
    run_test_category "CHATBOT INTEGRATION TESTS" "scripts/tests/test-chatbot-integration.sh" "CHATBOT_INTEGRATION"
    
    if [ "$DB_INTEGRATION" = 1 ] && [ "$MCP_INTEGRATION" = 1 ]; then
        INTEGRATION_TESTS=1
    fi
fi

# 4. System Tests
if [ "$RUN_SYSTEM" = true ]; then
    run_test_category "SYSTEM TESTS" "scripts/daemon-management/test_all_systems.sh" "SYSTEM_TESTS"
fi

# 5. Performance Tests (optional)
if [ "$RUN_PERFORMANCE" = true ]; then
    run_test_category "PERFORMANCE TESTS" "scripts/tests/test-performance.sh" "PERFORMANCE_TESTS"
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Final Summary
echo -e "\n${PURPLE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║ FINAL TEST SUMMARY${NC}"
echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Test Results:${NC}"
echo -e "Unit Tests: $([ "$UNIT_TESTS" = 1 ] && echo -e "${GREEN}PASSED${NC}" || echo -e "${RED}FAILED${NC}")"
echo -e "CLI Tests: $([ "$CLI_TESTS" = 1 ] && echo -e "${GREEN}PASSED${NC}" || echo -e "${RED}FAILED${NC}")"
echo -e "Integration Tests: $([ "$INTEGRATION_TESTS" = 1 ] && echo -e "${GREEN}PASSED${NC}" || echo -e "${RED}FAILED${NC}")"
echo -e "System Tests: $([ "$SYSTEM_TESTS" = 1 ] && echo -e "${GREEN}PASSED${NC}" || echo -e "${RED}FAILED${NC}")"
if [ "$RUN_PERFORMANCE" = true ]; then
    echo -e "Performance Tests: $([ "$PERFORMANCE_TESTS" = 1 ] && echo -e "${GREEN}PASSED${NC}" || echo -e "${RED}FAILED${NC}")"
fi

echo -e "\n${BLUE}Execution Time: ${DURATION} seconds${NC}"

if [ $FAILED_CATEGORIES -eq 0 ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED! 🎉${NC}"
    echo -e "${GREEN}Savant AI is ready for use.${NC}"
    
    # Optional: Show quick start instructions
    echo -e "\n${BLUE}Quick Start:${NC}"
    echo -e "  ./start-daemons     # Start all systems"
    echo -e "  ./monitor-daemons   # Monitor status"
    echo -e "  ./stop-daemons      # Stop when done"
    
    exit 0
else
    echo -e "\n${RED}❌ $FAILED_CATEGORIES test categories failed${NC}"
    echo -e "${RED}Check the output above for specific failures.${NC}"
    
    # Provide troubleshooting hints
    echo -e "\n${YELLOW}Troubleshooting:${NC}"
    echo -e "  ./verify-permissions    # Check system permissions"
    echo -e "  ./setup                 # Re-run setup if needed"
    echo -e "  cargo build --workspace # Rebuild components"
    
    exit 1
fi