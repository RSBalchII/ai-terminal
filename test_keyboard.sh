#!/bin/bash

echo "=== Testing X11 Keyboard Input Support ==="
echo ""

# Check DISPLAY
echo "1. DISPLAY variable:"
echo "   DISPLAY=$DISPLAY"
echo ""

# Check X11 connection
echo "2. X11 connection test:"
if xset q &>/dev/null; then
    echo "   ✓ X11 connection is working"
else
    echo "   ✗ X11 connection failed"
fi
echo ""

# Check keyboard layout
echo "3. Current keyboard layout:"
setxkbmap -query 2>/dev/null || echo "   Could not query keyboard layout"
echo ""

# Check input method
echo "4. Input method variables:"
echo "   XIM=$XIM"
echo "   XIM_PROGRAM=$XIM_PROGRAM"
echo "   XMODIFIERS=$XMODIFIERS"
echo "   GTK_IM_MODULE=$GTK_IM_MODULE"
echo "   QT_IM_MODULE=$QT_IM_MODULE"
echo ""

# Test with specific environment variables for better input support
echo "5. Running GUI with enhanced input support..."
echo "   Setting WINIT_X11_SCALE_FACTOR=1"
echo "   Setting WINIT_UNIX_BACKEND=x11"
echo ""

export WINIT_X11_SCALE_FACTOR=1
export WINIT_UNIX_BACKEND=x11
export RUST_BACKTRACE=1

# Run the GUI
cargo run --bin ai-terminal-gui
