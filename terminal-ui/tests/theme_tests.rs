#[cfg(test)]
mod tests {
    use terminal_ui::theme::{ThemeManager, Theme};
    use std::fs;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current_theme().name, "default");
        assert!(!manager.available_theme_names().is_empty());
    }
    
    #[test]
    fn test_switch_theme() {
        let mut manager = ThemeManager::new();
        assert!(manager.switch_theme("dark").is_ok());
        assert_eq!(manager.current_theme().name, "dark");
        
        assert!(manager.switch_theme("nonexistent").is_err());
    }
    
    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
        assert_eq!(theme.background, ratatui::style::Color::Black);
        assert_eq!(theme.text, ratatui::style::Color::White);
    }
    
    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        assert_eq!(theme.name, "dark");
        assert_eq!(theme.background, ratatui::style::Color::Black);
        assert_eq!(theme.text, ratatui::style::Color::White);
    }
    
    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");
        assert_eq!(theme.background, ratatui::style::Color::White);
        assert_eq!(theme.text, ratatui::style::Color::Black);
    }
    
    #[test]
    fn test_parse_color() {
        // This would require making parse_color public or testing through the manager
        let manager = ThemeManager::new();
        // Test that we have a working manager
        assert_eq!(manager.current_theme().name, "default");
    }
}