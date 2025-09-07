//! Enhanced command blocks for the AI Terminal
//!
//! This module provides enhanced functionality for command blocks,
//! including visual status indicators, grouping, and inline editing.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Represents a single command execution block in the terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandBlock {
    /// Unique identifier for this block
    pub id: Uuid,
    
    /// The command that was executed
    pub command: String,
    
    /// The complete output (stdout + stderr)
    pub output: String,
    
    /// Separate stdout for processing
    pub stdout: String,
    
    /// Separate stderr for error handling
    pub stderr: String,
    
    /// Exit code of the command
    pub exit_code: Option<i32>,
    
    /// When the command was started
    pub timestamp: DateTime<Local>,
    
    /// How long the command took to execute
    pub duration: Option<Duration>,
    
    /// Current state of the block
    pub state: BlockState,
    
    /// Working directory when command was executed
    pub working_dir: String,
    
    /// Whether this block is currently being edited
    pub is_editing: bool,
    
    /// Whether this block is selected/highlighted
    pub is_selected: bool,
}

/// Represents the current state of a command block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockState {
    /// Command is being edited/typed
    Editing,
    
    /// Command is currently running
    Running,
    
    /// Command completed successfully (exit code 0)
    Success,
    
    /// Command failed (non-zero exit code)
    Failed,
    
    /// Command was cancelled/interrupted
    Cancelled,
    
    /// Command timed out
    TimedOut,
}

impl CommandBlock {
    /// Create a new command block with the given command
    pub fn new(command: String, working_dir: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            command,
            output: String::new(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            timestamp: Local::now(),
            duration: None,
            state: BlockState::Editing,
            working_dir,
            is_editing: false,
            is_selected: false,
        }
    }
    
    /// Start executing this command
    pub fn start_execution(&mut self) {
        self.state = BlockState::Running;
        self.timestamp = Local::now();
        self.is_editing = false;
    }
    
    /// Mark the command as completed
    pub fn complete(&mut self, exit_code: i32, duration: Duration) {
        self.exit_code = Some(exit_code);
        self.duration = Some(duration);
        self.state = if exit_code == 0 {
            BlockState::Success
        } else {
            BlockState::Failed
        };
        self.is_editing = false;
    }
    
    /// Append output to the block
    pub fn append_output(&mut self, text: &str, is_stderr: bool) {
        if is_stderr {
            self.stderr.push_str(text);
        } else {
            self.stdout.push_str(text);
        }
        self.output.push_str(text);
    }
    
    /// Get a display-friendly status icon
    pub fn status_icon(&self) -> &str {
        match self.state {
            BlockState::Editing => "📝",
            BlockState::Running => "⚡",
            BlockState::Success => "✅",
            BlockState::Failed => "❌",
            BlockState::Cancelled => "⛔",
            BlockState::TimedOut => "⏱️",
        }
    }
    
    /// Get a color for this block based on its state
    pub fn status_color(&self) -> Color {
        match self.state {
            BlockState::Editing => Color::Gray,
            BlockState::Running => Color::Yellow,
            BlockState::Success => Color::Green,
            BlockState::Failed => Color::Red,
            BlockState::Cancelled => Color::Magenta,
            BlockState::TimedOut => Color::Blue,
        }
    }
    
    /// Check if this block has completed execution
    pub fn is_complete(&self) -> bool {
        matches!(
            self.state,
            BlockState::Success | BlockState::Failed | BlockState::Cancelled | BlockState::TimedOut
        )
    }
    
    /// Start editing this command block
    pub fn start_editing(&mut self) {
        self.is_editing = true;
        self.state = BlockState::Editing;
    }
    
    /// Finish editing this command block
    pub fn finish_editing(&mut self) {
        self.is_editing = false;
    }
    
    /// Set the selection state of this block
    pub fn set_selected(&mut self, selected: bool) {
        self.is_selected = selected;
    }
    
    /// Render the command block
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
        let block_style = if self.is_selected {
            Style::default().fg(theme.text).bg(theme.primary)
        } else {
            Style::default().fg(theme.text).bg(theme.background)
        };
        
        let block_widget = Block::default()
            .borders(Borders::ALL)
            .border_style(block_style);
            
        let inner_area = block_widget.inner(area);
        
        // Create the layout for the command block
        let block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Command line
                Constraint::Min(1),    // Output area
            ])
            .split(inner_area);
        
        // Render the command line
        let command_line = Line::from(vec![
            Span::styled(self.status_icon(), Style::default().fg(self.status_color())),
            Span::raw(" "),
            Span::styled(&self.command, Style::default().fg(theme.command)),
        ]);
        
        let command_widget = Paragraph::new(command_line)
            .style(block_style);
            
        f.render_widget(command_widget, block_layout[0]);
        
        // Render the output
        let output_widget = Paragraph::new(self.output.clone())
            .style(block_style);
            
        f.render_widget(output_widget, block_layout[1]);
        
        // Render the block border
        f.render_widget(block_widget, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_block_creation() {
        let block = CommandBlock::new("ls -la".to_string(), "/home/user".to_string());
        
        assert_eq!(block.command, "ls -la");
        assert_eq!(block.working_dir, "/home/user");
        assert_eq!(block.state, BlockState::Editing);
        assert_eq!(block.is_editing, false);
        assert_eq!(block.is_selected, false);
    }
    
    #[test]
    fn test_command_block_state_transitions() {
        let mut block = CommandBlock::new("ls -la".to_string(), "/home/user".to_string());
        
        // Start execution
        block.start_execution();
        assert_eq!(block.state, BlockState::Running);
        assert_eq!(block.is_editing, false);
        
        // Complete successfully
        block.complete(0, Duration::from_secs(1));
        assert_eq!(block.state, BlockState::Success);
        assert_eq!(block.is_editing, false);
        assert_eq!(block.exit_code, Some(0));
        assert!(block.duration.is_some());
        
        // Create a new block and fail it
        let mut block2 = CommandBlock::new("invalid-command".to_string(), "/home/user".to_string());
        block2.start_execution();
        block2.complete(1, Duration::from_secs(1));
        assert_eq!(block2.state, BlockState::Failed);
        assert_eq!(block2.exit_code, Some(1));
    }
    
    #[test]
    fn test_command_block_editing() {
        let mut block = CommandBlock::new("ls -la".to_string(), "/home/user".to_string());
        
        // Start editing
        block.start_editing();
        assert_eq!(block.is_editing, true);
        assert_eq!(block.state, BlockState::Editing);
        
        // Finish editing
        block.finish_editing();
        assert_eq!(block.is_editing, false);
    }
    
    #[test]
    fn test_command_block_selection() {
        let mut block = CommandBlock::new("ls -la".to_string(), "/home/user".to_string());
        
        // Select the block
        block.set_selected(true);
        assert_eq!(block.is_selected, true);
        
        // Deselect the block
        block.set_selected(false);
        assert_eq!(block.is_selected, false);
    }
    
    #[test]
    fn test_status_icon() {
        let mut block = CommandBlock::new("ls -la".to_string(), "/home/user".to_string());
        
        // Editing state
        assert_eq!(block.status_icon(), "📝");
        
        // Running state
        block.start_execution();
        assert_eq!(block.status_icon(), "⚡");
        
        // Success state
        block.complete(0, Duration::from_secs(1));
        assert_eq!(block.status_icon(), "✅");
        
        // Failed state
        let mut block2 = CommandBlock::new("invalid-command".to_string(), "/home/user".to_string());
        block2.start_execution();
        block2.complete(1, Duration::from_secs(1));
        assert_eq!(block2.status_icon(), "❌");
    }
}