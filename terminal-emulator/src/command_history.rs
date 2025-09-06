use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a command in the history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    /// The command that was executed
    pub command: String,
    /// When the command was executed
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Manages command history with persistence
pub struct CommandHistory {
    /// In-memory history entries
    entries: VecDeque<HistoryEntry>,
    /// Maximum number of entries to keep in memory
    max_entries: usize,
    /// Path to the history file
    history_file: PathBuf,
}

impl CommandHistory {
    /// Create a new command history manager
    pub fn new(max_entries: usize) -> Result<Self> {
        let history_file = Self::get_history_file_path()?;
        Self::with_file(max_entries, history_file)
    }
    
    /// Create a new command history manager with a specific history file
    pub fn with_file(max_entries: usize, history_file: PathBuf) -> Result<Self> {
        let mut history = Self {
            entries: VecDeque::new(),
            max_entries,
            history_file,
        };
        
        // Load existing history from file
        history.load_from_file()?;
        
        Ok(history)
    }
    
    /// Get the path to the history file
    fn get_history_file_path() -> Result<PathBuf> {
        // Try to get XDG data home, fallback to home directory
        let data_dir = if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home)
        } else if let Ok(home) = std::env::var("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".local/share");
            path
        } else {
            // Fallback to current directory
            std::env::current_dir()?
        };
        
        // Create the ai-terminal directory if it doesn't exist
        let mut history_dir = data_dir;
        history_dir.push("ai-terminal");
        std::fs::create_dir_all(&history_dir)?;
        
        // Return the history file path
        let mut history_file = history_dir;
        history_file.push("history.txt");
        Ok(history_file)
    }
    
    /// Load history from the history file
    fn load_from_file(&mut self) -> Result<()> {
        if !self.history_file.exists() {
            return Ok(());
        }
        
        let file = File::open(&self.history_file)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            // Try to parse as JSON first (newer format)
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                self.entries.push_back(entry);
            } else {
                // Fallback to simple text format (older format)
                self.entries.push_back(HistoryEntry {
                    command: line,
                    timestamp: chrono::Local::now(),
                });
            }
            
            // Limit the number of entries
            if self.entries.len() > self.max_entries {
                self.entries.pop_front();
            }
        }
        
        Ok(())
    }
    
    /// Save history to the history file
    fn save_to_file(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.history_file)?;
        
        for entry in &self.entries {
            let json = serde_json::to_string(entry)?;
            writeln!(file, "{}", json)?;
        }
        
        Ok(())
    }
    
    /// Add a command to the history
    pub fn add_command(&mut self, command: String) -> Result<()> {
        // Don't add empty commands or duplicates of the last command
        if command.trim().is_empty() {
            return Ok(());
        }
        
        if let Some(last_entry) = self.entries.back() {
            if last_entry.command == command {
                return Ok(());
            }
        }
        
        // Add the new entry
        self.entries.push_back(HistoryEntry {
            command,
            timestamp: chrono::Local::now(),
        });
        
        // Remove the oldest entry if we've exceeded the limit
        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
        
        // Save to file
        self.save_to_file()?;
        
        Ok(())
    }
    
    /// Get the history entries
    pub fn entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }
    
    /// Get a command by index (0 is the most recent)
    pub fn get_command(&self, index: usize) -> Option<&HistoryEntry> {
        // Index 0 is the most recent command, which is at the back of the deque
        let len = self.entries.len();
        if index < len {
            self.entries.get(len - 1 - index)
        } else {
            None
        }
    }
    
    /// Search for commands containing the given substring
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.command.contains(query))
            .collect()
    }
    
    /// Clear the history
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save_to_file()?;
        Ok(())
    }
}