//! Terminal UI for the AI Terminal application
//! 
//! This crate provides the terminal-based user interface using ratatui and crossterm.

// Existing imports
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io::{self, Stdout},
    time::Duration,
};

use terminal_emulator::{PtyExecutor, CommandBlock, BlockState, CommandHistory};
// Add ollama-client import
use ollama_client::{OllamaClient, OllamaRequest};
// Add futures_util import
use futures_util::StreamExt;

// New imports for our UI/UX improvements
use layout::LayoutManager;
use layout::PaneManager;
use layout::SplitOrientation;
use layout::TabManager;
use widgets::{CommandPalette, Command, ConfirmationModal, CommandBlock as UICommandBlock};
use theme::ThemeManager;
// Import our new markdown renderer
use markdown_renderer::render_markdown;

/// Application mode
#[derive(Debug, Clone)]
pub enum AppMode {
    Chat,
    Help,
}

/// UI data structure
#[derive(Debug, Clone)]
pub struct UIData {
    pub mode: AppMode,
    pub command_blocks: Vec<CommandBlock>,
    pub input: String,
    pub is_generating: bool,
    pub scroll_offset: u16,
}

/// UI state
#[derive(Debug, Clone)]
pub enum UIState {
    Normal,
    CommandPalette,
    ConfirmationModal,
}

/// Main terminal session struct
pub struct TerminalSession {
    pty_executor: PtyExecutor,
    command_blocks: Vec<CommandBlock>,
    input: String,
    mode: AppMode,
    should_quit: bool,
    is_generating: bool,
    command_history: CommandHistory,
    history_index: Option<usize>,
    // Add ollama_client field
    ollama_client: OllamaClient,
    scroll_offset: u16,
    // New fields for UI/UX improvements
    layout_manager: LayoutManager,
    pane_manager: PaneManager,
    tab_manager: TabManager,
    command_palette: CommandPalette,
    theme_manager: ThemeManager,
    ui_state: UIState,
    input_before_history: String,
    confirmation_modal: Option<ConfirmationModal>,
}

impl TerminalSession {
    /// Create a new terminal session
    pub fn new() -> Result<Self> {
        let terminal_size = crossterm::terminal::size()?;
        let layout_manager = LayoutManager::new(Rect::new(0, 0, terminal_size.0, terminal_size.1));
        let pane_manager = PaneManager::new(Rect::new(0, 0, terminal_size.0, terminal_size.1));
        let tab_manager = TabManager::new();
        let command_palette = CommandPalette::new();
        let mut theme_manager = ThemeManager::new();
        
        // Load user themes
        if let Err(e) = theme_manager.load_user_themes() {
            // Log the error but don't fail startup
            tracing::warn!("Failed to load user themes: {:?}", e);
        }
        
        let command_history = CommandHistory::new(1000)?; // Max 1000 history entries
        
        Ok(Self {
            pty_executor: PtyExecutor::new()?,
            command_blocks: Vec::new(),
            input: String::new(),
            mode: AppMode::Chat,
            should_quit: false,
            is_generating: false,
            command_history,
            history_index: None,
            // Add ollama_client initialization
            ollama_client: OllamaClient::new()?,
            scroll_offset: 0,
            // New fields
            layout_manager,
            pane_manager,
            tab_manager,
            command_palette,
            theme_manager,
            ui_state: UIState::Normal,
            input_before_history: String::new(),
            confirmation_modal: None,
        })
    }
    
    /// Setup the terminal for the TUI
    pub fn setup_terminal(&mut self) -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        // Update layout manager with terminal size
        let terminal_size = crossterm::terminal::size()?;
        self.layout_manager.update_size(Rect::new(0, 0, terminal_size.0, terminal_size.1));
        self.pane_manager.resize(Rect::new(0, 0, terminal_size.0, terminal_size.1));
        
        Ok(terminal)
    }
    
    /// Restore the terminal to its original state
    pub fn restore_terminal(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
    
    /// Run the terminal application
    pub async fn run(&mut self) -> Result<()> {
        // Add welcome message
        self.add_welcome_message();
        
        // Setup terminal
        let mut terminal = self.setup_terminal()?;
        
        loop {
            // Render the UI
            terminal.draw(|f| self.render(f))?;
            
            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.mode {
                        AppMode::Chat => self.handle_chat_key(key).await?,
                        AppMode::Help => self.handle_help_key(key).await?,
                    }
                    
                    if self.should_quit {
                        break;
                    }
                }
            }
        }
        
        // Restore terminal
        self.restore_terminal(&mut terminal)?;
        Ok(())
    }
    
    /// Add a welcome message to the terminal
    fn add_welcome_message(&mut self) {
        let mut welcome_block = CommandBlock::new(
            "Welcome to AI Terminal".to_string(),
            std::env::current_dir().unwrap_or_default().to_string_lossy().to_string()
        );
        welcome_block.state = BlockState::Success;
        welcome_block.append_output("Welcome to the AI Terminal! Type commands and press Enter to execute.
Use F1 for help, F10 to quit.", false);
        self.command_blocks.push(welcome_block);
    }
    
    /// Handle key events in chat mode
    async fn handle_chat_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.ui_state {
            UIState::Normal => {
                match key.code {
                    KeyCode::Enter => {
                        if !self.input.is_empty() {
                            // Add to history
                            self.command_history.add_command(self.input.clone())?;
                            
                            // Check if it's an AI command (starts with /)
                            if self.input.starts_with('/') {
                                // Handle AI command
                                self.handle_ai_command().await?;
                            } else {
                                // Create command block
                                let working_dir = self.pty_executor.working_dir().to_string();
                                let block = CommandBlock::new(self.input.clone(), working_dir);
                                
                                // Add block to the focused pane
                                if let Some(pane) = self.pane_manager.focused_pane_mut() {
                                    pane.add_command_block(block.clone());
                                }
                                
                                // Clear input
                                self.input.clear();
                                self.history_index = None;
                                
                                // Execute command
                                if let Some(pane) = self.pane_manager.focused_pane_mut() {
                                    if let Some(last_block) = pane.command_blocks.last_mut() {
                                        self.is_generating = true;
                                        self.pty_executor.execute_block(last_block).await?;
                                        self.is_generating = false;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.ui_state = UIState::CommandPalette;
                    }
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Show confirmation modal when trying to quit
                        self.show_confirmation_modal("Confirm Exit", "Are you sure you want to exit the AI Terminal?");
                    }
                    KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Split pane horizontally
                        let _ = self.pane_manager.split_focused_pane(SplitOrientation::Horizontal);
                    }
                    KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Split pane vertically
                        let _ = self.pane_manager.split_focused_pane(SplitOrientation::Vertical);
                    }
                    KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Close focused pane
                        let _ = self.pane_manager.close_focused_pane();
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Focus next pane
                        self.pane_manager.focus_next_pane();
                    }
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Focus previous pane
                        self.pane_manager.focus_prev_pane();
                    }
                    KeyCode::Char(c) => {
                        self.input.push(c);
                        self.history_index = None; // Reset history navigation when typing
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                        self.history_index = None; // Reset history navigation when typing
                    }
                    KeyCode::Up => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.scroll_offset = self.scroll_offset.saturating_add(1);
                        } else {
                            self.navigate_history_up();
                        }
                    }
                    KeyCode::Down => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.scroll_offset = self.scroll_offset.saturating_sub(1);
                        } else {
                            self.navigate_history_down();
                        }
                    }
                    KeyCode::Tab => {
                        self.handle_tab_completion();
                    }
                    KeyCode::Esc => {
                        self.input.clear();
                        self.history_index = None;
                    }
                    KeyCode::F(1) => {
                        self.mode = AppMode::Help;
                    }
                    KeyCode::F(10) => {
                        // Show confirmation modal when trying to quit
                        self.show_confirmation_modal("Confirm Exit", "Are you sure you want to exit the AI Terminal?");
                    }
                    // Page Up/Down for scrolling
                    KeyCode::PageUp => {
                        self.scroll_offset = self.scroll_offset.saturating_add(10);
                    }
                    KeyCode::PageDown => {
                        self.scroll_offset = self.scroll_offset.saturating_sub(10);
                    }
                    // Home/End for jumping to top/bottom
                    KeyCode::Home => {
                        self.scroll_offset = 0;
                    }
                    KeyCode::End => {
                        // Set to a large value to ensure we're at the bottom
                        self.scroll_offset = 1000;
                    }
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
                        // Clone the selected command to avoid borrowing issues
                        let selected_command = self.command_palette.get_selected_command().cloned();
                        if let Some(command) = selected_command {
                            self.execute_palette_command(&command)?;
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
            UIState::ConfirmationModal => {
                match key.code {
                    KeyCode::Esc => {
                        self.handle_confirmation_result("no");
                    }
                    KeyCode::Enter => {
                        if let Some(modal) = &self.confirmation_modal {
                            let result = modal.selected_button_id().to_string();
                            self.handle_confirmation_result(&result);
                        }
                    }
                    KeyCode::Left => {
                        if let Some(modal) = &mut self.confirmation_modal {
                            modal.select_previous();
                        }
                    }
                    KeyCode::Right => {
                        if let Some(modal) = &mut self.confirmation_modal {
                            modal.select_next();
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.handle_confirmation_result("yes");
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.handle_confirmation_result("no");
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle key events in help mode
    async fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::F(1) | KeyCode::Esc => {
                self.mode = AppMode::Chat;
            }
            KeyCode::F(10) => {
                self.should_quit = true;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle AI commands (starting with /)
    async fn handle_ai_command(&mut self) -> Result<()> {
        // Create a command block for the AI interaction
        let working_dir = self.pty_executor.working_dir().to_string();
        let block = CommandBlock::new(self.input.clone(), working_dir);
        
        // Add block to UI
        self.command_blocks.push(block.clone());
        
        // Clear input
        let ai_command = self.input.clone();
        self.input.clear();
        self.history_index = None;
        
        // Process the AI command
        self.is_generating = true;
        
        // For now, we'll just simulate an AI response
        // In a real implementation, this would call the Ollama API
        let ai_response = format!("This is a simulated response to: {}\n\nIn a real implementation, this would call the Ollama API.", &ai_command[1..]);
        
        // Update the last block with the AI response
        if let Some(last_block) = self.command_blocks.last_mut() {
            last_block.append_output(&ai_response, false);
            last_block.complete(0, std::time::Duration::from_millis(100));
        }
        
        self.is_generating = false;
        Ok(())
    }
    
    /// Navigate to the previous command in history
    fn navigate_history_up(&mut self) {
        let history_len = self.command_history.entries().len();
        if history_len == 0 {
            return;
        }
        
        // Save current input if we're just starting to navigate
        if self.history_index.is_none() {
            self.input_before_history = self.input.clone();
        }
        
        let current_index = self.history_index.unwrap_or(0);
        if current_index < history_len {
            self.history_index = Some(current_index + 1);
            if let Some(entry) = self.command_history.get_command(current_index) {
                self.input = entry.command.clone();
            }
        }
    }
    
    /// Navigate to the next command in history
    fn navigate_history_down(&mut self) {
        match self.history_index {
            Some(index) if index > 0 => {
                self.history_index = Some(index - 1);
                if let Some(entry) = self.command_history.get_command(index - 1) {
                    self.input = entry.command.clone();
                }
            }
            Some(0) => {
                // We've reached the end of history, restore the input
                self.input = self.input_before_history.clone();
                self.history_index = None;
            }
            _ => {
                // Not navigating history, do nothing
            }
        }
    }
    
    /// Handle tab completion for file paths
    fn handle_tab_completion(&mut self) {
        if self.input.is_empty() {
            return;
        }
        
        // Try to complete the file path
        let completed = self.complete_file_path(&self.input);
        if !completed.is_empty() {
            self.input = completed;
        }
    }
    
    /// Complete a file path
    fn complete_file_path(&self, path: &str) -> String {
        use std::path::Path;
        
        // Expand tilde to home directory
        let expanded_path = if path.starts_with('~') {
            if let Ok(home) = std::env::var("HOME") {
                path.replacen('~', &home, 1)
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        };
        
        // If it's an absolute path or starts with ~/, use it as is
        // Otherwise, make it relative to current directory
        let path_buf = if Path::new(&expanded_path).is_absolute() {
            Path::new(&expanded_path).to_path_buf()
        } else {
            let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            current_dir.join(&expanded_path)
        };
        
        // Get the parent directory and file name
        let parent_dir = path_buf.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path_buf.file_name().and_then(|s| s.to_str()).unwrap_or("");
        
        // Check if parent directory exists
        if !parent_dir.exists() || !parent_dir.is_dir() {
            return path.to_string();
        }
        
        // Read directory entries
        let entries: Vec<_> = std::fs::read_dir(parent_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .collect();
        
        // Find entries that match the file name
        let matches: Vec<_> = entries
            .iter()
            .filter(|entry| {
                entry.file_name()
                    .to_str()
                    .map(|name| name.starts_with(file_name))
                    .unwrap_or(false)
            })
            .collect();
        
        // If we have matches, complete with the first one
        if let Some(first_match) = matches.first() {
            let matched_name = first_match.file_name();
            let matched_path = parent_dir.join(&matched_name);
            
            // If it's a directory, add a trailing slash
            if matched_path.is_dir() {
                if path.starts_with("~/") || path == "~" {
                    // Convert back to ~/ form
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
                    if let Ok(relative) = matched_path.strip_prefix(&home) {
                        format!("~/{}/", relative.to_string_lossy())
                    } else {
                        format!("{}/", matched_path.to_string_lossy())
                    }
                } else if Path::new(path).is_absolute() {
                    format!("{}/", matched_path.to_string_lossy())
                } else {
                    // Make it relative to current directory
                    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
                    if let Ok(relative) = matched_path.strip_prefix(&current_dir) {
                        format!("{}/", relative.to_string_lossy())
                    } else {
                        format!("{}/", matched_path.to_string_lossy())
                    }
                }
            } else {
                // It's a file
                if path.starts_with("~/") {
                    // Convert back to ~/ form
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
                    if let Ok(relative) = matched_path.strip_prefix(&home) {
                        format!("~/{}", relative.to_string_lossy())
                    } else {
                        matched_path.to_string_lossy().to_string()
                    }
                } else if Path::new(path).is_absolute() {
                    matched_path.to_string_lossy().to_string()
                } else {
                    // Make it relative to current directory
                    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
                    if let Ok(relative) = matched_path.strip_prefix(&current_dir) {
                        relative.to_string_lossy().to_string()
                    } else {
                        matched_path.to_string_lossy().to_string()
                    }
                }
            }
        } else {
            path.to_string()
        }
    }
    
    /// Execute commands from the palette
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
                // Show confirmation modal instead of quitting immediately
                self.show_confirmation_modal("Confirm Exit", "Are you sure you want to exit the AI Terminal?");
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
            "test_confirmation" => {
                // Show a test confirmation modal
                self.show_confirmation_modal("Test Confirmation", "This is a test confirmation modal. Do you want to proceed?");
            }
            "save_theme" => {
                // For now, we'll just show a message that the theme was saved
                // In a real implementation, you would prompt for a theme name and save it
                let mut block = CommandBlock::new("Theme".to_string(), "".to_string());
                block.append_output("Current theme saved successfully!", false);
                self.command_blocks.push(block);
                
                // In a full implementation, we would do something like:
                // self.theme_manager.save_current_theme("my-theme")?;
            }
            "list_themes" => {
                let theme_names = self.theme_manager.available_theme_names();
                let mut output = "Available themes:
".to_string();
                for name in theme_names {
                    output.push_str(&format!("- {}
", name));
                }
                
                let mut block = CommandBlock::new("Themes".to_string(), "".to_string());
                block.append_output(&output, false);
                self.command_blocks.push(block);
            }
            _ => {
                // Handle unknown commands
            }
        }
        
        Ok(())
    }
    
    /// Show a confirmation modal
    fn show_confirmation_modal(&mut self, title: &str, message: &str) {
        let modal = ConfirmationModal::yes_no(title, message);
        self.confirmation_modal = Some(modal);
        self.ui_state = UIState::ConfirmationModal;
    }
    
    /// Handle confirmation modal result
    fn handle_confirmation_result(&mut self, result: &str) {
        // Store whether we should quit before resetting the modal
        let should_quit = self.confirmation_modal.as_ref()
            .map(|modal| modal.title() == "Confirm Exit")
            .unwrap_or(false);
        
        // Reset the modal state
        self.confirmation_modal = None;
        self.ui_state = UIState::Normal;
        
        // Handle the result
        match result {
            "yes" => {
                if should_quit {
                    self.should_quit = true;
                } else {
                    // For other confirmations, show a message that the action was confirmed
                    let mut block = CommandBlock::new("Confirmation".to_string(), "".to_string());
                    block.append_output("Action confirmed!", false);
                    self.command_blocks.push(block);
                }
            }
            "no" => {
                // User cancelled the action
                if !should_quit {
                    let mut block = CommandBlock::new("Confirmation".to_string(), "".to_string());
                    block.append_output("Action cancelled.", false);
                    self.command_blocks.push(block);
                }
                // If it was a quit confirmation, we just close the modal without quitting
            }
            _ => {}
        }
    }
    
    /// Render the UI
    fn render(&mut self, f: &mut Frame) {
        let ui_data = UIData {
            mode: self.mode.clone(),
            command_blocks: self.command_blocks.clone(),
            input: self.input.clone(),
            is_generating: self.is_generating,
            scroll_offset: self.scroll_offset,
        };
        
        match self.mode {
            AppMode::Chat => {
                // Render panes
                self.pane_manager.render(f);
                
                // Render command palette if in that state
                if let UIState::CommandPalette = self.ui_state {
                    // We need to render the command palette without borrowing self
                    let layout_manager = &self.layout_manager;
                    let command_palette = &self.command_palette;
                    render_command_palette(f, command_palette, layout_manager);
                }
                
                // Render confirmation modal if in that state
                if let UIState::ConfirmationModal = self.ui_state {
                    if let Some(modal) = &self.confirmation_modal {
                        let layout_manager = &self.layout_manager;
                        let popup_area = layout_manager.calculate_centered_rect(60, 20, f.area());
                        modal.render(f, popup_area);
                    }
                }
            }
            AppMode::Help => {
                render_help_ui(f, &self.layout_manager);
            }
        }
    }
}

/// Render the chat UI
fn render_chat_ui(f: &mut Frame, ui_data: &UIData, theme_manager: &ThemeManager) {
    let theme = theme_manager.current_theme();
    
    // Use the layout manager for calculating layout
    let layout_manager = LayoutManager::new(f.area());
    let main_layout = layout_manager.calculate_chat_layout();
    
    // Header
    let header_text = vec![
        Line::from(vec![
            "AI Terminal".fg(theme.primary),
            " v0.1.0".fg(theme.secondary),
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
            block.status_icon().bold(),
            " ".into(),
            block.command.clone().fg(theme.command).bold(),
            format!(" ({})", block.timestamp.format("%H:%M:%S")).into(),
        ]));
        
        // Display output with Markdown rendering
        if !block.output.is_empty() {
            // Render the output as Markdown
            let rendered_lines = render_markdown(&block.output);
            
            // Add each rendered line with proper indentation
            for mut line in rendered_lines {
                // Prepend indentation to the first span of each line
                if let Some(first_span) = line.spans.first_mut() {
                    let mut indented_text = "  ".to_string();  // Indentation
                    indented_text.push_str(&first_span.content);
                    first_span.content = indented_text.into();
                } else {
                    // If the line has no spans, add the indentation as a new span
                    line.spans.insert(0, Span::raw("  "));
                }
                messages_text.push(line);
            }
        }
        
        // Display status if completed
        if block.is_complete() {
            let status_color = match block.state {
                BlockState::Success => theme.success,
                BlockState::Failed => theme.error,
                _ => theme.text,
            };
            
            messages_text.push(Line::from(vec![
                "  ".into(),
                format!("Status: {:?} (Exit: {}) (Duration: {:?})", 
                         block.state, 
                         block.exit_code.unwrap_or(-1), 
                         block.duration.unwrap_or_default()).fg(status_color),
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
                .title("Input (Press Enter to execute, /command for AI, Ctrl+K for command palette)")
                .border_style(Style::default().fg(theme.secondary))
        );
    
    f.render_widget(input, main_layout[2]);
    
    // Status bar
    let status_text = if ui_data.is_generating {
        "⏳ EXECUTING (ESC to cancel) | F1: Help | F10: Exit | Ctrl+K: Command Palette"
    } else {
        "F1: Help | F10: Exit | Ctrl+K: Command Palette"
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().bg(theme.background).fg(theme.secondary))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(status, main_layout[3]);
}

/// Render the help UI
fn render_help_ui(f: &mut Frame, layout_manager: &LayoutManager) {
    let layout = layout_manager.calculate_chat_layout();
    
    let help_text = vec![
        "AI Terminal - Help".into(),
        "".into(),
        "Controls:".into(),
        "  Enter        - Execute command (/command for AI)".into(),
        "  Up/Down      - Navigate command history".into(),
        "  Tab          - File path completion".into(),
        "  Page Up/Down - Scroll through messages".into(),
        "  Ctrl+K       - Open command palette".into(),
        "  Ctrl+Q       - Quit with confirmation".into(),
        "  F1           - Toggle help".into(),
        "  F10          - Quit with confirmation".into(),
        "".into(),
        "Command Palette:".into(),
        "  Use Ctrl+K to open the command palette".into(),
        "  Type to search for commands".into(),
        "  Up/Down to navigate, Enter to select".into(),
        "".into(),
        "Theming:".into(),
        "  The AI Terminal supports custom themes.".into(),
        "  Themes can be created as TOML files in the config directory.".into(),
        "  Use the command palette to list, toggle, and save themes.".into(),
        "".into(),
        "Confirmation Dialogs:".into(),
        "  Some actions require confirmation (like quitting).".into(),
        "  Use Left/Right arrows to select options, Enter to confirm.".into(),
        "  You can also press Y/N for quick confirmation.".into(),
        "".into(),
        "Press F1 or Esc to return to the terminal".into(),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(help, layout[1]);
}

/// Render the command palette
fn render_command_palette(f: &mut Frame, command_palette: &CommandPalette, layout_manager: &LayoutManager) {
    let area = f.area();
    let popup_area = layout_manager.calculate_centered_rect(60, 50, area);
    command_palette.render(f, popup_area);
}

// Module declarations
pub mod layout;
pub mod widgets;
pub mod theme;
pub mod markdown_renderer;