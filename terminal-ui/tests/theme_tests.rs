#[cfg(test)]
mod tests {
    use terminal_ui::theme::presets::Theme;
    
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
    fn test_high_contrast_theme() {
        let theme = Theme::high_contrast();
        assert_eq!(theme.name, "high_contrast");
        assert_eq!(theme.background, ratatui::style::Color::White);
        assert_eq!(theme.text, ratatui::style::Color::Black);
    }
}