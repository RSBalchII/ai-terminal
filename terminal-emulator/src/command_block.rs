use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

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
        }
    }
    
    /// Start executing this command
    pub fn start_execution(&mut self) {
        self.state = BlockState::Running;
        self.timestamp = Local::now();
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
    pub fn status_color(&self) -> (u8, u8, u8) {
        match self.state {
            BlockState::Editing => (100, 100, 100),    // Gray
            BlockState::Running => (255, 193, 7),      // Amber
            BlockState::Success => (76, 175, 80),      // Green
            BlockState::Failed => (244, 67, 54),       // Red
            BlockState::Cancelled => (255, 152, 0),    // Orange
            BlockState::TimedOut => (156, 39, 176),    // Purple
        }
    }
    
    /// Check if this block has completed execution
    pub fn is_complete(&self) -> bool {
        matches!(
            self.state,
            BlockState::Success | BlockState::Failed | BlockState::Cancelled | BlockState::TimedOut
        )
    }
}