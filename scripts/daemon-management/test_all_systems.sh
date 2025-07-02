#!/bin/bash
echo "🧪 Testing all Savant AI systems..."

# Test prerequisites
echo "1. Testing dependencies..."
if command -v ollama &> /dev/null; then
    echo "   ✓ Ollama installed"
else
    echo "   ✗ Ollama not found"
fi

if command -v tesseract &> /dev/null; then
    echo "   ✓ Tesseract installed"
else
    echo "   ✗ Tesseract not found"
fi

if command -v convert &> /dev/null; then
    echo "   ✓ ImageMagick installed"
else
    echo "   ✗ ImageMagick not found"
fi

# Test Ollama connection
echo ""
echo "2. Testing Ollama server..."
if curl -s http://localhost:11434/api/tags > /dev/null; then
    echo "   ✓ Ollama server responding"
else
    echo "   ✗ Ollama server not responding"
fi

# Get the project root directory (two levels up from this script)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Test daemon status
echo ""
echo "3. Testing daemon status..."
if "$PROJECT_ROOT/sav" status &>/dev/null; then
    echo "   ✓ Audio daemon running"
else
    echo "   ⚠ Audio daemon not running"
fi

if "$PROJECT_ROOT/sav-video" status &>/dev/null; then
    echo "   ✓ Video daemon running"
else
    echo "   ⚠ Video daemon not running"
fi

# Test individual components
echo ""
echo "4. Testing individual components..."

echo "   Testing OCR..."
if (cd "$PROJECT_ROOT" && cargo run --package savant-ocr -- test) &>/dev/null; then
    echo "   ✓ OCR test passed"
else
    echo "   ✗ OCR test failed"
fi

echo "   Testing Computer Vision..."
if (cd "$PROJECT_ROOT" && cargo run --package savant-vision -- test) &>/dev/null; then
    echo "   ✓ Vision test passed"
else
    echo "   ✗ Vision test failed"
fi

echo "   Testing Synchronization..."
if (cd "$PROJECT_ROOT" && cargo run --package savant-sync -- test --count 5) &>/dev/null; then
    echo "   ✓ Sync test passed"
else
    echo "   ✗ Sync test failed"
fi

# Test with sample image if available
echo ""
echo "5. Testing with sample data..."
if [ -f "$PROJECT_ROOT/screenshot_small.png" ] || [ -f "$PROJECT_ROOT/screenshot.png" ]; then
    image_file="$PROJECT_ROOT/screenshot_small.png"
    [ -f "$PROJECT_ROOT/screenshot.png" ] && image_file="$PROJECT_ROOT/screenshot.png"
    
    echo "   Testing OCR on $(basename "$image_file")..."
    if (cd "$PROJECT_ROOT" && cargo run --package savant-ocr -- extract --input "$image_file" --fast) &>/dev/null; then
        echo "   ✓ OCR processing successful"
    else
        echo "   ✗ OCR processing failed"
    fi
    
    echo "   Testing Vision analysis on $(basename "$image_file")..."
    if (cd "$PROJECT_ROOT" && cargo run --package savant-vision -- analyze --input "$image_file" --detect-apps) &>/dev/null; then
        echo "   ✓ Vision analysis successful"
    else
        echo "   ✗ Vision analysis failed"
    fi
else
    echo "   ⚠ No test images found (screenshot.png or screenshot_small.png)"
fi

# Database test
echo ""
echo "6. Testing database..."
if (cd "$PROJECT_ROOT" && cargo run --package savant-db -- stats) &>/dev/null; then
    echo "   ✓ Database accessible"
else
    echo "   ✗ Database test failed"
fi

echo ""
echo "🧪 Test complete! Check results above for any issues."