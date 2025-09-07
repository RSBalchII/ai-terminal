#[cfg(test)]
mod tests {
    use terminal_ui::layout::pane::{PaneManager, SplitOrientation};
    use ratatui::layout::Rect;

    #[test]
    fn test_pane_manager_creation() {
        let rect = Rect::new(0, 0, 80, 24);
        let pane_manager = PaneManager::new(rect);
        
        assert_eq!(pane_manager.panes.len(), 1);
        assert_eq!(pane_manager.focused_pane_id, Some(0));
        assert_eq!(pane_manager.next_id, 1);
    }

    #[test]
    fn test_split_focused_pane() {
        let rect = Rect::new(0, 0, 80, 24);
        let mut pane_manager = PaneManager::new(rect);
        
        // Split the focused pane horizontally
        assert!(pane_manager.split_focused_pane(SplitOrientation::Horizontal).is_ok());
        assert_eq!(pane_manager.panes.len(), 2);
        assert_eq!(pane_manager.focused_pane_id, Some(1));
        
        // Split the focused pane vertically
        assert!(pane_manager.split_focused_pane(SplitOrientation::Vertical).is_ok());
        assert_eq!(pane_manager.panes.len(), 3);
        assert_eq!(pane_manager.focused_pane_id, Some(2));
    }

    #[test]
    fn test_close_focused_pane() {
        let rect = Rect::new(0, 0, 80, 24);
        let mut pane_manager = PaneManager::new(rect);
        
        // Split to create a second pane
        assert!(pane_manager.split_focused_pane(SplitOrientation::Horizontal).is_ok());
        assert_eq!(pane_manager.panes.len(), 2);
        
        // Close the focused pane
        assert!(pane_manager.close_focused_pane().is_ok());
        assert_eq!(pane_manager.panes.len(), 1);
        assert_eq!(pane_manager.focused_pane_id, Some(0));
    }

    #[test]
    fn test_focus_navigation() {
        let rect = Rect::new(0, 0, 80, 24);
        let mut pane_manager = PaneManager::new(rect);
        
        // Split to create a second pane
        assert!(pane_manager.split_focused_pane(SplitOrientation::Horizontal).is_ok());
        assert_eq!(pane_manager.focused_pane_id, Some(1));
        
        // Focus the next pane (should wrap around to the first)
        pane_manager.focus_next_pane();
        assert_eq!(pane_manager.focused_pane_id, Some(0));
        
        // Focus the previous pane (should wrap around to the second)
        pane_manager.focus_prev_pane();
        assert_eq!(pane_manager.focused_pane_id, Some(1));
    }
}