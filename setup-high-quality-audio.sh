#!/bin/bash
# Alternative setup for high-quality audio capture

echo "🎵 High-Quality Audio Setup Alternative"
echo "======================================="

echo "This setup uses separate routing for optimal quality:"
echo ""

echo "Method 1: Conditional Audio Capture"
echo "-----------------------------------"
echo "Only capture system audio when needed:"
echo ""
echo "# Normal use (high quality audio output)"
echo "./sav stop"
echo "# Switch to MacBook speakers in System Preferences"
echo ""
echo "# When you want to capture system audio"
echo "# Switch to Multi-Output Device in System Preferences"
echo "./sav start"
echo ""

echo "Method 2: Use SWIFTin Tool"
echo "--------------------------" 
echo "Advanced tool for better audio routing:"
echo ""
echo "1. Install SWIFTin: https://www.swiftin.com/"
echo "2. Configure high-quality virtual routing"
echo "3. Maintains 48kHz throughout the chain"
echo ""

echo "Method 3: Hardware Solution"
echo "---------------------------"
echo "Use external audio interface:"
echo ""
echo "1. USB audio interface with loopback capability"
echo "2. Route system audio through hardware"
echo "3. Capture from hardware input"
echo ""

echo "Recommended: Try Method 1 first (conditional capture)"
echo "It gives you the best of both worlds with manual control."