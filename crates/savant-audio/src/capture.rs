//! Audio capture implementation using CPAL

use crate::{AudioCapture, AudioConfig, AudioDevice, AudioSample, AudioStream, SampleFormat, StreamControl};
use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub struct CpalAudioCapture {
    host: Host,
}

impl CpalAudioCapture {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        info!("Initialized CPAL audio capture with host: {}", host.id().name());
        Ok(Self { host })
    }

    fn device_to_audio_device(&self, device: &Device, is_default_input: bool, is_default_output: bool) -> Result<AudioDevice> {
        let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
        
        // Get supported input configurations
        let input_configs: Vec<_> = device
            .supported_input_configs()
            .map(|configs| configs.collect())
            .unwrap_or_default();
        
        // Get supported output configurations  
        let output_configs: Vec<_> = device
            .supported_output_configs()
            .map(|configs| configs.collect())
            .unwrap_or_default();

        let has_input = !input_configs.is_empty();
        let has_output = !output_configs.is_empty();

        // Extract sample rates (prefer input configs for microphones)
        let sample_rates: Vec<u32> = if has_input {
            input_configs
                .iter()
                .flat_map(|config| {
                    vec![config.min_sample_rate().0, config.max_sample_rate().0]
                })
                .collect()
        } else {
            output_configs
                .iter()
                .flat_map(|config| {
                    vec![config.min_sample_rate().0, config.max_sample_rate().0]
                })
                .collect()
        };

        // Get max channels
        let channels = input_configs
            .iter()
            .chain(output_configs.iter())
            .map(|config| config.channels())
            .max()
            .unwrap_or(2);

        Ok(AudioDevice {
            id: name.clone(), // CPAL doesn't provide stable IDs, use name
            name,
            is_input: has_input,
            is_output: has_output,
            is_default: is_default_input || is_default_output,
            sample_rates,
            channels,
        })
    }
}

#[async_trait::async_trait]
impl AudioCapture for CpalAudioCapture {
    async fn list_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();

        let default_input = self.host.default_input_device();
        let default_output = self.host.default_output_device();

        let input_devices: Vec<_> = self.host
            .input_devices()
            .map_err(|e| anyhow!("Failed to enumerate input devices: {}", e))?
            .collect();

        let output_devices: Vec<_> = self.host
            .output_devices()
            .map_err(|e| anyhow!("Failed to enumerate output devices: {}", e))?
            .collect();

        // Process input devices
        for device in input_devices {
            let is_default = default_input
                .as_ref()
                .map(|d| d.name().unwrap_or_default() == device.name().unwrap_or_default())
                .unwrap_or(false);

            if let Ok(audio_device) = self.device_to_audio_device(&device, is_default, false) {
                devices.push(audio_device);
            }
        }

        // Process output devices (for system audio capture)
        for device in output_devices {
            let device_name = device.name().unwrap_or_default();
            
            // Skip if we already have this device from inputs
            if devices.iter().any(|d| d.name == device_name) {
                continue;
            }

            let is_default = default_output
                .as_ref()
                .map(|d| d.name().unwrap_or_default() == device_name)
                .unwrap_or(false);

            if let Ok(audio_device) = self.device_to_audio_device(&device, false, is_default) {
                devices.push(audio_device);
            }
        }

        info!("Found {} audio devices", devices.len());
        Ok(devices)
    }

    async fn default_input_device(&self) -> Result<Option<AudioDevice>> {
        if let Some(device) = self.host.default_input_device() {
            let audio_device = self.device_to_audio_device(&device, true, false)?;
            Ok(Some(audio_device))
        } else {
            Ok(None)
        }
    }

    async fn default_output_device(&self) -> Result<Option<AudioDevice>> {
        if let Some(device) = self.host.default_output_device() {
            let audio_device = self.device_to_audio_device(&device, false, true)?;
            Ok(Some(audio_device))
        } else {
            Ok(None)
        }
    }

    async fn start_capture(&self, config: AudioConfig) -> Result<AudioStream> {
        let device = if let Some(device_name) = &config.device_id {
            // Find device by name
            self.host
                .input_devices()
                .map_err(|e| anyhow!("Failed to enumerate devices: {}", e))?
                .find(|d| d.name().unwrap_or_default() == *device_name)
                .ok_or_else(|| anyhow!("Device not found: {}", device_name))?
        } else {
            // Use default input device
            self.host
                .default_input_device()
                .ok_or_else(|| anyhow!("No default input device available"))?
        };

        debug!("Using audio device: {}", device.name().unwrap_or_default());

        // Configure stream
        let stream_config = StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        let (tx, rx) = mpsc::channel(100);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Create stream based on format
        let stream = match config.format {
            SampleFormat::F32 => {
                let tx = tx.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if running_clone.load(Ordering::Relaxed) {
                            let sample = AudioSample {
                                data: data.to_vec(),
                                timestamp: chrono::Utc::now(),
                                sample_rate: config.sample_rate,
                                channels: config.channels,
                            };
                            
                            if let Err(e) = tx.try_send(sample) {
                                warn!("Failed to send audio sample: {}", e);
                            }
                        }
                    },
                    |err| error!("Audio stream error: {}", err),
                    None,
                )?
            }
            SampleFormat::I16 => {
                let tx = tx.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if running_clone.load(Ordering::Relaxed) {
                            // Convert i16 to f32
                            let f32_data: Vec<f32> = data
                                .iter()
                                .map(|&sample| sample as f32 / i16::MAX as f32)
                                .collect();

                            let sample = AudioSample {
                                data: f32_data,
                                timestamp: chrono::Utc::now(),
                                sample_rate: config.sample_rate,
                                channels: config.channels,
                            };
                            
                            if let Err(e) = tx.try_send(sample) {
                                warn!("Failed to send audio sample: {}", e);
                            }
                        }
                    },
                    |err| error!("Audio stream error: {}", err),
                    None,
                )?
            }
        };

        // Start the stream
        stream.play()?;
        info!("Started audio capture stream");

        let stream_control = Arc::new(CpalStreamControl {
            _stream: stream,
            running,
        });

        Ok(AudioStream::new(rx, stream_control))
    }

    async fn start_system_capture(&self, config: AudioConfig) -> Result<AudioStream> {
        #[cfg(target_os = "macos")]
        {
            // Use macOS-specific system audio capture
            return crate::macos::start_system_audio_capture(config).await;
        }

        #[cfg(not(target_os = "macos"))]
        {
            // For other platforms, try to use loopback device or return error
            warn!("System audio capture not fully implemented for this platform");
            
            // Try to find a loopback or monitor device
            let devices = self.list_devices().await?;
            let loopback_device = devices
                .iter()
                .find(|d| {
                    let name_lower = d.name.to_lowercase();
                    name_lower.contains("loopback") 
                        || name_lower.contains("monitor")
                        || name_lower.contains("stereo mix")
                        || name_lower.contains("what u hear")
                })
                .cloned();

            if let Some(device) = loopback_device {
                info!("Using loopback device for system audio: {}", device.name);
                let mut system_config = config;
                system_config.device_id = Some(device.id);
                return self.start_capture(system_config).await;
            }

            Err(anyhow!(
                "System audio capture not available on this platform. Please install a virtual audio driver like VB-Cable or BlackHole."
            ))
        }
    }
}

struct CpalStreamControl {
    _stream: Stream,
    running: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl StreamControl for CpalStreamControl {
    async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        info!("Stopped audio capture stream");
        Ok(())
    }

    async fn pause(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        debug!("Paused audio capture stream");
        Ok(())
    }

    async fn resume(&self) -> Result<()> {
        self.running.store(true, Ordering::Relaxed);
        debug!("Resumed audio capture stream");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}