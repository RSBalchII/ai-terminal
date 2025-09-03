#!/bin/bash

echo "🧪 Testing AI Terminal Functionality"
echo "===================================="

# Test system tools commands
echo "Testing system tools commands..."

# Test ls command directly with system tools
echo "1. Testing system tools crate directly:"
cargo run -p system-tools --bin test-tools 2>/dev/null | head -20

echo ""
echo "2. Testing if terminal can handle basic input (non-interactive):"

# This is tricky because the terminal expects interactive input
# Let's check if we can at least start it and send a signal
timeout 3s bash -c 'echo "ls ." | ./target/debug/ai-terminal' 2>/dev/null || echo "Terminal requires interactive mode"

echo ""
echo "3. Checking for any obvious runtime issues:"

# Run with verbose logging to see where it might fail
RUST_LOG=debug timeout 5s ./target/debug/ai-terminal > test_output.log 2>&1 &
PID=$!

sleep 2

if ps -p $PID > /dev/null 2>&1; then
    echo "✅ Terminal running without crashes"
    kill $PID 2>/dev/null
    wait $PID 2>/dev/null
else
    echo "❌ Terminal crashed or exited early"
fi

echo ""
echo "📋 Recent log output:"
tail -10 test_output.log 2>/dev/null || echo "No log file generated"

# Cleanup
rm -f test_output.log

echo ""
echo "💡 Issues to check:"
echo "   - Does the terminal freeze when you type a message?"
echo "   - Do system commands (ls, cat, ping, ps) work?"
echo "   - Are there any error messages in the UI?"
echo "   - Does the AI respond to simple prompts?"
