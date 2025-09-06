#!/bin/bash

echo "🔍 AI Terminal Issue Debugging Script"
echo "====================================="

# Function to run a test and capture output
run_test() {
    local test_name="$1"
    local command="$2"
    local timeout_duration="$3"
    
    echo ""
    echo "Testing: $test_name"
    echo "Command: $command"
    echo "----------------------------------------"
    
    if timeout "$timeout_duration" bash -c "$command" 2>&1; then
        echo "✅ Test completed successfully"
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo "⏱️  Test timed out after $timeout_duration seconds"
        else
            echo "❌ Test failed with exit code: $exit_code"
        fi
    fi
}

# Check basic functionality
echo "1. BASIC FUNCTIONALITY TESTS"
echo "=============================="

run_test "Ollama Connection" "curl -s http://localhost:11434/api/tags | head -3" "5s"
run_test "System Tools Direct Test" "cargo run -p system-tools --bin test-tools | head -10" "10s"
run_test "AI Terminal Help" "./target/debug/ai-terminal --help" "5s"

# Check for runtime issues
echo ""
echo "2. RUNTIME ANALYSIS"
echo "==================="

echo "Starting terminal with debug logging for 10 seconds..."
RUST_LOG=debug timeout 10s ./target/debug/ai-terminal > debug_terminal.log 2>&1 &
PID=$!

sleep 3

if ps -p $PID > /dev/null 2>&1; then
    echo "✅ Terminal is running (PID: $PID)"
    
    # Let it run for a bit more
    sleep 4
    
    if ps -p $PID > /dev/null 2>&1; then
        echo "✅ Terminal still running after 7 seconds"
        kill -TERM $PID 2>/dev/null
        wait $PID 2>/dev/null
        echo "✅ Terminal stopped cleanly"
    else
        echo "❌ Terminal crashed after a few seconds"
    fi
else
    echo "❌ Terminal failed to start or crashed immediately"
fi

echo ""
echo "3. LOG ANALYSIS"
echo "==============="

if [ -f debug_terminal.log ]; then
    echo "Debug log file size: $(wc -l debug_terminal.log | cut -d' ' -f1) lines"
    echo ""
    echo "Last 15 lines of debug log:"
    echo "----------------------------"
    tail -15 debug_terminal.log
    
    echo ""
    echo "Checking for specific issues:"
    echo "----------------------------"
    
    if grep -q "ERROR" debug_terminal.log; then
        echo "❌ Found ERROR messages:"
        grep "ERROR" debug_terminal.log
    else
        echo "✅ No ERROR messages found"
    fi
    
    if grep -q "panic" debug_terminal.log; then
        echo "❌ Found panic messages:"
        grep "panic" debug_terminal.log
    else
        echo "✅ No panic messages found"
    fi
    
    if grep -q "timeout" debug_terminal.log; then
        echo "⏱️  Found timeout messages:"
        grep "timeout" debug_terminal.log
    else
        echo "✅ No timeout messages found"
    fi
    
    if grep -q "System tools executor task started" debug_terminal.log; then
        echo "✅ System tools executor started successfully"
    else
        echo "❌ System tools executor may not have started"
    fi
    
else
    echo "❌ No debug log file generated"
fi

echo ""
echo "4. SYSTEM COMPATIBILITY"
echo "======================="

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "OS: $(uname -a)"
echo "Shell: $SHELL"

# Check dependencies
echo ""
echo "Checking critical dependencies:"
echo "- Tokio runtime: $(grep 'tokio.*=' Cargo.toml | head -1)"
echo "- Ollama connection: $(curl -s -o /dev/null -w '%{http_code}' http://localhost:11434/api/tags)"

# Memory usage check
echo "- Available memory: $(free -h | grep Mem | awk '{print $7}')"

echo ""
echo "5. RECOMMENDATIONS"
echo "=================="

if [ -f debug_terminal.log ]; then
    if grep -q "Connected to Ollama" debug_terminal.log && grep -q "System tools executor task started" debug_terminal.log; then
        echo "✅ Core components are initializing correctly"
        echo ""
        echo "💡 Potential issues to investigate:"
        echo "   1. Terminal UI responsiveness - try typing a simple message like 'hello'"
        echo "   2. System command execution - try 'ls .' or 'ps'"
        echo "   3. AI response generation - check if it times out after 15 seconds"
        echo "   4. Error handling - look for clear error messages in the UI"
        echo ""
        echo "🚀 To test manually:"
        echo "   ./target/debug/ai-terminal"
        echo "   Then try: hello (wait for AI response)"
        echo "   Then try: ls . (should execute immediately)"
        echo "   Then try: ping localhost (should show ping results)"
    else
        echo "❌ Core components failed to initialize"
        echo "   Check the debug log above for specific error messages"
    fi
else
    echo "❌ Could not analyze terminal behavior - no debug log generated"
fi

# Cleanup
rm -f debug_terminal.log

echo ""
echo "🏁 Debugging complete. Please share any specific error messages or behaviors you're seeing."
