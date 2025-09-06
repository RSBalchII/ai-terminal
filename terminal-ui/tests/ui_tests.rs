#[cfg(test)]
mod tests {
    use terminal_ui::{TerminalSession, AppMode};

    #[test]
    fn test_terminal_session_creation() {
        let session = TerminalSession::new();
        assert!(session.is_ok());
    }

    #[test]
    fn test_app_mode_variants() {
        let chat_mode = AppMode::Chat;
        let help_mode = AppMode::Help;
        
        assert_ne!(std::mem::discriminant(&chat_mode), std::mem::discriminant(&help_mode));
    }
}