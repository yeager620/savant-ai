Video Capture System Specification

## Executive Summary

This specification outlines a simplified video capture system that mirrors the existing audio capture daemon architecture. Following the project's UNIX philosophy and incremental development approach, the system provides periodic screen capture with basic analysis and seamless integration with the audio transcription pipeline.

## Design Principles Alignment

### UNIX Philosophy Implementation

- **Single Purpose**: Screen capture at configurable intervals - simple and reliable
- **Composability**: Follows savant-transcribe patterns for CLI and data flow
- **JSON I/O**: Matches existing audio metadata structure
- **Independence**: Mirrors savant-audio daemon management approach

### Privacy-First Architecture

- **Local Processing**: All capture and analysis happens on-device
- **Simple Controls**: Time-based scheduling and explicit consent
- **Visual Indicators**: Clear recording status like audio daemon
- **User Consent**: Follows audio system notification patterns

### Performance Requirements

- **Realistic Resource Usage**: 5-10% CPU during active capture
- **Simple Storage**: PNG screenshots with basic compression
- **Incremental Approach**: Start simple, optimize based on actual usage
- **Proven Technology**: Leverage existing screenshot capabilities

## System Architecture

### Core Components (Simplified)

```
Screen → Periodic Capture → PNG Storage → Basic Analysis → Database
   ↓         ↓               ↓             ↓              ↓
Display   Timer/Event     File System   OCR/Hash      SQLite
```

### Integration with Existing Systems

```
Audio Daemon → Transcription → Database
     ↓             ↓             ↓
Video Daemon → Screenshot → Synchronized Storage
     ↓             ↓             ↓
Combined Timeline → Basic Analysis → Unified Search
```

## Technical Specifications

### Platform-Specific Implementation

**macOS Implementation**
- Use existing `take_screenshot()` Tauri command
- Leverage Core Graphics APIs already in codebase
- Integrate with existing Accessibility API permissions

**Cross-Platform Approach**
- Follow existing screenshot patterns in `src-tauri/src/commands/system.rs`
- Platform-specific conditional compilation like audio system
- Reuse existing image handling libraries

### Capture Strategy (Simplified)

**Fixed Interval Capture**
```rust
pub struct CaptureConfig {
    interval_seconds: u32,        // 5-300 seconds
    enabled_hours: TimeRange,     // 09:00-17:00
    quality: ImageQuality,        // Low, Medium, High
    notify_user: bool,           // System tray indicator
}
```

**Change Detection (Basic)**
```rust
pub struct ChangeDetector {
    previous_hash: Option<String>,
    threshold: f32,              // 0.1 = 10% change required
    consecutive_identical: u32,   // Skip after N identical frames
}
```

## Storage Architecture

### Database Schema (Mirrors Audio Pattern)

```sql
-- Mirror segments table structure from audio system
CREATE TABLE IF NOT EXISTS video_segments (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    timestamp DATETIME NOT NULL,
    duration_seconds REAL DEFAULT 0,
    file_path TEXT NOT NULL,
    screen_resolution TEXT,
    change_detected BOOLEAN DEFAULT FALSE,
    metadata TEXT,  -- JSON blob like audio segments
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

-- Basic frame metadata (optional)
CREATE TABLE IF NOT EXISTS video_frames (
    id TEXT PRIMARY KEY,
    segment_id TEXT REFERENCES video_segments(id),
    timestamp DATETIME NOT NULL,
    image_hash TEXT,
    ocr_text TEXT,
    file_size_bytes INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### File Storage Strategy

**Directory Structure** (Follows Audio Pattern)
```
~/.config/savant-ai/
├── video-captures/
│   ├── 2025-07-02/
│   │   ├── screenshot_001.png
│   │   ├── screenshot_002.png
│   │   └── metadata.json
│   └── 2025-07-03/
└── daemon-logs/
    └── video-daemon.log
```

**File Naming Convention**
```
screenshot_{timestamp}_{session_id}.png
metadata_{session_id}.json
```

## Privacy and Security Controls

### Simple Privacy Controls

```rust
pub struct PrivacySettings {
    enabled: bool,
    recording_schedule: TimeRange,    // Work hours only
    notification_interval: u32,       // Remind every N minutes
    blocked_applications: Vec<String>, // Simple app name list
    require_explicit_start: bool,     // No auto-start
}
```

### Privacy Features

- **System Tray Indicator**: Always visible when recording
- **Periodic Notifications**: Remind user every 30 minutes
- **Simple App Blocking**: Basic application name filtering
- **Manual Control**: Explicit start/stop like audio daemon

## CLI Tool: savant-video

### Core Commands (Mirror Audio Pattern)

**Capture Control**
```bash
# Start capture (mirrors savant-transcribe)
savant-video start --interval 30 --duration 3600

# Stop capture
savant-video stop

# Status check
savant-video status
```

**Basic Analysis**
```bash
# Extract text from recent captures
savant-video ocr --since "1 hour ago"

# List recent captures
savant-video list --limit 10

# Search captured text
savant-video search "project alpha"
```

**Privacy and Management**
```bash
# Configure privacy settings
savant-video config --schedule "09:00-17:00"

# Delete old captures
savant-video cleanup --older-than "7 days"

# Export session data
savant-video export --session session-id --format json
```

## Implementation Phases

### Phase 1: Basic Capture (Week 1)
- Create `crates/savant-video/` library following audio patterns
- Periodic screenshot capture using existing Tauri commands
- Basic database storage with segments table
- Simple CLI with start/stop/status commands

### Phase 2: Analysis Integration (Week 2)
- Basic OCR using platform APIs (not external libraries)
- Text search integration with existing FTS5 system
- Change detection using image hashing
- Integration with existing database search

### Phase 3: Daemon Management (Week 3)
- Video daemon scripts following audio daemon patterns
- System tray integration and notifications
- Privacy controls and scheduling
- Log management and status reporting

### Phase 4: Audio Synchronization (Week 4)
- Temporal alignment with audio sessions
- Shared conversation grouping
- Basic multimodal search (text from both sources)
- Export functionality for combined data

## Configuration Management

### System Configuration (Matches Audio Pattern)

```toml
# ~/.config/savant-ai/config.toml
[video]
enabled = false
interval_seconds = 30
quality = "medium"
notify_user = true

[video.privacy]
recording_schedule = "09:00-17:00"
blocked_apps = ["1Password", "Keychain Access"]
require_explicit_start = true
notification_interval = 1800  # 30 minutes

[video.storage]
max_storage_gb = 10
retention_days = 30
cleanup_on_start = true
```

### Runtime Configuration

```rust
// Mirror audio configuration patterns
pub struct VideoConfig {
    pub interval_seconds: u32,
    pub quality: ImageQuality,
    pub privacy: PrivacySettings,
    pub storage: StorageSettings,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 60,
            quality: ImageQuality::Medium,
            privacy: PrivacySettings::default(),
            storage: StorageSettings::default(),
        }
    }
}
```

## Performance Considerations

### Resource Targets (Realistic)

- **CPU Usage**: 5-10% during active capture
- **Memory Usage**: 100-200MB for capture and basic processing
- **Storage Growth**: 50-100MB per hour with PNG compression
- **Network Impact**: Zero - all processing local

### Optimization Strategies

- **Incremental Capture**: Process one screenshot at a time
- **Basic Compression**: PNG with reasonable quality settings
- **Smart Scheduling**: Skip captures during idle periods
- **Background Processing**: Separate capture from analysis

## Integration Points

### Database Extensions

```sql
-- Link video and audio sessions (when Phase 4 ready)
CREATE TABLE IF NOT EXISTS multimodal_sessions (
    id TEXT PRIMARY KEY,
    audio_session_id TEXT,
    video_session_id TEXT,
    sync_offset_seconds INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Tauri Command Extensions

```rust
// Extend existing system commands
#[tauri::command]
pub async fn start_video_capture(
    config: VideoConfig,
) -> Result<String, String> {
    // Use existing screenshot patterns
}

#[tauri::command]
pub async fn get_video_status() -> Result<VideoStatus, String> {
    // Mirror audio status command
}
```

## Success Criteria

### Phase 1 Success
- ✅ Periodic screenshots working reliably
- ✅ Basic database storage functional
- ✅ CLI commands operational
- ✅ Privacy controls implemented

### Phase 2 Success
- ✅ Text extraction from screenshots
- ✅ Search integration working
- ✅ Change detection reducing storage
- ✅ Performance within targets

### Phase 3 Success
- ✅ Daemon management operational
- ✅ System tray integration working
- ✅ Privacy notifications functional
- ✅ Log management in place

### Phase 4 Success
- ✅ Audio-video timeline correlation
- ✅ Combined search functionality
- ✅ Export capabilities working
- ✅ Multimodal analysis basic features

## Risk Mitigation

### Technical Risks
- **Performance Impact**: Start with longer intervals, optimize based on usage
- **Storage Growth**: Implement aggressive cleanup policies
- **Platform Differences**: Leverage existing cross-platform screenshot code
- **Privacy Concerns**: Default to disabled, require explicit opt-in

### Implementation Risks
- **Scope Creep**: Stick to audio system patterns, resist advanced features
- **Complexity**: Maintain simplicity - PNG files and basic analysis only
- **Integration Issues**: Use existing database and search systems
- **User Experience**: Mirror audio daemon UX patterns

This simplified specification provides a realistic foundation for video capture that maintains the project's design principles while ensuring deliverable milestones and manageable complexity. The approach prioritizes proven patterns over innovation, reliability over optimization, and incremental development over comprehensive features.