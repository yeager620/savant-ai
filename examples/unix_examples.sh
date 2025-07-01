#!/bin/bash
# Examples demonstrating UNIX philosophy with Savant AI CLI tools

echo "=== Savant AI UNIX Philosophy Examples ==="
echo

echo "1. Simple LLM Query:"
echo "$ echo 'What is 2+2?' | savant-llm"
echo "What is 2+2?" | cargo run --package savant-llm
echo

echo "2. Extract just the answer using jq:"
echo "$ echo 'What is Rust?' | savant-llm | jq -r '.content'"
echo "What is Rust?" | cargo run --package savant-llm | jq -r '.content'
echo

echo "3. Test different models:"
echo "$ echo 'Hello' | savant-llm --model codellama"
# Note: This would work if codellama is available
echo

echo "4. Check processing time:"
echo "$ echo 'Complex question' | savant-llm | jq '.processing_time_ms'"
echo "What are the key principles of software architecture?" | cargo run --package savant-llm | jq '.processing_time_ms'
echo

echo "5. Batch processing (when implemented):"
echo "$ printf 'Question 1\nQuestion 2\nQuestion 3' | savant-llm batch"
echo

echo "6. Composability with other tools:"
echo "$ echo 'Explain async programming' | savant-llm | jq '.content' | wc -w"
echo "Explain async programming" | cargo run --package savant-llm | jq -r '.content' | wc -w
echo

echo "7. List available models:"
echo "$ savant-llm models"
cargo run --package savant-llm -- models
echo

echo "8. Test provider connection:"
echo "$ savant-llm test"
cargo run --package savant-llm -- test
echo

echo "=== Future Composability (when all tools are ready) ==="
echo "# Monitor browser content and auto-answer questions:"
echo "$ savant-browser --monitor | savant-browser --detect-questions | savant-llm --batch"
echo
echo "# Save conversation with history:"
echo "$ echo 'Hello' | savant-llm | savant-chat --save"
echo
echo "# Export and analyze chat patterns:"
echo "$ savant-chat --export | jq '.messages[] | select(.is_user==false) | .content' | wc -w"
echo
echo "# Auto-configure from environment:"
echo "$ savant-config --export | jq '.llm.default_model' | xargs -I {} savant-llm --model {}"