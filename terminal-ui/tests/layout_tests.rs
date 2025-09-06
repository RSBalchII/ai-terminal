#[cfg(test)]
mod tests {
    use terminal_ui::layout::{LayoutManager};
    use ratatui::layout::Rect;

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
        
        // Status should be 1 line tall
        assert_eq!(layout[3].height, 1);
    }
    
    #[test]
    fn test_calculate_centered_rect() {
        let rect = Rect::new(0, 0, 100, 50);
        let layout_manager = LayoutManager::new(rect);
        let centered = layout_manager.calculate_centered_rect(50, 50, rect);
        
        // Should be centered
        assert!(centered.x > 0);
        assert!(centered.y > 0);
        assert!(centered.width < rect.width);
        assert!(centered.height < rect.height);
    }
}