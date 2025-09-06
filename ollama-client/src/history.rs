use std::collections::VecDeque;

/// A simple history management system for conversations
#[derive(Debug, Clone)]
pub struct ConversationHistory {
    /// The history of messages
    history: VecDeque<HistoryEntry>,
    
    /// The maximum number of messages to keep in history
    max_entries: usize,
}

/// A single entry in the conversation history
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The role of the speaker (e.g., "user", "assistant")
    pub role: String,
    
    /// The content of the message
    pub content: String,
    
    /// Optional context from Ollama responses
    pub context: Option<Vec<i32>>,
}

impl HistoryEntry {
    /// Create a new HistoryEntry
    pub fn new(role: String, content: String, context: Option<Vec<i32>>) -> Self {
        Self { role, content, context }
    }
}

impl ConversationHistory {
    /// Create a new ConversationHistory with a maximum number of entries
    pub fn new(max_entries: usize) -> Self {
        Self {
            history: VecDeque::new(),
            max_entries,
        }
    }
    
    /// Add a new entry to the history
    pub fn add_entry(&mut self, role: String, content: String, context: Option<Vec<i32>>) {
        self.history.push_back(HistoryEntry::new(role, content, context));
        
        // Ensure we don't exceed the maximum number of entries
        if self.history.len() > self.max_entries {
            self.history.pop_front();
        }
    }
    
    /// Get the current history as a vector
    pub fn get_history(&self) -> Vec<HistoryEntry> {
        self.history.iter().cloned().collect()
    }
    
    /// Clear the history
    pub fn clear(&mut self) {
        self.history.clear();
    }
    
    /// Get the number of entries in the history
    pub fn len(&self) -> usize {
        self.history.len()
    }
    
    /// Check if the history is empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
    
    /// Get the context from the last assistant message
    pub fn get_last_context(&self) -> Option<Vec<i32>> {
        // Iterate through history in reverse to find the last assistant message with context
        for entry in self.history.iter().rev() {
            if entry.role == "assistant" && entry.context.is_some() {
                return entry.context.clone();
            }
        }
        None
    }
}