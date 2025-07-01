//! Whisper speech-to-text implementation

use crate::{SpeechToText, SttConfig, TranscriptionResult, TranscriptionSegment, WordTimestamp};
use anyhow::{anyhow, Result};
use std::time::Instant;
use tracing::{debug, info, warn};
use async_trait::async_trait;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState, WhisperContextParameters};

pub struct WhisperProcessor {
    context: Option<WhisperContext>,
    config: SttConfig,
}

impl WhisperProcessor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            context: None,
            config: SttConfig::default(),
        })
    }

    pub fn with_config(config: SttConfig) -> Result<Self> {
        Ok(Self {
            context: None,
            config,
        })
    }

    /// Create Whisper parameters from config
    fn create_params(&self) -> FullParams {
        let strategy = if self.config.temperature > 0.0 {
            SamplingStrategy::Greedy { best_of: 1 }
        } else {
            SamplingStrategy::Greedy { best_of: 1 }
        };

        let mut params = FullParams::new(strategy);

        // Configure parameters
        params.set_language(self.config.language.as_deref());
        params.set_translate(self.config.translate_to_english);
        params.set_no_context(true);
        params.set_single_segment(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(self.config.enable_timestamps);

        // Token limits
        if self.config.max_tokens > 0 {
            params.set_token_timestamps(true);
        }

        params
    }

}

#[async_trait(?Send)]
impl SpeechToText for WhisperProcessor {
    async fn load_model(&mut self, model_path: &str) -> Result<()> {
        info!("Loading Whisper model from: {}", model_path);
        
        if !std::path::Path::new(model_path).exists() {
            return Err(anyhow!("Model file not found: {}", model_path));
        }

        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| anyhow!("Failed to load Whisper model: {}", e))?;

        self.context = Some(ctx);
        self.config.model_path = model_path.to_string();

        info!("Successfully loaded Whisper model");
        Ok(())
    }

    async fn transcribe(&self, audio_data: &[f32], sample_rate: u32) -> Result<TranscriptionResult> {
        let ctx = self.context.as_ref()
            .ok_or_else(|| anyhow!("No model loaded"))?;

        if sample_rate != 16000 {
            warn!("Audio sample rate is {}Hz, Whisper expects 16kHz. Consider resampling.", sample_rate);
        }

        debug!("Starting transcription of {} samples", audio_data.len());
        let start_time = Instant::now();

        // Prepare audio data for Whisper
        let prepared_audio = crate::audio_utils::prepare_audio_for_whisper(audio_data, sample_rate, 1);

        // Create parameters
        let params = self.create_params();

        // Run transcription
        let mut state = ctx.create_state()
            .map_err(|e| anyhow!("Failed to create Whisper state: {}", e))?;

        state.full(params, &prepared_audio)
            .map_err(|e| anyhow!("Transcription failed: {}", e))?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Extract results using the state
        self.extract_results_from_state(&state, processing_time)
    }

    async fn transcribe_file(&self, file_path: &str) -> Result<TranscriptionResult> {
        info!("Transcribing audio file: {}", file_path);

        // Load audio file
        let audio_sample = load_audio_file(file_path)?;
        
        // Transcribe
        self.transcribe(&audio_sample.data, audio_sample.sample_rate).await
    }

    fn get_supported_languages(&self) -> Vec<String> {
        // Whisper supports many languages
        vec![
            "en".to_string(), "zh".to_string(), "de".to_string(), "es".to_string(),
            "ru".to_string(), "ko".to_string(), "fr".to_string(), "ja".to_string(),
            "pt".to_string(), "tr".to_string(), "pl".to_string(), "ca".to_string(),
            "nl".to_string(), "ar".to_string(), "sv".to_string(), "it".to_string(),
            "id".to_string(), "hi".to_string(), "fi".to_string(), "vi".to_string(),
            "he".to_string(), "uk".to_string(), "el".to_string(), "ms".to_string(),
            "cs".to_string(), "ro".to_string(), "da".to_string(), "hu".to_string(),
            "ta".to_string(), "no".to_string(), "th".to_string(), "ur".to_string(),
            "hr".to_string(), "bg".to_string(), "lt".to_string(), "la".to_string(),
            "mi".to_string(), "ml".to_string(), "cy".to_string(), "sk".to_string(),
            "te".to_string(), "fa".to_string(), "lv".to_string(), "bn".to_string(),
            "sr".to_string(), "az".to_string(), "sl".to_string(), "kn".to_string(),
            "et".to_string(), "mk".to_string(), "br".to_string(), "eu".to_string(),
            "is".to_string(), "hy".to_string(), "ne".to_string(), "mn".to_string(),
            "bs".to_string(), "kk".to_string(), "sq".to_string(), "sw".to_string(),
            "gl".to_string(), "mr".to_string(), "pa".to_string(), "si".to_string(),
            "km".to_string(), "sn".to_string(), "yo".to_string(), "so".to_string(),
            "af".to_string(), "oc".to_string(), "ka".to_string(), "be".to_string(),
            "tg".to_string(), "sd".to_string(), "gu".to_string(), "am".to_string(),
            "yi".to_string(), "lo".to_string(), "uz".to_string(), "fo".to_string(),
            "ht".to_string(), "ps".to_string(), "tk".to_string(), "nn".to_string(),
            "mt".to_string(), "sa".to_string(), "lb".to_string(), "my".to_string(),
            "bo".to_string(), "tl".to_string(), "mg".to_string(), "as".to_string(),
            "tt".to_string(), "haw".to_string(), "ln".to_string(), "ha".to_string(),
            "ba".to_string(), "jw".to_string(), "su".to_string(),
        ]
    }

    fn is_loaded(&self) -> bool {
        self.context.is_some()
    }
}

impl WhisperProcessor {
    /// Extract results from WhisperState instead of WhisperContext
    fn extract_results_from_state(&self, state: &WhisperState, processing_time: u64) -> Result<TranscriptionResult> {
        let num_segments = state.full_n_segments()?;
        let mut segments = Vec::new();
        let mut full_text = String::new();

        for i in 0..num_segments {
            let text = state.full_get_segment_text(i)?;
            let start_time = state.full_get_segment_t0(i)? as f64 / 100.0;
            let end_time = state.full_get_segment_t1(i)? as f64 / 100.0;

            // Extract word-level timestamps if enabled
            let words = None;

            segments.push(TranscriptionSegment {
                text: text.clone(),
                start_time,
                end_time,
                confidence: None,
                words,
            });

            full_text.push_str(&text);
            if i < num_segments - 1 {
                full_text.push(' ');
            }
        }

        Ok(TranscriptionResult {
            text: full_text.trim().to_string(),
            language: self.config.language.clone(),
            segments,
            processing_time_ms: processing_time,
            model_used: self.config.model_path.clone(),
        })
    }
}

/// Load audio file (supports WAV for now)
fn load_audio_file(file_path: &str) -> Result<AudioSample> {
    use hound::WavReader;

    let mut reader = WavReader::open(file_path)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?
        }
        hound::SampleFormat::Int => {
            reader.samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    Ok(AudioSample {
        data: samples,
        timestamp: chrono::Utc::now(),
        sample_rate: spec.sample_rate,
        channels: spec.channels,
    })
}

// Temporary struct until we properly import from savant-audio
#[derive(Debug, Clone)]
struct AudioSample {
    data: Vec<f32>,
    timestamp: chrono::DateTime<chrono::Utc>,
    sample_rate: u32,
    channels: u16,
}