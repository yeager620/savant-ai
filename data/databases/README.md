# Database Organization

This directory contains all Savant AI database files organized by purpose and lifecycle.

## Directory Structure

```
data/databases/
├── test/           # Test databases (committed to git)
│   ├── chatbot-integration.db  # Chatbot integration testing
│   ├── manual-import.db        # Manual import testing
│   ├── manual.db              # General manual testing
│   └── quick.db               # Quick test database
├── dev/            # Development databases (gitignored)
│   └── working.db             # Working/scratch database
├── tmp/            # Temporary databases (gitignored)
│   └── [auto-cleanup]         # Temporary test databases
└── fixtures/       # Test fixtures (committed to git)
    └── [sample data]          # Known test data sets
```

## Database Types

### Test Databases (`test/`)
- **Purpose**: Reproducible testing with known data sets
- **Lifecycle**: Committed to git for shared testing
- **Naming**: `{feature}-{purpose}.db`
- **Usage**: Integration tests, manual testing, CI/CD

### Development Databases (`dev/`)
- **Purpose**: Local development and experimentation  
- **Lifecycle**: Gitignored, developer-specific
- **Naming**: `{purpose}.db`
- **Usage**: Local development, debugging, prototyping

### Temporary Databases (`tmp/`)
- **Purpose**: Short-lived test databases
- **Lifecycle**: Auto-cleanup after 7 days
- **Naming**: `temp-{timestamp}-{purpose}.db`
- **Usage**: Automated testing, one-off experiments

### Fixtures (`fixtures/`)
- **Purpose**: Sample data for consistent testing
- **Lifecycle**: Committed to git
- **Naming**: `{scenario}-fixture.db`
- **Usage**: Baseline data for tests

## Production Database

The production database is stored separately:
```
~/.config/savant-ai/transcripts.db
```

## Cleanup

Run cleanup script to remove old temporary databases:
```bash
./scripts/cleanup-databases.sh
```

## Creating New Databases

### Test Database
```bash
# Copy production schema for testing
cp ~/.config/savant-ai/transcripts.db data/databases/test/new-feature.db
```

### Development Database  
```bash
# Create empty development database
cargo run --package savant-db -- --db-path data/databases/dev/my-feature.db init
```

### Temporary Database
```bash
# Use tmp/ directory - auto-cleanup after 7 days
cargo run --package savant-db -- --db-path data/databases/tmp/temp-$(date +%s)-test.db init
```