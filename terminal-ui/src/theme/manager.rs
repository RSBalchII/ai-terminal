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
    SerializeError(toml::ser::Error),
    ThemeNotFound(String),
    InvalidColor(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::IoError(e) => write!(f, "IO error: {}", e),
            ThemeError::ParseError(e) => write!(f, "Parse error: {}", e),
            ThemeError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            ThemeError::ThemeNotFound(name) => write!(f, "Theme not found: {}", name),
            ThemeError::InvalidColor(color) => write!(f, "Invalid color: {}", color),
        }
    }
}

impl std::error::Error for ThemeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ThemeError::IoError(e) => Some(e),
            ThemeError::ParseError(e) => Some(e),
            ThemeError::SerializeError(e) => Some(e),
            ThemeError::ThemeNotFound(_) => None,
            ThemeError::InvalidColor(_) => None,
        }
    }
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
    config_dir: Option<String>,
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
        
        // Try to get config directory
        let config_dir = dirs::config_dir()
            .map(|path| path.join("ai-terminal"))
            .and_then(|path| path.to_str().map(|s| s.to_string()));
        
        Self {
            current_theme: default_theme,
            available_themes: themes,
            config_dir,
        }
    }
    
    /// Load themes from the config directory
    pub fn load_user_themes(&mut self) -> Result<(), ThemeError> {
        if let Some(config_dir) = &self.config_dir {
            let themes_dir = Path::new(config_dir).join("themes");
            
            // Create themes directory if it doesn't exist
            if !themes_dir.exists() {
                fs::create_dir_all(&themes_dir)?;
            }
            
            // Load all .toml files in the themes directory
            if themes_dir.exists() {
                for entry in fs::read_dir(&themes_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.extension().map_or(false, |ext| ext == "toml") {
                        self.load_from_file(path.to_str().unwrap_or(""))?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Load a theme from a TOML configuration file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), ThemeError> {
        let content = fs::read_to_string(path)?;
        let config: ThemeConfig = toml::from_str(&content)?;
        
        // Convert string colors to ratatui colors
        let theme = Theme {
            name: config.name,
            primary: Self::parse_color(&config.primary)?,
            secondary: Self::parse_color(&config.secondary)?,
            background: Self::parse_color(&config.background)?,
            text: Self::parse_color(&config.text)?,
            accent: Self::parse_color(&config.accent)?,
            error: Self::parse_color(&config.error)?,
            success: Self::parse_color(&config.success)?,
            warning: Self::parse_color(&config.warning)?,
            command: Self::parse_color(&config.command)?,
            ai_response: Self::parse_color(&config.ai_response)?,
        };
        
        self.available_themes.insert(theme.name.clone(), theme.clone());
        self.current_theme = theme;
        
        Ok(())
    }
    
    /// Save the current theme to a TOML configuration file
    pub fn save_current_theme(&self, name: &str) -> Result<(), ThemeError> {
        if let Some(config_dir) = &self.config_dir {
            let themes_dir = Path::new(config_dir).join("themes");
            
            // Create themes directory if it doesn't exist
            fs::create_dir_all(&themes_dir)?;
            
            // Create theme config
            let config = ThemeConfig {
                name: name.to_string(),
                primary: Self::color_to_string(self.current_theme.primary),
                secondary: Self::color_to_string(self.current_theme.secondary),
                background: Self::color_to_string(self.current_theme.background),
                text: Self::color_to_string(self.current_theme.text),
                accent: Self::color_to_string(self.current_theme.accent),
                error: Self::color_to_string(self.current_theme.error),
                success: Self::color_to_string(self.current_theme.success),
                warning: Self::color_to_string(self.current_theme.warning),
                command: Self::color_to_string(self.current_theme.command),
                ai_response: Self::color_to_string(self.current_theme.ai_response),
            };
            
            // Serialize to TOML
            let toml_string = toml::to_string(&config)
                .map_err(|e| ThemeError::SerializeError(e))?;
            
            // Write to file
            let file_path = themes_dir.join(format!("{}.toml", name));
            fs::write(file_path, toml_string)?;
            
            Ok(())
        } else {
            Err(ThemeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Config directory not found",
            )))
        }
    }
    
    /// Parse a color string into a ratatui Color
    fn parse_color(color_str: &str) -> Result<ratatui::style::Color, ThemeError> {
        match color_str.to_lowercase().as_str() {
            "black" => Ok(ratatui::style::Color::Black),
            "red" => Ok(ratatui::style::Color::Red),
            "green" => Ok(ratatui::style::Color::Green),
            "yellow" => Ok(ratatui::style::Color::Yellow),
            "blue" => Ok(ratatui::style::Color::Blue),
            "magenta" => Ok(ratatui::style::Color::Magenta),
            "cyan" => Ok(ratatui::style::Color::Cyan),
            "gray" => Ok(ratatui::style::Color::Gray),
            "darkgray" => Ok(ratatui::style::Color::DarkGray),
            "lightred" => Ok(ratatui::style::Color::LightRed),
            "lightgreen" => Ok(ratatui::style::Color::LightGreen),
            "lightyellow" => Ok(ratatui::style::Color::LightYellow),
            "lightblue" => Ok(ratatui::style::Color::LightBlue),
            "lightmagenta" => Ok(ratatui::style::Color::LightMagenta),
            "lightcyan" => Ok(ratatui::style::Color::LightCyan),
            "white" => Ok(ratatui::style::Color::White),
            _ => {
                // Try to parse as RGB hex color (#RRGGBB)
                if color_str.starts_with('#') && color_str.len() == 7 {
                    let hex = &color_str[1..];
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        return Ok(ratatui::style::Color::Rgb(r, g, b));
                    }
                }
                Err(ThemeError::InvalidColor(color_str.to_string()))
            }
        }
    }
    
    /// Convert a ratatui Color to a string representation
    fn color_to_string(color: ratatui::style::Color) -> String {
        match color {
            ratatui::style::Color::Black => "black".to_string(),
            ratatui::style::Color::Red => "red".to_string(),
            ratatui::style::Color::Green => "green".to_string(),
            ratatui::style::Color::Yellow => "yellow".to_string(),
            ratatui::style::Color::Blue => "blue".to_string(),
            ratatui::style::Color::Magenta => "magenta".to_string(),
            ratatui::style::Color::Cyan => "cyan".to_string(),
            ratatui::style::Color::Gray => "gray".to_string(),
            ratatui::style::Color::DarkGray => "darkgray".to_string(),
            ratatui::style::Color::LightRed => "lightred".to_string(),
            ratatui::style::Color::LightGreen => "lightgreen".to_string(),
            ratatui::style::Color::LightYellow => "lightyellow".to_string(),
            ratatui::style::Color::LightBlue => "lightblue".to_string(),
            ratatui::style::Color::LightMagenta => "lightmagenta".to_string(),
            ratatui::style::Color::LightCyan => "lightcyan".to_string(),
            ratatui::style::Color::White => "white".to_string(),
            ratatui::style::Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            _ => "white".to_string(), // Default to white for unknown colors
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
    
    /// Create a new theme with custom colors
    pub fn create_theme(&mut self, name: &str, theme: Theme) {
        self.available_themes.insert(name.to_string(), theme);
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