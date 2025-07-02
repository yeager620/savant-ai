#!/bin/bash

# macOS Permission Helper Script
# Attempts various methods to streamline permission granting

echo "🔐 macOS Permission Helper"
echo "=========================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Detect current terminal
CURRENT_TERMINAL="Terminal"
TERMINAL_PATH="/System/Applications/Utilities/Terminal.app"

if [ "$TERM_PROGRAM" = "iTerm.app" ]; then
    CURRENT_TERMINAL="iTerm"
    TERMINAL_PATH="/Applications/iTerm.app"
elif [ "$TERM_PROGRAM" = "WarpTerminal" ]; then
    CURRENT_TERMINAL="Warp"
    TERMINAL_PATH="/Applications/Warp.app"
elif [ -n "$VSCODE_INJECTION" ]; then
    CURRENT_TERMINAL="VS Code"
    TERMINAL_PATH="/Applications/Visual Studio Code.app"
fi

echo "Detected terminal: $CURRENT_TERMINAL"
echo "Path: $TERMINAL_PATH"
echo ""

# Method 1: AppleScript approach (usually fails due to security)
print_info "Method 1: Attempting AppleScript automation..."

# Try to open System Preferences with AppleScript
osascript << EOF 2>/dev/null
tell application "System Preferences"
    activate
    set the current pane to pane id "com.apple.preference.security"
    delay 1
end tell
EOF

if [ $? -eq 0 ]; then
    print_success "System Preferences opened via AppleScript"
else
    print_warning "AppleScript method failed (expected due to security restrictions)"
fi

# Method 2: Direct URL schemes
print_info "Method 2: Using URL schemes to open specific Privacy panels..."

echo "Opening specific privacy settings:"
echo "1. Screen Recording..."
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
sleep 1

echo "2. Microphone..."
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
sleep 1

echo "3. Accessibility (optional)..."
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"

print_success "Privacy panels opened"

# Method 3: Terminal commands that might trigger permission dialogs
print_info "Method 3: Triggering permission dialogs..."

echo "Attempting to trigger Screen Recording permission dialog..."
# This command will fail but should trigger the permission dialog
screencapture -x /tmp/test_permission.png 2>/dev/null && rm -f /tmp/test_permission.png

echo "Attempting to trigger Microphone permission dialog..."
# This will attempt to access microphone
sox -d /tmp/test_audio.wav trim 0 0.1 2>/dev/null && rm -f /tmp/test_audio.wav

# Method 4: tccutil reset (requires admin)
print_info "Method 4: Checking if we can reset permissions (requires admin)..."

echo "This would clear existing permissions so you can re-grant them:"
echo "sudo tccutil reset ScreenCapture $TERMINAL_PATH"
echo "sudo tccutil reset Microphone $TERMINAL_PATH"
echo ""
read -p "Reset existing permissions? This requires admin password (y/N): " reset_perms

if [[ $reset_perms =~ ^[Yy]$ ]]; then
    echo "Resetting Screen Recording permission..."
    sudo tccutil reset ScreenCapture "$TERMINAL_PATH" 2>/dev/null
    
    echo "Resetting Microphone permission..."
    sudo tccutil reset Microphone "$TERMINAL_PATH" 2>/dev/null
    
    print_success "Permissions reset - you'll be prompted to re-grant them"
fi

# Method 5: Create a simple automation shortcut
print_info "Method 5: Creating macOS Shortcut for future permission management..."

# Check if Shortcuts app is available
if [ -d "/System/Applications/Shortcuts.app" ]; then
    cat > /tmp/savant_permissions.shortcut << 'EOF'
{
    "WFWorkflowMinimumClientVersion": 900,
    "WFWorkflowIcon": {
        "WFWorkflowIconStartColor": 431817727,
        "WFWorkflowIconGlyphNumber": 61440
    },
    "WFWorkflowName": "Savant AI Permissions",
    "WFWorkflowActions": [
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.openapp",
            "WFWorkflowActionParameters": {
                "WFAppIdentifier": "com.apple.systempreferences"
            }
        }
    ]
}
EOF
    
    print_info "Shortcut template created (manual import needed)"
    echo "You can import this into Shortcuts app for easy access"
fi

# Method 6: Database manipulation (advanced, risky)
print_warning "Method 6: Direct TCC database manipulation (NOT RECOMMENDED)"
echo ""
echo "⚠️  WARNING: Direct database manipulation can break macOS security!"
echo "This method involves:"
echo "1. Disabling System Integrity Protection (csrutil disable)"
echo "2. Modifying TCC.db directly with SQL"
echo "3. Re-enabling SIP"
echo ""
echo "This is NOT recommended and can cause system instability."
echo ""

# Provide the safest manual instructions
print_info "RECOMMENDED: Manual Permission Setup"
echo ""
echo "The safest and most reliable method:"
echo ""
echo "1. Open System Preferences → Security & Privacy → Privacy"
echo "2. Click 'Screen Recording' in left sidebar"
echo "3. Click the lock icon (🔒) and enter your password"
echo "4. Check the box next to '$CURRENT_TERMINAL'"
echo "5. Click 'Microphone' in left sidebar"
echo "6. Check the box next to '$CURRENT_TERMINAL'"
echo "7. Restart your terminal application"
echo ""

# Provide verification
echo "After granting permissions, verify with:"
echo "  ./verify-permissions"
echo ""

# Check if permissions are already granted
print_info "Current permission status:"

# Function to check TCC permission
check_permission() {
    local service=$1
    local app_path=$2
    
    local result=$(sqlite3 ~/Library/Application\ Support/com.apple.TCC/TCC.db \
        "SELECT allowed FROM access WHERE service='$service' AND client='$app_path';" 2>/dev/null)
    
    if [ "$result" = "1" ]; then
        return 0
    else
        return 1
    fi
}

if check_permission "kTCCServiceScreenCapture" "$TERMINAL_PATH"; then
    print_success "Screen Recording: Already granted"
else
    echo "❌ Screen Recording: Needs to be granted"
fi

if check_permission "kTCCServiceMicrophone" "$TERMINAL_PATH"; then
    print_success "Microphone: Already granted"
else
    echo "❌ Microphone: Needs to be granted"
fi

echo ""
print_info "Alternative: Use a different terminal"
echo ""
echo "If you're having issues with $CURRENT_TERMINAL, try:"
echo "• iTerm2: brew install --cask iterm2"
echo "• Warp: brew install --cask warp"
echo "• Alacritty: brew install --cask alacritty"
echo ""
echo "Different terminals may have different permission states."