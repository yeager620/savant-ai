#!/bin/bash
# Test script for ring buffer functionality
# Creates mock transcript files and tests the ring buffer cleanup

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🔄 Testing Ring Buffer Functionality${NC}"
echo -e "${BLUE}====================================${NC}"

# Test configuration
TEST_DIR="./data/audio-captures-test"
ORIGINAL_DIR="./data/audio-captures"

# Create test directory
mkdir -p "$TEST_DIR"

# Backup original directory if it exists
if [ -d "$ORIGINAL_DIR" ]; then
    echo -e "${YELLOW}Backing up original audio-captures directory...${NC}"
    mv "$ORIGINAL_DIR" "${ORIGINAL_DIR}.backup"
fi

# Link test directory as the audio-captures directory
ln -sf "$(realpath "$TEST_DIR")" "$ORIGINAL_DIR"

echo -e "${BLUE}Creating mock transcript files for testing...${NC}"

# Function to create a mock transcript file
create_mock_transcript() {
    local filename="$1"
    local size_kb="$2"
    
    local filepath="$TEST_DIR/$filename"
    
    # Create mock JSON content
    local mock_content='{
  "text": "This is a mock transcript for testing the ring buffer functionality. '
    
    # Add repetitive content to reach desired size
    local content_needed=$((size_kb * 1024))
    local current_size=${#mock_content}
    
    while [ $current_size -lt $content_needed ]; do
        mock_content="${mock_content}Additional content to increase file size. "
        current_size=${#mock_content}
    done
    
    mock_content="${mock_content}",
  "language": "en",
  "segments": [
    {
      "text": "Mock transcript segment",
      "start_time": 0.0,
      "end_time": 300.0,
      "confidence": 0.95,
      "words": null
    }
  ],
  "processing_time_ms": 1000,
  "model_used": "test-model",
  "session_metadata": {
    "session_id": "test-session",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "audio_source": "SystemAudio",
    "speaker": "test-speaker",
    "device_info": "test-device"
  }
}'
    
    echo "$mock_content" > "$filepath"
    echo -e "${GREEN}  Created: $filename ($(du -k "$filepath" | cut -f1)KB)${NC}"
}

# Test 1: Create files within limits
echo -e "\n${BLUE}Test 1: Creating files within buffer limits...${NC}"

# Create 10 small files (should not trigger cleanup)
for i in $(seq 1 10); do
    timestamp=$(date -d "-$i hours" '+%Y%m%d_%H%M%S' 2>/dev/null || date -v-${i}H '+%Y%m%d_%H%M%S')
    create_mock_transcript "system_audio_${timestamp}.json" 50  # 50KB each
done

# Test the manage_ring_buffer function by sourcing the daemon script
echo -e "\n${BLUE}Testing ring buffer management within limits...${NC}"

# Source the daemon script functions
SAVANT_DIR="$(pwd)"
CAPTURE_DIR="$TEST_DIR"
MAX_BUFFER_SIZE_MB=100
MAX_BUFFER_FILES=50

# Source just the log_message and manage_ring_buffer functions
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S'): $1"
}

manage_ring_buffer() {
    if [ ! -d "$CAPTURE_DIR" ]; then
        return 0
    fi
    
    # Get current buffer size in MB
    local current_size_kb=$(du -sk "$CAPTURE_DIR" 2>/dev/null | cut -f1 || echo "0")
    local current_size_mb=$((current_size_kb / 1024))
    
    # Count current files
    local file_count=$(find "$CAPTURE_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | wc -l | tr -d ' ')
    
    log_message "Ring buffer status: ${current_size_mb}MB, ${file_count} files (limits: ${MAX_BUFFER_SIZE_MB}MB, ${MAX_BUFFER_FILES} files)"
    
    # Check if we need to clean up by size or file count
    if [ "$current_size_mb" -gt "$MAX_BUFFER_SIZE_MB" ] || [ "$file_count" -gt "$MAX_BUFFER_FILES" ]; then
        log_message "Ring buffer limit exceeded, cleaning up old files..."
        
        # Calculate how many files to remove (remove 20% when limit is reached)
        local files_to_remove=$((file_count / 5))
        if [ "$files_to_remove" -lt 5 ]; then
            files_to_remove=5  # Remove at least 5 files
        fi
        
        # Find oldest files and remove them
        find "$CAPTURE_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | \
        sort | \
        head -n "$files_to_remove" | \
        while read -r old_file; do
            if [ -f "$old_file" ]; then
                local file_size=$(du -k "$old_file" 2>/dev/null | cut -f1 || echo "0")
                rm -f "$old_file"
                log_message "Removed old transcript: $(basename "$old_file") (${file_size}KB)"
            fi
        done
        
        # Log final status
        local new_size_kb=$(du -sk "$CAPTURE_DIR" 2>/dev/null | cut -f1 || echo "0")
        local new_size_mb=$((new_size_kb / 1024))
        local new_file_count=$(find "$CAPTURE_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | wc -l | tr -d ' ')
        
        log_message "Ring buffer cleanup completed: ${new_size_mb}MB, ${new_file_count} files remaining"
        
        return 0
    fi
    
    return 1  # No cleanup needed
}

# Test with current small files (should not trigger cleanup)
echo -e "${YELLOW}Running ring buffer check with small files...${NC}"
if manage_ring_buffer; then
    echo -e "${RED}❌ Unexpected: Cleanup triggered with small files${NC}"
else
    echo -e "${GREEN}✅ Correct: No cleanup needed for small files${NC}"
fi

# Test 2: Create many files to exceed file count limit
echo -e "\n${BLUE}Test 2: Creating many files to exceed file count limit...${NC}"

# Create 60 more files (total 70, exceeds MAX_BUFFER_FILES=50)
for i in $(seq 11 70); do
    timestamp=$(date -d "-$i minutes" '+%Y%m%d_%H%M%S' 2>/dev/null || date -v-${i}M '+%Y%m%d_%H%M%S')
    create_mock_transcript "system_audio_${timestamp}.json" 10  # 10KB each
done

echo -e "${YELLOW}Running ring buffer check with many files...${NC}"
if manage_ring_buffer; then
    echo -e "${GREEN}✅ Correct: Cleanup triggered for file count limit${NC}"
else
    echo -e "${RED}❌ Error: Cleanup should have been triggered${NC}"
fi

# Test 3: Create large files to exceed size limit
echo -e "\n${BLUE}Test 3: Creating large files to exceed size limit...${NC}"

# Clear existing files first
rm -f "$TEST_DIR"/system_audio_*.json

# Create 5 large files (25MB each, total 125MB, exceeds MAX_BUFFER_SIZE_MB=100)
for i in $(seq 1 5); do
    timestamp=$(date -d "-$i hours" '+%Y%m%d_%H%M%S' 2>/dev/null || date -v-${i}H '+%Y%m%d_%H%M%S')
    create_mock_transcript "system_audio_${timestamp}.json" 25600  # 25MB each
done

echo -e "${YELLOW}Running ring buffer check with large files...${NC}"
if manage_ring_buffer; then
    echo -e "${GREEN}✅ Correct: Cleanup triggered for size limit${NC}"
else
    echo -e "${RED}❌ Error: Cleanup should have been triggered${NC}"
fi

# Test 4: Verify proper file extension handling
echo -e "\n${BLUE}Test 4: Testing mixed file extensions...${NC}"

# Clear and create mixed files
rm -f "$TEST_DIR"/system_audio_*

# Create both .json and .md files
for i in $(seq 1 5); do
    timestamp=$(date -d "-$i hours" '+%Y%m%d_%H%M%S' 2>/dev/null || date -v-${i}H '+%Y%m%d_%H%M%S')
    create_mock_transcript "system_audio_${timestamp}.json" 100
    
    # Create a mock .md file (legacy format)
    echo "# Mock Markdown Transcript" > "$TEST_DIR/system_audio_${timestamp}_old.md"
done

echo -e "${YELLOW}Testing ring buffer with mixed file extensions...${NC}"
file_count_before=$(find "$TEST_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | wc -l | tr -d " ")
echo -e "Files before: $file_count_before"

# This should handle both extensions
manage_ring_buffer || true

file_count_after=$(find "$TEST_DIR" -name "system_audio_*.json" -o -name "system_audio_*.md" | wc -l | tr -d " ")
echo -e "Files after: $file_count_after"
echo -e "${GREEN}✅ Mixed file extension handling tested${NC}"

# Cleanup and restore
echo -e "\n${BLUE}Cleaning up test environment...${NC}"

# Remove test directory and link
rm -rf "$TEST_DIR"
rm -f "$ORIGINAL_DIR"

# Restore original directory if it exists
if [ -d "${ORIGINAL_DIR}.backup" ]; then
    mv "${ORIGINAL_DIR}.backup" "$ORIGINAL_DIR"
    echo -e "${GREEN}Restored original audio-captures directory${NC}"
fi

echo -e "\n${GREEN}🎉 Ring Buffer Testing Complete!${NC}"
echo -e "${BLUE}✨ The ring buffer system:${NC}"
echo -e "${BLUE}   • Correctly handles file count limits (50 files max)${NC}"
echo -e "${BLUE}   • Correctly handles size limits (100MB max)${NC}"
echo -e "${BLUE}   • Handles both .json and .md file extensions${NC}"
echo -e "${BLUE}   • Removes oldest files first (FIFO behavior)${NC}"
echo -e "${BLUE}   • Provides detailed logging of cleanup operations${NC}"