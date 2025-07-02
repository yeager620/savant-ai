use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use indexmap::IndexMap;

use crate::{VideoEvent, AudioEvent, SyncWindow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineEvent {
    Video(VideoEvent),
    Audio(AudioEvent),
    Sync(SyncEvent),
    System(SystemEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub sync_type: SyncEventType,
    pub affected_events: Vec<String>,
    pub sync_quality: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEventType {
    WindowProcessed,
    CorrelationDetected,
    InsightGenerated,
    SyncIssueDetected,
    TimelineCompacted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: SystemEventType,
    pub metadata: SystemEventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEventType {
    SessionStarted,
    SessionEnded,
    ConfigurationChanged,
    ErrorOccurred,
    PerformanceMetrics,
    DataCompaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEventMetadata {
    pub session_id: Option<String>,
    pub error_message: Option<String>,
    pub performance_data: Option<PerformanceMetrics>,
    pub config_changes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub processing_time_ms: u64,
    pub memory_usage_mb: f32,
    pub events_processed: usize,
    pub correlations_found: usize,
    pub sync_quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Video,
    Audio,
    Sync,
    System,
}

pub struct TimelineManager {
    events: BTreeMap<DateTime<Utc>, Vec<TimelineEvent>>,
    event_index: IndexMap<String, DateTime<Utc>>,
    retention_policy: RetentionPolicy,
    compaction_config: CompactionConfig,
    last_compaction: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub max_timeline_duration: Duration,
    pub max_events_per_type: usize,
    pub auto_cleanup_enabled: bool,
    pub cleanup_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct CompactionConfig {
    pub enabled: bool,
    pub compaction_threshold: usize,
    pub preserve_correlations: bool,
    pub min_event_interval: Duration,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_timeline_duration: Duration::hours(24),
            max_events_per_type: 10000,
            auto_cleanup_enabled: true,
            cleanup_interval: Duration::hours(1),
        }
    }
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            compaction_threshold: 5000,
            preserve_correlations: true,
            min_event_interval: Duration::milliseconds(100),
        }
    }
}

impl TimelineManager {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
            event_index: IndexMap::new(),
            retention_policy: RetentionPolicy::default(),
            compaction_config: CompactionConfig::default(),
            last_compaction: None,
        }
    }

    pub fn with_config(retention: RetentionPolicy, compaction: CompactionConfig) -> Self {
        Self {
            events: BTreeMap::new(),
            event_index: IndexMap::new(),
            retention_policy: retention,
            compaction_config: compaction,
            last_compaction: None,
        }
    }

    pub async fn add_event(&mut self, event: TimelineEvent) -> Result<()> {
        let timestamp = self.get_event_timestamp(&event);
        let event_id = self.get_event_id(&event);

        // Check for duplicate events
        if self.event_index.contains_key(&event_id) {
            return Ok(()); // Skip duplicate
        }

        // Add to timeline
        self.events.entry(timestamp).or_default().push(event);
        self.event_index.insert(event_id, timestamp);

        // Check if cleanup/compaction is needed
        self.maybe_cleanup().await?;
        self.maybe_compact().await?;

        Ok(())
    }

    pub async fn get_events_in_window(&self, window: &SyncWindow) -> Result<Vec<TimelineEvent>> {
        let mut events = Vec::new();

        for (timestamp, event_list) in self.events.range(window.start_time..=window.end_time) {
            if window.contains_timestamp(timestamp) {
                events.extend(event_list.clone());
            }
        }

        Ok(events)
    }

    pub async fn get_video_events_in_window(&self, window: &SyncWindow) -> Result<Vec<VideoEvent>> {
        let events = self.get_events_in_window(window).await?;
        let video_events = events.into_iter()
            .filter_map(|e| match e {
                TimelineEvent::Video(video_event) => Some(video_event),
                _ => None,
            })
            .collect();

        Ok(video_events)
    }

    pub async fn get_audio_events_in_window(&self, window: &SyncWindow) -> Result<Vec<AudioEvent>> {
        let events = self.get_events_in_window(window).await?;
        let audio_events = events.into_iter()
            .filter_map(|e| match e {
                TimelineEvent::Audio(audio_event) => Some(audio_event),
                _ => None,
            })
            .collect();

        Ok(audio_events)
    }

    pub async fn get_events_by_type(&self, event_type: EventType, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<TimelineEvent>> {
        let mut events = Vec::new();

        for (timestamp, event_list) in self.events.range(start..=end) {
            for event in event_list {
                if self.matches_event_type(event, &event_type) {
                    events.push(event.clone());
                }
            }
        }

        Ok(events)
    }

    pub async fn get_event_by_id(&self, event_id: &str) -> Result<Option<TimelineEvent>> {
        if let Some(timestamp) = self.event_index.get(event_id) {
            if let Some(event_list) = self.events.get(timestamp) {
                for event in event_list {
                    if self.get_event_id(event) == event_id {
                        return Ok(Some(event.clone()));
                    }
                }
            }
        }
        Ok(None)
    }

    pub async fn get_recent_events_count(&self, duration: Duration) -> Result<usize> {
        let start_time = Utc::now() - duration;
        let mut count = 0;

        for (timestamp, event_list) in self.events.range(start_time..) {
            count += event_list.len();
        }

        Ok(count)
    }

    pub async fn get_timeline_stats(&self) -> Result<TimelineStats> {
        let mut stats = TimelineStats::default();

        for (_, event_list) in &self.events {
            for event in event_list {
                stats.total_events += 1;
                match event {
                    TimelineEvent::Video(_) => stats.video_events += 1,
                    TimelineEvent::Audio(_) => stats.audio_events += 1,
                    TimelineEvent::Sync(_) => stats.sync_events += 1,
                    TimelineEvent::System(_) => stats.system_events += 1,
                }
            }
        }

        if let Some((earliest, _)) = self.events.first_key_value() {
            stats.earliest_timestamp = Some(*earliest);
        }

        if let Some((latest, _)) = self.events.last_key_value() {
            stats.latest_timestamp = Some(*latest);
        }

        stats.memory_usage_estimate = self.estimate_memory_usage();

        Ok(stats)
    }

    async fn maybe_cleanup(&mut self) -> Result<()> {
        if !self.retention_policy.auto_cleanup_enabled {
            return Ok(());
        }

        let now = Utc::now();
        let cutoff_time = now - self.retention_policy.max_timeline_duration;

        // Remove old events
        let keys_to_remove: Vec<DateTime<Utc>> = self.events
            .range(..cutoff_time)
            .map(|(k, _)| *k)
            .collect();

        for key in keys_to_remove {
            if let Some(events) = self.events.remove(&key) {
                // Remove from index
                for event in events {
                    let event_id = self.get_event_id(&event);
                    self.event_index.remove(&event_id);
                }
            }
        }

        Ok(())
    }

    async fn maybe_compact(&mut self) -> Result<()> {
        if !self.compaction_config.enabled {
            return Ok(());
        }

        let total_events = self.event_index.len();
        if total_events < self.compaction_config.compaction_threshold {
            return Ok(());
        }

        // Simple compaction: merge events that are very close in time
        let mut compacted_events = BTreeMap::new();
        let mut compacted_index = IndexMap::new();

        for (timestamp, events) in &self.events {
            // Find the appropriate compaction bucket
            let bucket_timestamp = self.get_compaction_bucket(*timestamp);
            
            compacted_events.entry(bucket_timestamp).or_insert_with(Vec::new).extend(events.clone());
            
            for event in events {
                let event_id = self.get_event_id(event);
                compacted_index.insert(event_id, bucket_timestamp);
            }
        }

        self.events = compacted_events;
        self.event_index = compacted_index;
        self.last_compaction = Some(Utc::now());

        Ok(())
    }

    fn get_compaction_bucket(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        let interval_ms = self.compaction_config.min_event_interval.num_milliseconds();
        let bucket_ms = (timestamp.timestamp_millis() / interval_ms) * interval_ms;
        DateTime::from_timestamp(bucket_ms / 1000, ((bucket_ms % 1000) * 1_000_000) as u32).unwrap_or(timestamp)
    }

    fn get_event_timestamp(&self, event: &TimelineEvent) -> DateTime<Utc> {
        match event {
            TimelineEvent::Video(e) => e.timestamp,
            TimelineEvent::Audio(e) => e.timestamp,
            TimelineEvent::Sync(e) => e.timestamp,
            TimelineEvent::System(e) => e.timestamp,
        }
    }

    fn get_event_id(&self, event: &TimelineEvent) -> String {
        match event {
            TimelineEvent::Video(e) => e.event_id.clone(),
            TimelineEvent::Audio(e) => e.event_id.clone(),
            TimelineEvent::Sync(e) => e.event_id.clone(),
            TimelineEvent::System(e) => e.event_id.clone(),
        }
    }

    fn matches_event_type(&self, event: &TimelineEvent, event_type: &EventType) -> bool {
        match (event, event_type) {
            (TimelineEvent::Video(_), EventType::Video) => true,
            (TimelineEvent::Audio(_), EventType::Audio) => true,
            (TimelineEvent::Sync(_), EventType::Sync) => true,
            (TimelineEvent::System(_), EventType::System) => true,
            _ => false,
        }
    }

    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate of memory usage
        let events_size = self.events.len() * std::mem::size_of::<(DateTime<Utc>, Vec<TimelineEvent>)>();
        let index_size = self.event_index.len() * std::mem::size_of::<(String, DateTime<Utc>)>();
        
        // Rough estimate of event content size
        let avg_event_size = 500; // bytes
        let total_events: usize = self.events.values().map(|v| v.len()).sum();
        let content_size = total_events * avg_event_size;

        events_size + index_size + content_size
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimelineStats {
    pub total_events: usize,
    pub video_events: usize,
    pub audio_events: usize,
    pub sync_events: usize,
    pub system_events: usize,
    pub earliest_timestamp: Option<DateTime<Utc>>,
    pub latest_timestamp: Option<DateTime<Utc>>,
    pub memory_usage_estimate: usize,
}