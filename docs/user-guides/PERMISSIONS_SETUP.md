# macOS Permissions Setup Guide

This guide walks you through configuring all macOS system permissions required for Savant AI's multimodal functionality.

## 🔍 Quick Verification

First, run the automated verification script:

```bash
./verify-permissions
```

This will check all permissions and dependencies, providing specific instructions for any issues found.

## 📋 Required Permissions Checklist

### **1. Screen Recording Permission** ⭐ **CRITICAL**
**Required for**: Video capture, OCR, computer vision analysis

**How to enable**:
1. Open **System Preferences** → **Security & Privacy** → **Privacy**
2. Click **Screen Recording** in the left sidebar
3. Click the **lock icon** and enter your password
4. Check the box next to your terminal application:
   - **Terminal** (default macOS terminal)
   - **iTerm2** (if using iTerm)
   - **Warp** (if using Warp terminal)
   - **VS Code** (if running from VS Code terminal)

**To verify**: Run `./verify-permissions` or try screen capture:
```bash
./sav-video start --interval 5
./sav-video status
```

### **2. Microphone Permission** ⭐ **CRITICAL**
**Required for**: Audio capture, speech-to-text, multimodal correlation

**How to enable**:
1. Open **System Preferences** → **Security & Privacy** → **Privacy**
2. Click **Microphone** in the left sidebar
3. Check the box next to your terminal application
4. Also enable for **any IDE or editor** you're using

**To verify**: Run audio capture test:
```bash
./sav start
./sav test
```

### **3. Accessibility Permission** (Optional but Recommended)
**Required for**: Advanced UI detection, window management

**How to enable**:
1. Open **System Preferences** → **Security & Privacy** → **Privacy**
2. Click **Accessibility** in the left sidebar
3. Check the box next to your terminal application

### **4. Full Disk Access** (Optional)
**Required for**: Advanced file monitoring, system-wide analysis

**How to enable**:
1. Open **System Preferences** → **Security & Privacy** → **Privacy**
2. Click **Full Disk Access** in the left sidebar
3. Add your terminal application if needed

## 🎵 Audio System Configuration

### **System Audio Capture (Advanced)**

For capturing system audio (not just microphone), you need a virtual audio device:

#### **Option 1: BlackHole (Recommended)**
```bash
# Download and install BlackHole
open https://github.com/ExistentialAudio/BlackHole

# After installation, configure:
# 1. Open Audio MIDI Setup app
# 2. Create Multi-Output Device
# 3. Include both built-in output + BlackHole
# 4. Set as default output device
```

#### **Option 2: SoundFlower (Alternative)**
```bash
brew install --cask soundflower
```

### **Audio Device Verification**
```bash
# List available audio devices
./scripts/audio/audio-devices.sh

# Test system audio capture
./scripts/audio/capture-system-audio.sh
```

## 🖥️ Display & Graphics

### **Multiple Displays**
If using multiple monitors:
1. Screen recording permission applies to **all displays**
2. Test capture on each display:
```bash
# Specify display for capture
./sav-video start --display 1
```

### **High DPI/Retina Displays**
For optimal OCR on high-resolution displays:
```bash
# Test OCR with different DPI settings
cargo run --package savant-ocr -- extract --input screenshot.png --fast
```

## 🔒 Security & Firewall

### **Firewall Configuration**
If macOS Firewall is enabled:

1. **System Preferences** → **Security & Privacy** → **Firewall**
2. Click **Firewall Options**
3. Ensure **Ollama** is allowed (or disable firewall temporarily)

### **Gatekeeper Issues**
If you get "unidentified developer" warnings:
```bash
# For specific files (if needed)
sudo xattr -rd com.apple.quarantine /path/to/file

# Or allow in Security & Privacy after the warning appears
```

## 🧪 Testing Your Configuration

### **Complete System Test**
```bash
# Run comprehensive test suite
./test-systems

# Test individual components
cargo run --package savant-ocr -- test
cargo run --package savant-vision -- test
cargo run --package savant-sync -- test
```

### **Audio Test**
```bash
# Test microphone capture
./sav test

# Test transcription
./sav start
# Speak into microphone, then:
./sav logs
```

### **Video Test**
```bash
# Test screen capture
./sav-video test

# Test OCR on current screen
screencapture -x test_screenshot.png
cargo run --package savant-ocr -- extract --input test_screenshot.png --fast
rm test_screenshot.png
```

### **Integration Test**
```bash
# Test multimodal correlation
./start-daemons
sleep 10
./test-systems
./stop-daemons
```

## 🚨 Common Issues & Solutions

### **"Operation not permitted" Errors**
- **Cause**: Missing Screen Recording or Microphone permissions
- **Fix**: Grant permissions in System Preferences, restart terminal

### **"No audio devices found"**
- **Cause**: Audio drivers or permissions
- **Fix**: Check Audio MIDI Setup app, verify microphone permissions

### **"Screen capture failed"**
- **Cause**: Screen Recording permission not granted
- **Fix**: Enable in Security & Privacy, may need to restart terminal/IDE

### **"Ollama not responding"**
- **Cause**: Ollama server not running or firewall blocking
- **Fix**: 
```bash
ollama serve  # Start server
curl http://localhost:11434/api/tags  # Test connection
```

### **OCR "Unknown format" errors**
- **Cause**: Image format or Tesseract configuration
- **Fix**:
```bash
brew reinstall tesseract
tesseract --list-langs  # Verify installation
```

### **Performance Issues**
- **Cause**: Insufficient permissions or system resources
- **Fix**: 
  - Grant Full Disk Access for better performance
  - Close unnecessary applications
  - Check available memory: `./verify-permissions`

## 📱 Application-Specific Setup

### **VS Code Users**
If running from VS Code integrated terminal:
1. Grant VS Code the same permissions as Terminal
2. Or run from external terminal for better compatibility

### **Docker Users**
If using Docker containers:
1. Audio/video capture won't work from inside containers
2. Run Savant AI on host system
3. Use Docker only for isolated testing

### **Remote/SSH Users**
- Screen capture requires local display access
- Audio capture requires local audio devices
- Consider using X11 forwarding for remote display

## ✅ Verification Checklist

After setup, verify each component:

- [ ] **Dependencies installed**: `brew list | grep -E "(ollama|tesseract|imagemagick)"`
- [ ] **Screen Recording**: Terminal in Security & Privacy → Screen Recording
- [ ] **Microphone**: Terminal in Security & Privacy → Microphone  
- [ ] **Ollama running**: `curl -s http://localhost:11434/api/tags`
- [ ] **Audio devices**: `./scripts/audio/audio-devices.sh`
- [ ] **OCR working**: `cargo run --package savant-ocr -- test`
- [ ] **All systems**: `./verify-permissions`

## 🎯 Quick Start After Setup

Once permissions are configured:

```bash
# Start everything
./start-daemons

# Monitor in real-time
./monitor-daemons

# Test complete system
./test-systems

# Stop when done
./stop-daemons
```

For ongoing issues, check:
- Console app for system-level error messages
- `./sav logs` and `./sav-video logs` for application logs
- Activity Monitor for resource usage