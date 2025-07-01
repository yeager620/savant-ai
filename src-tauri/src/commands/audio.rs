use anyhow::Result;
use tauri::AppHandle;
use savant_audio::{create_audio_capture, AudioConfig, AudioBuffer, AudioBufferConfig};
use savant_stt::{create_speech_to_text, SttConfig, markdown};
use std::time::{Duration, Instant};

/// Capture audio and return markdown transcript
#[tauri::command]
pub async fn transcribe_audio(duration: u32, system: bool) -> Result<String, String> {
    match internal_transcribe(duration, system).await {
        Ok(md) => Ok(md),
        Err(e) => Err(e.to_string()),
    }
}

async fn internal_transcribe(duration: u32, system: bool) -> Result<String> {
    let mut config = AudioConfig::default();
    config.capture_system_audio = system;

    let capture = create_audio_capture()?;
    let mut stream = if system {
        capture.start_system_capture(config.clone()).await?
    } else {
        capture.start_capture(config.clone()).await?
    };

    let mut buffer = AudioBuffer::new(AudioBufferConfig {
        sample_rate: config.sample_rate,
        channels: config.channels,
        max_duration_seconds: duration as f32,
    });

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(duration as u64) {
        if let Some(sample) = stream.receiver.recv().await {
            buffer.push(&sample);
        }
    }
    stream.stop().await?;

    let audio = buffer.get_sample();

    let mut stt = create_speech_to_text()?;
    let stt_cfg = SttConfig::default();
    stt.load_model(&stt_cfg.model_path).await?;

    let result = stt.transcribe(&audio.data, audio.sample_rate).await?;
    Ok(markdown::format_transcription_markdown(&result, None, chrono::Utc::now()))
}
