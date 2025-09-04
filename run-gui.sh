#!/bin/bash

# AI Terminal GUI Launcher Script
# Sets up proper environment for keyboard input in WSL

echo "Starting AI Terminal GUI with WSL input support..."

# Essential X11/Winit environment variables for WSL
export WINIT_X11_SCALE_FACTOR=1.0
export WINIT_UNIX_BACKEND=x11

# Input method configuration
# These help with keyboard input in X11 applications
export XMODIFIERS=@im=local
export GTK_IM_MODULE=xim
export QT_IM_MODULE=xim

# Additional X11 settings that can help
export GDK_BACKEND=x11

# Debug information (optional)
if [ "$1" = "--debug" ]; then
    echo "Environment settings:"
    echo "  DISPLAY=$DISPLAY"
    echo "  WINIT_UNIX_BACKEND=$WINIT_UNIX_BACKEND"
    echo "  XMODIFIERS=$XMODIFIERS"
    echo "  GTK_IM_MODULE=$GTK_IM_MODULE"
    export RUST_BACKTRACE=1
fi

# Run the GUI application
exec cargo run --bin ai-terminal-gui "$@"
