#!/bin/bash

# Automated System Audio Setup
# This script automates BlackHole installation and audio routing configuration

set -e  # Exit on any error

echo "🎵 Automated System Audio Setup"
echo "==============================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script only works on macOS"
    exit 1
fi

# Request administrator privileges
echo "🔐 This setup requires administrator privileges for audio configuration..."
sudo -v

# Keep sudo alive
while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done 2>/dev/null &

print_info "Step 1: Installing BlackHole..."

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    print_warning "Homebrew not found. Installing Homebrew first..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install BlackHole
if brew list blackhole-2ch >/dev/null 2>&1; then
    print_status "BlackHole is already installed"
else
    print_info "Installing BlackHole 2ch..."
    brew install blackhole-2ch
    print_status "BlackHole installed successfully"
fi

print_info "Step 2: Configuring Audio MIDI Setup..."

# Create AppleScript to automate Audio MIDI Setup
cat << 'EOF' > /tmp/setup_audio_midi.applescript
tell application "Audio MIDI Setup"
    activate
    delay 2
    
    -- Try to create Multi-Output Device
    tell application "System Events"
        tell process "Audio MIDI Setup"
            -- Click the + button
            try
                click button 1 of group 1 of splitter group 1 of window 1
                delay 1
                
                -- Select "Create Multi-Output Device"
                click menu item "Create Multi-Output Device" of menu 1 of button 1 of group 1 of splitter group 1 of window 1
                delay 2
                
                -- The Multi-Output Device should now be created and selected
                -- We need to check the checkboxes for built-in output and BlackHole
                
                -- Look for checkboxes in the device list
                repeat with i from 1 to 10
                    try
                        set checkboxName to name of checkbox i of scroll area 1 of splitter group 1 of window 1
                        if checkboxName contains "Built-in Output" or checkboxName contains "MacBook Pro Speakers" then
                            click checkbox i of scroll area 1 of splitter group 1 of window 1
                            delay 0.5
                        end if
                        if checkboxName contains "BlackHole" then
                            click checkbox i of scroll area 1 of splitter group 1 of window 1
                            delay 0.5
                        end if
                    on error
                        -- Continue if checkbox doesn't exist
                    end try
                end repeat
                
            on error theError
                display dialog "Error creating Multi-Output Device: " & theError
            end try
        end tell
    end tell
    
    delay 2
    quit
end tell
EOF

print_info "Running Audio MIDI Setup automation..."
osascript /tmp/setup_audio_midi.applescript 2>/dev/null || print_warning "Audio MIDI Setup automation had issues, manual verification may be needed"

# Clean up temporary file
rm -f /tmp/setup_audio_midi.applescript

print_info "Step 3: Setting up system audio routing..."

# Create AppleScript to set default audio devices
cat << 'EOF' > /tmp/set_audio_devices.applescript
tell application "System Preferences"
    activate
    set current pane to pane "com.apple.preference.sound"
    delay 2
    
    tell application "System Events"
        tell process "System Preferences"
            -- Click on Output tab
            try
                click button "Output" of tab group 1 of window 1
                delay 1
                
                -- Look for Multi-Output Device and select it
                repeat with i from 1 to 10
                    try
                        set deviceName to name of row i of table 1 of scroll area 1 of tab group 1 of window 1
                        if deviceName contains "Multi-Output Device" then
                            click row i of table 1 of scroll area 1 of tab group 1 of window 1
                            exit repeat
                        end if
                    on error
                        -- Continue if row doesn't exist
                    end try
                end repeat
                
            on error theError
                display dialog "Error setting output device: " & theError
            end try
        end tell
    end tell
    
    delay 1
    quit
end tell
EOF

print_info "Configuring system audio output..."
osascript /tmp/set_audio_devices.applescript 2>/dev/null || print_warning "System audio configuration had issues, manual verification may be needed"

# Clean up temporary file
rm -f /tmp/set_audio_devices.applescript

print_info "Step 4: Verifying setup..."

# Test audio device availability
BLACKHOLE_AVAILABLE=$(cargo run --package savant-audio --bin list-devices 2>/dev/null | grep -i "blackhole" || echo "")

if [[ -n "$BLACKHOLE_AVAILABLE" ]]; then
    print_status "BlackHole device detected and available"
else
    print_warning "BlackHole device not detected in audio devices list"
fi

print_info "Step 5: Creating system audio capture daemon..."

# Create a background service script
cat << 'EOF' > ~/savant-audio-daemon.sh
#!/bin/bash

# Savant Audio Daemon - Continuous system audio capture
# This runs in the background and captures all system audio

CAPTURE_DIR="$HOME/savant-audio-captures"
SEGMENT_DURATION=300  # 5 minutes per segment
LOG_FILE="$HOME/savant-audio-daemon.log"

# Create capture directory
mkdir -p "$CAPTURE_DIR"

# Function to log with timestamp
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S'): $1" | tee -a "$LOG_FILE"
}

# Function to capture audio segment
capture_segment() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local output_file="$CAPTURE_DIR/system_audio_$timestamp.md"
    
    log_message "Starting capture segment: $output_file"
    
    cd ~/Documents/savant-ai
    
    timeout $SEGMENT_DURATION cargo run --package savant-transcribe -- \
        --duration $SEGMENT_DURATION \
        --system \
        --output "$output_file" 2>>"$LOG_FILE"
    
    if [ $? -eq 0 ]; then
        log_message "Capture completed: $output_file"
    else
        log_message "Capture failed for segment $timestamp"
    fi
}

# Main daemon loop
log_message "Savant Audio Daemon started (PID: $$)"

while true; do
    capture_segment
    sleep 5  # Brief pause between segments
done
EOF

chmod +x ~/savant-audio-daemon.sh

print_status "Audio capture daemon created at ~/savant-audio-daemon.sh"

print_info "Step 6: Creating launch daemon for automatic startup..."

# Create LaunchAgent plist for automatic startup
sudo mkdir -p /Library/LaunchAgents

cat << EOF > /tmp/com.savant.audio.daemon.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.savant.audio.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$HOME/savant-audio-daemon.sh</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$HOME/savant-audio-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>$HOME/savant-audio-daemon.log</string>
    <key>WorkingDirectory</key>
    <string>$HOME/Documents/savant-ai</string>
</dict>
</plist>
EOF

sudo cp /tmp/com.savant.audio.daemon.plist /Library/LaunchAgents/
sudo chown root:wheel /Library/LaunchAgents/com.savant.audio.daemon.plist
rm /tmp/com.savant.audio.daemon.plist

print_status "Launch daemon installed for automatic startup"

echo ""
echo "🎉 SETUP COMPLETE!"
echo "=================="
echo ""
print_status "System audio capture is now configured and ready!"
echo ""
print_info "What was configured:"
echo "  • BlackHole 2ch virtual audio device"
echo "  • Multi-Output Device (speakers + BlackHole)"
echo "  • System audio routing"
echo "  • Background capture daemon"
echo "  • Automatic startup service"
echo ""
print_info "How it works:"
echo "  • You'll still hear audio through your speakers/headphones"
echo "  • Your microphone still works normally"
echo "  • System captures ALL audio in the background"
echo "  • Transcripts saved to ~/savant-audio-captures/"
echo ""
print_info "Controls:"
echo "  • Start daemon: sudo launchctl load /Library/LaunchAgents/com.savant.audio.daemon.plist"
echo "  • Stop daemon:  sudo launchctl unload /Library/LaunchAgents/com.savant.audio.daemon.plist"
echo "  • View logs:    tail -f ~/savant-audio-daemon.log"
echo "  • Captures:     ls ~/savant-audio-captures/"
echo ""
print_warning "Note: The daemon will start automatically on next reboot"
echo ""
echo "🚀 Start the daemon now? [y/N]"
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    sudo launchctl load /Library/LaunchAgents/com.savant.audio.daemon.plist
    print_status "Audio capture daemon started!"
    echo ""
    print_info "Monitor activity with: tail -f ~/savant-audio-daemon.log"
fi

echo ""
print_status "Setup completed successfully! 🎵"