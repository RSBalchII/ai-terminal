//! Layout management for the AI Terminal UI
//! 
//! This module provides functionality for managing the terminal layout,
//! including dynamic resizing and multi-pane arrangements.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

/// Manages the overall layout of the terminal UI
pub struct LayoutManager {
    terminal_size: Rect,
}

impl LayoutManager {
    /// Create a new layout manager with the given terminal size
    pub fn new(terminal_size: Rect) -> Self {
        Self { terminal_size }
    }
    
    /// Get the current terminal size
    pub fn terminal_size(&self) -> Rect {
        self.terminal_size
    }
    
    /// Update the layout based on the current terminal size
    pub fn update_size(&mut self, terminal_size: Rect) {
        self.terminal_size = terminal_size;
    }
    
    /// Calculate the main layout sections for the chat UI
    pub fn calculate_chat_layout(&self) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(1),    // Main content area
                Constraint::Length(3), // Input area
                Constraint::Length(1), // Status bar
            ])
            .split(self.terminal_size)
            .to_vec()
    }
    
    /// Calculate the layout for the main content area (messages)
    pub fn calculate_content_layout(&self, content_area: Rect) -> Vec<Rect> {
        // For now, we just return the content area as is
        // In the future, this could be split into multiple panes
        vec![content_area]
    }
    
    /// Calculate a centered rectangle for modal dialogs
    pub fn calculate_centered_rect(&self, percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);
            
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layout_manager_creation() {
        let rect = Rect::new(0, 0, 80, 24);
        let layout_manager = LayoutManager::new(rect);
        assert_eq!(layout_manager.terminal_size, rect);
    }
    
    #[test]
    fn test_layout_manager_update_size() {
        let rect1 = Rect::new(0, 0, 80, 24);
        let rect2 = Rect::new(0, 0, 120, 40);
        
        let mut layout_manager = LayoutManager::new(rect1);
        assert_eq!(layout_manager.terminal_size, rect1);
        
        layout_manager.update_size(rect2);
        assert_eq!(layout_manager.terminal_size, rect2);
    }
    
    #[test]
    fn test_calculate_chat_layout() {
        let rect = Rect::new(0, 0, 80, 24);
        let layout_manager = LayoutManager::new(rect);
        let layout = layout_manager.calculate_chat_layout();
        
        // Should have 4 sections: header, content, input, status
        assert_eq!(layout.len(), 4);
        
        // Header should be 1 line tall
        assert_eq!(layout[0].height, 1);
        
        // Input should be 3 lines tall
        assert_eq!(layout[2].height, 3);
        
        // Status should be 1 line tall
        assert_eq!(layout[3].height, 1);
    }
}