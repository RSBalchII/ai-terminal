// Draft of updated lib.rs with new UI/UX improvements

// ... existing imports ...
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io::{self, IsTerminal, Stdout},
    time::Duration,
};
use tracing::{debug, info};
use chrono::Local;

use terminal_emulator::{PtyExecutor, ExecutionEvent, CommandBlock, BlockState};
use ollama_client::{OllamaClient, OllamaRequest};

// New imports for our UI/UX improvements
use layout::LayoutManager;
use widgets::CommandPalette;
use theme::ThemeManager;

// ... existing enums and structs ...

#[derive(Debug, Clone)]
pub enum AppMode {
    Chat,
    Help,
}

#[derive(Debug, Clone)]
pub struct UIData {
    pub mode: AppMode,
    pub command_blocks: Vec<CommandBlock>,
    pub input: String,
    pub is_generating: bool,
    pub scroll_offset: u16,
}

// New enum for UI state
#[derive(Debug, Clone)]
pub enum UIState {
    Normal,
    CommandPalette,
}

// ... existing TerminalSession struct with new fields ...

pub struct TerminalSession {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    pty_executor: PtyExecutor,
    command_blocks: Vec<CommandBlock>,
    input: String,
    mode: AppMode,
    should_quit: bool,
    is_generating: bool,
    command_history: CommandHistory,
    history_index: Option<usize>,
    ollama_client: OllamaClient,
    scroll_offset: u16,
    // New fields for UI/UX improvements
    layout_manager: LayoutManager,
    command_palette: CommandPalette,
    theme_manager: ThemeManager,
    ui_state: UIState,
}

// ... existing implementation with modifications ...

impl TerminalSession {
    pub fn new() -> Result<Self> {
        // ... existing initialization ...
        
        let terminal_size = crossterm::terminal::size()?;
        let layout_manager = LayoutManager::new(Rect::new(0, 0, terminal_size.0, terminal_size.1));
        let command_palette = CommandPalette::new();
        let theme_manager = ThemeManager::new();
        
        Ok(Self {
            terminal: None,
            pty_executor: PtyExecutor::new()?,
            command_blocks: Vec::new(),
            input: String::new(),
            mode: AppMode::Chat,
            should_quit: false,
            is_generating: false,
            command_history: CommandHistory::new(),
            history_index: None,
            ollama_client: OllamaClient::new("http://localhost:11434".to_string()),
            scroll_offset: 0,
            // New fields
            layout_manager,
            command_palette,
            theme_manager,
            ui_state: UIState::Normal,
        })
    }
    
    // ... existing methods ...
    
    pub fn setup_terminal(&mut self) -> Result<Terminal<CrosstermBackend<Stdout>>> {
        // ... existing setup code ...
        
        // Update layout manager with terminal size
        let terminal_size = crossterm::terminal::size()?;
        self.layout_manager.update_size(Rect::new(0, 0, terminal_size.0, terminal_size.1));
        
        // ... rest of existing setup code ...
    }
    
    // ... existing methods ...
    
    async fn handle_chat_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.ui_state {
            UIState::Normal => {
                // ... existing key handling ...
                
                // Add new key bindings for UI/UX improvements
                match key.code {
                    KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.ui_state = UIState::CommandPalette;
                        return Ok(());
                    }
                    // ... other key bindings ...
                    _ => {}
                }
            }
            UIState::CommandPalette => {
                match key.code {
                    KeyCode::Esc => {
                        self.ui_state = UIState::Normal;
                        self.command_palette.reset();
                    }
                    KeyCode::Enter => {
                        if let Some(command) = self.command_palette.get_selected_command() {
                            self.execute_palette_command(command)?;
                        }
                        self.ui_state = UIState::Normal;
                        self.command_palette.reset();
                    }
                    KeyCode::Backspace => {
                        self.command_palette.handle_backspace();
                    }
                    KeyCode::Up => {
                        self.command_palette.move_selection_up();
                    }
                    KeyCode::Down => {
                        self.command_palette.move_selection_down();
                    }
                    KeyCode::Char(c) => {
                        self.command_palette.handle_input(&c.to_string());
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    // New method to execute commands from the palette
    fn execute_palette_command(&mut self, command: &Command) -> Result<()> {
        match command.id.as_str() {
            "new_session" => {
                // Implement new session logic
                self.command_blocks.clear();
                self.add_welcome_message();
            }
            "clear_screen" => {
                self.command_blocks.clear();
            }
            "toggle_help" => {
                self.mode = match self.mode {
                    AppMode::Chat => AppMode::Help,
                    AppMode::Help => AppMode::Chat,
                };
            }
            "quit" => {
                self.should_quit = true;
            }
            "scroll_up" => {
                self.scroll_offset = self.scroll_offset.saturating_add(5);
            }
            "scroll_down" => {
                self.scroll_offset = self.scroll_offset.saturating_sub(5);
            }
            "toggle_theme" => {
                // Simple theme toggle between default and dark
                let current_theme = self.theme_manager.current_theme().name.clone();
                let next_theme = if current_theme == "default" { "dark" } else { "default" };
                let _ = self.theme_manager.switch_theme(next_theme);
            }
            _ => {
                // Handle unknown commands
            }
        }
        
        Ok(())
    }
    
    // ... rest of existing implementation ...
}

// ... existing render functions with theme support ...

fn render_chat_ui(f: &mut Frame, ui_data: &UIData, theme_manager: &ThemeManager) {
    let theme = theme_manager.current_theme();
    
    // Use the layout manager for calculating layout
    let layout_manager = LayoutManager::new(f.area());
    let main_layout = layout_manager.calculate_chat_layout();
    
    // Header
    let header_text = vec![
        Line::from(vec![
            Span::styled("AI Terminal", Style::default().fg(theme.primary)),
            Span::styled(" v0.1.0", Style::default().fg(theme.secondary)),
        ])
    ];
    
    let header = Paragraph::new(header_text)
        .style(Style::default().bg(theme.background).fg(theme.text))
        .block(Block::default().borders(Borders::NONE));
    
    f.render_widget(header, main_layout[0]);
    
    // Messages area with proper word wrapping
    let mut messages_text = Vec::new();
    
    for block in &ui_data.command_blocks {
        // Display command with theme colors
        messages_text.push(Line::from(vec![
            Span::styled(block.status_icon(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(&block.command, Style::default().fg(theme.command).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" ({})", block.timestamp.format("%H:%M:%S"))),
        ]));
        
        // Display output
        if !block.output.is_empty() {
            messages_text.push(Line::from(vec![
                Span::raw("  "), // Indent output
                Span::styled(block.output.clone(), Style::default().fg(theme.text)),
            ]));
        }
        
        // Display status if completed
        if block.is_complete() {
            let status_color = match block.state {
                BlockState::CompletedSuccess => theme.success,
                BlockState::CompletedError => theme.error,
                _ => theme.text,
            };
            
            messages_text.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("Status: {:?} (Exit: {}) (Duration: {:?})", 
                             block.state, 
                             block.exit_code.unwrap_or(-1), 
                             block.duration.unwrap_or_default()),
                    Style::default().fg(status_color)
                ),
            ]));
        }
        
        // Add empty line between blocks for readability
        messages_text.push(Line::from(""));
    }
    
    let messages_paragraph = Paragraph::new(messages_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Messages")
                .border_style(Style::default().fg(theme.secondary))
        )
        .style(Style::default().bg(theme.background).fg(theme.text))
        .wrap(Wrap { trim: true })
        .scroll((ui_data.scroll_offset, 0));
    
    f.render_widget(messages_paragraph, main_layout[1]);
    
    // Input area
    let input = Paragraph::new(ui_data.input.as_str())
        .style(Style::default().bg(theme.background).fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input (Press Enter to execute, / for AI commands, Ctrl+K for command palette)")
                .border_style(Style::default().fg(theme.secondary))
        );
    
    f.render_widget(input, main_layout[2]);
    
    // Status bar
    let status_text = if ui_data.is_generating {
        "⏳ EXECUTING (ESC to cancel) | F1: Help | F10: Exit | Ctrl+K: Command Palette".to_string()
    } else {
        "F1: Help | F10: Exit | Ctrl+K: Command Palette".to_string()
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().bg(theme.background).fg(theme.secondary))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(status, main_layout[3]);
}

// ... existing render_help_ui function ...

// New render function for command palette
fn render_command_palette(f: &mut Frame, command_palette: &CommandPalette, layout_manager: &LayoutManager) {
    let area = f.area();
    let popup_area = layout_manager.calculate_centered_rect(60, 50, area);
    command_palette.render(f, popup_area);
}

// ... existing centered_rect function ...

// Module declarations at the end of the file
mod layout;
mod widgets;
mod theme;

use crossterm::event::KeyModifiers;
use ratatui::layout::Rect;
use terminal_emulator::CommandHistory;
use widgets::Command;