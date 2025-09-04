use crate::command_block::{CommandBlock, BlockState};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;
use tracing::{debug, info};

/// Manages the history of command blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    /// All command blocks in chronological order
    blocks: VecDeque<CommandBlock>,
    
    /// Currently selected block index
    selected_index: Option<usize>,
    
    /// Maximum number of blocks to keep in memory
    max_blocks: usize,
    
    /// Current working directory
    current_dir: String,
}

impl CommandHistory {
    /// Create a new command history
    pub fn new(max_blocks: usize) -> Self {
        let current_dir = std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
            
        Self {
            blocks: VecDeque::new(),
            selected_index: None,
            max_blocks,
            current_dir,
        }
    }
    
    /// Add a new command block
    pub fn add_block(&mut self, mut block: CommandBlock) -> Uuid {
        let id = block.id;
        
        // Deselect all other blocks
        for b in &mut self.blocks {
            b.is_selected = false;
        }
        
        // Select the new block
        block.is_selected = true;
        
        // Add the block
        self.blocks.push_back(block);
        self.selected_index = Some(self.blocks.len() - 1);
        
        // Prune old blocks if necessary
        while self.blocks.len() > self.max_blocks {
            self.blocks.pop_front();
            // Adjust selected index
            if let Some(idx) = self.selected_index {
                self.selected_index = Some(idx.saturating_sub(1));
            }
        }
        
        info!("Added command block {}", id);
        id
    }
    
    /// Get a block by ID
    pub fn get_block(&self, id: &Uuid) -> Option<&CommandBlock> {
        self.blocks.iter().find(|b| &b.id == id)
    }
    
    /// Get a mutable block by ID
    pub fn get_block_mut(&mut self, id: &Uuid) -> Option<&mut CommandBlock> {
        self.blocks.iter_mut().find(|b| &b.id == id)
    }
    
    /// Get the currently selected block
    pub fn selected_block(&self) -> Option<&CommandBlock> {
        self.selected_index.and_then(|idx| self.blocks.get(idx))
    }
    
    /// Get the currently selected block mutably
    pub fn selected_block_mut(&mut self) -> Option<&mut CommandBlock> {
        self.selected_index.and_then(move |idx| self.blocks.get_mut(idx))
    }
    
    /// Navigate to the previous block
    pub fn select_previous(&mut self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }
        
        let new_index = match self.selected_index {
            Some(idx) if idx > 0 => idx - 1,
            Some(_) => 0,  // Covers idx == 0 and any other case
            None => self.blocks.len() - 1,
        };
        
        self.select_index(new_index)
    }
    
    /// Navigate to the next block
    pub fn select_next(&mut self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }
        
        let new_index = match self.selected_index {
            Some(idx) if idx < self.blocks.len() - 1 => idx + 1,
            Some(idx) => idx,
            None => 0,
        };
        
        self.select_index(new_index)
    }
    
    /// Select a block by index
    pub fn select_index(&mut self, index: usize) -> bool {
        if index >= self.blocks.len() {
            return false;
        }
        
        // Deselect all blocks
        for block in &mut self.blocks {
            block.is_selected = false;
        }
        
        // Select the target block
        if let Some(block) = self.blocks.get_mut(index) {
            block.is_selected = true;
            self.selected_index = Some(index);
            debug!("Selected block at index {}", index);
            true
        } else {
            false
        }
    }
    
    /// Select a block by ID
    pub fn select_block(&mut self, id: &Uuid) -> bool {
        if let Some(index) = self.blocks.iter().position(|b| &b.id == id) {
            self.select_index(index)
        } else {
            false
        }
    }
    
    /// Get all blocks
    pub fn blocks(&self) -> &VecDeque<CommandBlock> {
        &self.blocks
    }
    
    /// Get all blocks mutably
    pub fn blocks_mut(&mut self) -> &mut VecDeque<CommandBlock> {
        &mut self.blocks
    }
    
    /// Clear all blocks
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.selected_index = None;
        info!("Cleared command history");
    }
    
    /// Remove a block by ID
    pub fn remove_block(&mut self, id: &Uuid) -> Option<CommandBlock> {
        if let Some(index) = self.blocks.iter().position(|b| &b.id == id) {
            // Adjust selected index if necessary
            if let Some(selected) = self.selected_index {
                if index < selected {
                    self.selected_index = Some(selected - 1);
                } else if index == selected {
                    self.selected_index = None;
                }
            }
            
            info!("Removed block {}", id);
            Some(self.blocks.remove(index)?)
        } else {
            None
        }
    }
    
    /// Get blocks that are currently running
    pub fn running_blocks(&self) -> Vec<&CommandBlock> {
        self.blocks
            .iter()
            .filter(|b| b.state == BlockState::Running)
            .collect()
    }
    
    /// Get blocks that failed
    pub fn failed_blocks(&self) -> Vec<&CommandBlock> {
        self.blocks
            .iter()
            .filter(|b| b.state == BlockState::Failed)
            .collect()
    }
    
    /// Get blocks that succeeded
    pub fn successful_blocks(&self) -> Vec<&CommandBlock> {
        self.blocks
            .iter()
            .filter(|b| b.state == BlockState::Success)
            .collect()
    }
    
    /// Search blocks by command text
    pub fn search(&self, query: &str) -> Vec<&CommandBlock> {
        let query_lower = query.to_lowercase();
        self.blocks
            .iter()
            .filter(|b| b.command.to_lowercase().contains(&query_lower))
            .collect()
    }
    
    /// Get the last N blocks
    pub fn last_n(&self, n: usize) -> Vec<&CommandBlock> {
        let start = self.blocks.len().saturating_sub(n);
        self.blocks.iter().skip(start).collect()
    }
    
    /// Get statistics about the command history
    pub fn stats(&self) -> HistoryStats {
        let total = self.blocks.len();
        let running = self.running_blocks().len();
        let successful = self.successful_blocks().len();
        let failed = self.failed_blocks().len();
        
        let avg_duration = {
            let durations: Vec<_> = self.blocks
                .iter()
                .filter_map(|b| b.duration)
                .collect();
                
            if durations.is_empty() {
                None
            } else {
                let sum: std::time::Duration = durations.iter().sum();
                Some(sum / durations.len() as u32)
            }
        };
        
        HistoryStats {
            total_blocks: total,
            running: running,
            successful: successful,
            failed: failed,
            average_duration: avg_duration,
        }
    }
    
    /// Update current working directory
    pub fn set_current_dir(&mut self, dir: String) {
        self.current_dir = dir;
    }
    
    /// Get current working directory
    pub fn current_dir(&self) -> &str {
        &self.current_dir
    }
}

/// Statistics about the command history
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_blocks: usize,
    pub running: usize,
    pub successful: usize,
    pub failed: usize,
    pub average_duration: Option<std::time::Duration>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(10000) // Default to 10k blocks
    }
}
