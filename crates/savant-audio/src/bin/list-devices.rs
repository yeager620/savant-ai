use savant_audio::create_audio_capture;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Audio I/O Devices\n");
    println!("=================\n");

    let capture = create_audio_capture()?;
    let devices = capture.list_devices().await?;

    if devices.is_empty() {
        println!("No audio devices found.");
        return Ok(());
    }

    // Separate input and output devices
    let input_devices: Vec<_> = devices.iter().filter(|d| d.is_input).collect();
    let output_devices: Vec<_> = devices.iter().filter(|d| d.is_output).collect();

    // Show input devices
    if !input_devices.is_empty() {
        println!("INPUT DEVICES (Microphones):");
        println!("================================");
        for (i, device) in input_devices.iter().enumerate() {
            let default_marker = if device.is_default { " [DEFAULT]" } else { "" };
            println!("{}. {}{}", i + 1, device.name, default_marker);
            println!("   ID: {}", device.id);
            println!("   Channels: {}", device.channels);
            if !device.sample_rates.is_empty() {
                let mut rates = device.sample_rates.clone();
                rates.sort();
                rates.dedup();
                println!("   Sample Rates: {:?} Hz", rates);
            }
            println!();
        }
    }

    // Show output devices  
    if !output_devices.is_empty() {
        println!("OUTPUT DEVICES (Speakers/Headphones):");
        println!("========================================");
        for (i, device) in output_devices.iter().enumerate() {
            let default_marker = if device.is_default { " [DEFAULT]" } else { "" };
            println!("{}. {}{}", i + 1, device.name, default_marker);
            println!("   ID: {}", device.id);
            println!("   Channels: {}", device.channels);
            if !device.sample_rates.is_empty() {
                let mut rates = device.sample_rates.clone();
                rates.sort();
                rates.dedup();
                println!("   Sample Rates: {:?} Hz", rates);
            }
            
            // Check if this might be a loopback device for system audio
            let name_lower = device.name.to_lowercase();
            if name_lower.contains("blackhole") 
                || name_lower.contains("loopback") 
                || name_lower.contains("soundflower")
                || name_lower.contains("virtual") {
                println!("   SYSTEM AUDIO CAPABLE (Loopback Device)");
            }
            println!();
        }
    }

    // Show devices that are both input and output
    let bidirectional: Vec<_> = devices.iter().filter(|d| d.is_input && d.is_output).collect();
    if !bidirectional.is_empty() {
        println!("BIDIRECTIONAL DEVICES (Input + Output):");
        println!("==========================================");
        for (i, device) in bidirectional.iter().enumerate() {
            let default_marker = if device.is_default { " [DEFAULT]" } else { "" };
            println!("{}. {}{}", i + 1, device.name, default_marker);
            println!("   ID: {}", device.id);
            println!("   Channels: {}", device.channels);
            println!();
        }
    }

    println!("USAGE EXAMPLES:");
    println!("===============");
    println!("# Record from default microphone:");
    println!("cargo run --package savant-transcribe -- --duration 10");
    println!();
    println!("# Record from specific device:");
    println!("cargo run --package savant-transcribe -- --device \"DEVICE_NAME\" --duration 10");
    println!();
    println!("# Record system audio (requires loopback device like BlackHole):");
    println!("cargo run --package savant-transcribe -- --system --duration 10");
    println!();
    
    if !output_devices.iter().any(|d| {
        let name = d.name.to_lowercase();
        name.contains("blackhole") || name.contains("loopback") 
    }) {
        println!("TIP: To capture system audio, install BlackHole:");
        println!("   brew install blackhole-2ch");
        println!("   or download from: https://github.com/ExistentialAudio/BlackHole");
    }

    Ok(())
}