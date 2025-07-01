//! # Savant Speech-to-Text Library
//!
//! Local offline speech-to-text processing using Whisper
//! Supports multiple model sizes and languages

use anyhow::Result;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

pub mod whisper;
pub mod models;

pub use whisper::*;
pub use models::*;

/// Speech-to-text configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub model_path: String,
    pub language: Option<String>,
    pub translate_to_english: bool,
    pub temperature: f32,
    pub no_speech_threshold: f32,
    pub enable_timestamps: bool,
    pub enable_word_timestamps: bool,
    pub max_tokens: u32,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model_path: "models/ggml-base.en.bin".to_string(),
            language: None, // Auto-detect
            translate_to_english: false,
            temperature: 0.0,
            no_speech_threshold: 0.6,
            enable_timestamps: true,
            enable_word_timestamps: false,
            max_tokens: 0, // No limit
        }
    }
}

/// Transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub segments: Vec<TranscriptionSegment>,
    pub processing_time_ms: u64,
    pub model_used: String,
}

/// Individual transcription segment with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: Option<f32>,
    pub words: Option<Vec<WordTimestamp>>,
}

/// Word-level timestamp information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTimestamp {
    pub word: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: Option<f32>,
}

/// Speech-to-text processor trait
#[async_trait(?Send)]
pub trait SpeechToText {
    /// Load model from path
    async fn load_model(&mut self, model_path: &str) -> Result<()>;

    /// Transcribe audio samples
    async fn transcribe(&self, audio_data: &[f32], sample_rate: u32) -> Result<TranscriptionResult>;

    /// Transcribe audio file
    async fn transcribe_file(&self, file_path: &str) -> Result<TranscriptionResult>;

    /// Get available languages
    fn get_supported_languages(&self) -> Vec<String>;

    /// Check if model is loaded
    fn is_loaded(&self) -> bool;
}

/// Create speech-to-text processor
pub fn create_speech_to_text() -> Result<Box<dyn SpeechToText>> {
    Ok(Box::new(WhisperProcessor::new()?))
}

/// Utility functions for audio preprocessing
pub mod audio_utils {

    /// Ensure audio is in correct format for Whisper (16kHz, mono, f32)
    pub fn prepare_audio_for_whisper(audio_data: &[f32], sample_rate: u32, channels: u16) -> Vec<f32> {
        let mut processed = audio_data.to_vec();

        // Convert to mono if needed
        if channels > 1 {
            processed = convert_to_mono(&processed, channels);
        }

        // Resample if needed
        if sample_rate != 16000 {
            processed = resample_audio(&processed, sample_rate, 16000);
        }

        processed
    }

    /// Convert stereo/multi-channel to mono by averaging
    fn convert_to_mono(audio_data: &[f32], channels: u16) -> Vec<f32> {
        if channels == 1 {
            return audio_data.to_vec();
        }

        let mut mono_data = Vec::with_capacity(audio_data.len() / channels as usize);
        
        for chunk in audio_data.chunks_exact(channels as usize) {
            let avg = chunk.iter().sum::<f32>() / channels as f32;
            mono_data.push(avg);
        }

        mono_data
    }

    /// Simple linear interpolation resampling
    fn resample_audio(audio_data: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
        if source_rate == target_rate {
            return audio_data.to_vec();
        }

        let ratio = source_rate as f64 / target_rate as f64;
        let output_len = (audio_data.len() as f64 / ratio) as usize;
        let mut output = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let pos = i as f64 * ratio;
            let index = pos as usize;
            
            if index + 1 < audio_data.len() {
                let frac = pos - index as f64;
                let sample = audio_data[index] * (1.0 - frac as f32) + audio_data[index + 1] * frac as f32;
                output.push(sample);
            } else if index < audio_data.len() {
                output.push(audio_data[index]);
            }
        }

        output
    }

    /// Normalize audio to prevent clipping
    pub fn normalize_audio(audio_data: &mut [f32]) {
        let max_amplitude = audio_data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        
        if max_amplitude > 1.0 {
            let scale = 0.95 / max_amplitude;
            for sample in audio_data.iter_mut() {
                *sample *= scale;
            }
        }
    }

    /// Apply simple noise gate
    pub fn apply_noise_gate(audio_data: &mut [f32], threshold: f32) {
        for sample in audio_data.iter_mut() {
            if sample.abs() < threshold {
                *sample = 0.0;
            }
        }
    }
}

/// Markdown formatting utilities
pub mod markdown {
    use super::*;
    use chrono::{DateTime, Utc};

    /// Format transcription result as markdown
    pub fn format_transcription_markdown(
        result: &TranscriptionResult,
        session_name: Option<&str>,
        timestamp: DateTime<Utc>,
    ) -> String {
        let mut markdown = String::new();

        // Header
        if let Some(name) = session_name {
            markdown.push_str(&format!("# Audio Transcription: {}\n\n", name));
        } else {
            markdown.push_str("# Audio Transcription\n\n");
        }

        // Metadata
        markdown.push_str(&format!("**Date:** {}\n", timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        markdown.push_str(&format!("**Model:** {}\n", result.model_used));
        markdown.push_str(&format!("**Processing Time:** {}ms\n", result.processing_time_ms));
        
        if let Some(lang) = &result.language {
            markdown.push_str(&format!("**Language:** {}\n", lang));
        }

        markdown.push_str("\n---\n\n");

        // Full text
        markdown.push_str("## Full Transcript\n\n");
        markdown.push_str(&result.text);
        markdown.push_str("\n\n");

        // Segments with timestamps
        if !result.segments.is_empty() {
            markdown.push_str("## Timestamped Segments\n\n");
            
            for (i, segment) in result.segments.iter().enumerate() {
                let start_min = (segment.start_time / 60.0) as u64;
                let start_sec = (segment.start_time % 60.0) as u64;
                let end_min = (segment.end_time / 60.0) as u64;
                let end_sec = (segment.end_time % 60.0) as u64;

                markdown.push_str(&format!(
                    "### Segment {} ({:02}:{:02} - {:02}:{:02})\n\n",
                    i + 1, start_min, start_sec, end_min, end_sec
                ));

                markdown.push_str(&segment.text);
                markdown.push_str("\n\n");

                // Word-level timestamps if available
                if let Some(words) = &segment.words {
                    markdown.push_str("**Word Timestamps:**\n\n");
                    for word in words {
                        let word_start = word.start_time as u64;
                        let word_end = word.end_time as u64;
                        markdown.push_str(&format!(
                            "- `{}` ({}s - {}s)\n",
                            word.word, word_start, word_end
                        ));
                    }
                    markdown.push_str("\n");
                }
            }
        }

        markdown
    }

    /// Format as simple timestamped transcript
    pub fn format_simple_transcript(result: &TranscriptionResult) -> String {
        let mut transcript = String::new();

        for segment in &result.segments {
            let timestamp = format_timestamp(segment.start_time);
            transcript.push_str(&format!("[{}] {}\n", timestamp, segment.text));
        }

        transcript
    }

    /// Format timestamp as MM:SS
    fn format_timestamp(seconds: f64) -> String {
        let min = (seconds / 60.0) as u64;
        let sec = (seconds % 60.0) as u64;
        format!("{:02}:{:02}", min, sec)
    }
}