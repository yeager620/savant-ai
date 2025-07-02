#!/bin/bash
# Comprehensive video/screen capture data analyzer

echo "📹 Screen Capture Data Analyzer"
echo "==============================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURE_DIR="$HOME/Library/Application Support/savant-ai/video-captures"

if [ ! -d "$CAPTURE_DIR" ]; then
    echo "❌ Video captures directory not found: $CAPTURE_DIR"
    echo "Make sure the video daemon has been running."
    exit 1
fi

# Summary statistics
echo -e "${BLUE}📊 Capture Summary${NC}"
echo "=================="

total_images=$(find "$CAPTURE_DIR" -name "*.png" -type f | wc -l | tr -d ' ')
total_size=$(du -sh "$CAPTURE_DIR" | cut -f1)
latest_capture=$(find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -1 | cut -d' ' -f2-)
oldest_capture=$(find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -n | head -1 | cut -d' ' -f2-)

echo "📸 Total screenshots: $total_images"
echo "💾 Total storage used: $total_size"
echo "🕐 Latest capture: $(basename "$latest_capture" 2>/dev/null || echo "None")"
echo "🕐 Oldest capture: $(basename "$oldest_capture" 2>/dev/null || echo "None")"

# Capture frequency analysis
echo ""
echo -e "${BLUE}📈 Capture Timeline${NC}"
echo "=================="

echo "Screenshots by hour today:"
find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%Sm" -t "%H" {} \; | sort | uniq -c | while read count hour; do
    printf "%02d:00 - %02d:59: %3d screenshots\n" "$hour" "$hour" "$count"
done

echo ""
echo "What would you like to analyze?"
echo "1. 🔍 Extract text from recent screenshots (OCR)"
echo "2. 👁️  Analyze visual content (computer vision)"
echo "3. 📱 View screenshots in Finder"
echo "4. 🖼️  Generate thumbnail gallery"
echo "5. 📊 Activity analysis (detect apps/websites)"
echo "6. 🔎 Search screenshots by content"
echo "7. 📝 Export analysis report"
echo "8. 🗑️  Cleanup old captures"
echo ""

read -p "Choose an option (1-8): " choice

case $choice in
    1)
        echo -e "\n${BLUE}🔍 OCR Text Extraction${NC}"
        echo "====================="
        echo "Extracting text from 5 most recent screenshots..."
        
        find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -5 | while read timestamp filepath; do
            filename=$(basename "$filepath")
            echo ""
            echo -e "${YELLOW}📸 $filename${NC}"
            echo "$(date -r $timestamp '+%Y-%m-%d %H:%M:%S')"
            echo "----------------------------------------"
            
            # Run OCR on the image
            if cargo run --package savant-ocr -- extract --input "$filepath" --format text --fast 2>/dev/null; then
                echo "✅ OCR completed"
            else
                echo "❌ OCR failed"
            fi
        done
        ;;
        
    2)
        echo -e "\n${BLUE}👁️  Computer Vision Analysis${NC}"
        echo "============================="
        echo "Analyzing visual content of 3 most recent screenshots..."
        
        find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -3 | while read timestamp filepath; do
            filename=$(basename "$filepath")
            echo ""
            echo -e "${YELLOW}📸 $filename${NC}"
            echo "$(date -r $timestamp '+%Y-%m-%d %H:%M:%S')"
            echo "----------------------------------------"
            
            # Run computer vision analysis
            if cargo run --package savant-vision -- analyze --input "$filepath" --detect-apps --classify-activity --format summary 2>/dev/null; then
                echo "✅ Vision analysis completed"
            else
                echo "❌ Vision analysis failed"
            fi
        done
        ;;
        
    3)
        echo -e "\n${BLUE}📱 Opening in Finder${NC}"
        echo "==================="
        open "$CAPTURE_DIR"
        echo "✅ Opened video captures directory in Finder"
        ;;
        
    4)
        echo -e "\n${BLUE}🖼️  Generating Thumbnail Gallery${NC}"
        echo "==============================="
        
        gallery_dir="$PROJECT_ROOT/video-gallery"
        mkdir -p "$gallery_dir"
        
        echo "Creating thumbnails of recent captures..."
        
        find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -20 | while read timestamp filepath; do
            filename=$(basename "$filepath" .png)
            thumbnail_path="$gallery_dir/${filename}_thumb.png"
            
            # Create thumbnail using sips
            sips -Z 400 "$filepath" --out "$thumbnail_path" >/dev/null 2>&1
            echo "📷 Created thumbnail: $(basename "$thumbnail_path")"
        done
        
        # Create HTML gallery
        cat > "$gallery_dir/gallery.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Savant AI Screen Captures</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .gallery { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; }
        .capture { border: 1px solid #ddd; padding: 10px; border-radius: 8px; }
        .capture img { max-width: 100%; height: auto; border-radius: 4px; }
        .capture-info { margin-top: 10px; font-size: 14px; color: #666; }
    </style>
</head>
<body>
    <h1>📹 Savant AI Screen Captures</h1>
    <div class="gallery">
EOF

        find "$gallery_dir" -name "*_thumb.png" | sort -r | while read thumb; do
            filename=$(basename "$thumb" _thumb.png)
            echo "        <div class=\"capture\">" >> "$gallery_dir/gallery.html"
            echo "            <img src=\"$(basename "$thumb")\" alt=\"Screenshot\">" >> "$gallery_dir/gallery.html"
            echo "            <div class=\"capture-info\">$filename</div>" >> "$gallery_dir/gallery.html"
            echo "        </div>" >> "$gallery_dir/gallery.html"
        done

        cat >> "$gallery_dir/gallery.html" << 'EOF'
    </div>
</body>
</html>
EOF

        echo "✅ Gallery created: $gallery_dir/gallery.html"
        echo "🌐 Opening in browser..."
        open "$gallery_dir/gallery.html"
        ;;
        
    5)
        echo -e "\n${BLUE}📊 Activity Analysis${NC}"
        echo "==================="
        echo "Analyzing app usage and activity patterns..."
        
        # Analyze recent screenshots for apps and activities
        temp_analysis="/tmp/savant_activity_analysis.json"
        echo "[]" > "$temp_analysis"
        
        find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -10 | while read timestamp filepath; do
            echo "Analyzing $(basename "$filepath")..."
            
            # Run vision analysis and collect results
            cargo run --package savant-vision -- analyze --input "$filepath" --detect-apps --classify-activity --format json 2>/dev/null || true
        done
        
        echo "✅ Activity analysis completed"
        echo "💡 Use the MCP server to query activity data with natural language"
        ;;
        
    6)
        echo -e "\n${BLUE}🔎 Search Screenshots${NC}"
        echo "===================="
        read -p "Enter text to search for: " search_text
        
        echo "Searching for '$search_text' in screenshots..."
        echo ""
        
        found=0
        find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -20 | while read timestamp filepath; do
            # Extract text and search
            extracted_text=$(cargo run --package savant-ocr -- extract --input "$filepath" --format text --fast 2>/dev/null || echo "")
            
            if echo "$extracted_text" | grep -i "$search_text" >/dev/null; then
                echo -e "${GREEN}✅ Found in: $(basename "$filepath")${NC}"
                echo "📅 $(date -r $timestamp '+%Y-%m-%d %H:%M:%S')"
                echo "📝 Text context:"
                echo "$extracted_text" | grep -i "$search_text" | head -3
                echo ""
                found=$((found + 1))
            fi
        done
        
        if [ $found -eq 0 ]; then
            echo "❌ No screenshots found containing '$search_text'"
        fi
        ;;
        
    7)
        echo -e "\n${BLUE}📝 Export Analysis Report${NC}"
        echo "========================="
        
        report_file="$PROJECT_ROOT/screen-capture-report-$(date +%Y%m%d-%H%M%S).md"
        
        cat > "$report_file" << EOF
# Screen Capture Analysis Report
Generated: $(date)

## Summary
- Total screenshots: $total_images
- Storage used: $total_size
- Date range: $(date -r $(stat -f "%m" "$oldest_capture" 2>/dev/null || echo 0) '+%Y-%m-%d') to $(date -r $(stat -f "%m" "$latest_capture" 2>/dev/null || echo 0) '+%Y-%m-%d')

## Recent Activity (Last 10 Screenshots)
EOF

        find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | head -10 | while read timestamp filepath; do
            filename=$(basename "$filepath")
            datetime=$(date -r $timestamp '+%Y-%m-%d %H:%M:%S')
            size=$(ls -lh "$filepath" | awk '{print $5}')
            
            echo "- **$datetime** - $filename ($size)" >> "$report_file"
        done
        
        echo "" >> "$report_file"
        echo "## Analysis Commands" >> "$report_file"
        echo "- OCR text extraction: \`cargo run --package savant-ocr -- extract --input <image> --fast\`" >> "$report_file"
        echo "- Computer vision: \`cargo run --package savant-vision -- analyze --input <image> --detect-apps\`" >> "$report_file"
        echo "- Database query: \`cargo run --package savant-db -- query --text \"keyword\"\`" >> "$report_file"
        
        echo "✅ Report exported: $report_file"
        echo "📖 Opening report..."
        open "$report_file"
        ;;
        
    8)
        echo -e "\n${BLUE}🗑️  Cleanup Old Captures${NC}"
        echo "======================"
        echo "Current storage: $total_size"
        echo ""
        echo "Cleanup options:"
        echo "1. Delete captures older than 7 days"
        echo "2. Delete captures older than 30 days"
        echo "3. Keep only last 100 captures"
        echo "4. Custom cleanup"
        echo ""
        
        read -p "Choose cleanup option (1-4): " cleanup_choice
        
        case $cleanup_choice in
            1)
                echo "Deleting captures older than 7 days..."
                find "$CAPTURE_DIR" -name "*.png" -type f -mtime +7 -delete
                echo "✅ Cleanup completed"
                ;;
            2)
                echo "Deleting captures older than 30 days..."
                find "$CAPTURE_DIR" -name "*.png" -type f -mtime +30 -delete
                echo "✅ Cleanup completed"
                ;;
            3)
                echo "Keeping only last 100 captures..."
                find "$CAPTURE_DIR" -name "*.png" -type f -exec stat -f "%m %N" {} \; | sort -nr | tail -n +101 | cut -d' ' -f2- | xargs rm -f 2>/dev/null || true
                echo "✅ Cleanup completed"
                ;;
            4)
                read -p "Delete captures older than how many days? " days
                if [[ "$days" =~ ^[0-9]+$ ]] && [ "$days" -gt 0 ]; then
                    echo "Deleting captures older than $days days..."
                    find "$CAPTURE_DIR" -name "*.png" -type f -mtime +$days -delete
                    echo "✅ Cleanup completed"
                else
                    echo "❌ Invalid number of days"
                fi
                ;;
        esac
        
        # Show new storage usage
        new_size=$(du -sh "$CAPTURE_DIR" | cut -f1)
        echo "Storage after cleanup: $new_size"
        ;;
        
    *)
        echo "❌ Invalid option"
        ;;
esac

echo ""
echo -e "${BLUE}💡 More Analysis Options:${NC}"
echo "========================"
echo "🔍 Search database: cargo run --package savant-db -- query --text \"keyword\""
echo "🧠 MCP queries: Natural language questions about your screen activity"
echo "📊 Performance: cargo run --package savant-vision -- benchmark"
echo "🎯 Specific image: cargo run --package savant-ocr -- extract --input \"path/to/image.png\""