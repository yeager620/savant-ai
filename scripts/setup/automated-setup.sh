#!/bin/bash

# Savant AI Automated Setup Script
# Automates everything possible via CLI, then guides through required GUI steps

echo "🚀 Savant AI Automated Setup"
echo "============================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BOLD}ℹ $1${NC}"
}

# Track what needs GUI interaction
GUI_STEPS=()

print_step "Phase 1: Automated Dependency Installation"
echo ""

# Check and install Homebrew
if ! command -v brew &> /dev/null; then
    print_step "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add to PATH
    echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zshrc
    eval "$(/opt/homebrew/bin/brew shellenv)"
    print_success "Homebrew installed"
else
    print_success "Homebrew already installed"
fi

# Install required packages
print_step "Installing required packages..."
packages_to_install=()

if ! command -v ollama &> /dev/null; then
    packages_to_install+=("ollama")
fi

if ! command -v tesseract &> /dev/null; then
    packages_to_install+=("tesseract")
fi

if ! command -v convert &> /dev/null && ! command -v magick &> /dev/null; then
    packages_to_install+=("imagemagick")
fi

if [ ${#packages_to_install[@]} -gt 0 ]; then
    print_step "Installing: ${packages_to_install[*]}"
    brew install "${packages_to_install[@]}"
    print_success "Packages installed"
else
    print_success "All required packages already installed"
fi

# Install Rust if needed
if ! command -v rustc &> /dev/null; then
    print_step "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed"
else
    print_success "Rust already installed"
fi

print_step "Phase 2: Service Configuration"
echo ""

# Start and configure Ollama
if command -v ollama &> /dev/null; then
    # Start Ollama server if not running
    if ! pgrep -f "ollama serve" > /dev/null; then
        print_step "Starting Ollama server..."
        ollama serve &
        sleep 3
        print_success "Ollama server started"
    else
        print_success "Ollama server already running"
    fi
    
    # Install devstral model if not present
    if ! ollama list 2>/dev/null | grep -q "devstral"; then
        print_step "Installing devstral model (this may take a few minutes)..."
        ollama pull devstral
        print_success "Devstral model installed"
    else
        print_success "Devstral model already available"
    fi
else
    print_error "Ollama installation failed"
fi

# Create necessary directories
print_step "Creating application directories..."
mkdir -p ~/.config/savant-ai
mkdir -p "$(pwd)/data"
print_success "Directories created"

print_step "Phase 3: Optional Audio Enhancement Setup"
echo ""

# Check for BlackHole
if ! system_profiler SPAudioDataType | grep -q "BlackHole"; then
    print_warning "BlackHole virtual audio device not detected"
    echo "BlackHole enables system audio capture for comprehensive monitoring."
    echo ""
    read -p "Install BlackHole for system audio capture? (y/N): " install_blackhole
    
    if [[ $install_blackhole =~ ^[Yy]$ ]]; then
        print_step "Opening BlackHole download page..."
        open "https://github.com/ExistentialAudio/BlackHole/releases"
        print_info "Download and install BlackHole, then run this script again"
        GUI_STEPS+=("Install BlackHole from GitHub releases page")
    fi
else
    print_success "BlackHole virtual audio device detected"
fi

print_step "Phase 4: Project Build & Verification"
echo ""

# Build the project
print_step "Building Savant AI project..."
if cargo build --workspace; then
    print_success "Project built successfully"
else
    print_error "Project build failed - check Rust installation"
fi

print_step "Phase 5: macOS Permissions Setup"
echo ""

print_warning "The following permissions MUST be granted manually via System Preferences:"
echo ""

# Check current permissions and build GUI steps list
BUNDLE_ID="/System/Applications/Utilities/Terminal.app"
if [ "$TERM_PROGRAM" = "iTerm.app" ]; then
    BUNDLE_ID="/Applications/iTerm.app"
elif [ "$TERM_PROGRAM" = "WarpTerminal" ]; then
    BUNDLE_ID="/Applications/Warp.app"
fi

# Function to check TCC permission
check_tcc_permission() {
    local service=$1
    local bundle_id=$2
    
    local result=$(sqlite3 ~/Library/Application\ Support/com.apple.TCC/TCC.db \
        "SELECT allowed FROM access WHERE service='$service' AND client='$bundle_id';" 2>/dev/null)
    
    [ "$result" = "1" ]
}

# Check Screen Recording
if ! check_tcc_permission "kTCCServiceScreenCapture" "$BUNDLE_ID"; then
    GUI_STEPS+=("Screen Recording permission for $(basename "$BUNDLE_ID")")
    echo "❌ Screen Recording: Required for video capture and OCR"
else
    echo "✅ Screen Recording: Already granted"
fi

# Check Microphone
if ! check_tcc_permission "kTCCServiceMicrophone" "$BUNDLE_ID"; then
    GUI_STEPS+=("Microphone permission for $(basename "$BUNDLE_ID")")
    echo "❌ Microphone: Required for audio transcription"
else
    echo "✅ Microphone: Already granted"
fi

# Check Accessibility (optional)
if ! check_tcc_permission "kTCCServiceAccessibility" "$BUNDLE_ID"; then
    echo "⚠️  Accessibility: Optional, for advanced UI detection"
else
    echo "✅ Accessibility: Already granted"
fi

if [ ${#GUI_STEPS[@]} -gt 0 ]; then
    echo ""
    print_step "REQUIRED: Manual Permission Setup"
    echo ""
    print_info "Opening System Preferences to Privacy settings..."
    echo ""
    
    # Open System Preferences to the right location
    open "x-apple.systempreferences:com.apple.preference.security?Privacy"
    
    echo "In the opened System Preferences window:"
    echo ""
    
    for i in "${!GUI_STEPS[@]}"; do
        step="${GUI_STEPS[$i]}"
        echo "$(($i + 1)). $step"
        
        if [[ $step == *"Screen Recording"* ]]; then
            echo "   → Click 'Screen Recording' in left sidebar"
            echo "   → Check the box next to your terminal app"
            echo "   → You may need to quit and restart your terminal"
        elif [[ $step == *"Microphone"* ]]; then
            echo "   → Click 'Microphone' in left sidebar"  
            echo "   → Check the box next to your terminal app"
        fi
        echo ""
    done
    
    echo "After granting permissions:"
    echo "• Quit and restart your terminal application"
    echo "• Run: ./verify-permissions"
    echo "• Once all permissions are granted, run: ./start-daemons"
    
    print_info "Press Enter when you've completed the permission setup..."
    read -r
else
    print_success "All required permissions already granted!"
fi

print_step "Phase 6: Final Verification & Testing"
echo ""

# Run verification
print_step "Running system verification..."
if ./verify-permissions; then
    print_success "System verification passed!"
    echo ""
    print_step "🎉 Setup Complete! Ready to start Savant AI"
    echo ""
    echo "Next steps:"
    echo "1. Start all systems: ./start-daemons"
    echo "2. Monitor operation: ./monitor-daemons"  
    echo "3. Test functionality: ./test-systems"
    echo "4. Run main app: cargo tauri dev"
    echo ""
    
    # Offer to start immediately
    read -p "Start Savant AI systems now? (Y/n): " start_now
    if [[ ! $start_now =~ ^[Nn]$ ]]; then
        print_step "Starting Savant AI systems..."
        ./start-daemons
    fi
else
    print_warning "System verification found issues"
    echo ""
    echo "Common fixes:"
    echo "• Restart your terminal after granting permissions"
    echo "• Run: ./verify-permissions again"
    echo "• Check the detailed guide: docs/user-guides/PERMISSIONS_SETUP.md"
fi

print_step "Setup Summary"
echo ""
print_info "What was automated:"
echo "✓ Homebrew installation"
echo "✓ Package installation (ollama, tesseract, imagemagick)"
echo "✓ Rust installation (if needed)"
echo "✓ Ollama server startup"
echo "✓ Model installation (llama3.2)"
echo "✓ Directory creation"
echo "✓ Project compilation"
echo ""

if [ ${#GUI_STEPS[@]} -gt 0 ]; then
    print_info "What requires manual setup:"
    for step in "${GUI_STEPS[@]}"; do
        echo "• $step"
    done
    echo ""
    echo "These permissions cannot be automated due to macOS security requirements."
else
    print_success "No manual setup required - everything is automated!"
fi