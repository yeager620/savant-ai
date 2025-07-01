//! macOS-specific audio capture implementation using Core Audio

use crate::{AudioCapture, AudioConfig, AudioStream, StreamControl};
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info};
use async_trait::async_trait;

/// macOS system audio capture using Core Audio APIs
pub struct MacOSSystemCapture {
    running: Arc<AtomicBool>,
}

impl MacOSSystemCapture {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait(?Send)]
impl StreamControl for MacOSSystemCapture {
    async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        info!("Stopped macOS system audio capture");
        Ok(())
    }

    async fn pause(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        debug!("Paused macOS system audio capture");
        Ok(())
    }

    async fn resume(&self) -> Result<()> {
        self.running.store(true, Ordering::Relaxed);
        debug!("Resumed macOS system audio capture");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

/// Start system audio capture on macOS using BlackHole or similar loopback device
pub async fn start_system_audio_capture(config: AudioConfig) -> Result<AudioStream> {
    info!("Starting macOS system audio capture");

    // Check if BlackHole or similar loopback device is available
    if !check_loopback_device_available().await? {
        return Err(anyhow!(
            "System audio capture requires a loopback audio device like BlackHole. Please install BlackHole from: https://github.com/ExistentialAudio/BlackHole"
        ));
    }

    // Use CPAL to capture from the loopback device
    let cpal_capture = crate::capture::CpalAudioCapture::new()?;
    
    // Find BlackHole or similar loopback device
    let devices = cpal_capture.list_devices().await?;
    let loopback_device = devices
        .iter()
        .find(|d| {
            let name_lower = d.name.to_lowercase();
            name_lower.contains("blackhole") 
                || name_lower.contains("loopback")
                || name_lower.contains("soundflower")
                || name_lower.contains("virtual")
        })
        .ok_or_else(|| anyhow!(
            "No loopback device found. Please install BlackHole or configure a loopback device."
        ))?;

    info!("Using loopback device for system audio: {}", loopback_device.name);
    
    // Configure audio capture to use the loopback device
    let mut system_config = config;
    system_config.device_id = Some(loopback_device.id.clone());
    
    // Start capture using CPAL with the loopback device
    let stream = cpal_capture.start_capture(system_config).await?;
    
    // Replace the stream control with our macOS-specific one
    let macos_control = Arc::new(MacOSSystemCapture::new());
    macos_control.running.store(true, Ordering::Relaxed);
    
    Ok(AudioStream::new(stream.receiver, macos_control))
}

/// Check if a loopback device is available for system audio capture
async fn check_loopback_device_available() -> Result<bool> {
    let cpal_capture = crate::capture::CpalAudioCapture::new()?;
    let devices = cpal_capture.list_devices().await?;
    
    let has_loopback = devices.iter().any(|d| {
        let name_lower = d.name.to_lowercase();
        name_lower.contains("blackhole") 
            || name_lower.contains("loopback")
            || name_lower.contains("soundflower")
            || name_lower.contains("virtual")
    });
    
    Ok(has_loopback)
}



/// Request audio capture permissions
pub async fn request_audio_permissions() -> Result<bool> {
    info!("Audio capture permissions will be requested when starting capture");
    Ok(true)
}

/// Get list of available audio devices using CPAL (includes loopback devices)
pub async fn get_core_audio_devices() -> Result<Vec<crate::AudioDevice>> {
    let cpal_capture = crate::capture::CpalAudioCapture::new()?;
    cpal_capture.list_devices().await
}

#[cfg(test)]
mod tests {
    use super::*;

    

    #[tokio::test]
    async fn test_loopback_device_check() {
        let result = check_loopback_device_available().await;
        assert!(result.is_ok());
    }
}