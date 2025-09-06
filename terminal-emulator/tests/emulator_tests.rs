#[cfg(test)]
mod tests {
    use terminal_emulator::{CommandBlock, BlockState, PtyExecutor};
    use std::time::Duration;

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
    fn test_block_state_transitions() {
        let mut block = CommandBlock::new(
            "test command".to_string(),
            "/tmp".to_string(),
        );
        
        assert_eq!(block.state, BlockState::Editing);
        
        block.start_execution();
        assert_eq!(block.state, BlockState::Running);
        
        block.complete(0, Duration::from_millis(100));
        assert_eq!(block.state, BlockState::Success);
        assert!(block.is_complete());
    }
    
    #[test]
    fn test_pty_executor_creation() {
        let executor = PtyExecutor::new();
        assert!(executor.is_ok());
    }
}