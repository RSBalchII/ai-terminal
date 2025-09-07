//! Confirmation modal widget for the AI Terminal
//!
//! This widget provides a modal dialog for confirming user actions
//! with customizable buttons and messages.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Color, Modifier},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Represents a button in the confirmation modal
#[derive(Debug, Clone, PartialEq)]
pub struct ModalButton {
    pub id: String,
    pub text: String,
    pub is_default: bool,
}

impl ModalButton {
    /// Create a new modal button
    pub fn new(id: &str, text: &str, is_default: bool) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            is_default,
        }
    }
}

/// Confirmation modal widget
pub struct ConfirmationModal {
    title: String,
    message: String,
    buttons: Vec<ModalButton>,
    selected_button: usize,
}

impl ConfirmationModal {
    /// Create a new confirmation modal
    pub fn new(title: &str, message: &str, buttons: Vec<ModalButton>) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            buttons,
            selected_button: 0,
        }
    }

    /// Create a simple Yes/No confirmation modal
    pub fn yes_no(title: &str, message: &str) -> Self {
        let buttons = vec![
            ModalButton::new("yes", "Yes", true),
            ModalButton::new("no", "No", false),
        ];
        
        Self::new(title, message, buttons)
    }

    /// Select the next button
    pub fn select_next(&mut self) {
        self.selected_button = (self.selected_button + 1) % self.buttons.len();
    }

    /// Select the previous button
    pub fn select_previous(&mut self) {
        if self.selected_button > 0 {
            self.selected_button -= 1;
        } else {
            self.selected_button = self.buttons.len() - 1;
        }
    }

    /// Get the ID of the selected button
    pub fn selected_button_id(&self) -> &str {
        &self.buttons[self.selected_button].id
    }

    /// Get the title of the modal
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the message of the modal
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the buttons of the modal
    pub fn buttons(&self) -> &[ModalButton] {
        &self.buttons
    }

    /// Get the selected button index
    pub fn selected_button(&self) -> usize {
        self.selected_button
    }

    /// Render the confirmation modal
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Clear the area behind the popup
        f.render_widget(Clear, area);

        // Create the main block
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        // Calculate inner area
        let inner_area = block.inner(area);

        // Split the inner area into message and buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3), // Message area
                Constraint::Length(3), // Buttons area
            ])
            .split(inner_area);

        // Render the message
        let message = Paragraph::new(Text::raw(&self.message))
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        f.render_widget(message, chunks[0]);

        // Render the buttons
        let button_count = self.buttons.len();
        let button_constraints: Vec<Constraint> = vec![Constraint::Ratio(1, button_count as u32); button_count];
        
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(button_constraints)
            .split(chunks[1]);

        for (i, button) in self.buttons.iter().enumerate() {
            let is_selected = i == self.selected_button;
            
            let button_style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(Color::Black)
                    .fg(Color::White)
            };

            let button_text = if button.is_default {
                format!("[{}] (default)", button.text)
            } else {
                format!("[{}]", button.text)
            };

            let button_widget = Paragraph::new(Text::raw(button_text))
                .style(button_style)
                .alignment(Alignment::Center);

            f.render_widget(button_widget, button_chunks[i]);
        }

        // Render the main block
        f.render_widget(block, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(modal.buttons(), &buttons);
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