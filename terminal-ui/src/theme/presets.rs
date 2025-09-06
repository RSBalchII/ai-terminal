//! Predefined themes for the AI Terminal

use ratatui::style::Color;

/// Represents a color theme for the terminal UI
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub text: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub command: Color,
    pub ai_response: Color,
}

impl Theme {
    /// Create the default theme
    pub fn default() -> Self {
        Self {
            name: "default".to_string(),
            primary: Color::Blue,
            secondary: Color::Gray,
            background: Color::Black,
            text: Color::White,
            accent: Color::Cyan,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            command: Color::Cyan,
            ai_response: Color::Green,
        }
    }
    
    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            primary: Color::Blue,
            secondary: Color::DarkGray,
            background: Color::Black,
            text: Color::White,
            accent: Color::Cyan,
            error: Color::LightRed,
            success: Color::LightGreen,
            warning: Color::LightYellow,
            command: Color::Cyan,
            ai_response: Color::LightGreen,
        }
    }
    
    /// Create a light theme
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            primary: Color::Blue,
            secondary: Color::Gray,
            background: Color::White,
            text: Color::Black,
            accent: Color::Cyan,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            command: Color::Blue,
            ai_response: Color::Green,
        }
    }
    
    /// Create a high contrast theme
    pub fn high_contrast() -> Self {
        Self {
            name: "high_contrast".to_string(),
            primary: Color::Blue,
            secondary: Color::Gray,
            background: Color::White,
            text: Color::Black,
            accent: Color::Cyan,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            command: Color::Magenta,
            ai_response: Color::Blue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
        assert_eq!(theme.background, Color::Black);
        assert_eq!(theme.text, Color::White);
    }
    
    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        assert_eq!(theme.name, "dark");
        assert_eq!(theme.background, Color::Black);
        assert_eq!(theme.text, Color::White);
    }
    
    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");
        assert_eq!(theme.background, Color::White);
        assert_eq!(theme.text, Color::Black);
    }
    
    #[test]
    fn test_high_contrast_theme() {
        let theme = Theme::high_contrast();
        assert_eq!(theme.name, "high_contrast");
        assert_eq!(theme.background, Color::White);
        assert_eq!(theme.text, Color::Black);
    }
}