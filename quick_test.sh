#!/bin/bash

echo "============================================================"
echo "🚀 AI Terminal Quick Verification"
echo "============================================================"

# Quick build check
echo -e "\n1. Build Check:"
if cargo build --bin ai-terminal --bin ai-terminal-gui 2>/dev/null; then
    echo "   ✅ Binaries compile"
else
    echo "   ❌ Build failed"
fi

# Check Ollama
echo -e "\n2. Ollama Check:"
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo "   ✅ Ollama is running"
    MODELS=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name' 2>/dev/null | head -1)
    echo "   Model: $MODELS"
else
    echo "   ⚠️  Ollama not running"
fi

# Quick component test (no hanging tests)
echo -e "\n3. Component Tests:"
cargo test -p system-tools --lib --quiet 2>/dev/null && echo "   ✅ System tools" || echo "   ❌ System tools"
cargo test -p python-bridge --lib --quiet 2>/dev/null && echo "   ✅ Python bridge" || echo "   ❌ Python bridge"
cargo test -p terminal-emulator --lib --quiet 2>/dev/null && echo "   ✅ Terminal emulator" || echo "   ❌ Terminal emulator"

# Check files exist
echo -e "\n4. File Structure:"
[ -d "spec-kit/specs" ] && echo "   ✅ Spec-kit exists" || echo "   ❌ Spec-kit missing"
[ -f "PROGRESS_REPORT.md" ] && echo "   ✅ Documentation exists" || echo "   ❌ Documentation missing"

echo -e "\n============================================================"
echo "✅ Quick verification complete!"
echo ""
echo "To run the application:"
echo "  • TUI: cargo run --bin ai-terminal"
echo "  • GUI: cargo run --bin ai-terminal-gui"
echo "============================================================"
