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
- **Configurable AI model and system prompts**

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

## Configuration

The AI-Terminal can be configured using a `config.toml` file in the root directory. The configuration file has the following structure:

```toml
[ollama]
# The default model to use for Ollama requests
model = "llama3"

# Optional system prompt to guide the model's behavior
system_prompt = """
You are an expert terminal assistant. You help users with terminal commands, 
shell scripting, and system administration tasks. Provide concise, accurate 
responses and always consider the context of the user's operating system.
"""

# Custom prompts that can be referenced by name in the application
[custom_prompts]
terminal_expert = """
You are an expert terminal assistant. You help users with terminal commands, 
shell scripting, and system administration tasks. Provide concise, accurate 
responses and always consider the context of the user's operating system.
"""

shell_scripting = """
You are a shell scripting expert. Help users write, debug, and optimize 
shell scripts for various tasks. Explain your solutions clearly and 
provide examples when helpful.
"""
```

### Configuration Options

- `ollama.model`: The default model to use for Ollama requests. This can be any model that is available in your Ollama installation.

- `ollama.system_prompt`: An optional system prompt that will be sent to the model to guide its behavior. If not specified, the model's default system prompt will be used.

- `custom_prompts`: A section for defining custom prompts that can be referenced by name in the application. These prompts can be used to provide specific guidance to the AI for different types of tasks.

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