//! Theme manager for the AI Terminal

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::theme::presets::Theme;

/// Error types for theme operations
#[derive(Debug)]
pub enum ThemeError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    ThemeNotFound(String),
}

impl From<std::io::Error> for ThemeError {
    fn from(error: std::io::Error) -> Self {
        ThemeError::IoError(error)
    }
}

impl From<toml::de::Error> for ThemeError {
    fn from(error: toml::de::Error) -> Self {
        ThemeError::ParseError(error)
    }
}

/// Theme configuration structure for TOML parsing
#[derive(Debug, Deserialize, Serialize)]
struct ThemeConfig {
    name: String,
    primary: String,
    secondary: String,
    background: String,
    text: String,
    accent: String,
    error: String,
    success: String,
    warning: String,
    command: String,
    ai_response: String,
}

/// Manages themes for the terminal UI
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: HashMap<String, Theme>,
}

impl ThemeManager {
    /// Create a new theme manager with default themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        
        let default_theme = Theme::default();
        let dark_theme = Theme::dark();
        let light_theme = Theme::light();
        let high_contrast_theme = Theme::high_contrast();
        
        themes.insert(default_theme.name.clone(), default_theme.clone());
        themes.insert(dark_theme.name.clone(), dark_theme.clone());
        themes.insert(light_theme.name.clone(), light_theme.clone());
        themes.insert(high_contrast_theme.name.clone(), high_contrast_theme.clone());
        
        Self {
            current_theme: default_theme,
            available_themes: themes,
        }
    }
    
    /// Load a theme from a TOML configuration file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), ThemeError> {
        let content = fs::read_to_string(path)?;
        let config: ThemeConfig = toml::from_str(&content)?;
        
        // Convert string colors to ratatui colors (simplified for now)
        let theme = Theme {
            name: config.name,
            primary: Self::parse_color(&config.primary),
            secondary: Self::parse_color(&config.secondary),
            background: Self::parse_color(&config.background),
            text: Self::parse_color(&config.text),
            accent: Self::parse_color(&config.accent),
            error: Self::parse_color(&config.error),
            success: Self::parse_color(&config.success),
            warning: Self::parse_color(&config.warning),
            command: Self::parse_color(&config.command),
            ai_response: Self::parse_color(&config.ai_response),
        };
        
        self.available_themes.insert(theme.name.clone(), theme.clone());
        self.current_theme = theme;
        
        Ok(())
    }
    
    /// Parse a color string into a ratatui Color
    fn parse_color(color_str: &str) -> ratatui::style::Color {
        match color_str.to_lowercase().as_str() {
            "black" => ratatui::style::Color::Black,
            "red" => ratatui::style::Color::Red,
            "green" => ratatui::style::Color::Green,
            "yellow" => ratatui::style::Color::Yellow,
            "blue" => ratatui::style::Color::Blue,
            "magenta" => ratatui::style::Color::Magenta,
            "cyan" => ratatui::style::Color::Cyan,
            "gray" => ratatui::style::Color::Gray,
            "darkgray" => ratatui::style::Color::DarkGray,
            "lightred" => ratatui::style::Color::LightRed,
            "lightgreen" => ratatui::style::Color::LightGreen,
            "lightyellow" => ratatui::style::Color::LightYellow,
            "lightblue" => ratatui::style::Color::LightBlue,
            "lightmagenta" => ratatui::style::Color::LightMagenta,
            "lightcyan" => ratatui::style::Color::LightCyan,
            "white" => ratatui::style::Color::White,
            _ => ratatui::style::Color::White, // Default to white if unknown
        }
    }
    
    /// Get the current theme
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }
    
    /// Switch to a different theme
    pub fn switch_theme(&mut self, theme_name: &str) -> Result<(), ThemeError> {
        if let Some(theme) = self.available_themes.get(theme_name) {
            self.current_theme = theme.clone();
            Ok(())
        } else {
            Err(ThemeError::ThemeNotFound(theme_name.to_string()))
        }
    }
    
    /// Get a list of available theme names
    pub fn available_theme_names(&self) -> Vec<&String> {
        self.available_themes.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
}