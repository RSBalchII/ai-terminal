#[cfg(test)]
mod tests {
    use terminal_emulator::{CommandHistory, HistoryEntry};

    #[test]
    fn test_history_entry_creation() {
        let entry = HistoryEntry {
            command: "ls -la".to_string(),
            timestamp: chrono::Local::now(),
        };
        
        assert_eq!(entry.command, "ls -la");
    }
    
    #[test]
    fn test_command_history_creation() {
        let history = CommandHistory::new(100);
        assert!(history.is_ok());
    }
    
    #[test]
    fn test_add_command() {
        let mut history = CommandHistory::new(100).unwrap();
        let result = history.add_command("ls -la".to_string());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_add_empty_command() {
        let mut history = CommandHistory::new(100).unwrap();
        let initial_len = history.entries().len();
        let result = history.add_command("".to_string());
        assert!(result.is_ok());
        assert_eq!(history.entries().len(), initial_len);
    }
    
    #[test]
    fn test_add_duplicate_command() {
        let mut history = CommandHistory::new(100).unwrap();
        history.add_command("ls -la".to_string()).unwrap();
        let initial_len = history.entries().len();
        history.add_command("ls -la".to_string()).unwrap();
        assert_eq!(history.entries().len(), initial_len);
    }
    
    #[test]
    fn test_get_command() {
        let mut history = CommandHistory::new(100).unwrap();
        history.add_command("ls -la".to_string()).unwrap();
        history.add_command("pwd".to_string()).unwrap();
        
        // Index 0 should be the most recent command
        let entry = history.get_command(0);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().command, "pwd");
        
        // Index 1 should be the previous command
        let entry = history.get_command(1);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().command, "ls -la");
        
        // Index out of bounds should return None
        let entry = history.get_command(10);
        assert!(entry.is_none());
    }
    
    #[test]
    fn test_search_commands() {
        // Use a temporary file to avoid interference from existing history
        let temp_dir = std::env::temp_dir();
        let history_file = temp_dir.join("ai_terminal_test_search_history.txt");
        
        // Make sure the file doesn't exist
        let _ = std::fs::remove_file(&history_file);
        
        let mut history = CommandHistory::with_file(100, history_file.clone()).unwrap();
        history.add_command("ls -la".to_string()).unwrap();
        history.add_command("ls -l".to_string()).unwrap();
        history.add_command("pwd".to_string()).unwrap();
        
        let results = history.search("ls");
        assert_eq!(results.len(), 2);
        
        let results = history.search("pwd");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "pwd");
        
        // Clean up
        let _ = std::fs::remove_file(&history_file);
    }
    
    #[test]
    fn test_history_limit() {
        let mut history = CommandHistory::new(3).unwrap();
        history.add_command("cmd1".to_string()).unwrap();
        history.add_command("cmd2".to_string()).unwrap();
        history.add_command("cmd3".to_string()).unwrap();
        history.add_command("cmd4".to_string()).unwrap();
        
        assert_eq!(history.entries().len(), 3);
        // The oldest command should be removed
        assert_eq!(history.get_command(2).unwrap().command, "cmd2");
    }
}