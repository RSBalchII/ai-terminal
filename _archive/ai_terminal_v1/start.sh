#!/bin/bash

echo "╔══════════════════════════════════════╗"
echo "║      AI Terminal Launcher            ║"
echo "╚══════════════════════════════════════╝"
echo ""
echo "Which interface would you like to use?"
echo ""
echo "1) Terminal UI (TUI) - Text-based in terminal"
echo "2) Desktop GUI - Graphical window interface"
echo "3) Exit"
echo ""
read -p "Enter your choice (1-3): " choice

case $choice in
    1)
        echo ""
        echo "Starting Terminal UI..."
        echo "Controls:"
        echo "  • Type message and press Enter to send"
        echo "  • Ctrl+C to exit"
        echo "  • Tab to switch focus"
        echo ""
        sleep 2
        cargo run --bin ai-terminal
        ;;
    2)
        echo ""
        echo "Starting Desktop GUI..."
        echo "A new window will open shortly..."
        echo ""
        cargo run --bin ai-terminal-gui
        ;;
    3)
        echo "Goodbye!"
        exit 0
        ;;
    *)
        echo "Invalid choice. Please run ./start.sh again"
        exit 1
        ;;
esac
