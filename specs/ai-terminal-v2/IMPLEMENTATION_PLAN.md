# Implementation Plan: UI/UX Improvements for AI-Terminal TUI

## Overview
This document outlines the implementation plan for the three UI/UX improvements specified for Coda U-001 (The Designer):
1. Enhanced `ratatui` layout for better information density and readability
2. Implementation of a new sophisticated UI widget (command palette)
3. Addition of a basic color configuration system

## Current State Analysis
Based on code review, the current UI implementation includes:
- A vertical layout with three main sections: messages area, input area, and status bar
- Basic chat UI with command blocks showing commands, output, and status
- Help modal that appears when F1 is pressed
- Simple color scheme with white text, cyan for commands, and status-colored output

## 1. Enhanced Layout Implementation

### Current Issues
- Limited information hierarchy
- No visual distinction between different types of content
- Fixed layout that doesn't adapt well to different terminal sizes

### Proposed Improvements

#### A. Multi-Pane Layout
```
┌─────────────────────────────────────────────────────────────┐
│ AI Terminal v0.1.0                    [Mode: Chat] [🕒 14:30]│
├─────────────────────────────────────────────────────────────┤
│ ┌[📝 Command Block]────────────────────────────────────────┐ │
│ │$ ls -la                                                   │ │
│ │total 20                                                   │ │
│ │drwxr-xr-x  4 user user 4096 Sep  6 14:30 .               │ │
│ │drwxr-xr-x 11 user user 4096 Sep  6 14:25 ..              │ │
│ │-rw-r--r--  1 user user  497 Sep  6 14:20 Cargo.toml      │ │
│ │└ Status: Completed (Exit: 0) (Duration: 120ms)           │ │
│ ├[🤖 AI Response]───────────────────────────────────────────┤ │
│ │The directory contains 4 items including 2 directories     │ │
│ │and 2 files. The Cargo.toml file is 497 bytes in size.    │ │
│ │└ Status: Completed (Duration: 850ms)                     │ │
│ └───────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│ > Enter command...                                          │
├─────────────────────────────────────────────────────────────┤
│ [F1 Help] [F10 Exit] [PgUp/PgDn Scroll] [Ctrl+C Cancel]     │
└─────────────────────────────────────────────────────────────┘
```

#### B. Implementation Steps
1. Refactor the `render_chat_ui` function to support a multi-pane layout
2. Add visual separators and distinct styling for different content types
3. Implement dynamic resizing based on terminal dimensions
4. Add timestamp display in the header
5. Improve status bar with more contextual information

## 2. Command Palette Widget Implementation

### Design
A modal overlay that appears when a specific key combination is pressed (e.g., Ctrl+K), allowing users to quickly access commands and features.

### Features
- Fuzzy search for commands
- Keyboard navigation
- Categorized commands (File, Edit, View, AI, etc.)
- Recent commands history

### Implementation Steps
1. Create a new `command_palette` module in the terminal-ui crate
2. Implement the widget rendering logic using `ratatui`
3. Add event handling for keyboard input
4. Integrate with the main application event loop
5. Add commands for common actions (new session, clear screen, etc.)

## 3. Color Configuration System

### Design
A theme system that allows users to customize the terminal appearance through a configuration file.

### Theme Structure
```rust
pub struct Theme {
    pub name: String,
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub text: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub command: Color,
    pub ai_response: Color,
}
```

### Implementation Steps
1. Create a `theme` module in the terminal-ui crate
2. Implement theme loading from a TOML configuration file
3. Add predefined themes (default, dark, light, high-contrast)
4. Modify existing UI components to use theme colors
5. Add a command to switch themes

## File Structure Changes

```
terminal-ui/
├── src/
│   ├── layout/
│   │   ├── mod.rs
│   │   └── manager.rs
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── command_block.rs
│   │   └── command_palette.rs
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   └── presets.rs
│   ├── lib.rs
│   └── ui.rs
├── config/
│   └── theme.toml
└── tests/
    ├── layout_tests.rs
    ├── widget_tests.rs
    └── theme_tests.rs
```

## Testing Strategy

### 1. Layout Tests
- Test layout calculations with different terminal sizes
- Verify proper rendering of multi-pane layout
- Test dynamic resizing behavior

### 2. Widget Tests
- Test command palette search functionality
- Verify keyboard navigation
- Test event handling

### 3. Theme Tests
- Test theme loading from configuration files
- Verify color application to UI components
- Test theme switching

## Implementation Timeline

### Phase 1: Foundation (2 days)
- Refactor existing layout code
- Create theme system infrastructure
- Set up test framework for new components

### Phase 2: Command Palette (3 days)
- Implement command palette widget
- Add search and filtering logic
- Integrate with main event loop

### Phase 3: Theme System (2 days)
- Implement theme loading and management
- Modify UI components to use themes
- Add predefined themes

### Phase 4: Integration & Testing (2 days)
- Integrate all components
- Write comprehensive tests
- Conduct usability testing

## Success Criteria
1. Improved information density and readability in the terminal UI
2. Functional command palette widget that enhances user productivity
3. Working theme system that allows for customization
4. All new features are covered by tests
5. No degradation in performance or responsiveness