# Test Scripts

Comprehensive test suite for Savant AI components following UNIX philosophy principles.

## Test Categories

### Database Integration Tests
- **`test-database-sql.sh`** - Direct SQL database queries and CLI tool testing
- **`test-mcp-natural-queries.sh`** - MCP server natural language query integration

## Running Tests

```bash
# Run all database tests
./scripts/tests/test-database-sql.sh

# Run MCP server tests  
./scripts/tests/test-mcp-natural-queries.sh

# Make scripts executable if needed
chmod +x scripts/tests/*.sh
```

## Test Philosophy

Tests follow UNIX philosophy:
- **Single Purpose**: Each test script focuses on one component
- **Composability**: Tests can be run independently or together
- **Clear Output**: Structured output with clear pass/fail indicators
- **Exit Codes**: Proper exit codes for automation integration

## Test Coverage

- ✅ Database connection and initialization
- ✅ SQL query functionality 
- ✅ MCP server JSON-RPC 2.0 protocol
- ✅ Natural language query processing
- ✅ CLI tool integration
- ✅ UNIX workflow compositions
- ✅ Error handling and edge cases

## Adding New Tests

Follow the established patterns:
1. Clear test descriptions with numbered steps
2. Proper error handling and exit codes
3. Informative output with color coding
4. Test isolation (no dependencies between tests)