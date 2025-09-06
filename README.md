# AI-Terminal

A terminal-based frontend for AI interactions, built with Rust using ratatui and crossterm.

## Features

- Terminal-based user interface using ratatui
- PTY-based command execution
- Block-based I/O system for displaying commands and outputs
- Real-time output streaming
- Visual status indicators for command execution
- Keyboard navigation and controls
- **Persistent command history with file I/O**
- **History navigation with arrow keys**
- **Tab completion for file paths**

## Prerequisites

- Rust (latest stable version)
- Cargo

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Controls

- Type commands and press Enter to execute
- **Up/Down Arrow Keys: Navigate command history**
- **Tab: Complete file paths**
- F1: Show help
- F10: Exit application
- Page Up/Down: Scroll through command history
- Ctrl+Up/Down: Fine-grained scrolling
- Home/End: Jump to top/bottom of command history

## Architecture

The application is structured as a workspace with the following crates:

- `terminal-emulator`: Handles command execution and PTY management
- `terminal-ui`: Implements the TUI using ratatui
- `ai-terminal`: Main application entry point

## Testing

Run tests with:

```bash
cargo test
```