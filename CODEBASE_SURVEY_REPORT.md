# AI-Terminal Codebase Survey Report

## Directory Structure

```
ai-terminal/
├── _archive/
│   └── ai_terminal_v1/
├── ai-terminal/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── mcp/
│           ├── client.rs
│           └── mod.rs
├── ollama-client/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── api.rs
│   │   ├── bin/
│   │   │   └── mcp_server.rs
│   │   ├── error.rs
│   │   ├── history.rs
│   │   ├── lib.rs
│   │   ├── mcp.rs
│   │   └── models.rs
│   └── tests/
│       ├── api_tests.rs
│       └── history_tests.rs
├── specs/
├── target/
├── terminal-emulator/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── command_block.rs
│   │   ├── command_history.rs
│   │   ├── lib.rs
│   │   └── pty_executor.rs
│   └── tests/
│       ├── command_history_tests.rs
│       ├── emulator_tests.rs
│       └── persistence_test.rs
├── terminal-ui/
│   ├── Cargo.toml
│   ├── config/
│   │   └── theme.toml
│   ├── example-theme.toml
│   ├── sample-theme.toml
│   ├── src/
│   │   ├── layout/
│   │   │   ├── manager.rs
│   │   │   └── mod.rs
│   │   ├── lib.rs
│   │   ├── theme/
│   │   │   ├── manager.rs
│   │   │   ├── mod.rs
│   │   │   └── presets.rs
│   │   └── widgets/
│   │       ├── command_palette.rs
│   │       ├── confirmation_modal.rs
│   │       └── mod.rs
│   └── tests/
│       ├── confirmation_modal_tests.rs
│       ├── integration_tests.rs
│       ├── layout_tests.rs
│       ├── theme_tests.rs
│       └── widget_tests.rs
├── Cargo.lock
├── Cargo.toml
├── FINAL_IMPLEMENTATION_REPORT.md
├── FINAL_SUMMARY.md
├── IMPLEMENTATION_SUMMARY.md
├── MCP_INTEGRATION_PLAN.md
├── OLLAMA_MCP_TOOLS_PLAN.md
├── QWEN.md
├── README.md
└── run.sh
```

## File-by-File Analysis

### Main Application (`ai-terminal/src`)

#### `main.rs`
This is the entry point for the AI Terminal application. It initializes the terminal UI and starts the main event loop.

**Key Components:**
- Uses `clap` for command-line argument parsing
- Sets up logging with `tracing`
- Creates and configures a `TerminalSession` from the `terminal_ui` crate
- Runs the terminal application and handles cleanup

#### `mcp/client.rs`
Implements an MCP (Model Context Protocol) client that can connect to MCP servers and interact with their tools.

**Key Components:**
- `MCPClient`: Main struct for managing the connection to an MCP server
- Methods for listing tools (`list_tools`) and calling tools (`call_tool`)
- Handles JSON-RPC communication with the server
- Manages process spawning and I/O for the server connection

#### `mcp/mod.rs`
Module declaration for the MCP functionality in the main application.

### Ollama Client (`ollama-client/src`)

#### `lib.rs`
Main library file that declares the modules and re-exports key types for the Ollama client.

**Key Components:**
- Declares modules: `api`, `models`, `history`, `error`, and `mcp`
- Re-exports `OllamaClient` and `OllamaRequest`/`OllamaResponse` models

#### `api.rs`
Implements the main client for interacting with the Ollama API.

**Key Components:**
- `OllamaClient`: Main struct for sending requests to the Ollama API
- `chat` method for sending requests and receiving streaming responses
- Handles HTTP communication using `reqwest`
- Implements streaming response processing

#### `models.rs`
Defines the data structures for Ollama API requests and responses.

**Key Components:**
- `OllamaRequest`: Structure for API requests with fields for model, prompt, system message, context, etc.
- `OllamaResponse`: Structure for API responses with fields for model, response text, done flag, context, and timing information

#### `history.rs`
Implements a simple history management system for conversations.

**Key Components:**
- `ConversationHistory`: Main struct for managing conversation history
- `HistoryEntry`: Represents a single entry in the conversation history
- Methods for adding entries, getting history, clearing history, and retrieving the last context

#### `error.rs`
Defines error types for the Ollama client.

**Key Components:**
- `OllamaError` enum with variants for request failures, JSON errors, IO errors, invalid responses, and missing environment variables

#### `mcp.rs`
Implements an MCP server that exposes Ollama functionality as MCP tools.

**Key Components:**
- `OllamaMcp` trait defining the MCP methods for the Ollama server
- `OllamaMcpServer`: Implementation of the MCP server for Ollama
- Methods for listing tools and calling tools via JSON-RPC
- `OllamaChatTool` and `OllamaChatResult` structures for the chat tool

#### `bin/mcp_server.rs`
Binary that runs an MCP server exposing Ollama functionality as MCP tools.

**Key Components:**
- Main function that initializes logging and creates an `OllamaMcpServer`
- Sets up stdin/stdout for JSON-RPC communication
- Main loop for processing JSON-RPC requests

### Terminal Emulator (`terminal-emulator/src`)

#### `lib.rs`
Main library file that declares the modules and re-exports key types for the terminal emulator.

**Key Components:**
- Declares modules: `command_block`, `command_history`, and `pty_executor`
- Re-exports main types: `BlockState`, `CommandBlock`, `CommandHistory`, `HistoryEntry`, `ExecutionEvent`, and `PtyExecutor`

#### `command_block.rs`
Defines structures for representing command execution blocks in the terminal.

**Key Components:**
- `CommandBlock`: Represents a single command execution with fields for command, output, exit code, timestamp, duration, state, and working directory
- `BlockState` enum representing the state of a command block (Editing, Running, Success, Failed, Cancelled, TimedOut)
- Methods for managing command block state and output

#### `command_history.rs`
Implements command history management with persistence.

**Key Components:**
- `HistoryEntry`: Represents a command in the history with command text and timestamp
- `CommandHistory`: Main struct for managing command history with in-memory storage and file persistence
- Methods for adding commands, loading/saving history, searching, and clearing history

#### `pty_executor.rs`
Implements PTY-based command execution.

**Key Components:**
- `PtyExecutor`: Main struct for executing commands in a PTY
- `ExecutionEvent` enum representing events that can occur during command execution
- Methods for executing commands and streaming events
- Uses `portable_pty` for cross-platform PTY support

### Terminal UI (`terminal-ui/src`)

#### `lib.rs`
Main library file that implements the terminal-based user interface.

**Key Components:**
- `TerminalSession`: Main struct for managing the terminal session
- UI data structures: `UIData`, `AppMode`, and `UIState`
- Methods for setting up/restoring the terminal, handling key events, rendering the UI, and managing command history
- Integration with `terminal_emulator` and `ollama_client` crates
- Implementation of command palette and confirmation modal widgets

#### `layout/manager.rs`
Manages the overall layout of the terminal UI.

**Key Components:**
- `LayoutManager`: Main struct for managing terminal layout
- Methods for calculating chat layout, content layout, and centered rectangles for modals
- Handles dynamic resizing based on terminal size

#### `layout/mod.rs`
Module declaration for the layout functionality.

#### `theme/manager.rs`
Manages themes for the terminal UI.

**Key Components:**
- `ThemeManager`: Main struct for managing available themes and the current theme
- Methods for loading/saving themes from/to TOML files
- Color parsing and conversion utilities
- Theme switching functionality

#### `theme/presets.rs`
Defines predefined themes for the terminal UI.

**Key Components:**
- `Theme`: Structure representing a color theme
- Preset themes: `default`, `dark`, `light`, and `high_contrast`

#### `theme/mod.rs`
Module declaration for the theme functionality.

#### `widgets/command_palette.rs`
Implements a command palette widget for accessing terminal commands.

**Key Components:**
- `Command`: Represents a command in the palette with ID, name, description, category, and icon
- `CommandPalette`: Main struct for the command palette widget
- Methods for filtering commands, handling input, and managing selection
- Fuzzy matching for command search using `fuzzy_matcher`

#### `widgets/confirmation_modal.rs`
Implements a confirmation modal widget for user actions.

**Key Components:**
- `ModalButton`: Represents a button in the confirmation modal
- `ConfirmationModal`: Main struct for the confirmation modal widget
- Methods for button selection and rendering
- Predefined Yes/No modal constructor

#### `widgets/mod.rs`
Module declaration for the widgets functionality.