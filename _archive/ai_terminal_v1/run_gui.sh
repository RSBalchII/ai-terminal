#!/bin/bash

# Get the Windows host IP (for WSL2)
export DISPLAY=$(ip route list default | awk '{print $3}'):0

# Alternative: If you're using WSL1, use:
# export DISPLAY=:0

# Force X11 backend instead of Wayland
export WINIT_UNIX_BACKEND=x11
export GDK_BACKEND=x11

# OpenGL settings for better compatibility
export LIBGL_ALWAYS_INDIRECT=1
export MESA_GL_VERSION_OVERRIDE=3.3

echo "Display configured as: $DISPLAY"
echo "Starting AI Terminal GUI..."

cargo run --bin ai-terminal-gui "$@"
