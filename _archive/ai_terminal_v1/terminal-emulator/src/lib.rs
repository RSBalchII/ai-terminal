pub mod command_block;
pub mod command_history;
pub mod pty_executor;

// Re-export main types for convenience
pub use command_block::{BlockState, CommandBlock};
pub use command_history::{CommandHistory, HistoryStats};
pub use pty_executor::{ExecutionEvent, PtyExecutor};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_block_creation() {
        let block = CommandBlock::new(
            "echo 'Hello, World!'".to_string(),
            "/home/user".to_string(),
        );
        
        assert_eq!(block.state, BlockState::Editing);
        assert!(block.command.contains("echo"));
        assert!(block.output.is_empty());
    }
    
    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new(100);
        
        let block = CommandBlock::new(
            "ls -la".to_string(),
            "/home/user".to_string(),
        );
        
        let id = history.add_block(block);
        
        assert_eq!(history.blocks().len(), 1);
        assert!(history.get_block(&id).is_some());
    }
    
    #[test]
    fn test_block_state_transitions() {
        let mut block = CommandBlock::new(
            "test command".to_string(),
            "/tmp".to_string(),
        );
        
        assert_eq!(block.state, BlockState::Editing);
        
        block.start_execution();
        assert_eq!(block.state, BlockState::Running);
        
        block.complete(0, std::time::Duration::from_millis(100));
        assert_eq!(block.state, BlockState::Success);
        assert!(block.is_complete());
    }
}
