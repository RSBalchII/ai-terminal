#[cfg(test)]
mod tests {
    use terminal_ui::widgets::{Command, CommandPalette};
    
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
        assert!(palette.command_count() > 0);
        assert_eq!(palette.filtered_command_count(), palette.command_count());
        assert_eq!(palette.input(), "");
        assert_eq!(palette.selected_index(), 0);
    }
    
    #[test]
    fn test_filtering() {
        let mut palette = CommandPalette::new();
        assert_eq!(palette.filtered_command_count(), palette.command_count());
        
        palette.handle_input("clear");
        assert!(palette.filtered_command_count() <= palette.command_count());
        
        palette.reset();
        assert_eq!(palette.filtered_command_count(), palette.command_count());
    }
    
    #[test]
    fn test_selection_movement() {
        let mut palette = CommandPalette::new();
        palette.handle_input("test"); // This will likely result in 0 matches, but that's fine for the test
        
        let initial_index = palette.selected_index();
        palette.move_selection_down();
        assert_eq!(palette.selected_index(), (initial_index + 1) % palette.filtered_command_count());
        
        palette.move_selection_up();
        assert_eq!(palette.selected_index(), initial_index);
    }
}

#[cfg(test)]
mod confirmation_modal_tests {
    use terminal_ui::widgets::{ConfirmationModal, ModalButton};
    
    #[test]
    fn test_modal_button_creation() {
        let button = ModalButton::new("test", "Test Button", true);
        assert_eq!(button.id, "test");
        assert_eq!(button.text, "Test Button");
        assert_eq!(button.is_default, true);
    }

    #[test]
    fn test_confirmation_modal_creation() {
        let buttons = vec![
            ModalButton::new("yes", "Yes", true),
            ModalButton::new("no", "No", false),
        ];
        
        let modal = ConfirmationModal::new("Test Title", "Test Message", buttons.clone());
        assert_eq!(modal.title(), "Test Title");
        assert_eq!(modal.message(), "Test Message");
        assert_eq!(modal.buttons(), buttons.as_slice());
        assert_eq!(modal.selected_button(), 0);
    }

    #[test]
    fn test_yes_no_modal() {
        let modal = ConfirmationModal::yes_no("Confirm", "Are you sure?");
        assert_eq!(modal.title(), "Confirm");
        assert_eq!(modal.message(), "Are you sure?");
        assert_eq!(modal.buttons().len(), 2);
        assert_eq!(modal.buttons()[0].id, "yes");
        assert_eq!(modal.buttons()[1].id, "no");
    }

    #[test]
    fn test_button_selection() {
        let mut modal = ConfirmationModal::yes_no("Confirm", "Are you sure?");
        
        // Test next selection
        modal.select_next();
        assert_eq!(modal.selected_button(), 1);
        
        // Test wrapping around
        modal.select_next();
        assert_eq!(modal.selected_button(), 0);
        
        // Test previous selection
        modal.select_previous();
        assert_eq!(modal.selected_button(), 1);
        
        // Test wrapping around
        modal.select_previous();
        assert_eq!(modal.selected_button(), 0);
    }

    #[test]
    fn test_selected_button_id() {
        let modal = ConfirmationModal::yes_no("Confirm", "Are you sure?");
        assert_eq!(modal.selected_button_id(), "yes");
    }
}