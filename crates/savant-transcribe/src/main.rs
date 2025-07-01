use clap::Parser;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use savant_audio::{create_audio_capture, AudioConfig, AudioBuffer, AudioBufferConfig};
use savant_stt::{create_speech_to_text_with_config, SttConfig, markdown, SessionMetadata, AudioSource};
use anyhow::Result;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "savant-transcribe", about = "Record audio and output a markdown transcript", long_about = None)]
struct Cli {
    /// Duration to record in seconds
    #[arg(short, long, default_value = "10")]
    duration: u32,
    /// Capture system audio instead of microphone
    #[arg(long)]
    system: bool,
    /// Audio device name
    #[arg(long)]
    device: Option<String>,
    /// Path to Whisper model
    #[arg(long, default_value = "models/ggml-base.en.bin")]
    model: String,
    /// Language to transcribe in (e.g., "en", "zh"). Auto-detects if not specified.
    #[arg(long)]
    language: Option<String>,
    /// Output file. If not provided, prints to stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Output format: json or markdown
    #[arg(long, default_value = "json")]
    format: String,
    /// Speaker identifier for this audio source
    #[arg(long)]
    speaker: Option<String>,
    /// Session ID to group related recordings
    #[arg(long)]
    session_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Simple logger
    let subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).ok();

    let cli = Cli::parse();

    let mut audio_config = AudioConfig::default();
    audio_config.capture_system_audio = cli.system;
    if let Some(dev) = cli.device.clone() {
        audio_config.device_id = Some(dev);
    }

    let capture = create_audio_capture()?;
    let mut stream = if cli.system {
        capture.start_system_capture(audio_config.clone()).await?
    } else {
        capture.start_capture(audio_config.clone()).await?
    };

    let mut buffer = AudioBuffer::new(AudioBufferConfig {
        sample_rate: audio_config.sample_rate,
        channels: audio_config.channels,
        max_duration_seconds: cli.duration as f32,
    });

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(cli.duration as u64) {
        if let Some(sample) = stream.receiver.recv().await {
            buffer.push(&sample);
        }
    }
    stream.stop().await?;

    let audio_sample = buffer.get_sample();

    let mut stt_cfg = SttConfig::default();
    stt_cfg.model_path = cli.model.clone();
    stt_cfg.language = cli.language.clone();
    let mut stt = create_speech_to_text_with_config(stt_cfg.clone())?;
    stt.load_model(&stt_cfg.model_path).await?;

    let mut result = stt
        .transcribe(&audio_sample.data, audio_sample.sample_rate)
        .await?;

    // Add session metadata
    let session_metadata = SessionMetadata {
        session_id: cli.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        timestamp: chrono::Utc::now(),
        audio_source: if cli.system { AudioSource::SystemAudio } else { AudioSource::Microphone },
        speaker: cli.speaker,
        device_info: Some(format!("savant-transcribe-{}", env!("CARGO_PKG_VERSION"))),
    };
    
    result.session_metadata = Some(session_metadata);

    let output_content = match cli.format.as_str() {
        "json" => serde_json::to_string_pretty(&result)?,
        "markdown" | "md" => markdown::format_transcription_markdown(&result, None, chrono::Utc::now()),
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", cli.format)),
    };

    if let Some(path) = cli.output {
        std::fs::write(&path, output_content)?;
        println!("Saved transcript to {}", path.display());
    } else {
        println!("{}", output_content);
    }

    Ok(())
}
