#!/bin/bash

# Savant AI - Complete Application Packaging and Distribution Script
# This script packages the entire Savant AI application for distribution

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
PACKAGE_NAME="savant-ai"
VERSION="1.0.0"
DIST_DIR="$PROJECT_DIR/dist-package"

echo "🚀 Savant AI Packaging Script"
echo "================================"
echo "Project: $PROJECT_DIR"
echo "Version: $VERSION"
echo "Package: $PACKAGE_NAME"
echo ""

# Clean and create distribution directory
echo "📁 Creating distribution directory..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/$PACKAGE_NAME"

# Copy essential files
echo "📋 Copying project files..."
cp -r "$PROJECT_DIR"/{src,src-tauri,crates,scripts,docs,examples} "$DIST_DIR/$PACKAGE_NAME/"
cp "$PROJECT_DIR"/{Cargo.toml,Cargo.lock,Trunk.toml,index.html,styles.css} "$DIST_DIR/$PACKAGE_NAME/"
cp "$PROJECT_DIR"/{README.md,CLAUDE.md} "$DIST_DIR/$PACKAGE_NAME/"

# Create data directories
echo "📂 Creating data directories..."
mkdir -p "$DIST_DIR/$PACKAGE_NAME/data"/{audio-captures,daemon-logs,test-captures}

# Copy models directory if it exists
if [ -d "$PROJECT_DIR/models" ]; then
    echo "🤖 Copying AI models..."
    cp -r "$PROJECT_DIR/models" "$DIST_DIR/$PACKAGE_NAME/"
fi

# Create installation script
echo "⚙️ Creating installation script..."
cat > "$DIST_DIR/$PACKAGE_NAME/install.sh" << 'EOF'
#!/bin/bash

# Savant AI Installation Script
# Installs and configures the complete Savant AI application

set -e

INSTALL_DIR="$HOME/Applications/savant-ai"
PROJECT_NAME="savant-ai"

echo "🚀 Installing Savant AI"
echo "======================"
echo "Target: $INSTALL_DIR"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✅ Rust installed successfully"
fi

# Check if Homebrew is installed (for BlackHole)
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew not found. Installing..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    echo "✅ Homebrew installed successfully"
fi

# Create installation directory
echo "📁 Creating installation directory..."
mkdir -p "$INSTALL_DIR"

# Copy application files
echo "📋 Installing application files..."
cp -r ./* "$INSTALL_DIR/"

# Make scripts executable
echo "🔧 Setting up permissions..."
chmod +x "$INSTALL_DIR"/scripts/**/*.sh
chmod +x "$INSTALL_DIR"/examples/*.sh

# Install dependencies
echo "📦 Installing dependencies..."
cd "$INSTALL_DIR"

# Install BlackHole
echo "🎵 Installing BlackHole audio driver..."
brew install blackhole-2ch

# Download Whisper model if not present
if [ ! -f "models/ggml-base.en.bin" ]; then
    echo "🤖 Downloading Whisper AI model..."
    mkdir -p models
    curl -L -o models/ggml-base.en.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
fi

# Build the application
echo "🔨 Building Savant AI..."
cargo build --release

# Set up audio capture daemon
echo "🎙️ Setting up audio capture daemon..."
./scripts/setup/auto-setup-system-audio.sh

# Create desktop shortcuts/aliases
echo "🔗 Creating command line aliases..."
SHELL_RC="$HOME/.zshrc"
if [ "$SHELL" = "/bin/bash" ]; then
    SHELL_RC="$HOME/.bashrc"
fi

# Add aliases if they don't exist
if ! grep -q "savant-ai" "$SHELL_RC" 2>/dev/null; then
    echo "" >> "$SHELL_RC"
    echo "# Savant AI aliases" >> "$SHELL_RC"
    echo "alias savant-ai='cd $INSTALL_DIR && cargo tauri dev'" >> "$SHELL_RC"
    echo "alias savant-audio='cd $INSTALL_DIR && ./scripts/audio/savant-audio-control.sh'" >> "$SHELL_RC"
    echo "alias savant-devices='cd $INSTALL_DIR && ./scripts/audio/audio-devices.sh'" >> "$SHELL_RC"
fi

# Installation complete
echo ""
echo "✅ Savant AI installation complete!"
echo ""
echo "🎯 Quick Start:"
echo "  1. Restart your terminal (for aliases)"
echo "  2. Set up Multi-Output Device in Audio MIDI Setup"
echo "  3. Run: savant-ai (to start the application)"
echo "  4. Run: savant-audio status (to check audio capture)"
echo ""
echo "📚 Documentation: $INSTALL_DIR/docs/README.md"
echo "🆘 Support: $INSTALL_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"
echo ""
echo "🎵 Next steps:"
echo "  - Open Audio MIDI Setup and create Multi-Output Device"
echo "  - Include both MacBook Speakers and BlackHole 2ch"
echo "  - Set Multi-Output Device as default audio output"
echo "  - Start audio capture: savant-audio start"
EOF

chmod +x "$DIST_DIR/$PACKAGE_NAME/install.sh"

# Create uninstall script
echo "🗑️ Creating uninstall script..."
cat > "$DIST_DIR/$PACKAGE_NAME/uninstall.sh" << 'EOF'
#!/bin/bash

# Savant AI Uninstall Script

INSTALL_DIR="$HOME/Applications/savant-ai"

echo "🗑️ Uninstalling Savant AI"
echo "========================"

# Stop audio daemon
echo "⏹️ Stopping audio daemon..."
launchctl unload ~/Library/LaunchAgents/com.savant.audio.daemon.plist 2>/dev/null || true

# Remove LaunchAgent
echo "🧹 Removing system services..."
rm -f ~/Library/LaunchAgents/com.savant.audio.daemon.plist

# Remove aliases from shell config
echo "🔗 Removing aliases..."
SHELL_RC="$HOME/.zshrc"
if [ "$SHELL" = "/bin/bash" ]; then
    SHELL_RC="$HOME/.bashrc"
fi

if [ -f "$SHELL_RC" ]; then
    sed -i '' '/# Savant AI aliases/,+3d' "$SHELL_RC" 2>/dev/null || true
fi

# Ask about removing application directory
echo ""
read -p "Remove application directory ($INSTALL_DIR)? [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$INSTALL_DIR"
    echo "✅ Application directory removed"
else
    echo "📁 Application directory preserved"
fi

# Ask about removing BlackHole
echo ""
read -p "Remove BlackHole audio driver? [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    brew uninstall blackhole-2ch 2>/dev/null || true
    echo "✅ BlackHole removed"
else
    echo "🎵 BlackHole preserved"
fi

echo ""
echo "✅ Savant AI uninstallation complete"
echo "💡 You may need to restart your terminal for alias changes to take effect"
EOF

chmod +x "$DIST_DIR/$PACKAGE_NAME/uninstall.sh"

# Create README for distribution
echo "📖 Creating distribution README..."
cat > "$DIST_DIR/$PACKAGE_NAME/INSTALL_README.md" << 'EOF'
# Savant AI - Installation Guide

## 🚀 Quick Installation

```bash
# Extract the package
cd savant-ai

# Run the installation script
./install.sh
```

The installer will:
- Install Rust (if needed)
- Install Homebrew (if needed)
- Install BlackHole audio driver
- Download AI models
- Build the application
- Set up audio capture daemon
- Create convenient command aliases

## 📋 System Requirements

- **macOS**: 10.13+ (High Sierra or newer)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 2GB free space
- **Internet**: Required for initial setup

## 🎯 Quick Start

After installation:

1. **Set up audio routing**:
   - Open Audio MIDI Setup (`Cmd+Space`, type "Audio MIDI Setup")
   - Create Multi-Output Device (+ button)
   - Check both "MacBook Pro Speakers" and "BlackHole 2ch"
   - Set as default output in System Preferences → Sound

2. **Start the application**:
   ```bash
   savant-ai  # Opens the chat interface
   ```

3. **Start audio capture**:
   ```bash
   savant-audio start  # Begins background audio transcription
   ```

## 📚 Documentation

- **User Guide**: `docs/user-guides/SYSTEM_AUDIO_SETUP.md`
- **Development**: `docs/development/`
- **API Reference**: `docs/api/`

## 🆘 Troubleshooting

- **Audio not capturing**: Check Multi-Output Device setup
- **Build errors**: Ensure Rust is properly installed
- **Permission issues**: Make sure scripts are executable

## 🗑️ Uninstalling

```bash
./uninstall.sh
```

## 📞 Support

- Check documentation in the `docs/` directory
- Review troubleshooting guides
- Ensure all system requirements are met
EOF

# Create version info
echo "📊 Creating version information..."
cat > "$DIST_DIR/$PACKAGE_NAME/VERSION" << EOF
Savant AI v$VERSION
Built: $(date)
Platform: macOS
Components: Audio Capture, Chat Interface, AI Transcription
EOF

# Create package archive
echo "📦 Creating distribution archive..."
cd "$DIST_DIR"
tar -czf "${PACKAGE_NAME}-v${VERSION}.tar.gz" "$PACKAGE_NAME"

# Create installer disk image (macOS)
if command -v hdiutil &> /dev/null; then
    echo "💽 Creating disk image..."
    hdiutil create -srcfolder "$PACKAGE_NAME" -format UDZO "${PACKAGE_NAME}-v${VERSION}.dmg"
fi

echo ""
echo "✅ Packaging complete!"
echo ""
echo "📦 Distribution files created:"
echo "  • Archive: $DIST_DIR/${PACKAGE_NAME}-v${VERSION}.tar.gz"
if [ -f "$DIST_DIR/${PACKAGE_NAME}-v${VERSION}.dmg" ]; then
echo "  • Disk Image: $DIST_DIR/${PACKAGE_NAME}-v${VERSION}.dmg"
fi
echo "  • Directory: $DIST_DIR/$PACKAGE_NAME/"
echo ""
echo "🚀 To distribute:"
echo "  1. Share the .tar.gz or .dmg file"
echo "  2. Recipients extract and run ./install.sh"
echo "  3. Follow setup instructions in INSTALL_README.md"
echo ""
echo "📋 Distribution includes:"
echo "  • Complete source code"
echo "  • Automated installer"
echo "  • Documentation"
echo "  • Setup scripts"
echo "  • Uninstaller"