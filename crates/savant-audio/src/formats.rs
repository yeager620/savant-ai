//! Audio format utilities and conversions

use crate::{AudioSample, SampleFormat};
use anyhow::Result;
use std::io::Cursor;
use hound::{WavReader, WavSpec, WavWriter};

/// Audio format converter
pub struct AudioConverter;

impl AudioConverter {
    /// Convert audio sample to different format
    pub fn convert_format(sample: &AudioSample, target_format: SampleFormat) -> AudioSample {
        let converted_data = match target_format {
            SampleFormat::F32 => sample.data.clone(), // Already F32
            SampleFormat::I16 => {
                sample.data
                    .iter()
                    .map(|&f| (f * i16::MAX as f32) as i16)
                    .map(|i| i as f32 / i16::MAX as f32) // Convert back to F32 for internal use
                    .collect()
            }
        };

        AudioSample {
            data: converted_data,
            timestamp: sample.timestamp,
            sample_rate: sample.sample_rate,
            channels: sample.channels,
        }
    }

    /// Resample audio to target sample rate
    pub fn resample(sample: &AudioSample, target_rate: u32) -> AudioSample {
        if sample.sample_rate == target_rate {
            return sample.clone();
        }

        let resampled_data = crate::prepare_for_whisper(&sample.data, sample.sample_rate, target_rate);

        AudioSample {
            data: resampled_data,
            timestamp: sample.timestamp,
            sample_rate: target_rate,
            channels: sample.channels,
        }
    }

    /// Convert stereo to mono
    pub fn to_mono(sample: &AudioSample) -> AudioSample {
        if sample.channels == 1 {
            return sample.clone();
        }

        let mono_data = if sample.channels == 2 {
            // Simple stereo to mono conversion (average channels)
            sample.data
                .chunks_exact(2)
                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                .collect()
        } else {
            // For more than 2 channels, take first channel
            sample.data
                .iter()
                .step_by(sample.channels as usize)
                .copied()
                .collect()
        };

        AudioSample {
            data: mono_data,
            timestamp: sample.timestamp,
            sample_rate: sample.sample_rate,
            channels: 1,
        }
    }

    /// Prepare audio sample for Whisper (16kHz, mono, F32)
    pub fn prepare_for_whisper(sample: &AudioSample) -> AudioSample {
        let mut result = sample.clone();

        // Convert to mono if needed
        if result.channels > 1 {
            result = Self::to_mono(&result);
        }

        // Resample to 16kHz if needed
        if result.sample_rate != 16000 {
            result = Self::resample(&result, 16000);
        }

        result
    }
}

/// WAV file utilities
pub struct WavUtils;

impl WavUtils {
    /// Save audio sample as WAV file
    pub fn save_wav(sample: &AudioSample, path: &str) -> Result<()> {
        let spec = WavSpec {
            channels: sample.channels,
            sample_rate: sample.sample_rate,
            bits_per_sample: 32, // F32 format
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = WavWriter::create(path, spec)?;
        
        for &sample_val in &sample.data {
            writer.write_sample(sample_val)?;
        }

        writer.finalize()?;
        Ok(())
    }

    /// Load WAV file as audio sample
    pub fn load_wav(path: &str) -> Result<AudioSample> {
        let mut reader = WavReader::open(path)?;
        let spec = reader.spec();

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?
            }
            hound::SampleFormat::Int => {
                reader.samples::<i32>().map(|s| {
                    s.map(|sample| sample as f32 / i32::MAX as f32)
                }).collect::<Result<Vec<_>, _>>()?
            }
        };

        Ok(AudioSample {
            data: samples,
            timestamp: chrono::Utc::now(),
            sample_rate: spec.sample_rate,
            channels: spec.channels,
        })
    }

    /// Convert audio sample to WAV bytes
    pub fn to_wav_bytes(sample: &AudioSample) -> Result<Vec<u8>> {
        let spec = WavSpec {
            channels: sample.channels,
            sample_rate: sample.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)?;
            for &sample_val in &sample.data {
                writer.write_sample(sample_val)?;
            }
            writer.finalize()?;
        }

        Ok(cursor.into_inner())
    }

    /// Load audio from WAV bytes
    pub fn from_wav_bytes(data: &[u8]) -> Result<AudioSample> {
        let cursor = Cursor::new(data);
        let mut reader = WavReader::new(cursor)?;
        let spec = reader.spec();

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?
            }
            hound::SampleFormat::Int => {
                reader.samples::<i32>().map(|s| {
                    s.map(|sample| sample as f32 / i32::MAX as f32)
                }).collect::<Result<Vec<_>, _>>()?
            }
        };

        Ok(AudioSample {
            data: samples,
            timestamp: chrono::Utc::now(),
            sample_rate: spec.sample_rate,
            channels: spec.channels,
        })
    }
}

/// Audio buffer for accumulating samples
pub struct AudioBuffer {
    buffer: Vec<f32>,
    config: AudioBufferConfig,
}

#[derive(Debug, Clone)]
pub struct AudioBufferConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub max_duration_seconds: f32,
}

impl AudioBuffer {
    pub fn new(config: AudioBufferConfig) -> Self {
        let max_samples = (config.sample_rate as f32 * config.max_duration_seconds) as usize * config.channels as usize;
        
        Self {
            buffer: Vec::with_capacity(max_samples),
            config,
        }
    }

    /// Add audio sample to buffer
    pub fn push(&mut self, sample: &AudioSample) {
        // Ensure sample is compatible
        if sample.sample_rate != self.config.sample_rate || sample.channels != self.config.channels {
            // TODO: Convert sample to match buffer config
            return;
        }

        self.buffer.extend_from_slice(&sample.data);

        // Trim buffer if it exceeds max duration
        let max_samples = (self.config.sample_rate as f32 * self.config.max_duration_seconds) as usize * self.config.channels as usize;
        if self.buffer.len() > max_samples {
            let excess = self.buffer.len() - max_samples;
            self.buffer.drain(0..excess);
        }
    }

    /// Get current buffer as audio sample
    pub fn get_sample(&self) -> AudioSample {
        AudioSample {
            data: self.buffer.clone(),
            timestamp: chrono::Utc::now(),
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
        }
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get buffer duration in seconds
    pub fn duration_seconds(&self) -> f32 {
        self.buffer.len() as f32 / (self.config.sample_rate as f32 * self.config.channels as f32)
    }

    /// Check if buffer has enough data for transcription
    pub fn has_sufficient_data(&self, min_seconds: f32) -> bool {
        self.duration_seconds() >= min_seconds
    }
}