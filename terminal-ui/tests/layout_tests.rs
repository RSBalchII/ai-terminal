#[cfg(test)]
mod tests {
    use ratatui::prelude::Rect;
    use terminal_ui::layout::LayoutManager;
    
    #[test]
    fn test_layout_manager_creation() {
        let rect = Rect::new(0, 0, 80, 24);
        let layout_manager = LayoutManager::new(rect);
        assert_eq!(layout_manager.terminal_size(), rect);
    }
    
    #[test]
    fn test_layout_manager_update_size() {
        let rect1 = Rect::new(0, 0, 80, 24);
        let rect2 = Rect::new(0, 0, 120, 40);
        
        let mut layout_manager = LayoutManager::new(rect1);
        assert_eq!(layout_manager.terminal_size(), rect1);
        
        layout_manager.update_size(rect2);
        assert_eq!(layout_manager.terminal_size(), rect2);
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