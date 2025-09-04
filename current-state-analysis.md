# Current State Analysis

This document provides a detailed analysis of the existing codebase, outlining the purpose and key functionalities of each module and component within the `ai-terminal` project.

## Workspace Structure

The project is organized as a Rust workspace, comprising several crates, each responsible for a specific aspect of the terminal application. The `Cargo.toml` at the root defines the workspace members and shared dependencies.

## Core Crates

### `ai-terminal` (Root Crate)

*   **Purpose**: The main application crate, orchestrating the various components. It defines the main entry points for both the terminal UI and the EGUI-based GUI.
*   **Key Dependencies**: Depends on all other workspace crates (`terminal-ui`, `egui-ui`, `ollama-client`, `python-bridge`, `system-tools`) and common utilities like `tokio`, `anyhow`, `tracing`, `clap`, `serde`, `serde_json`, `reqwest`, `egui`, `eframe`, `egui_extras`.

### `terminal-ui`

*   **Purpose**: Responsible for rendering the Text User Interface (TUI) of the terminal. It handles user input and displays command output.
*   **Key Dependencies**: `ratatui`, `crossterm` (for TUI rendering), `tokio` (async), `serde` (serialization), `ollama-client`, `python-bridge` (for interacting with AI and Python functionalities).
*   **Current Shell Integration**: The `TerminalApp` in this crate processes user input by either dispatching it as a system tool request (via `python-bridge`) or sending it to the `ollama-client` for AI response generation. It does *not* directly execute user input as shell commands.

### `egui-ui`

*   **Purpose**: Provides the Graphical User Interface (GUI) for the terminal, built using the `egui` framework. It offers an alternative visual interface to the TUI.
*   **Key Dependencies**: `egui`, `eframe` (GUI framework), `tokio` (async), `chrono` (time handling), `ollama-client`, `python-bridge`, `system-tools` (for interacting with AI, Python, and system functionalities).

### `terminal-emulator`

*   **Purpose**: The core component for emulating a terminal. It manages pseudo-terminal (PTY) interactions, executes commands, and parses terminal output.
*   **Key Modules**:
    *   `command_block`: Represents a single command execution block, tracking its state, command, and output.
    *   `command_history`: Manages a history of `CommandBlock`s.
    *   `pty_executor`: This module is responsible for the actual execution of shell commands. It uses `portable-pty` to open a PTY, spawns the command using the system's default shell, and streams its stdout/stderr. It provides `ExecutionEvent`s to report the command's progress and output.
*   **Current Integration**: While `terminal-emulator` is capable of executing shell commands, it is currently *not* directly integrated into the `terminal-ui`'s main input processing loop for direct shell command execution. Its functionalities are available but not directly utilized for user-typed shell commands in the TUI.

### `ollama-client`

*   **Purpose**: Facilitates communication with the Ollama API, enabling local-first AI capabilities within the terminal. It handles requests and responses to the LLM.
*   **Key Dependencies**: `reqwest` (HTTP client), `serde` (serialization), `tokio` (async), `futures-util`.

### `python-bridge`

*   **Purpose**: Provides a bridge for interoperability between Rust and Python code. This allows for the integration of Python-based functionalities or plugins.
*   **Key Dependencies**: `pyo3` (Python binding), `tokio` (async), `serde` (serialization), `serde_yaml`, `system-tools` (suggesting Python can interact with system functionalities via this bridge).

### `system-tools`

*   **Purpose**: Offers a suite of system-level functionalities, including process management, file system operations, and potentially network interactions. It provides the terminal with capabilities to interact with the underlying operating system.
*   **Key Dependencies**: `tokio` (async), `regex`, `walkdir` (file system), `sysinfo`, `nix`, `libc` (system information and low-level OS interaction), `reqwest`.