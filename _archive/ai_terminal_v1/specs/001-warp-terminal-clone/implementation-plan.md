# Implementation Plan: Warp Terminal Clone with Local LLMs

**Feature**: Warp Terminal Clone  
**Spec**: [feature-spec.md](./feature-spec.md)  
**Created**: 2025-09-03  
**Status**: Updated (GUI Terminal → Persistence)

## Architecture Overview (As-Built + Planned)

### Technology Stack
- **Language**: Rust (performance, safety)
- **GUI Framework**: egui (primary UX)
- **Terminal Emulator**: portable-pty via terminal-emulator crate (PTY execution engine)
- **AI Backend**: Ollama (local LLMs)
- **Database**: SQLite (rusqlite) for sessions/history (Phase 2)
- **Syntax Highlighting**: syntect (optional, later)
- **Shell Integration**: PTY for real shell execution

### Component Architecture
```
┌───────────────────────────────────────────────┐
│               AI Terminal (GUI)               │
│                   (egui)                      │
├───────────────────────────────────────────────┤
│  TerminalController      │    ChatController  │
│  (Blocks, History, PTY)  │    (Existing)      │
├───────────────┬──────────┴──────────┬─────────┤
│ terminal-emulator (PTY, CommandBlock, History) │
│ ollama-client (local AI)  │ python-bridge (tools)│
├───────────────────────────┴────────────────────┤
│        system-tools (filesystem/network/process)│
└────────────────────────────────────────────────┘
```

- TerminalController orchestrates PTY execution via terminal-emulator, maintains CommandHistory, and updates GUI blocks.
- ChatController remains as-is; both modes co-exist in AiChatApp.

## Phase 1: GUI Terminal Mode (egui)

### 1.1 Terminal Mode Infrastructure
- Add `AppMode::Terminal` to egui app
- Introduce `TerminalController` struct: holds CommandHistory, manages PtyExecutor tasks and event channels
- Add menu entry and shortcuts to switch between Chat and Terminal

### 1.2 PTY Execution Pipeline
- Spawn PtyExecutor per command; stream ExecutionEvent to GUI
- Update CommandBlock incrementally (stdout/stderr -> output aggregation)
- Handle exit codes, duration, timestamps, state transitions (Running → Success/Failed)
- Support interactive commands where feasible (initially focus on non-interactive; document limits)

### 1.3 Block Rendering and Actions
- Per-block header: status icon/color, command line, timestamp, duration, exit code
- Output: collapsible area with word-wrap; initial non-virtualized scrolling, iterate as needed
- Actions: copy command/output, collapse/expand, delete block

### 1.4 Input & Navigation
- Multiline input (Shift+Enter newline, Enter execute)
- Keyboard navigation: Ctrl+Up/Down for block focus; PageUp/Down scroll; Esc clear selection
- CWD indicator; implement `cd` handling to update TerminalController.working_dir

## Phase 2: Persistence (SQLite)

### 2.1 Schema and Storage
- Tables: sessions(id, name, created_at, updated_at), blocks(id, session_id, cmd, stdout, stderr, exit_code, ts, duration_ms, cwd, collapsed)
- DAO layer with rusqlite; migrations and versioning

### 2.2 Integration
- Auto-save block records upon completion
- Load last session on startup (configurable)
- Session selector UI; export/import as JSON/YAML

## Security Update (Local, Permissive by Default)
- Default `ToolSecurity` to permissive (max_auto_level = Dangerous, require_confirmation = false)
- Implement filesystem write/move/delete/copy with canonicalize() + restricted path checks and basic prompts (optional)
- Settings UI toggle for security mode (Permissive/Restrictive)

## Implementation Timeline (Milestone-oriented)

### Milestone A: Terminal Mode MVP
- [ ] AppMode::Terminal + menu navigation
- [ ] TerminalController with PTY execution and event handling
- [ ] Render blocks with status, timestamps, exit code; collapsible output
- [ ] Input + keyboard navigation + CWD indicator + basic cd support

### Milestone B: Persistence
- [ ] rusqlite schema + migrations
- [ ] Save blocks incrementally
- [ ] Load last session; session selector
- [ ] Export/import

### Milestone C: Security & Polish
- [ ] Permissive ToolSecurity default + toggles
- [ ] Implement filesystem write/move/delete/copy
- [ ] Improve performance (virtualized scrolling); themes; minor UX refinements

## Testing Strategy
- Unit: CommandBlock ops; DAO CRUD; ToolSecurity paths
- Integration: PTY execution; event streaming; SQLite save/load; destructive ops dry-runs
- UI: Block navigation; copy actions; persistence flows; settings toggles
- Performance: Long output scrolling; 10k blocks; startup time; memory usage

## Risks and Mitigations
- PTY interactivity: start with non-interactive focus; provide clear docs; later enhancements
- Large outputs: add collapsing now; plan virtualization next
- Destructive operations: canonicalize paths, maintain restricted list; expose toggles

## Success Criteria
- Terminal Mode MVP accepted (all acceptance scenarios met)
- Persistence works reliably across restarts
- Security posture is configurable; defaults align with local-only usage
- Performance within targets (smooth scrolling; reasonable memory usage)
