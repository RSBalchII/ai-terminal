# AI Terminal Constitution

## Core Principles

### I. Specification-First Development
- Every feature begins with a complete specification before any code
- Specifications define the "what" and "why", never the "how"
- Code serves specifications, not the other way around
- All changes trace back to specification requirements

### II. Modular Architecture
- Functionality separated into distinct, reusable crates
- Each crate has a single, well-defined responsibility
- Inter-crate dependencies must be explicit and minimal
- Libraries must be independently testable

### III. GUI-First Interface
- Primary interface is the standalone egui GUI application
- Terminal interface maintained as secondary option
- All features accessible through GUI menus and visual elements
- Keyboard shortcuts and accessibility are first-class citizens

### IV. AI Integration Excellence
- Ollama integration must be robust with proper timeout handling
- Support multiple AI models with seamless switching
- Streaming responses with real-time UI updates
- Graceful degradation when AI services unavailable

### V. System Tools Safety
- All system operations must go through security validation
- Sandboxed execution with explicit permissions
- Clear user feedback for all operations
- Comprehensive logging for audit trails

### VI. Async-First Design
- All I/O operations must be non-blocking
- Proper timeout protection on all network calls
- UI must remain responsive during long operations
- Clear visual feedback for async operations

### VII. User Experience Priority
- Word wrap and scrolling must work seamlessly
- Settings persistence across sessions
- Intuitive menu structure and navigation
- Error messages must be clear and actionable

## Architecture Constraints

### Technology Stack
- **Language**: Rust (stable channel)
- **GUI Framework**: egui/eframe
- **Async Runtime**: Tokio
- **AI Backend**: Ollama (local)
- **Build System**: Cargo workspace
- **Storage**: Local configuration files

### Performance Standards
- GUI must maintain 60 FPS during normal operation
- AI response timeout: 45 seconds (configurable)
- Startup time: < 3 seconds
- Memory usage: < 200MB baseline

## Development Workflow

### Specification Process
1. Create feature specification using /specify command
2. Define user stories and acceptance criteria
3. Mark all ambiguities with [NEEDS CLARIFICATION]
4. Review and approve specification before implementation

### Implementation Process
1. Generate implementation plan using /plan command
2. Validate plan against constitution principles
3. Implement following the specification exactly
4. Test against acceptance criteria

### Quality Gates
- [ ] Specification reviewed and approved
- [ ] Implementation plan validated
- [ ] All tests passing
- [ ] UI remains responsive
- [ ] No blocking operations in event loops
- [ ] Error handling comprehensive

## Governance

- This constitution supersedes all other development practices
- Amendments require documentation of rationale and migration plan
- All specifications must align with constitutional principles
- Every PR must verify constitutional compliance
- Complexity must be justified by specification requirements

**Version**: 1.0.0 | **Ratified**: 2025-09-03 | **Last Amended**: 2025-09-03
