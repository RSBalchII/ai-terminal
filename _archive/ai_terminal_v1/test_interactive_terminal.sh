#!/bin/bash
set -e

echo "=== AI Terminal Interactive Test ==="
echo "Current directory: $(pwd)"
echo "Testing build: $(which cargo)"
echo

# Clean build first
echo "1. Building project..."
cargo build --quiet
echo "✅ Build successful"
echo

# Test help command (non-interactive)
echo "2. Testing help command..."
./target/debug/ai-terminal --help > /dev/null
echo "✅ Help command works"
echo

# Test version command (non-interactive)
echo "3. Testing version command..."
./target/debug/ai-terminal --version > /dev/null
echo "✅ Version command works"
echo

# Test error handling for non-interactive mode
echo "4. Testing non-interactive error handling..."
echo "" | ./target/debug/ai-terminal 2>&1 | grep -q "not a tty"
if [ $? -eq 0 ]; then
    echo "✅ Properly detects non-interactive environment"
else
    echo "❌ Failed to detect non-interactive environment"
fi
echo

# Test that system tools executor starts in the logs
echo "5. Testing system tools executor startup..."
echo "" | ./target/debug/ai-terminal 2>&1 | grep -q "System tools executor task started"
if [ $? -eq 0 ]; then
    echo "✅ System tools executor task starts properly"
else
    echo "❌ System tools executor task does not start"
fi
echo

echo "6. Testing interactive mode (if terminal is available)..."
if [ -t 0 ] && [ -t 1 ] && [ -n "$TERM" ]; then
    echo "Interactive terminal detected - testing startup..."
    
    # Create an expect script to test interactive mode with timeout
    cat > /tmp/test_ai_terminal.exp << 'EOF'
#!/usr/bin/expect -f
set timeout 10

spawn ./target/debug/ai-terminal

expect {
    "Welcome to AI Terminal" {
        send_user "✅ Terminal starts successfully\n"
        send "\x1b"
        expect eof
        exit 0
    }
    timeout {
        send_user "❌ Terminal startup timed out\n"
        exit 1
    }
}
EOF

    chmod +x /tmp/test_ai_terminal.exp
    
    if command -v expect > /dev/null 2>&1; then
        cd /home/rsbiiw/projects/ai-terminal
        /tmp/test_ai_terminal.exp
    else
        echo "⚠️  'expect' not available, skipping interactive test"
        echo "   To install: sudo apt-get install expect"
    fi
    
    rm -f /tmp/test_ai_terminal.exp
else
    echo "⚠️  Not in an interactive terminal, skipping interactive test"
    echo "   This test should be run in a proper terminal session"
fi
echo

echo "=== Test Summary ==="
echo "✅ Build successful"
echo "✅ Help command works"
echo "✅ Version command works"
echo "✅ Non-interactive error handling works"
echo "✅ System tools executor starts properly"
echo "🎉 All basic functionality tests passed!"
echo
echo "To test interactively:"
echo "  1. Open a terminal"
echo "  2. cd $(pwd)"
echo "  3. ./target/debug/ai-terminal"
echo "  4. Type 'hello' and press Enter"
echo "  5. Press Esc to exit"
