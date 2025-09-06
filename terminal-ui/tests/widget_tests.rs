#[cfg(test)]
mod tests {
    use terminal_ui::widgets::{CommandPalette, Command};

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
    
    #[test]
    fn test_selection_navigation() {
        let mut palette = CommandPalette::new();
        let initial_index = palette.selected_index;
        
        palette.move_selection_down();
        assert_eq!(palette.selected_index, (initial_index + 1) % palette.filtered_commands.len());
        
        palette.move_selection_up();
        assert_eq!(palette.selected_index, initial_index);
    }
    
    #[test]
    fn test_get_selected_command() {
        let palette = CommandPalette::new();
        let selected = palette.get_selected_command();
        assert!(selected.is_some());
    }
}