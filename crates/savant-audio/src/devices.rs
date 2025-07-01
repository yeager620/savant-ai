//! Audio device management and enumeration

use crate::{AudioDevice, SampleFormat};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Audio device manager for cross-platform device handling
pub struct DeviceManager {
    cached_devices: Option<Vec<AudioDevice>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            cached_devices: None,
        }
    }

    /// Refresh device cache
    pub async fn refresh(&mut self) -> Result<()> {
        let capture = crate::create_audio_capture()?;
        self.cached_devices = Some(capture.list_devices().await?);
        Ok(())
    }

    /// Get cached devices (refreshes if none cached)
    pub async fn get_devices(&mut self) -> Result<&Vec<AudioDevice>> {
        if self.cached_devices.is_none() {
            self.refresh().await?;
        }
        Ok(self.cached_devices.as_ref().unwrap())
    }

    /// Find device by name
    pub async fn find_device_by_name(&mut self, name: &str) -> Result<Option<AudioDevice>> {
        let devices = self.get_devices().await?;
        Ok(devices.iter().find(|d| d.name == name).cloned())
    }

    /// Get default input device
    pub async fn get_default_input(&mut self) -> Result<Option<AudioDevice>> {
        let devices = self.get_devices().await?;
        Ok(devices.iter().find(|d| d.is_default && d.is_input).cloned())
    }

    /// Get default output device
    pub async fn get_default_output(&mut self) -> Result<Option<AudioDevice>> {
        let devices = self.get_devices().await?;
        Ok(devices.iter().find(|d| d.is_default && d.is_output).cloned())
    }

    /// Filter devices by capability
    pub async fn filter_devices<F>(&mut self, predicate: F) -> Result<Vec<AudioDevice>>
    where
        F: Fn(&AudioDevice) -> bool,
    {
        let devices = self.get_devices().await?;
        Ok(devices.iter().filter(|d| predicate(d)).cloned().collect())
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Device capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub supported_sample_rates: Vec<u32>,
    pub supported_formats: Vec<SampleFormat>,
    pub min_channels: u16,
    pub max_channels: u16,
    pub has_system_capture: bool,
}

impl DeviceCapabilities {
    /// Check if device supports a specific configuration
    pub fn supports_config(&self, sample_rate: u32, format: SampleFormat, channels: u16) -> bool {
        self.supported_sample_rates.contains(&sample_rate)
            && self.supported_formats.contains(&format)
            && channels >= self.min_channels
            && channels <= self.max_channels
    }

    /// Get best supported configuration for Whisper
    pub fn whisper_compatible_config(&self) -> Option<(u32, SampleFormat, u16)> {
        // Prefer 16kHz, F32, mono for Whisper
        if self.supports_config(16000, SampleFormat::F32, 1) {
            return Some((16000, SampleFormat::F32, 1));
        }

        // Try other sample rates that can be resampled
        for &rate in &[48000, 44100, 22050, 8000] {
            if self.supports_config(rate, SampleFormat::F32, 1) {
                return Some((rate, SampleFormat::F32, 1));
            }
        }

        // Try I16 format
        for &rate in &[16000, 48000, 44100, 22050] {
            if self.supports_config(rate, SampleFormat::I16, 1) {
                return Some((rate, SampleFormat::I16, 1));
            }
        }

        None
    }
}

/// Platform-specific device utilities
pub mod platform {
    use super::*;

    #[cfg(target_os = "macos")]
    pub mod macos {
        use super::*;

        /// Check if audio permissions are granted
        pub async fn check_audio_permissions() -> Result<bool> {
            // This would check macOS audio permissions
            // Implementation would use Core Audio APIs
            Ok(true) // Placeholder
        }

        /// Request audio permissions
        pub async fn request_audio_permissions() -> Result<bool> {
            // This would request macOS audio permissions
            // Implementation would show system permission dialog
            Ok(true) // Placeholder
        }

        /// Check if system audio capture is available (macOS 14.4+)
        pub fn supports_system_capture() -> bool {
            // Check macOS version and API availability
            true // Placeholder - would check actual OS version
        }
    }

    #[cfg(target_os = "windows")]
    pub mod windows {
        use super::*;

        /// Check if Windows audio session APIs are available
        pub fn supports_system_capture() -> bool {
            true // Most Windows versions support this
        }

        /// Find stereo mix or similar loopback device
        pub async fn find_loopback_device() -> Result<Option<AudioDevice>> {
            // Implementation would search for "Stereo Mix" or "What U Hear"
            Ok(None) // Placeholder
        }
    }

    #[cfg(target_os = "linux")]
    pub mod linux {
        use super::*;

        /// Check for PulseAudio monitor devices
        pub async fn find_monitor_devices() -> Result<Vec<AudioDevice>> {
            // Implementation would find PulseAudio monitor sources
            Ok(vec![]) // Placeholder
        }

        /// Check if PipeWire is available (better for modern Linux)
        pub fn has_pipewire() -> bool {
            false // Placeholder
        }
    }
}