#!/bin/bash

echo "🧪 Testing AI Terminal Response (with timeout fix)"
echo "=================================================="

# Start the terminal in background
RUST_LOG=info timeout 15s ./target/debug/ai-terminal > terminal_output.log 2>&1 &
TERMINAL_PID=$!

# Wait for startup
sleep 3

# Check if it's running
if ps -p $TERMINAL_PID > /dev/null 2>&1; then
    echo "✅ Terminal started successfully"
    
    # Try to send some input (this is tricky without interactive mode)
    # Let's just let it run for a bit to see if it freezes
    echo "⏱️  Waiting 5 seconds to see if terminal remains responsive..."
    sleep 5
    
    if ps -p $TERMINAL_PID > /dev/null 2>&1; then
        echo "✅ Terminal still running after 5 seconds"
        
        # Kill it cleanly
        kill -TERM $TERMINAL_PID 2>/dev/null
        sleep 2
        
        # Force kill if still running
        if ps -p $TERMINAL_PID > /dev/null 2>&1; then
            kill -9 $TERMINAL_PID 2>/dev/null
        fi
        
        echo "✅ Terminal stopped"
        echo ""
        echo "📋 Last few lines of output:"
        tail -n 10 terminal_output.log
        
    else
        echo "❌ Terminal crashed or exited unexpectedly"
        echo "📋 Output:"
        cat terminal_output.log
    fi
else
    echo "❌ Terminal failed to start"
    echo "📋 Output:"
    cat terminal_output.log
fi

# Cleanup
rm -f terminal_output.log
