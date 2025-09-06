# Task Breakdown: Warp Terminal Clone Implementation (Updated)

This plan prioritizes the GUI Terminal Mode (egui) and Persistence, aligned with the Brownfield analysis and approved decisions.

## Immediate Plan (Approved)

### Phase A: GUI Terminal Mode (egui)
1. Terminal Mode Infrastructure
   - [ ] Add `AppMode::Terminal` to egui app
   - [ ] Create `TerminalController` (history, PTY task mgmt, event channels)
   - [ ] Add menu entries and keyboard shortcuts to switch modes
2. PTY Execution Pipeline
   - [ ] Connect `PtyExecutor` to controller; stream `ExecutionEvent`
   - [ ] Update `CommandBlock` incrementally; handle stderr/stdout, exit code, duration, timestamps
3. Block UI and Actions
   - [ ] Render block headers (status icon/color, cmd, timestamp, duration, exit code)
   - [ ] Collapsible output with word-wrap; scrollable history
   - [ ] Actions: copy command/output; collapse/expand; delete block
4. Input & Navigation
   - [ ] Multiline input (Shift+Enter newline, Enter execute)
   - [ ] Keyboard navigation (Ctrl+Up/Down blocks; PageUp/Down scroll; Esc clear)
   - [ ] CWD indicator; implement `cd` to update working dir

Acceptance (Phase A)
- Running a command creates a block with correct status and output; navigation and copying work; CWD updates via `cd`.

### Phase B: Persistence (SQLite)
1. Storage
   - [ ] Add `rusqlite`; define schema (sessions, blocks)
   - [ ] Implement migrations and DAO layer
2. Integration
   - [ ] Auto-save block on completion
   - [ ] Load last session on startup (toggleable)
   - [ ] Session selector UI
   - [ ] Export/Import sessions (JSON/YAML)

Acceptance (Phase B)
- Blocks persist across restarts; sessions can be selected and exported/imported reliably.

### Security Update (Permissive Local Default)
- [ ] Set `ToolSecurity` default to permissive (Dangerous, confirmations off)
- [ ] Implement filesystem write/move/delete/copy with canonicalize() + restricted path checks
- [ ] Add Settings toggle for security profile (Permissive/Restrictive); optional confirmations

Acceptance (Security)
- Destructive ops work locally by default with basic guardrails; toggles available in Settings.

---

## Reference (Original Detailed Backlog)

The following is the previously defined comprehensive backlog, preserved for traceability and future phases.

## Phase 1: Core Terminal Foundation (Week 1-2)

### 1. PTY Integration Setup
- [ ] Add `portable-pty` crate to dependencies
- [ ] Create `terminal-emulator` crate in workspace
- [ ] Implement basic PTY spawn and command execution
- [ ] Handle stdin/stdout/stderr properly
- [ ] Test with simple commands (ls, echo, etc.)

### 2. Command Block Data Structure
- [ ] Define `CommandBlock` struct with all fields
- [ ] Implement `BlockState` enum for status tracking
- [ ] Create `CommandHistory` manager
- [ ] Add serialization for persistence
- [ ] Write unit tests for block operations

### 3. Basic Block Rendering
- [ ] Create `BlockWidget` egui component
- [ ] Render command input area
- [ ] Display command output
- [ ] Show status indicators (running/success/failed)
- [ ] Implement basic scrolling

## Phase 2: Enhanced Command Blocks (Week 3-4)

### 4. Block Navigation System
- [ ] Implement Ctrl+Up/Down navigation
- [ ] Add visual selection indicator
- [ ] Create focused block highlighting
- [ ] Support Page Up/Down for fast navigation
- [ ] Add Home/End shortcuts

### 5. Selection and Copying
- [ ] Implement text selection within blocks
- [ ] Add block-level selection
- [ ] Create copy command (Ctrl+C)
- [ ] Support copying with formatting
- [ ] Add clipboard integration

### 6. Syntax Highlighting
- [ ] Add `syntect` crate dependency
- [ ] Load shell syntax definitions
- [ ] Implement command highlighting
- [ ] Add output highlighting (errors in red)
- [ ] Cache highlighted text for performance

### 7. Visual Status Indicators
- [ ] Create animated spinner for running commands
- [ ] Add success checkmark (✓) indicator
- [ ] Add error cross (✗) indicator
- [ ] Show execution time for completed blocks
- [ ] Implement exit code display

## Phase 3: AI Integration (Week 5-6)

### 8. Enhanced Ollama Client
- [ ] Extend existing `OllamaClient` with new methods
- [ ] Add `translate_nl_to_command` function
- [ ] Implement `explain_command` capability
- [ ] Create `explain_error` analyzer
- [ ] Build `suggest_completion` system

### 9. Natural Language Input
- [ ] Create NL input widget (Ctrl+` activation)
- [ ] Show AI thinking indicator
- [ ] Display command translation
- [ ] Allow editing before execution
- [ ] Add confidence scoring display

### 10. Error Explanation System
- [ ] Detect command failures
- [ ] Parse error messages
- [ ] Send context to LLM
- [ ] Display explanation panel
- [ ] Show fix suggestions

### 11. Inline AI Suggestions
- [ ] Implement typing detection
- [ ] Create ghost text rendering
- [ ] Add Tab to accept suggestion
- [ ] Show multiple suggestions dropdown
- [ ] Cache frequent patterns

### 12. AI Response Caching
- [ ] Create LRU cache structure
- [ ] Implement cache key generation
- [ ] Add SQLite cache persistence
- [ ] Build cache invalidation logic
- [ ] Monitor cache hit rates

## Phase 4: Notebook System (Week 7-8)

### 13. Notebook Data Model
- [ ] Define notebook YAML schema
- [ ] Create `Notebook` struct
- [ ] Implement variable system
- [ ] Add command sequencing
- [ ] Build serialization/deserialization

### 14. Notebook Storage
- [ ] Create notebooks directory structure
- [ ] Implement file-based storage
- [ ] Add notebook listing API
- [ ] Create import/export functions
- [ ] Build version control

### 15. Notebook Editor UI
- [ ] Create notebook creation dialog
- [ ] Build command list editor
- [ ] Add variable definition UI
- [ ] Implement drag-and-drop reordering
- [ ] Create preview mode

### 16. Variable Substitution Engine
- [ ] Parse variable placeholders ({{VAR}})
- [ ] Create variable input dialog
- [ ] Implement substitution logic
- [ ] Add validation for variables
- [ ] Support environment variables

### 17. Notebook Execution
- [ ] Build execution engine
- [ ] Add step-by-step mode
- [ ] Implement pause/continue
- [ ] Create error handling options
- [ ] Log execution history

## Phase 5: Modern UI/UX (Week 9-10)

### 18. Warp-like Visual Design
- [ ] Create modern color palette
- [ ] Design block borders and spacing
- [ ] Implement gradient backgrounds
- [ ] Add subtle shadows
- [ ] Create visual hierarchy

### 19. Theme System
- [ ] Define `Theme` struct
- [ ] Create default dark theme
- [ ] Build light theme
- [ ] Add theme switching UI
- [ ] Persist theme preference

### 20. Animations
- [ ] Implement smooth scrolling
- [ ] Add block appear animation
- [ ] Create panel slide transitions
- [ ] Build fade effects for AI suggestions
- [ ] Add loading animations

### 21. Custom Fonts
- [ ] Add JetBrains Mono font
- [ ] Include Fira Code as alternative
- [ ] Implement font size adjustment
- [ ] Add ligature support
- [ ] Create font preference UI

### 22. Multi-line Editor
- [ ] Extend input area for multi-line
- [ ] Add line numbers
- [ ] Implement auto-indent
- [ ] Add bracket matching
- [ ] Support Shift+Enter for newlines

## Phase 6: Performance & Polish (Week 11-12)

### 23. Virtual Scrolling
- [ ] Implement viewport calculation
- [ ] Create lazy rendering system
- [ ] Add block recycling
- [ ] Optimize re-renders
- [ ] Test with 10k+ blocks

### 24. SQLite Integration
- [ ] Design database schema
- [ ] Implement command history table
- [ ] Add full-text search index
- [ ] Create query optimization
- [ ] Build data migration system

### 25. Background Processing
- [ ] Move AI calls to background threads
- [ ] Implement job queue
- [ ] Add cancellation support
- [ ] Create priority system
- [ ] Monitor thread pool

### 26. Memory Management
- [ ] Profile memory usage
- [ ] Implement block pruning
- [ ] Add output truncation
- [ ] Create memory limits
- [ ] Build cleanup routines

## Testing & Documentation

### 27. Integration Tests
- [ ] Test PTY with various shells
- [ ] Verify AI integration
- [ ] Test notebook execution
- [ ] Check theme switching
- [ ] Validate persistence

### 28. Performance Benchmarks
- [ ] Measure startup time
- [ ] Test scrolling FPS
- [ ] Check AI response times
- [ ] Monitor memory growth
- [ ] Profile CPU usage

### 29. User Documentation
- [ ] Write user guide
- [ ] Create video tutorials
- [ ] Build interactive onboarding
- [ ] Document keyboard shortcuts
- [ ] Add troubleshooting guide

### 30. Beta Testing
- [ ] Recruit beta testers
- [ ] Create feedback system
- [ ] Track usage metrics
- [ ] Gather feature requests
- [ ] Iterate based on feedback

## Priority Order

1. **Critical** (Must have for MVP):
   - Phase A (GUI Terminal Mode)
   - Phase B (Persistence)
2. **Important** (Key differentiators):
   - Security update & polish, theme system, virtualized scrolling
3. **Nice to Have** (Polish):
   - Notebooks, advanced editing, animations, docs
