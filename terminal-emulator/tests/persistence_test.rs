#[test]
fn test_persistent_history() {
    use std::fs;
    use terminal_emulator::CommandHistory;
    
    // Create a temporary history file
    let temp_dir = std::env::temp_dir();
    let history_file = temp_dir.join("ai_terminal_test_history.txt");
    
    // Make sure the file doesn't exist
    let _ = fs::remove_file(&history_file);
    
    // Create a custom history manager for testing
    let mut history = CommandHistory::with_file(100, history_file.clone()).unwrap();
    history.add_command("test command 1".to_string()).unwrap();
    history.add_command("test command 2".to_string()).unwrap();
    
    // Create a new history instance to test loading from file
    let history2 = CommandHistory::with_file(100, history_file.clone()).unwrap();
    
    // Check that we can access the commands
    assert_eq!(history2.entries().len(), 2);
    assert_eq!(history2.get_command(0).unwrap().command, "test command 2");
    assert_eq!(history2.get_command(1).unwrap().command, "test command 1");
    
    // Clean up
    let _ = fs::remove_file(&history_file);
}

#[test]
fn test_persistent_history_json_format() {
    use std::fs;
    use terminal_emulator::{CommandHistory, HistoryEntry};
    use chrono::Local;
    
    // Create a temporary history file
    let temp_dir = std::env::temp_dir();
    let history_file = temp_dir.join("ai_terminal_test_history_json.txt");
    
    // Make sure the file doesn't exist
    let _ = fs::remove_file(&history_file);
    
    // Create a history file with JSON format entries
    {
        let mut file = std::fs::File::create(&history_file).unwrap();
        let entry = HistoryEntry {
            command: "test command".to_string(),
            timestamp: Local::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        std::io::Write::write_all(&mut file, format!("{}\n", json).as_bytes()).unwrap();
    }
    
    // Create a history instance to test loading from file
    let history = CommandHistory::with_file(100, history_file.clone()).unwrap();
    
    // Check that we can access the command
    assert_eq!(history.entries().len(), 1);
    assert_eq!(history.get_command(0).unwrap().command, "test command");
    
    // Clean up
    let _ = fs::remove_file(&history_file);
}