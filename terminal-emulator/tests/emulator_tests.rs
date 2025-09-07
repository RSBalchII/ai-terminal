#[cfg(test)]
mod tests {
    use terminal_emulator::{CommandBlock, BlockState, PtyExecutor, ExecutionEvent};
    use std::time::Duration;
    use tokio::sync::mpsc;

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
    fn test_block_state_transitions_failure() {
        let mut block = CommandBlock::new(
            "test command".to_string(),
            "/tmp".to_string(),
        );
        
        block.start_execution();
        assert_eq!(block.state, BlockState::Running);
        
        block.complete(1, Duration::from_millis(100));
        assert_eq!(block.state, BlockState::Failed);
        assert!(block.is_complete());
    }
    
    #[test]
    fn test_block_append_output() {
        let mut block = CommandBlock::new(
            "test command".to_string(),
            "/tmp".to_string(),
        );
        
        block.append_output("stdout output", false);
        assert_eq!(block.stdout, "stdout output");
        assert_eq!(block.output, "stdout output");
        
        block.append_output("stderr output", true);
        assert_eq!(block.stderr, "stderr output");
        assert_eq!(block.output, "stdout outputstderr output");
    }
    
    #[test]
    fn test_pty_executor_creation() {
        let executor = PtyExecutor::new();
        assert!(executor.is_ok());
    }
    
    #[tokio::test]
    async fn test_pty_executor_execute_simple_command() {
        let executor = PtyExecutor::new().unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Execute a simple command
        let result = executor.execute("echo 'test'", tx).await;
        assert!(result.is_ok());
        
        // Collect events
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        
        // Should have at least a Started and Completed event
        assert!(events.iter().any(|e| matches!(e, ExecutionEvent::Started)));
        assert!(events.iter().any(|e| matches!(e, ExecutionEvent::Completed { .. })));
    }
    
    #[tokio::test]
    async fn test_pty_executor_execute_block() {
        let executor = PtyExecutor::new().unwrap();
        let mut block = CommandBlock::new("echo 'test'".to_string(), "/tmp".to_string());
        
        // Execute the block
        let result = executor.execute_block(&mut block).await;
        assert!(result.is_ok());
        
        // Check that the block was completed successfully
        assert_eq!(block.state, BlockState::Success);
        assert!(block.output.contains("test"));
    }
    
    #[tokio::test]
    async fn test_pty_executor_working_dir() {
        let mut executor = PtyExecutor::new().unwrap();
        let original_dir = executor.working_dir().to_string();
        
        // Change working directory
        executor.set_working_dir("/tmp".to_string());
        assert_eq!(executor.working_dir(), "/tmp");
        
        // Reset to original
        executor.set_working_dir(original_dir);
    }
}