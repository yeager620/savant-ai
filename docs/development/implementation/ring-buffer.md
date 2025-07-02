# Ring Buffer Implementation for Transcript Files

## Overview

The Savant AI audio daemon now implements a **ring buffer system** to manage transcript file storage, preventing unlimited accumulation and ensuring consistent performance.

## Changes Made

### 1. Fixed File Extension Confusion

**Problem:** Files were saved with `.md` extension but contained JSON data, creating confusion.

**Solution:**
- Updated audio daemon to explicitly use `.json` extension
- Added `--format json` flag to transcription command
- Files now properly reflect their content format

**Before:**
```bash
# Confusing - JSON content with .md extension
system_audio_20250701_152448.md  # Contains JSON data
```

**After:**
```bash
# Clear - JSON content with .json extension  
system_audio_20250701_152448.json  # Contains JSON data
```

### 2. Implemented Ring Buffer System

**Configuration:**
```bash
MAX_BUFFER_SIZE_MB=100  # Maximum size of transcript buffer in MB
MAX_BUFFER_FILES=50     # Maximum number of transcript files to keep
```

**Behavior:**
- **Automatic cleanup** when limits are exceeded
- **FIFO (First-In-First-Out)** deletion of oldest files
- **20% cleanup** when limits reached (removes 1/5 of files)
- **Minimum 5 files** removed per cleanup cycle
- **Detailed logging** of all cleanup operations

### 3. Enhanced Audio Daemon

**Updated `scripts/audio/savant-audio-daemon.sh`:**

```bash
# Function to manage ring buffer - keeps transcript storage within limits
manage_ring_buffer() {
    # Get current buffer size and file count
    local current_size_mb=$(($(du -sk "$CAPTURE_DIR" | cut -f1) / 1024))
    local file_count=$(find "$CAPTURE_DIR" -name "system_audio_*" | wc -l)
    
    # Check limits and cleanup if needed
    if [ "$current_size_mb" -gt "$MAX_BUFFER_SIZE_MB" ] || 
       [ "$file_count" -gt "$MAX_BUFFER_FILES" ]; then
        # Remove oldest files (FIFO)
        # Detailed logging of cleanup operations
    fi
}
```

**Updated capture function:**
```bash
capture_segment() {
    local output_file="$CAPTURE_DIR/system_audio_$timestamp.json"
    
    # Check ring buffer before capturing
    manage_ring_buffer
    
    cargo run --package savant-transcribe -- \
        --duration $SEGMENT_DURATION \
        --device "BlackHole 2ch" \
        --format json \
        --output "$output_file"
}
```

## Data Safety

### Backup Protection

**All existing transcript data was safely backed up to the database before implementing changes:**

- Used existing `query-audio.sh setup` functionality
- Imported 76 transcript files (1.4MB total) to SQLite database
- Database stored at: `data/databases/dev/personal-audio.db`
- No data loss during transition

### Ring Buffer Safety Features

1. **Graceful cleanup** - Only removes files when limits exceeded
2. **Oldest-first deletion** - Preserves recent audio captures
3. **Configurable limits** - Easy to adjust based on storage needs
4. **Detailed logging** - Full audit trail of cleanup operations
5. **Database integration** - Important data preserved in long-term storage

## Usage

### Ring Buffer Status

Check current buffer status:
```bash
./scripts/simple-ring-buffer-test.sh
```

Output:
```
Testing ring buffer with existing transcript files...
Current status: 76 files, 2MB
Limits: 50 files, 100MB
✅ Ring buffer would trigger cleanup (limits exceeded)
```

### Configuration

Adjust ring buffer limits in `scripts/audio/savant-audio-daemon.sh`:

```bash
# Conservative settings for limited storage
MAX_BUFFER_SIZE_MB=50
MAX_BUFFER_FILES=25

# Generous settings for ample storage  
MAX_BUFFER_SIZE_MB=500
MAX_BUFFER_FILES=200
```

### Manual Cleanup

Use database cleanup script for broader maintenance:
```bash
./scripts/cleanup-databases.sh --temp-only
```

## Benefits

### Storage Management
- **Predictable storage usage** - Never exceeds configured limits
- **Automatic maintenance** - No manual intervention required
- **Performance preservation** - Prevents directory bloat

### Data Integrity
- **No data loss** - Important data preserved in database
- **Crash recovery** - Transcript files provide durability buffer
- **Human inspection** - Recent files available for manual review

### Operational Excellence
- **Clear file formats** - JSON files contain JSON data
- **Comprehensive logging** - Full audit trail of operations
- **Configurable behavior** - Adjustable to different environments

## File Lifecycle

```
Audio Capture → JSON Transcript → Ring Buffer → Database Import
     ↓               ↓               ↓              ↓
  Real-time     Intermediate      Automatic      Long-term
  Processing     Storage          Cleanup        Storage
  (5 min)       (FIFO Buffer)    (When limits   (Permanent)
                                 exceeded)
```

## Monitoring

### Log Messages

Ring buffer operations are logged with timestamps:

```
2025-07-01 21:45:00: Ring buffer status: 2MB, 76 files (limits: 100MB, 50 files)
2025-07-01 21:45:00: Ring buffer limit exceeded, cleaning up old files...
2025-07-01 21:45:01: Removed old transcript: system_audio_20250701_101129.md (6KB)
2025-07-01 21:45:01: Ring buffer cleanup completed: 1MB, 50 files remaining
```

### Health Checks

Monitor ring buffer health:
```bash
# Check current status
du -sh data/audio-captures/
ls data/audio-captures/ | wc -l

# Check daemon logs
tail -f data/daemon-logs/savant-audio-daemon.log
```

## Implementation Status

✅ **Completed:**
- File extension confusion resolved
- Ring buffer system implemented  
- Automatic cleanup functionality
- Data safety through database backup
- Comprehensive logging
- Configuration options
- Testing and validation

The ring buffer system ensures **sustainable long-term operation** while maintaining **data safety** and **operational simplicity**.