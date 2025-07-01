#!/bin/bash

# Update all file paths in documentation and scripts to reflect new organization

SAVANT_DIR="$HOME/Documents/savant-ai"

echo "🔄 Updating file paths to new organized structure..."

# Update SYSTEM_AUDIO_SETUP.md with new paths
sed -i '' 's|~/savant-audio-captures/\*\.md|~/Documents/savant-ai/data/audio-captures/\*.md|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"
sed -i '' 's|~/savant-audio-daemon\.log|~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"
sed -i '' 's|~/savant-audio-daemon\.sh|~/Documents/savant-ai/scripts/audio/savant-audio-daemon.sh|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"

# Update script references in documentation
sed -i '' 's|\./audio-devices\.sh|./scripts/audio/audio-devices.sh|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"
sed -i '' 's|\./savant-audio-control\.sh|./scripts/audio/savant-audio-control.sh|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"
sed -i '' 's|\./auto-setup-system-audio\.sh|./scripts/setup/auto-setup-system-audio.sh|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"

# Update log commands in documentation
sed -i '' 's|tail -20 ~/savant-audio-daemon\.log|tail -20 ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"
sed -i '' 's|tail -f ~/savant-audio-daemon\.log|tail -f ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"
sed -i '' 's|cat ~/savant-audio-daemon\.log|cat ~/Documents/savant-ai/data/daemon-logs/savant-audio-daemon.log|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"

# Update capture directory references
sed -i '' 's|~/savant-audio-captures/|~/Documents/savant-ai/data/audio-captures/|g' "$SAVANT_DIR/docs/user-guides/SYSTEM_AUDIO_SETUP.md"

# Update README.md references if they exist
if [ -f "$SAVANT_DIR/README.md" ]; then
    sed -i '' 's|\./audio-devices\.sh|./scripts/audio/audio-devices.sh|g' "$SAVANT_DIR/README.md"
    sed -i '' 's|\./savant-audio-control\.sh|./scripts/audio/savant-audio-control.sh|g' "$SAVANT_DIR/README.md"
fi

echo "✅ File paths updated successfully!"
echo ""
echo "📁 New organized structure:"
echo "  Scripts: $SAVANT_DIR/scripts/"
echo "  Data: $SAVANT_DIR/data/"
echo "  Docs: $SAVANT_DIR/docs/"
echo ""
echo "🎯 Key commands with new paths:"
echo "  Audio devices: ./scripts/audio/audio-devices.sh"
echo "  Audio control: ./scripts/audio/savant-audio-control.sh"
echo "  View captures: ls -la data/audio-captures/"
echo "  View logs: tail -f data/daemon-logs/savant-audio-daemon.log"