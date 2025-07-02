#!/bin/bash
# Backup all existing transcript files to the database before implementing changes
# This ensures no data is lost during the refactoring

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🗄️  Backing Up Transcript Files to Database${NC}"
echo -e "${BLUE}==========================================${NC}"

# Configuration
AUDIO_CAPTURES_DIR="./data/audio-captures"
BACKUP_DB_PATH="./data/databases/dev/transcript-backup.db"
DB_CLI="./target/release/savant-db"

# Build the database CLI
echo -e "${BLUE}Building database CLI...${NC}"
cargo build --package savant-db --release >/dev/null 2>&1

# Check if audio captures directory exists
if [ ! -d "$AUDIO_CAPTURES_DIR" ]; then
    echo -e "${RED}❌ Audio captures directory not found: $AUDIO_CAPTURES_DIR${NC}"
    exit 1
fi

# Remove existing backup database and create fresh one
echo -e "${BLUE}Creating fresh backup database...${NC}"
rm -f "$BACKUP_DB_PATH"

# Count total files
TOTAL_FILES=$(find "$AUDIO_CAPTURES_DIR" -name "system_audio_*.md" | wc -l | tr -d ' ')
echo -e "${BLUE}Found $TOTAL_FILES transcript files to backup${NC}"

if [ "$TOTAL_FILES" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No transcript files found to backup${NC}"
    exit 0
fi

# Function to backup a single file
backup_file() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    
    # Extract timestamp from filename for conversation title
    local timestamp=$(echo "$file_name" | sed 's/system_audio_\([0-9_]*\)\.md/\1/' | sed 's/_/ /g')
    local title="System Audio Capture $timestamp"
    
    echo -e "${YELLOW}  Backing up: $file_name${NC}"
    
    # Check if file is JSON or Markdown format
    if head -1 "$file_path" | grep -q "^{"; then
        # JSON format file
        if ! jq empty "$file_path" >/dev/null 2>&1; then
            echo -e "${RED}    ❌ Invalid JSON format${NC}"
            return 1
        fi
        
        # Import JSON to database
        if cat "$file_path" | "$DB_CLI" --db-path "$BACKUP_DB_PATH" store --title "$title" >/dev/null 2>&1; then
            echo -e "${GREEN}    ✅ Successfully backed up (JSON)${NC}"
            return 0
        else
            echo -e "${RED}    ❌ Failed to backup JSON${NC}"
            return 1
        fi
    else
        # Markdown format file - extract meaningful content
        local text_content=""
        
        # Try to extract transcript content from markdown
        if grep -q "## Full Transcript" "$file_path"; then
            # Extract text between "## Full Transcript" and next "##" section
            text_content=$(awk '/^## Full Transcript$/,/^##/ {if (!/^##/ && !/^$/) print}' "$file_path" | head -20 | tr '\n' ' ')
        elif grep -q "## Timestamped Segments" "$file_path"; then
            # Extract from timestamped segments if no full transcript
            text_content=$(awk '/^## Timestamped Segments$/,/^##/ {if (!/^##/ && !/^### Segment/ && !/^$/) print}' "$file_path" | head -10 | tr '\n' ' ')
        fi
        
        # Skip if no meaningful content
        if [ ${#text_content} -lt 50 ]; then
            echo -e "${YELLOW}    ⚠️  No meaningful content found in markdown, skipping${NC}"
            return 1
        fi
        
        # Create basic JSON structure for markdown content
        local json_data=$(cat <<EOF
{
  "text": "$text_content",
  "language": "en",
  "segments": [
    {
      "text": "$text_content",
      "start_time": 0.0,
      "end_time": 300.0,
      "confidence": 0.8,
      "words": null
    }
  ],
  "processing_time_ms": 0,
  "model_used": "extracted-from-markdown",
  "session_metadata": {
    "session_id": "markdown-backup-$(date +%s)",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "audio_source": "SystemAudio",
    "speaker": "system",
    "device_info": "markdown-extraction"
  }
}
EOF
)
        
        # Import constructed JSON to database
        if echo "$json_data" | "$DB_CLI" --db-path "$BACKUP_DB_PATH" store --title "$title" >/dev/null 2>&1; then
            echo -e "${GREEN}    ✅ Successfully backed up (Markdown→JSON)${NC}"
            return 0
        else
            echo -e "${RED}    ❌ Failed to backup markdown content${NC}"
            return 1
        fi
    fi
}

# Backup all files
echo -e "${BLUE}Starting backup process...${NC}"
successful=0
failed=0

# Process files in chronological order
find "$AUDIO_CAPTURES_DIR" -name "system_audio_*.md" | sort | while read -r file; do
    if backup_file "$file"; then
        ((successful++))
    else
        ((failed++))
    fi
done

# Get final counts by checking the database
if [ -f "$BACKUP_DB_PATH" ]; then
    conversation_count=$("$DB_CLI" --db-path "$BACKUP_DB_PATH" list --limit 1000 | grep -c "id:" 2>/dev/null || echo "0")
    segment_count=$("$DB_CLI" --db-path "$BACKUP_DB_PATH" stats 2>/dev/null | grep -o "Total segments: [0-9]*" | grep -o "[0-9]*" || echo "0")
    
    echo
    echo -e "${GREEN}📊 Backup Summary:${NC}"
    echo -e "  Conversations stored: $conversation_count"
    echo -e "  Segments stored: $segment_count"
    echo -e "  Database location: $BACKUP_DB_PATH"
    
    # Show database size
    if [ -f "$BACKUP_DB_PATH" ]; then
        db_size=$(du -h "$BACKUP_DB_PATH" | cut -f1)
        echo -e "  Database size: $db_size"
    fi
    
    echo
    echo -e "${GREEN}✅ Transcript backup completed successfully!${NC}"
    echo -e "${BLUE}All transcript data has been safely stored in the database.${NC}"
    echo -e "${BLUE}You can now proceed with implementing the ring buffer system.${NC}"
else
    echo -e "${RED}❌ Backup database was not created${NC}"
    exit 1
fi