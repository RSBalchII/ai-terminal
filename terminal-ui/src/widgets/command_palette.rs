//! Command palette widget for the AI Terminal
//! 
//! This widget provides a searchable interface for accessing terminal commands
//! and features through a modal overlay.

use ratatui::{
    layout::Rect,
    style::{Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Clear},
    Frame,
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Represents a command in the palette
#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: String,
}

impl Command {
    /// Create a new command
    pub fn new(id: &str, name: &str, description: &str, category: &str, icon: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            icon: icon.to_string(),
        }
    }
}

/// Command palette widget
pub struct CommandPalette {
    commands: Vec<Command>,
    filtered_commands: Vec<Command>,
    input: String,
    selected_index: usize,
    matcher: SkimMatcherV2,
}

impl CommandPalette {
    /// Create a new command palette with default commands
    pub fn new() -> Self {
        let commands = vec![
            Command::new("new_session", "New Session", "Create a new AI session", "Session", "📝"),
            Command::new("clear_screen", "Clear Screen", "Clear the terminal screen", "View", "🧹"),
            Command::new("toggle_help", "Toggle Help", "Show/hide the help modal", "View", "❓"),
            Command::new("quit", "Quit", "Exit the application", "Session", "🚪"),
            Command::new("scroll_up", "Scroll Up", "Scroll the chat up by 5 lines", "Navigation", "⬆️"),
            Command::new("scroll_down", "Scroll Down", "Scroll the chat down by 5 lines", "Navigation", "⬇️"),
            Command::new("toggle_theme", "Toggle Theme", "Switch between light and dark themes", "View", "🎨"),
            Command::new("test_confirmation", "Test Confirmation", "Show a test confirmation modal", "Test", "✅"),
            Command::new("save_theme", "Save Theme", "Save the current theme to a file", "View", "💾"),
            Command::new("list_themes", "List Themes", "Show all available themes", "View", "📋"),
        ];
        
        let matcher = SkimMatcherV2::default();
        
        Self {
            commands: commands.clone(),
            filtered_commands: commands,
            input: String::new(),
            selected_index: 0,
            matcher,
        }
    }
    
    /// Get the number of commands
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
    
    /// Get the number of filtered commands
    pub fn filtered_command_count(&self) -> usize {
        self.filtered_commands.len()
    }
    
    /// Get the current input
    pub fn input(&self) -> &str {
        &self.input
    }
    
    /// Get the selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
    
    /// Update the filter based on user input
    pub fn update_filter(&mut self) {
        if self.input.is_empty() {
            self.filtered_commands = self.commands.clone();
        } else {
            let mut scored_commands: Vec<(i64, Command)> = self.commands
                .iter()
                .filter_map(|cmd| {
                    // Match against both name and description
                    let match_text = format!("{} {}", cmd.name, cmd.description);
                    self.matcher.fuzzy_match(&match_text, &self.input)
                        .map(|score| (score, cmd.clone()))
                })
                .collect();
                
            // Sort by score (highest first)
            scored_commands.sort_by(|a, b| b.0.cmp(&a.0));
            
            self.filtered_commands = scored_commands
                .into_iter()
                .map(|(_, cmd)| cmd)
                .collect();
        }
        
        // Reset selection to top
        self.selected_index = 0;
    }
    
    /// Handle user input
    pub fn handle_input(&mut self, input: &str) {
        self.input.push_str(input);
        self.update_filter();
    }
    
    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        self.input.pop();
        self.update_filter();
    }
    
    /// Move selection up
    pub fn move_selection_up(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_commands.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
    
    /// Move selection down
    pub fn move_selection_down(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_commands.len();
        }
    }
    
    /// Get the currently selected command
    pub fn get_selected_command(&self) -> Option<&Command> {
        self.filtered_commands.get(self.selected_index)
    }
    
    /// Render the command palette
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Clear the area behind the popup
        f.render_widget(Clear, area);
        
        // Create list items
        let items: Vec<ListItem> = self.filtered_commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let line = Line::from(vec![
                    Span::styled(cmd.icon.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(cmd.name.clone(), style),
                    Span::raw(" - "),
                    Span::styled(cmd.description.clone(), style),
                ]);
                
                ListItem::new(line)
            })
            .collect();
        
        // Create the list widget
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Command Palette ({})", self.input))
            );
        
        f.render_widget(list, area);
    }
    
    /// Reset the command palette
    pub fn reset(&mut self) {
        self.input.clear();
        self.filtered_commands = self.commands.clone();
        self.selected_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_creation() {
        let cmd = Command::new("test", "Test Command", "A test command", "Test", "🧪");
        assert_eq!(cmd.id, "test");
        assert_eq!(cmd.name, "Test Command");
        assert_eq!(cmd.description, "A test command");
        assert_eq!(cmd.category, "Test");
        assert_eq!(cmd.icon, "🧪");
    }
    
    #[test]
    fn test_command_palette_creation() {
        let palette = CommandPalette::new();
        assert!(!palette.commands.is_empty());
        assert_eq!(palette.filtered_commands.len(), palette.commands.len());
        assert_eq!(palette.input, "");
        assert_eq!(palette.selected_index, 0);
    }
    
    #[test]
    fn test_filtering() {
        let mut palette = CommandPalette::new();
        assert_eq!(palette.filtered_commands.len(), palette.commands.len());
        
        palette.handle_input("clear");
        assert!(palette.filtered_commands.len() <= palette.commands.len());
        
        palette.reset();
        assert_eq!(palette.filtered_commands.len(), palette.commands.len());
    }
}