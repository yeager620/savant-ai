use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub enabled: bool,
    pub recording_schedule: Option<crate::config::TimeRange>,
    pub notification_interval: u32, // seconds
    pub blocked_applications: HashSet<String>,
    pub require_explicit_start: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        let mut blocked_apps = HashSet::new();
        blocked_apps.insert("1Password".to_string());
        blocked_apps.insert("Keychain Access".to_string());
        blocked_apps.insert("System Preferences".to_string());
        blocked_apps.insert("Security & Privacy".to_string());

        Self {
            enabled: true, // Enable for testing
            recording_schedule: None,
            notification_interval: 1800, // 30 minutes
            blocked_applications: blocked_apps,
            require_explicit_start: false, // Allow immediate capture
        }
    }
}

pub struct PrivacyController {
    settings: PrivacySettings,
    last_notification: Option<std::time::Instant>,
}

impl PrivacyController {
    pub fn new(settings: PrivacySettings) -> Self {
        Self {
            settings,
            last_notification: None,
        }
    }

    pub fn should_capture(&self, active_app: Option<&str>) -> bool {
        if !self.settings.enabled {
            return false;
        }

        // Check if app is blocked
        if let Some(app) = active_app {
            if self.settings.blocked_applications.contains(app) {
                return false;
            }
        }

        // Check schedule
        if let Some(schedule) = &self.settings.recording_schedule {
            let now = chrono::Local::now();
            let hour = now.hour() as u8;
            let minute = now.minute() as u8;
            if !schedule.is_within_range(hour, minute) {
                return false;
            }
        }

        true
    }

    pub fn should_notify(&mut self) -> bool {
        if self.settings.notification_interval == 0 {
            return false;
        }

        let now = std::time::Instant::now();
        match self.last_notification {
            None => {
                self.last_notification = Some(now);
                true
            }
            Some(last) => {
                if now.duration_since(last).as_secs() >= self.settings.notification_interval as u64 {
                    self.last_notification = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn is_sensitive_content(&self, _text: Option<&str>) -> bool {
        // TODO: Implement basic PII detection
        // For now, just return false
        false
    }
}