#!/bin/bash

echo "🎵 Setting up System Audio Capture"
echo "=================================="
echo ""

# Check if BlackHole is installed
if brew list blackhole-2ch >/dev/null 2>&1; then
    echo "✅ BlackHole is already installed"
else
    echo "📦 Installing BlackHole..."
    brew install blackhole-2ch
    if [ $? -eq 0 ]; then
        echo "✅ BlackHole installed successfully"
    else
        echo "❌ Failed to install BlackHole"
        exit 1
    fi
fi

echo ""
echo "🔧 MANUAL SETUP REQUIRED:"
echo "========================"
echo "1. Open 'Audio MIDI Setup' (Applications > Utilities)"
echo "2. Click '+' button → 'Create Multi-Output Device'"
echo "3. Check both:"
echo "   ☑️  MacBook Pro Speakers"
echo "   ☑️  BlackHole 2ch"
echo "4. Go to System Preferences > Sound > Output"
echo "5. Select the Multi-Output Device as default"
echo ""
echo "After setup, test with:"
echo "🎙️  ./capture-system-audio.sh 60"
echo ""