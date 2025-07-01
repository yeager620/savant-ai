Video Capture System Specification

  Executive Summary

  This specification outlines a system-level video capture daemon that parallels the existing audio capture system, implementing continuous screen recording with intelligent compression, privacy controls, and seamless integration with the
  audio transcription pipeline for multimodal analysis.

  Design Principles Alignment

  UNIX Philosophy Implementation

  - Single Purpose: Video capture daemon does one thing well - continuous screen recording
  - Composability: Pipes with existing audio pipeline and database systems
  - JSON I/O: Structured metadata output for integration
  - Independence: Operates standalone or integrated with main application

  Privacy-First Architecture

  - Local Processing: All video analysis happens on-device
  - Selective Capture: Configurable application/window filtering
  - Automatic Redaction: Sensitive content detection and masking
  - User Consent: Explicit recording permissions and notifications

  Performance Requirements

  - Lightweight Storage: Efficient compression without quality loss for analysis
  - Real-time Processing: Frame-by-frame analysis without blocking capture
  - Resource Management: CPU/memory limits to avoid system impact
  - Scalable Storage: Automatic cleanup and archival strategies

  System Architecture

  Core Components

  Video Capture Pipeline:
  Screen → Frame Capture → Compression → Analysis → Storage → Indexing
     ↓         ↓             ↓            ↓         ↓         ↓
  Raw Pixels  RGB/YUV    WebP/AV1    OCR/Motion  SQLite   Timeline

  Integration with Existing Systems

  Audio Daemon → Transcription → Database
       ↓              ↓             ↓
  Video Daemon → Frame Analysis → Synchronized Storage
       ↓              ↓             ↓
  Combined Data → Multimodal Analysis → Unified Timeline

  Technical Specifications

  Platform-Specific Capture Methods

  macOS Implementation

  - Core Graphics: CGDisplayCreateImage for full screen capture
  - Screen Capture Kit: iOS 15+ API for efficient window-level capture
  - Accessibility API: Integration with existing browser monitoring
  - Privacy Controls: System permission integration

  Windows Implementation

  - DirectX/DXGI: Hardware-accelerated screen capture
  - Windows Graphics Capture API: Modern capture with window isolation
  - GDI Fallback: Legacy system compatibility
  - UWP Integration: Modern Windows app compatibility

  Linux Implementation

  - X11/Xorg: Traditional X11 screen capture
  - Wayland: Modern compositor integration
  - DRM/KMS: Direct hardware access for efficiency
  - Portal Integration: Flatpak/Snap compatibility

  Frame Capture Strategy

  Intelligent Frame Rate Adaptation

  pub struct AdaptiveFrameRate {
      base_fps: u8,                    // 1-5 FPS baseline
      motion_threshold: f32,           // 0.05 change detection
      activity_boost_fps: u8,          // 10-15 FPS during activity
      idle_reduction_fps: u8,          // 0.5 FPS during idle
      window_focus_multiplier: f32,    // 2x for active applications
  }

  Motion Detection Algorithm

  - Frame Differencing: Compare consecutive frames for change detection
  - Region of Interest: Focus capture on changed screen areas
  - Activity Classification: Distinguish typing, scrolling, video playback
  - Adaptive Quality: Higher compression for static content

  Compression and Storage

  Video Codec Selection

  Primary: WebP Animation
  - Lossless compression for text/UI elements
  - ~70% size reduction vs PNG sequences
  - Frame-by-frame access for analysis
  - Wide tooling support

  Secondary: AV1 with Keyframe Strategy
  - Ultra-efficient compression for video content
  - Keyframes every 30 frames for analysis access
  - Hardware acceleration on modern systems
  - Future-proof codec selection

  Storage Architecture

  -- Video segments table
  CREATE TABLE video_segments (
      id UUID PRIMARY KEY,
      session_id UUID NOT NULL,
      start_time TIMESTAMPTZ NOT NULL,
      end_time TIMESTAMPTZ NOT NULL,
      file_path TEXT NOT NULL,
      codec TEXT NOT NULL,
      resolution TEXT NOT NULL,           -- "1920x1080"
      frame_count INTEGER NOT NULL,
      file_size_bytes INTEGER NOT NULL,
      compression_ratio REAL,
      motion_score REAL,                  -- 0-1 activity level
      applications JSON,                  -- Active apps during segment
      created_at TIMESTAMPTZ DEFAULT NOW()
  );

  -- Frame-level metadata for analysis
  CREATE TABLE video_frames (
      id UUID PRIMARY KEY,
      segment_id UUID REFERENCES video_segments(id),
      frame_number INTEGER NOT NULL,
      timestamp TIMESTAMPTZ NOT NULL,
      motion_delta REAL,                  -- Change from previous frame
      ocr_text TEXT,                      -- Extracted text content
      dominant_application TEXT,          -- Primary app in focus
      screenshot_hash TEXT,               -- Duplicate detection
      analysis_metadata JSON             -- OCR coordinates, UI elements
  );

  Privacy and Security Controls

  Application-Level Filtering

  pub struct PrivacyControls {
      blocked_applications: HashSet<String>,     // Never record these apps
      secure_applications: HashSet<String>,      // Banking, password managers
      redaction_patterns: Vec<Regex>,           // Credit cards, SSNs, etc.
      recording_schedule: TimeRanges,           // Work hours only
      user_consent_required: bool,              // Explicit recording permission
  }

  Automatic Content Redaction

  - PII Detection: Credit cards, social security numbers, phone numbers
  - Password Field Recognition: Input field type detection
  - Secure App Blacklisting: Banking, password managers, private browsing
  - User-Defined Sensitive Areas: Manual screen region blocking

  Recording Notifications

  - System Tray Indicator: Always-visible recording status
  - Periodic Notifications: Hourly recording reminders
  - Activity Summaries: Daily/weekly recording statistics
  - Privacy Dashboard: Recorded content review and deletion

  Synchronization with Audio Pipeline

  Temporal Alignment

  pub struct SynchronizedCapture {
      audio_session_id: UUID,
      video_session_id: UUID,
      sync_offset_ms: i64,           // Audio-video time difference
      shared_timeline: Timeline,      // Unified event timeline
      correlation_confidence: f32,    // Sync accuracy score
  }

  Multimodal Event Detection

  - Screen Activity Correlation: Match screen changes with audio events
  - Application Context: Link browser tabs with audio transcription
  - Meeting Detection: Identify video calls with participant audio
  - Workflow Analysis: Screen actions + voice commands correlation

  Storage Optimization

  Hierarchical Storage Management

  pub struct StorageStrategy {
      hot_storage_days: u32,         // 7 days - high frame rate, instant access
      warm_storage_days: u32,        // 30 days - reduced frame rate, compressed
      cold_storage_days: u32,        // 365 days - keyframes only, archived
      analysis_cache_frames: u32,    // Recent frames kept uncompressed
  }

  Intelligent Compression Levels

  - Text/UI Content: Lossless compression for readability
  - Video Content: Lossy compression for size optimization
  - Static Screens: Single frame + duration for efficiency
  - High Activity: Reduced compression for motion clarity

  Analysis and Indexing

  Computer Vision Pipeline

  pub struct FrameAnalysis {
      ocr_engine: TesseractEngine,           // Text extraction
      ui_detector: UIElementDetector,        // Buttons, menus, forms
      motion_analyzer: MotionAnalyzer,       // Activity classification
      object_detector: ObjectDetector,       // Optional: YOLO for objects
      face_detector: Option<FaceDetector>,   // Optional: Meeting analysis
  }

  Content Indexing Strategy

  - OCR Text Extraction: Full-text search across all screen content
  - UI Element Recognition: Button clicks, form submissions, navigation
  - Application State Tracking: Window focus, tab changes, app launches
  - Activity Classification: Work, entertainment, communication, browsing

  Search and Query Capabilities

  // CLI interface for video analysis
  savant-video search "project alpha discussion" --timeframe "2025-07-01"
  savant-video analyze --session video-session-id --extract-text
  savant-video timeline --sync-audio audio-session-id
  savant-video export --format webp --start "10:30" --duration "00:05:00"

  CLI Tool: savant-video

  Core Commands

  Capture Control

  # Start continuous capture
  savant-video start --quality medium --fps adaptive

  # Stop capture and process
  savant-video stop --analyze

  # Status and statistics
  savant-video status
  savant-video stats --timeframe today

  Analysis Operations

  # Extract text from video timeline
  savant-video ocr --session session-id --output transcript.txt

  # Analyze application usage patterns
  savant-video analyze apps --timeframe week

  # Generate activity timeline
  savant-video timeline --sync-audio --format json

  # Search across recorded content
  savant-video search "github repository" --limit 10

  Privacy and Management

  # Configure privacy settings
  savant-video privacy --block-app "1Password" --add

  # Review and delete sensitive content
  savant-video review --flagged-content
  savant-video delete --session session-id

  # Export for external analysis
  savant-video export --session session-id --format frames

  Integration Commands

  # Synchronize with audio transcription
  savant-video sync --audio-session audio-id --video-session video-id

  # Combined multimodal analysis
  savant-video correlate --audio-db --output insights.json

  # Pipeline integration
  savant-video stream | savant-analyze-frames | savant-db store-visual

  Configuration Management

  System Configuration

  # ~/.config/savant-ai/video-config.toml
  [capture]
  enabled = true
  default_fps = 2
  adaptive_fps = true
  max_fps = 15
  resolution = "native"
  codec = "webp"

  [privacy]
  recording_schedule = "09:00-17:00"
  blocked_apps = ["1Password", "Keychain Access", "Private Browsing"]
  require_consent = true
  notification_interval = 3600  # seconds

  [storage]
  max_storage_gb = 50
  retention_days = 90
  hot_storage_days = 7
  compression_level = "medium"

  [analysis]
  ocr_enabled = true
  motion_detection = true
  ui_analysis = true
  face_detection = false

  Runtime Controls

  pub struct VideoCaptureDaemon {
      config: VideoConfig,
      privacy_controller: PrivacyController,
      capture_engine: CaptureEngine,
      storage_manager: StorageManager,
      analysis_pipeline: AnalysisPipeline,
      sync_coordinator: SyncCoordinator,
  }

  Performance Considerations

  Resource Management

  - CPU Usage: Target <5% average CPU utilization
  - Memory Usage: <200MB RAM for capture and compression
  - Storage Growth: ~100MB per hour with medium compression
  - Network Impact: Zero - all processing local

  Optimization Strategies

  - Hardware Acceleration: GPU-assisted compression when available
  - Incremental Compression: Process frames as captured, not batch
  - Smart Caching: Keep recent frames uncompressed for analysis
  - Background Processing: Analysis pipeline separate from capture

  Scalability Limits

  - Concurrent Sessions: Support multiple monitor capture
  - Storage Scaling: Automatic archival to external storage
  - Analysis Queuing: Background processing queue for compute-heavy tasks
  - Memory Management: Streaming processing for large video segments

  Integration Points

  Database Schema Extensions

  -- Link video and audio sessions
  CREATE TABLE multimodal_sessions (
      id UUID PRIMARY KEY,
      audio_session_id UUID REFERENCES audio_sessions(id),
      video_session_id UUID REFERENCES video_sessions(id),
      sync_offset_ms INTEGER,
      correlation_score REAL,
      created_at TIMESTAMPTZ DEFAULT NOW()
  );

  -- Combined timeline events
  CREATE TABLE timeline_events (
      id UUID PRIMARY KEY,
      session_id UUID,
      event_type TEXT,                    -- 'audio_segment', 'screen_change', 'app_switch'
      timestamp TIMESTAMPTZ NOT NULL,
      content_reference TEXT,             -- File path or database reference
      metadata JSON,
      confidence_score REAL
  );

  Audio-Video Correlation

  pub struct MultimodalAnalyzer {
      audio_transcripts: Vec<AudioSegment>,
      video_frames: Vec<VideoFrame>,
      correlation_engine: CorrelationEngine,
  }

  impl MultimodalAnalyzer {
      pub fn correlate_events(&self) -> Vec<CorrelatedEvent> {
          // Match screen activity with audio transcription
          // Identify context switches (app changes + conversation topics)
          // Generate unified timeline with confidence scores
      }
  }

  Unified CLI Interface

  # Combined audio-video analysis
  savant analyze multimodal --session combined-session-id
  savant search "project meeting" --include-audio --include-video
  savant export --session session-id --format unified-json

  Implementation Phases

  Phase 1: Core Capture (Weeks 1-2)

  - Platform-specific screen capture implementation
  - Basic WebP frame storage
  - Privacy controls and application filtering
  - CLI interface for start/stop/status

  Phase 2: Analysis Pipeline (Weeks 3-4)

  - OCR text extraction from frames
  - Motion detection and activity classification
  - Database schema and storage implementation
  - Search functionality across captured content

  Phase 3: Audio Synchronization (Weeks 5-6)

  - Temporal alignment with audio transcription
  - Multimodal event correlation
  - Unified timeline generation
  - Combined analysis tools

  Phase 4: Advanced Features (Weeks 7-8)

  - Adaptive compression and quality optimization
  - UI element recognition and interaction tracking
  - Advanced privacy controls and content redaction
  - Performance optimization and resource management

  Security and Compliance

  Data Protection

  - Encryption at Rest: AES-256 encryption for stored video files
  - Access Controls: File system permissions and application-level security
  - Audit Logging: Complete access logs for compliance
  - Data Minimization: Configurable retention and automatic deletion

  Privacy Compliance

  - User Consent: Explicit opt-in for video recording
  - Transparency: Clear recording indicators and activity logs
  - Data Portability: Export functionality for user data ownership
  - Right to Deletion: Complete data removal capabilities

  Technical Security

  - Input Validation: Sanitize all user inputs and configuration
  - Memory Safety: Rust's memory safety for capture and analysis code
  - Sandboxing: Isolated processes for video analysis
  - Secure Defaults: Privacy-first default configuration

  This specification provides a comprehensive foundation for implementing system-level video capture that maintains the project's design principles while enabling powerful multimodal analysis capabilities. The modular architecture ensures
  each component can be developed and tested independently while integrating seamlessly with the existing audio transcription and database systems.
