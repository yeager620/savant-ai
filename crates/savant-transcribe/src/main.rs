use clap::Parser;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use savant_audio::{create_audio_capture, AudioConfig, AudioBuffer, AudioBufferConfig};
use savant_stt::{create_speech_to_text, SttConfig, markdown};
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
    /// Output file (markdown). If not provided, prints to stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
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

    let mut stt = create_speech_to_text()?;
    let mut stt_cfg = SttConfig::default();
    stt_cfg.model_path = cli.model.clone();
    stt.load_model(&stt_cfg.model_path).await?;

    let result = stt
        .transcribe(&audio_sample.data, audio_sample.sample_rate)
        .await?;

    let markdown = markdown::format_transcription_markdown(&result, None, chrono::Utc::now());

    if let Some(path) = cli.output {
        std::fs::write(&path, markdown)?;
        println!("Saved transcript to {}", path.display());
    } else {
        println!("{}", markdown);
    }

    Ok(())
}
