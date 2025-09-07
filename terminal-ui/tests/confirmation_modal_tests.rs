#[cfg(test)]
mod tests {
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