# Current-State Analysis — AI Terminal (Brownfield)

Updated: 2025-09-04
Path: specs/current-state-analysis.md

Summary
- This document captures the as-built architecture and capabilities of the AI Terminal project to ground the Brownfield workflow and align upcoming specifications with the user’s goals: GUI-first Warp-like terminal, local-only AI via Ollama, and permissive local security.

Workspace overview (Rust workspace)
- ai-terminal (root)
  - src/main.rs: Ratatui TUI entry (chat-centric)
  - src/gui_main.rs: egui GUI entry (chat-centric)
  - src/system_tools_integration.rs: integrates system-tools crate; bridges Python bridge requests
- terminal-ui (crate): ratatui + crossterm TUI app (Chat, Model Selector, Help)
- egui-ui (crate): egui GUI app (AiChatApp; Chat, Settings, Model Selector)
- terminal-emulator (crate): PTY execution + command block primitives
  - command_block.rs: CommandBlock, BlockState; output aggregation, status helpers
  - command_history.rs: CommandHistory; selection, search, stats, pruning
  - pty_executor.rs: PtyExecutor using portable-pty; ExecutionEvent streaming
- ollama-client (crate): reqwest client for /api/tags, /api/generate (streaming and non-streaming), model mgmt
- python-bridge (crate): PyO3 bridge; tool execution channel; simple tool intent parser
- system-tools (crate): Filesystem, Network, Process tools; ToolExecutor; ToolSecurity

External dependencies
- Ollama server (localhost:11434)
- Tokio async runtime
- ratatui/crossterm for TUI
- egui/eframe for GUI
- PyO3 for Python integration

As-built capabilities
- Chat experiences
  - TUI (terminal-ui) chat with input, system messages, model selection overlay, offline toggle
  - GUI (egui-ui) chat with multiline input, model list, basic settings; system tools integrated
- Ollama integration (ollama-client)
  - List models, set model with timeout, generate text (with streaming API available)
  - Availability checks and basic errors
- Python bridge
  - Initializes interpreter; loads config; exposes tool execution via channel
  - parse_system_tool_request for simple patterns (ls/cat/find/ping/ps)
- System tools (local-only)
  - Filesystem: list/read/search/find fully implemented; write/move/delete/copy stubbed
  - Network: ping/resolve/curl/port-scan/test-connect implemented; netcat send/listen guarded
  - Process: list/info implemented; kill/signal disabled for safety
  - ToolExecutor with timeout + security levels; restricted paths; basic descriptions
- Terminal-emulator (PTY)
  - PtyExecutor streams output events; CommandBlock aggregation; History management

Current UX shape vs goal
- Current: Chat-first in both TUI and GUI; model selection; system tools are callable via chat commands
- Goal: Warp-like block-based terminal in GUI, with command blocks (PTY) as the primary modality; retain Chat mode

Gaps and issues (relative to Warp-like target)
- No GUI Terminal Mode integrating PTY command blocks
- No block-based UI in GUI: no block headers/status, collapsible outputs, per-block actions
- No SQLite persistence for command history/sessions
- No CWD prompts/visualization or cd behavior routed into PtyExecutor within GUI
- Security posture is conservative; destructive filesystem/process/network operations disabled or stubbed
- No AI “explain command”/“explain error” integrated in terminal flow (available as future phase)
- No theme system, virtualized scrolling for large output, or advanced editor features

Architectural strengths
- Clean separation of concerns with dedicated crates for UI, PTY, system tools, and AI client
- Streaming model for PTY output is already in place (ExecutionEvent)
- System tools integration is channel-based and composable with both TUI and GUI
- Ollama client provides both blocking and streaming pathways

Key technical risks
- PTY integration in GUI: ensuring responsive streaming updates and non-blocking UI
- Output volume: need virtualization/collapsing for large outputs to maintain 60 FPS
- State management for sessions and persistence design
- Security changes: enabling destructive ops safely (path normalization, guardrails)

Recommendations (approved direction)
- Phase 1 (GUI Terminal Mode in egui)
  - Add Terminal mode to AiChatApp (egui)
  - Introduce TerminalController managing CommandHistory + PtyExecutor lifecycle
  - Map ExecutionEvent -> CommandBlock UI with status, timestamps, duration, exit code
  - Implement command input (Enter execute, Shift+Enter newline), Ctrl+Up/Down block navigation, PageUp/Down scroll
  - Show CWD; support cd to update working dir for future commands
- Phase 2 (Persistence via SQLite)
  - rusqlite schema for sessions and blocks; auto-save on completion
  - Load last session on startup; session selector; export/import
- Security posture update (local-only permissive by default)
  - Set ToolSecurity default to Dangerous (confirmation off), add guardrails and optional confirmations in GUI Settings
  - Implement filesystem write/move/delete/copy with canonicalize() and restricted-path checks

Acceptance criteria (Phase 1)
- GUI shows a Terminal mode; commands run in PTY; blocks render status, timestamp, duration, exit code
- Collapsible output; scrollable history; keyboard navigation works
- CWD indicator visible; cd updates CWD for subsequent commands
- Chat mode remains fully functional; TUI unchanged

Acceptance criteria (Phase 2)
- Blocks persist across restarts; session manager allows load/save
- Export/import of sessions; user-controllable persistence settings

Traceability
- Aligns with specs/001-warp-terminal-clone/feature-spec.md and implementation-plan.md (to be updated) to reflect GUI-first terminal focus
- This document serves as Brownfield Step 1 (Current-State Analysis) and informs subsequent reverse specs and gap updates

