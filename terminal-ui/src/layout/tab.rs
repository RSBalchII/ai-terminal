//! Tab management for the AI Terminal UI
//!
//! This module provides functionality for managing multiple tabs within the terminal,
//! including creation, switching, and closing tabs.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Color},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use std::collections::HashMap;

/// Represents a tab in the terminal UI
#[derive(Debug, Clone)]
pub struct Tab {
    /// Unique identifier for the tab
    pub id: usize,
    /// Name of the tab
    pub name: String,
    /// Index of the tab (for ordering)
    pub index: usize,
}

impl Tab {
    /// Create a new tab
    pub fn new(id: usize, name: String, index: usize) -> Self {
        Self { id, name, index }
    }
}

/// Manages multiple tabs in the terminal UI
#[derive(Debug)]
pub struct TabManager {
    /// List of tabs
    tabs: HashMap<usize, Tab>,
    /// ID of the currently active tab
    active_tab_id: Option<usize>,
    /// Next ID to assign to a new tab
    next_id: usize,
    /// Next index to assign to a new tab
    next_index: usize,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new() -> Self {
        let mut tabs = HashMap::new();
        let initial_tab = Tab::new(0, "Tab 1".to_string(), 0);
        tabs.insert(0, initial_tab);
        
        Self {
            tabs,
            active_tab_id: Some(0),
            next_id: 1,
            next_index: 1,
        }
    }

    /// Get the currently active tab
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab_id.and_then(|id| self.tabs.get(&id))
    }

    /// Get the currently active tab ID
    pub fn active_tab_id(&self) -> Option<usize> {
        self.active_tab_id
    }

    /// Create a new tab
    pub fn create_tab(&mut self, name: Option<String>) -> usize {
        let tab_id = self.next_id;
        self.next_id += 1;
        
        let tab_name = name.unwrap_or_else(|| format!("Tab {}", self.next_index + 1));
        let tab = Tab::new(tab_id, tab_name, self.next_index);
        self.next_index += 1;
        
        self.tabs.insert(tab_id, tab);
        self.active_tab_id = Some(tab_id);
        
        tab_id
    }

    /// Switch to a specific tab by ID
    pub fn switch_to_tab(&mut self, tab_id: usize) -> Result<(), &'static str> {
        if self.tabs.contains_key(&tab_id) {
            self.active_tab_id = Some(tab_id);
            Ok(())
        } else {
            Err("Tab not found")
        }
    }

    /// Switch to the next tab
    pub fn switch_to_next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        
        let current_id = self.active_tab_id.unwrap_or(0);
        let current_index = self.tabs.get(&current_id).map(|tab| tab.index).unwrap_or(0);
        
        // Find the next tab by index
        let next_index = (current_index + 1) % self.tabs.len();
        let next_tab_id = self.tabs
            .values()
            .find(|tab| tab.index == next_index)
            .map(|tab| tab.id);
        
        if let Some(id) = next_tab_id {
            self.active_tab_id = Some(id);
        }
    }

    /// Switch to the previous tab
    pub fn switch_to_prev_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        
        let current_id = self.active_tab_id.unwrap_or(0);
        let current_index = self.tabs.get(&current_id).map(|tab| tab.index).unwrap_or(0);
        
        // Find the previous tab by index
        let prev_index = if current_index > 0 {
            current_index - 1
        } else {
            self.tabs.len() - 1
        };
        
        let prev_tab_id = self.tabs
            .values()
            .find(|tab| tab.index == prev_index)
            .map(|tab| tab.id);
        
        if let Some(id) = prev_tab_id {
            self.active_tab_id = Some(id);
        }
    }

    /// Close a specific tab by ID
    pub fn close_tab(&mut self, tab_id: usize) -> Result<(), &'static str> {
        if self.tabs.len() <= 1 {
            return Err("Cannot close the last tab");
        }
        
        if !self.tabs.contains_key(&tab_id) {
            return Err("Tab not found");
        }
        
        // Remove the tab
        self.tabs.remove(&tab_id);
        
        // If we closed the active tab, switch to the previous tab
        if self.active_tab_id == Some(tab_id) {
            let prev_tab_id = self.tabs.keys().next().cloned().unwrap_or(0);
            self.active_tab_id = Some(prev_tab_id);
        }
        
        Ok(())
    }

    /// Rename a tab
    pub fn rename_tab(&mut self, tab_id: usize, new_name: String) -> Result<(), &'static str> {
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.name = new_name;
            Ok(())
        } else {
            Err("Tab not found")
        }
    }

    /// Get a list of all tabs, sorted by index
    pub fn tabs(&self) -> Vec<&Tab> {
        let mut tabs: Vec<&Tab> = self.tabs.values().collect();
        tabs.sort_by_key(|tab| tab.index);
        tabs
    }

    /// Render the tab bar
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let tabs: Vec<String> = self.tabs().iter().map(|tab| tab.name.clone()).collect();
        let active_index = self.active_tab()
            .and_then(|active_tab| {
                self.tabs().iter().position(|tab| tab.id == active_tab.id)
            })
            .unwrap_or(0);
        
        let tabs_widget = Tabs::new(tabs)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(active_index);
        
        f.render_widget(tabs_widget, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(1, "Test Tab".to_string(), 0);
        
        assert_eq!(tab.id, 1);
        assert_eq!(tab.name, "Test Tab");
        assert_eq!(tab.index, 0);
    }

    #[test]
    fn test_tab_manager_creation() {
        let tab_manager = TabManager::new();
        
        assert_eq!(tab_manager.tabs.len(), 1);
        assert_eq!(tab_manager.active_tab_id, Some(0));
        assert_eq!(tab_manager.next_id, 1);
        assert_eq!(tab_manager.next_index, 1);
    }

    #[test]
    fn test_create_tab() {
        let mut tab_manager = TabManager::new();
        
        let tab_id = tab_manager.create_tab(Some("New Tab".to_string()));
        assert_eq!(tab_manager.tabs.len(), 2);
        assert_eq!(tab_manager.active_tab_id, Some(tab_id));
        
        let tab = tab_manager.tabs.get(&tab_id).unwrap();
        assert_eq!(tab.name, "New Tab");
        assert_eq!(tab.index, 1);
    }

    #[test]
    fn test_switch_tabs() {
        let mut tab_manager = TabManager::new();
        let tab_id = tab_manager.create_tab(Some("New Tab".to_string()));
        
        // Switch to the new tab
        assert!(tab_manager.switch_to_tab(tab_id).is_ok());
        assert_eq!(tab_manager.active_tab_id, Some(tab_id));
        
        // Try to switch to a non-existent tab
        assert!(tab_manager.switch_to_tab(999).is_err());
    }

    #[test]
    fn test_close_tab() {
        let mut tab_manager = TabManager::new();
        let tab_id = tab_manager.create_tab(Some("New Tab".to_string()));
        
        // Close the new tab
        assert!(tab_manager.close_tab(tab_id).is_ok());
        assert_eq!(tab_manager.tabs.len(), 1);
        assert_eq!(tab_manager.active_tab_id, Some(0)); // Should switch to the remaining tab
        
        // Try to close the last tab (should fail)
        assert!(tab_manager.close_tab(0).is_err());
    }

    #[test]
    fn test_rename_tab() {
        let mut tab_manager = TabManager::new();
        let tab_id = tab_manager.create_tab(Some("New Tab".to_string()));
        
        // Rename the tab
        assert!(tab_manager.rename_tab(tab_id, "Renamed Tab".to_string()).is_ok());
        
        let tab = tab_manager.tabs.get(&tab_id).unwrap();
        assert_eq!(tab.name, "Renamed Tab");
        
        // Try to rename a non-existent tab
        assert!(tab_manager.rename_tab(999, "Non-existent".to_string()).is_err());
    }
}