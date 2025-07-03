# Savant AI Daemon Management Scripts

This directory contains integrated scripts for managing all Savant AI multimodal daemons.

## Available Scripts

### 🚀 **start_all_daemons.sh**
Comprehensive startup script that:
- Verifies dependencies (ollama, tesseract, imagemagick)
- Installs devstral model if missing
- Starts Ollama server
- Launches audio daemon
- Launches video daemon with full multimodal analysis
- Provides status verification and usage instructions

### 🛑 **stop_all_daemons.sh**
Graceful shutdown script that:
- Stops video daemon first (dependencies)
- Stops audio daemon
- Provides final status verification
- Optional Ollama server shutdown (commented by default)

### 🔄 **restart_daemons.sh**
Clean restart sequence that:
- Stops all daemons gracefully
- Waits for clean shutdown
- Restarts all systems

### 📊 **monitor_daemons.sh**
Real-time monitoring dashboard that shows:
- Live daemon status with color coding
- System resource usage (CPU, memory, disk)
- Recent log entries from both daemons
- Auto-refreshes every 5 seconds
- Press Ctrl+C to exit

### 🧪 **test_all_systems.sh**
Comprehensive system test that verifies:
- Dependencies installation
- Ollama server connectivity
- Individual component functionality (OCR, vision, sync)
- Sample image processing (if available)
- Database connectivity

## Quick Usage

```bash
# Start everything
./start_all_daemons.sh

# Monitor in real-time
./monitor_daemons.sh

# Test all systems
./test_all_systems.sh

# Stop everything
./stop_all_daemons.sh
```

## What Gets Started

When you run `./start_all_daemons.sh`, you get:

### Audio Pipeline
- Microphone capture
- Real-time speech-to-text
- Speaker identification
- Audio analytics

### Video Pipeline  
- Screen capture every 30 seconds
- OCR text extraction (0.9s fast mode)
- Computer vision analysis
- Application detection
- Activity classification

### Multimodal Intelligence
- Audio-video correlation
- Context synchronization
- Proactive insights
- Enhanced database integration

## Monitoring

The `monitor_daemons.sh` script provides a live dashboard showing:

```
🔍 Savant AI Daemon Monitor - 14:32:15
Press Ctrl+C to exit
==========================
🟢 Audio Daemon : Running
🟢 Video Daemon : Running  
🟢 Ollama Server: Running

📊 System Resources:
CPU: 15.2%
Memory: 78% free
Disk: 45% used

📝 Recent Audio Logs (last 3 lines):
  [INFO] Audio capture initialized
  [INFO] Speaker detection active
  [INFO] Processing audio segment...

📝 Recent Video Logs (last 3 lines):
  [INFO] Screen capture successful
  [INFO] OCR processing completed (0.9s)
  [INFO] Application detected: Visual Studio Code
```

## Troubleshooting

### Dependencies Missing
```bash
# Install required dependencies
brew install ollama tesseract imagemagick

# Pull AI model
ollama pull llama3.2
```

### Permissions Required
- **Screen Recording**: System Preferences > Security & Privacy > Privacy
- **Microphone**: System Preferences > Security & Privacy > Privacy

### Port Conflicts
- Ollama uses port 11434 by default
- Check with: `lsof -i :11434`

### Log Analysis
```bash
# View detailed logs
./sav logs --verbose
./sav-video logs --verbose

# Or monitor specific log files
tail -f ~/.config/savant-ai/logs/audio.log
tail -f ~/.config/savant-ai/logs/video.log
```

## Performance Notes

- **OCR Fast Mode**: 0.9s per screenshot (real-time suitable)
- **Memory Usage**: ~200MB baseline + 50MB per active analysis
- **Storage**: 5-20MB per hour of captured data
- **CPU**: 10-20% during active processing

## Integration with Main App

These daemons integrate seamlessly with the main Tauri application:

```bash
# Start daemons first
./start_all_daemons.sh

# Then run main app
cargo tauri dev
```

The main app will automatically connect to running daemons and provide a unified interface for all multimodal intelligence features.