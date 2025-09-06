use ollama_client::history::ConversationHistory;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history() {
        let history = ConversationHistory::new(5);
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_add_entry() {
        let mut history = ConversationHistory::new(5);
        history.add_entry("user".to_string(), "Hello".to_string(), None);
        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_get_history() {
        let mut history = ConversationHistory::new(5);
        history.add_entry("user".to_string(), "Hello".to_string(), None);
        history.add_entry("assistant".to_string(), "Hi there!".to_string(), None);
        
        let entries = history.get_history();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].role, "user");
        assert_eq!(entries[0].content, "Hello");
        assert_eq!(entries[1].role, "assistant");
        assert_eq!(entries[1].content, "Hi there!");
    }

    #[test]
    fn test_clear_history() {
        let mut history = ConversationHistory::new(5);
        history.add_entry("user".to_string(), "Hello".to_string(), None);
        assert_eq!(history.len(), 1);
        
        history.clear();
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_history_limit() {
        let mut history = ConversationHistory::new(3);
        
        // Add more entries than the limit
        history.add_entry("user".to_string(), "1".to_string(), None);
        history.add_entry("assistant".to_string(), "2".to_string(), None);
        history.add_entry("user".to_string(), "3".to_string(), None);
        history.add_entry("assistant".to_string(), "4".to_string(), None);
        
        // Verify that the history is limited to the last 3 entries
        assert_eq!(history.len(), 3);
        let entries = history.get_history();
        assert_eq!(entries[0].content, "2");
        assert_eq!(entries[1].content, "3");
        assert_eq!(entries[2].content, "4");
    }
    
    #[test]
    fn test_get_last_context() {
        let mut history = ConversationHistory::new(5);
        
        // Add entries without context
        history.add_entry("user".to_string(), "Hello".to_string(), None);
        history.add_entry("assistant".to_string(), "Hi there!".to_string(), None);
        
        // Verify that no context is returned
        assert_eq!(history.get_last_context(), None);
        
        // Add an entry with context
        history.add_entry("user".to_string(), "How are you?".to_string(), None);
        history.add_entry("assistant".to_string(), "I'm fine, thanks!".to_string(), Some(vec![1, 2, 3]));
        
        // Verify that the context is returned
        assert_eq!(history.get_last_context(), Some(vec![1, 2, 3]));
        
        // Add another entry without context
        history.add_entry("user".to_string(), "That's good to hear.".to_string(), None);
        
        // Verify that the last context is still returned
        assert_eq!(history.get_last_context(), Some(vec![1, 2, 3]));
    }
}