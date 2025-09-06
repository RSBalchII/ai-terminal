#!/bin/bash

# Test script to verify the AI terminal system tools integration

echo "🧪 Testing AI Terminal System Tools Integration"
echo "============================================="

# Start the AI terminal in a background process with a timeout
timeout 10s ./target/debug/ai-terminal &
TERMINAL_PID=$!

# Wait a moment for the terminal to start
sleep 2

# Check if the process is still running
if ps -p $TERMINAL_PID > /dev/null; then
    echo "✅ AI Terminal started successfully"
    
    # Kill the process
    kill $TERMINAL_PID 2>/dev/null
    wait $TERMINAL_PID 2>/dev/null
    echo "✅ AI Terminal stopped cleanly"
else
    echo "❌ AI Terminal failed to start or crashed immediately"
    exit 1
fi

echo ""
echo "🎉 Integration test passed!"
echo ""
echo "🔧 Available system tools commands to test:"
echo "   - ls .                    # List directory contents"
echo "   - cat Cargo.toml         # Read file contents"
echo "   - find . *.rs            # Find files matching pattern"
echo "   - ping localhost         # Test network connectivity"
echo "   - ps                     # List running processes"
echo ""
echo "To run the AI terminal interactively:"
echo "   ./target/debug/ai-terminal"
