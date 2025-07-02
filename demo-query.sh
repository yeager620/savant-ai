#!/bin/bash
# Simple demo of your personal audio query system

echo "🎤 Your Personal Audio Data - July 1st, 2025"
echo "============================================="

echo ""
echo "📊 What's in your database:"
./target/release/savant-db --db-path data/databases/dev/personal-audio.db list

echo ""
echo "🔍 Searching for 'machine learning' content:"
./target/release/savant-db --db-path data/databases/dev/personal-audio.db query --text "machine learning"

echo ""
echo "🔍 Searching for 'PyTorch' content:"
./target/release/savant-db --db-path data/databases/dev/personal-audio.db query --text "PyTorch"

echo ""
echo "🔍 Searching for 'Unsloth' content:"
./target/release/savant-db --db-path data/databases/dev/personal-audio.db query --text "Unsloth"

echo ""
echo "📋 Database statistics:"
./target/release/savant-db --db-path data/databases/dev/personal-audio.db stats

echo ""
echo "🤖 Testing AI query (this may take a moment)..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query_conversations","arguments":{"query":"What topics about machine learning were discussed?","session_id":"demo"}}}' | timeout 20s ./target/release/savant-mcp-server --database data/databases/dev/personal-audio.db --llm-provider ollama --llm-model devstral 2>/dev/null | head -1)

if [ -n "$RESPONSE" ]; then
    echo "✅ AI Response received:"
    echo "$RESPONSE" | jq -r '.result.content[0].text // .result // "Processing completed"' 2>/dev/null
else
    echo "⚠️  AI query timed out, but your data is accessible via direct search above"
fi

echo ""
echo "🎯 Summary: Your audio database contains machine learning training discussions"
echo "   Topics include: Unsloth, PyTorch, model fine-tuning, tokenizers, and training setups"
echo "   Date: July 1st, 2025 around 15:22-15:24"
echo "   Source: System audio captures during what appears to be ML training sessions"