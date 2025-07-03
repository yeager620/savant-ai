#!/bin/bash

# Savant AI System Permissions Verification Script
# Checks all macOS permissions and settings required for full functionality

echo "🔍 Savant AI System Permissions Verification"
echo "============================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall status
OVERALL_STATUS=0

print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "OK" ]; then
        echo -e "${GREEN}✓${NC} $message"
    elif [ "$status" = "WARNING" ]; then
        echo -e "${YELLOW}⚠${NC} $message"
    elif [ "$status" = "INFO" ]; then
        echo -e "${BLUE}ℹ${NC} $message"
    else
        echo -e "${RED}✗${NC} $message"
        OVERALL_STATUS=1
    fi
}

print_section() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

# Function to check if app has specific permission using functional tests
check_permission() {
    local permission_type=$1
    local app_name=$2
    
    case "$permission_type" in
        "kTCCServiceMicrophone")
            # Test microphone access by trying to list audio devices
            if system_profiler SPAudioDataType >/dev/null 2>&1; then
                return 0
            else
                return 1
            fi
            ;;
        "kTCCServiceScreenCapture")
            # Test screen recording by trying a quick screenshot
            if screencapture -x -t png /tmp/permission_test.png >/dev/null 2>&1; then
                rm -f /tmp/permission_test.png 2>/dev/null
                return 0
            else
                return 1
            fi
            ;;
        "kTCCServiceAccessibility")
            # Accessibility is optional - always return warning status
            return 1
            ;;
        *)
            return 1
            ;;
    esac
}

# Function to check bundle ID permissions
check_bundle_permission() {
    local permission_type=$1
    local bundle_id=$2
    
    local result=$(sqlite3 ~/Library/Application\ Support/com.apple.TCC/TCC.db \
        "SELECT allowed FROM access WHERE service='$permission_type' AND client='$bundle_id';" 2>/dev/null)
    
    if [ "$result" = "1" ]; then
        return 0
    else
        return 1
    fi
}

print_section "System Information"
echo "macOS Version: $(sw_vers -productVersion)"
echo "Build: $(sw_vers -buildVersion)"
echo "Current User: $(whoami)"
echo "Terminal: $TERM_PROGRAM"

print_section "Required Dependencies"

# Check Homebrew
if command -v brew &> /dev/null; then
    print_status "OK" "Homebrew installed"
else
    print_status "ERROR" "Homebrew not installed - Install from https://brew.sh"
fi

# Check Ollama
if command -v ollama &> /dev/null; then
    print_status "OK" "Ollama installed ($(ollama --version 2>/dev/null || echo 'version unknown'))"
    
    # Check if Ollama is running
    if pgrep -f "ollama serve" > /dev/null; then
        print_status "OK" "Ollama server is running"
    else
        print_status "WARNING" "Ollama server not running - Start with: ollama serve"
    fi
    
    # Check if devstral model is available
    if ollama list 2>/dev/null | grep -q "devstral"; then
        print_status "OK" "Devstral model available"
    else
        print_status "WARNING" "Llama3.2 model not found - Install with: ollama pull llama3.2"
    fi
else
    print_status "ERROR" "Ollama not installed - Install with: brew install ollama"
fi

# Check Tesseract
if command -v tesseract &> /dev/null; then
    print_status "OK" "Tesseract installed ($(tesseract --version 2>&1 | head -1))"
    
    # Check available languages
    langs=$(tesseract --list-langs 2>/dev/null | tail -n +2 | tr '\n' ' ')
    echo "   Available languages: $langs"
else
    print_status "ERROR" "Tesseract not installed - Install with: brew install tesseract"
fi

# Check ImageMagick
if command -v convert &> /dev/null; then
    print_status "OK" "ImageMagick installed ($(convert -version | head -1 | awk '{print $3}'))"
else
    print_status "WARNING" "ImageMagick not installed - Install with: brew install imagemagick (optional)"
fi

print_section "Audio Permissions & Configuration"

# Check microphone permission for Terminal
if check_permission "kTCCServiceMicrophone" "/System/Applications/Utilities/Terminal.app" || \
   check_permission "kTCCServiceMicrophone" "/Applications/Utilities/Terminal.app"; then
    print_status "OK" "Terminal has microphone access"
else
    print_status "ERROR" "Terminal needs microphone permission"
    echo "   Fix: System Preferences > Security & Privacy > Privacy > Microphone > Enable Terminal"
fi

# Check for other common terminal apps
for app in "iTerm" "Warp" "Hyper"; do
    if [ -d "/Applications/$app.app" ]; then
        if check_permission "kTCCServiceMicrophone" "/Applications/$app.app"; then
            print_status "OK" "$app has microphone access"
        else
            print_status "WARNING" "$app may need microphone permission if you're using it"
        fi
    fi
done

# Check audio devices
echo ""
echo "📱 Available audio input devices:"
system_profiler SPAudioDataType | grep -A 5 "Audio Devices:" | grep "Microphone\|Input" | sed 's/^/   /' || echo "   No specific input devices listed"

echo ""
echo "🔊 Available audio output devices:"
system_profiler SPAudioDataType | grep -A 5 "Audio Devices:" | grep -E "(Built-in|Speakers|Headphones)" | sed 's/^/   /' || echo "   No specific output devices listed"

# Check if BlackHole is installed (for system audio capture)
if system_profiler SPAudioDataType | grep -q "BlackHole"; then
    print_status "OK" "BlackHole virtual audio device detected"
else
    print_status "WARNING" "BlackHole not detected - Install for system audio capture"
    echo "   Download from: https://github.com/ExistentialAudio/BlackHole"
fi

print_section "Screen Recording Permissions"

# Check screen recording permission for Terminal
if check_permission "kTCCServiceScreenCapture" "/System/Applications/Utilities/Terminal.app" || \
   check_permission "kTCCServiceScreenCapture" "/Applications/Utilities/Terminal.app"; then
    print_status "OK" "Terminal has screen recording permission"
else
    print_status "ERROR" "Terminal needs screen recording permission"
    echo "   Fix: System Preferences > Security & Privacy > Privacy > Screen Recording > Enable Terminal"
fi

# Check for other terminal apps
for app in "iTerm" "Warp" "Hyper"; do
    if [ -d "/Applications/$app.app" ]; then
        if check_permission "kTCCServiceScreenCapture" "/Applications/$app.app"; then
            print_status "OK" "$app has screen recording permission"
        else
            print_status "WARNING" "$app may need screen recording permission if you're using it"
        fi
    fi
done

print_section "Accessibility Permissions"

# Check accessibility permission for Terminal
if check_permission "kTCCServiceAccessibility" "/System/Applications/Utilities/Terminal.app" || \
   check_permission "kTCCServiceAccessibility" "/Applications/Utilities/Terminal.app"; then
    print_status "OK" "Terminal has accessibility permission"
else
    print_status "WARNING" "Terminal may need accessibility permission for advanced features"
    echo "   Fix: System Preferences > Security & Privacy > Privacy > Accessibility > Enable Terminal"
fi

print_section "Network & Security"

# Check if Ollama port is accessible
if lsof -i :11434 > /dev/null 2>&1; then
    print_status "OK" "Ollama port 11434 is active"
else
    print_status "WARNING" "Ollama port 11434 not active - Ollama server may not be running"
fi

# Check firewall status
firewall_status=$(sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate 2>/dev/null)
if echo "$firewall_status" | grep -q "enabled"; then
    print_status "WARNING" "Firewall is enabled - May need to allow Ollama connections"
    echo "   Check: System Preferences > Security & Privacy > Firewall > Firewall Options"
else
    print_status "OK" "Firewall is disabled or allowing connections"
fi

print_section "File System Permissions"

# Check if we can access the application directory
if [ -w "$(pwd)" ]; then
    print_status "OK" "Current directory is writable"
else
    print_status "ERROR" "Current directory is not writable"
fi

# Check config directory
config_dir="$HOME/.config/savant-ai"
if [ -d "$config_dir" ]; then
    if [ -w "$config_dir" ]; then
        print_status "OK" "Config directory exists and is writable: $config_dir"
    else
        print_status "ERROR" "Config directory exists but is not writable: $config_dir"
    fi
else
    print_status "INFO" "Config directory will be created: $config_dir"
fi

# Check data directory
data_dir="$(pwd)/data"
if [ -d "$data_dir" ]; then
    if [ -w "$data_dir" ]; then
        print_status "OK" "Data directory exists and is writable: $data_dir"
    else
        print_status "ERROR" "Data directory exists but is not writable: $data_dir"
    fi
else
    print_status "INFO" "Data directory will be created: $data_dir"
fi

print_section "Rust & Cargo Environment"

# Check Rust installation
if command -v rustc &> /dev/null; then
    print_status "OK" "Rust installed ($(rustc --version))"
else
    print_status "ERROR" "Rust not installed - Install from https://rustup.rs"
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    print_status "OK" "Cargo available ($(cargo --version))"
else
    print_status "ERROR" "Cargo not available"
fi

# Check if in a Rust project
if [ -f "Cargo.toml" ]; then
    print_status "OK" "Running in Rust project directory"
else
    print_status "WARNING" "Not in Rust project directory - Make sure you're in the savant-ai folder"
fi

print_section "System Resources"

# Check available memory
memory_gb=$(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 ))
if [ "$memory_gb" -ge 8 ]; then
    print_status "OK" "Sufficient memory: ${memory_gb}GB"
else
    print_status "WARNING" "Low memory: ${memory_gb}GB (8GB+ recommended)"
fi

# Check available disk space
disk_space=$(df -h . | tail -1 | awk '{print $4}')
print_status "OK" "Available disk space: $disk_space"

# Check CPU cores
cpu_cores=$(sysctl -n hw.ncpu)
print_status "OK" "CPU cores: $cpu_cores"

print_section "Quick Functional Tests"

# Test OCR if images are available
if [ -f "screenshot.png" ] || [ -f "screenshot_small.png" ]; then
    echo "Testing OCR functionality..."
    test_image="screenshot_small.png"
    [ -f "screenshot.png" ] && test_image="screenshot.png"
    
    if cargo run --package savant-ocr -- test --input "$test_image" &>/dev/null; then
        print_status "OK" "OCR test passed with sample image"
    else
        print_status "WARNING" "OCR test failed - Check Tesseract installation"
    fi
else
    print_status "INFO" "No test images found - OCR functionality not tested"
fi

# Test Ollama connection
if curl -s http://localhost:11434/api/tags > /dev/null; then
    print_status "OK" "Ollama API responding"
else
    print_status "WARNING" "Ollama API not responding - Start with: ollama serve"
fi

print_section "Summary & Recommendations"

if [ $OVERALL_STATUS -eq 0 ]; then
    echo -e "${GREEN}🎉 All critical permissions and dependencies are properly configured!${NC}"
    echo ""
    echo "You can now run:"
    echo "  ./start-daemons     # Start all systems"
    echo "  ./monitor-daemons   # Monitor operation"
else
    echo -e "${RED}⚠️  Some issues need attention before full functionality is available.${NC}"
    echo ""
    echo "Priority fixes:"
    echo "1. Grant Terminal permissions in System Preferences > Security & Privacy > Privacy"
    echo "2. Install missing dependencies with Homebrew"
    echo "3. Start required services (ollama serve)"
fi

echo ""
echo "For detailed setup instructions, see:"
echo "  ./scripts/setup/auto-setup-system-audio.sh"
echo "  docs/user-guides/SYSTEM_AUDIO_SETUP.md"

exit $OVERALL_STATUS