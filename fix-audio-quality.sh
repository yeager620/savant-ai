#!/bin/bash
# Fix audio quality degradation caused by sample rate mismatch

echo "🎵 Fixing Audio Quality Issues"
echo "=============================="

echo "The issue: Sample rate mismatch between BlackHole (16kHz) and MacBook speakers (48kHz)"
echo ""

echo "🔧 Step 1: Open Audio MIDI Setup"
echo "Press Cmd+Space and type 'Audio MIDI Setup', then press Enter"
echo ""
read -p "Press Enter when Audio MIDI Setup is open..."

echo ""
echo "🔧 Step 2: Configure BlackHole sample rate"
echo "1. In the left sidebar, click on 'BlackHole 2ch'"
echo "2. In the right panel, find 'Format'"
echo "3. Change the sample rate from 16000 Hz to 48000 Hz"
echo "4. Keep everything else the same"
echo ""
read -p "Press Enter when BlackHole is set to 48000 Hz..."

echo ""
echo "🔧 Step 3: Configure Multi-Output Device"
echo "1. In the left sidebar, click on 'Multi-Output Device'"
echo "2. In the right panel, find 'Format'"
echo "3. Change the sample rate from 16000 Hz to 48000 Hz"
echo "4. Ensure both 'Built-in Output' and 'BlackHole 2ch' are checked"
echo "5. Set 'Built-in Output' as Master Device (right-click → Use This Device For Sound Output)"
echo ""
read -p "Press Enter when Multi-Output Device is configured..."

echo ""
echo "🔧 Step 4: Test the fix"
echo "Now try playing audio and starting the daemon again."
echo "The audio quality should remain high."
echo ""

echo "✅ Configuration complete!"
echo ""
echo "If you still experience issues, try:"
echo "  1. Restart your MacBook"
echo "  2. Delete and recreate the Multi-Output Device"
echo "  3. Use the alternative solution below"