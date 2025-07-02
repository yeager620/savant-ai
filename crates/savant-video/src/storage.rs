use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub base_path: PathBuf,
    pub max_storage_gb: u32,
    pub retention_days: u32,
    pub cleanup_on_start: bool,
}

impl Default for StorageSettings {
    fn default() -> Self {
        let base_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("savant-ai")
            .join("video-captures");

        Self {
            base_path,
            max_storage_gb: 10,
            retention_days: 30,
            cleanup_on_start: true,
        }
    }
}

pub struct StorageManager {
    settings: StorageSettings,
}

impl StorageManager {
    pub fn new(settings: StorageSettings) -> Self {
        Self { settings }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Create base directory
        fs::create_dir_all(&self.settings.base_path)
            .await
            .context("Failed to create video captures directory")?;

        // Create daemon logs directory
        let logs_dir = self.settings.base_path.parent()
            .unwrap_or(Path::new("."))
            .join("daemon-logs");
        fs::create_dir_all(&logs_dir)
            .await
            .context("Failed to create daemon logs directory")?;

        if self.settings.cleanup_on_start {
            self.cleanup_old_files().await?;
        }

        Ok(())
    }

    pub async fn get_session_dir(&self, session_id: &str) -> Result<PathBuf> {
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let session_dir = self.settings.base_path.join(date).join(session_id);
        
        fs::create_dir_all(&session_dir)
            .await
            .context("Failed to create session directory")?;
        
        Ok(session_dir)
    }

    pub async fn save_frame(&self, session_id: &str, frame_data: &[u8]) -> Result<PathBuf> {
        let session_dir = self.get_session_dir(session_id).await?;
        let timestamp = chrono::Utc::now().timestamp_millis();
        let filename = format!("screenshot_{}_{}.png", timestamp, session_id);
        let file_path = session_dir.join(filename);

        fs::write(&file_path, frame_data)
            .await
            .context("Failed to save frame")?;

        Ok(file_path)
    }

    pub async fn save_metadata(&self, session_id: &str, metadata: &serde_json::Value) -> Result<()> {
        let session_dir = self.get_session_dir(session_id).await?;
        let metadata_path = session_dir.join(format!("metadata_{}.json", session_id));

        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(&metadata_path, json)
            .await
            .context("Failed to save metadata")?;

        Ok(())
    }

    pub async fn cleanup_old_files(&self) -> Result<()> {
        let cutoff_date = chrono::Local::now() - chrono::Duration::days(self.settings.retention_days as i64);
        
        let mut entries = fs::read_dir(&self.settings.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    let modified_date: chrono::DateTime<chrono::Local> = modified.into();
                    if modified_date < cutoff_date {
                        if entry.path().is_dir() {
                            fs::remove_dir_all(entry.path()).await?;
                        } else {
                            fs::remove_file(entry.path()).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn get_storage_usage(&self) -> Result<u64> {
        let mut total_size = 0u64;
        self.calculate_dir_size(&self.settings.base_path, &mut total_size).await?;
        Ok(total_size)
    }

    async fn calculate_dir_size(&self, path: &Path, total: &mut u64) -> Result<()> {
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                Box::pin(self.calculate_dir_size(&entry.path(), total)).await?;
            } else {
                *total += metadata.len();
            }
        }
        Ok(())
    }
}