//! # Savant Audio Capture Library
//!
//! Cross-platform audio capture library with support for:
//! - Microphone input capture
//! - System audio capture (macOS 14.4+, Windows, Linux)
//! - Real-time audio streaming
//! - Multiple audio device management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use async_trait::async_trait;

pub mod capture;
pub mod devices;
pub mod formats;

#[cfg(target_os = "macos")]
pub mod macos;

pub use capture::*;
pub use devices::*;
pub use formats::*;

/// Audio sample rate (16kHz for Whisper compatibility)
pub const SAMPLE_RATE: u32 = 16000;

/// Audio channels (mono for Whisper compatibility)
pub const CHANNELS: u16 = 1;

/// Audio sample format
pub const SAMPLE_FORMAT: SampleFormat = SampleFormat::F32;

/// Audio sample format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat {
    I16,
    F32,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_default: bool,
    pub sample_rates: Vec<u32>,
    pub channels: u16,
}

/// Audio capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub device_id: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub format: SampleFormat,
    pub capture_system_audio: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device_id: None,
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            buffer_size: 4096,
            format: SAMPLE_FORMAT,
            capture_system_audio: false,
        }
    }
}

/// Audio sample with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSample {
    pub data: Vec<f32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Audio capture stream handle
pub struct AudioStream {
    pub receiver: mpsc::Receiver<AudioSample>,
    handle: Arc<dyn StreamControl>,
}

impl AudioStream {
    pub fn new(receiver: mpsc::Receiver<AudioSample>, handle: Arc<dyn StreamControl>) -> Self {
        Self { receiver, handle }
    }

    pub async fn stop(self) -> Result<()> {
        self.handle.stop().await
    }
}

/// Stream control interface
#[async_trait(?Send)]
pub trait StreamControl {
    async fn stop(&self) -> Result<()>;
    async fn pause(&self) -> Result<()>;
    async fn resume(&self) -> Result<()>;
    fn is_running(&self) -> bool;
}

/// Main audio capture interface
#[async_trait(?Send)]
pub trait AudioCapture {
    /// List available audio devices
    async fn list_devices(&self) -> Result<Vec<AudioDevice>>;

    /// Get default input device
    async fn default_input_device(&self) -> Result<Option<AudioDevice>>;

    /// Get default output device (for system audio capture)
    async fn default_output_device(&self) -> Result<Option<AudioDevice>>;

    /// Start audio capture stream
    async fn start_capture(&self, config: AudioConfig) -> Result<AudioStream>;

    /// Start system audio capture (requires permissions)
    async fn start_system_capture(&self, config: AudioConfig) -> Result<AudioStream>;
}

/// Create platform-specific audio capture instance
pub fn create_audio_capture() -> Result<Box<dyn AudioCapture>> {
    Ok(Box::new(capture::CpalAudioCapture::new()?))
}

/// Utility function to convert audio to Whisper-compatible format
pub fn prepare_for_whisper(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if source_rate == target_rate {
        return samples.to_vec();
    }

    // Simple linear interpolation resampling
    let ratio = source_rate as f64 / target_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let pos = i as f64 * ratio;
        let index = pos as usize;
        
        if index + 1 < samples.len() {
            let frac = pos - index as f64;
            let sample = samples[index] * (1.0 - frac as f32) + samples[index + 1] * frac as f32;
            output.push(sample);
        } else if index < samples.len() {
            output.push(samples[index]);
        }
    }

    output
}

/// Audio buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioBufferConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub max_duration_seconds: f32,
}

/// Audio buffer for accumulating samples
pub struct AudioBuffer {
    config: AudioBufferConfig,
    samples: Vec<f32>,
    max_samples: usize,
}

impl AudioBuffer {
    pub fn new(config: AudioBufferConfig) -> Self {
        let max_samples = (config.sample_rate as f32 * config.max_duration_seconds * config.channels as f32) as usize;
        Self {
            config,
            samples: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn push(&mut self, sample: &AudioSample) {
        // Ensure we don't exceed max capacity
        let remaining_capacity = self.max_samples.saturating_sub(self.samples.len());
        let samples_to_add = sample.data.len().min(remaining_capacity);
        
        self.samples.extend_from_slice(&sample.data[..samples_to_add]);
    }

    pub fn get_sample(&self) -> AudioSample {
        AudioSample {
            data: self.samples.clone(),
            timestamp: chrono::Utc::now(),
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
        }
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}