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

pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    pty_executor: PtyExecutor,
    command_blocks: Vec<CommandBlock>,
    input: String,
    mode: AppMode,
    should_quit: bool,
    is_generating: bool,
    scroll_offset: u16,
    command_events_rx: tokio::sync::mpsc::UnboundedReceiver<ExecutionEvent>,
    command_events_tx: tokio::sync::mpsc::UnboundedSender<ExecutionEvent>,
    current_command_block: Option<CommandBlock>,
}

impl TerminalSession {
    pub fn new() -> Result<Self> {
        let (command_events_tx, command_events_rx) = tokio::sync::mpsc::unbounded_channel();
        
        let session = Self {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
            pty_executor: PtyExecutor::new()?,
            command_blocks: Vec::new(),
            input: String::new(),
            mode: AppMode::Chat,
            should_quit: false,
            is_generating: false,
            scroll_offset: 0,
            command_events_rx,
            command_events_tx,
            current_command_block: None,
        };

        Ok(session)
    }

    pub fn setup_terminal(&mut self) -> Result<Terminal<CrosstermBackend<Stdout>>> {
        debug!("Creating TerminalSession - step 1: checking terminal environment");
        
        // Check if stdin is actually a terminal
        if !std::io::stdin().is_terminal() {
            return Err(anyhow::anyhow!("Not running in an interactive terminal. stdin is not a tty."));
        }
        
        // Check if we're in a proper terminal environment
        let term_env = std::env::var("TERM").unwrap_or_default();
        if term_env.is_empty() {
            return Err(anyhow::anyhow!("No TERM environment variable set. Not running in a proper terminal."));
        }
        debug!("Terminal environment detected: {}", term_env);
        
        debug!("Creating TerminalSession - step 2: enabling raw mode");
        enable_raw_mode().map_err(|e| {
            anyhow::anyhow!("Failed to enable raw mode: {}", e)
        })?;
        
        debug!("Creating TerminalSession - step 2: setting up stdout");
        let mut stdout = io::stdout();
        debug!("Creating TerminalSession - step 3: executing terminal commands");
        
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
            anyhow::anyhow!("Failed to setup terminal (not in interactive environment?): {}", e)
        })?;
        
        debug!("Creating TerminalSession - step 4: creating backend");
        let backend = CrosstermBackend::new(stdout);
        debug!("Creating TerminalSession - step 5: creating terminal");
        let terminal = Terminal::new(backend)?;
        debug!("Creating TerminalSession - step 6: completed successfully");
        Ok(terminal)
    }

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

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting terminal UI...");
        
        // Add welcome message
        self.add_welcome_message();
        
        while !self.should_quit {
            // Create UI components based on current state
            let ui_data = self.prepare_ui_data();
            
            self.terminal.draw(|f| {
                render_ui(f, &ui_data);
            })?;
            
            // Check for terminal events (key, mouse, resize)
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        debug!("Key event received: {:?}", key);
                        self.handle_key_event(key).await?;
                    }
                    Event::Resize(width, height) => {
                        debug!("Resize event: {}x{}", width, height);
                        // Handle terminal resize
                    }
                    _ => {
                        debug!("Other event received");
                    }
                }
            }
            
            // Poll for command execution events
            if let Ok(event) = self.command_events_rx.try_recv() {
                self.handle_command_event(event).await?;
            }
        }
        
        Ok(())
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Chat => self.handle_chat_key(key).await?,
            AppMode::Help => self.handle_help_key(key)?,
        }
        Ok(())
    }
    
    async fn handle_chat_key(&mut self, key: KeyEvent) -> Result<()> {
        debug!("Handling chat key: {:?}", key);
        match key.code {
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    let user_input = self.input.clone();
                    self.input.clear();
                    debug!("Processing Enter key with input: {}", user_input);
                    
                    // Process command
                    self.process_user_input(user_input).await?;
                    debug!("Finished process_user_input");
                }
            }
            KeyCode::F(1) => {
                self.mode = AppMode::Help;
            }
            KeyCode::Esc => {
                if self.is_generating {
                    // Cancel generation
                    self.is_generating = false;
                }
            }
            // Scrolling controls
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(5);
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(5);
            }
            KeyCode::Up if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::Home => {
                self.scroll_offset = 0; // Scroll to top
            }
            KeyCode::End => {
                // Auto-scroll to bottom when new messages arrive
                self.scroll_offset = 0;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::F(1) => {
                self.mode = AppMode::Chat;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_command_event(&mut self, event: ExecutionEvent) -> Result<()> {
        match event {
            ExecutionEvent::StdoutData(data) => {
                if let Some(block) = &mut self.current_command_block {
                    block.append_output(&data, false);
                }
            }
            ExecutionEvent::StderrData(data) => {
                if let Some(block) = &mut self.current_command_block {
                    block.append_output(&data, true);
                }
            }
            ExecutionEvent::Completed { exit_code, duration } => {
                if let Some(block) = self.current_command_block.take() {
                    let mut completed_block = block.clone();
                    completed_block.complete(exit_code, duration);
                    self.command_blocks.push(completed_block);
                }
            }
            ExecutionEvent::Failed(error) => {
                if let Some(block) = self.current_command_block.take() {
                    let mut failed_block = block.clone();
                    failed_block.state = BlockState::Failed;
                    failed_block.append_output(&format!("\n[Error: {}]", error), true);
                    self.command_blocks.push(failed_block);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn process_user_input(&mut self, input: String) -> Result<()> {
        debug!("Starting process_user_input with: {}", input);
        
        // Create a new command block
        let command_block = CommandBlock::new(
            input.clone(), 
            self.pty_executor.working_dir().to_string()
        );
        
        // Store the command block and set state to running
        self.current_command_block = Some(command_block.clone());
        
        let pty_executor_clone = self.pty_executor.clone();
        let command_events_tx_clone = self.command_events_tx.clone();
        
        // Spawn a task to execute the command
        tokio::spawn(async move {
            let mut block_to_execute = command_block.clone();
            match pty_executor_clone.execute_block(&mut block_to_execute).await {
                Ok(_) => {
                    // Send the final state of the block
                    command_events_tx_clone.send(ExecutionEvent::Completed { 
                        exit_code: block_to_execute.exit_code.unwrap_or(0), 
                        duration: block_to_execute.duration.unwrap_or_default() 
                    }).unwrap();
                }
                Err(e) => {
                    command_events_tx_clone.send(ExecutionEvent::Failed(format!("Execution error: {}", e))).unwrap();
                }
            }
        });
        
        Ok(())
    }
    
    fn add_welcome_message(&mut self) {
        let mut block = CommandBlock::new(String::new(), self.pty_executor.working_dir().to_string());
        block.append_output("Welcome to AI Terminal! Type your command and press Enter to execute. Press F1 for help.", false);
        block.timestamp = Local::now();
        block.complete(0, Duration::new(0, 0)); // Mark as completed immediately
        self.command_blocks.push(block);
        
        // Auto-scroll to bottom when new messages arrive
        self.scroll_offset = 0;
    }
    
    fn prepare_ui_data(&self) -> UIData {
        UIData {
            mode: self.mode.clone(),
            command_blocks: self.command_blocks.clone(),
            input: self.input.clone(),
            is_generating: self.is_generating,
            scroll_offset: self.scroll_offset,
        }
    }
}

fn render_ui(f: &mut Frame, ui_data: &UIData) {
    match ui_data.mode {
        AppMode::Chat => render_chat_ui(f, ui_data),
        AppMode::Help => render_help_ui(f),
    }
}

fn render_chat_ui(f: &mut Frame, ui_data: &UIData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());
    
    // Messages area with proper word wrapping
    let mut messages_text = Vec::new();
    
    for block in &ui_data.command_blocks {
        // Display command
        messages_text.push(Line::from(vec![
            Span::styled(block.status_icon(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(&block.command, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" ({})", block.timestamp.format("%H:%M:%S"))),
        ]));
        
        // Display output
        if !block.output.is_empty() {
            messages_text.push(Line::from(vec![
                Span::raw("  "), // Indent output
                Span::styled(block.output.clone(), Style::default().fg(Color::White)),
            ]));
        }
        
        // Display status if completed
        if block.is_complete() {
            let status_color = block.status_color();
            messages_text.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("Status: {:?} (Exit: {}) (Duration: {:?})", 
                             block.state, 
                             block.exit_code.unwrap_or(-1), 
                             block.duration.unwrap_or_default()),
                    Style::default().fg(Color::Rgb(status_color.0, status_color.1, status_color.2))
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
                .title("AI Terminal")
        )
        .wrap(Wrap { trim: true })
        .scroll((ui_data.scroll_offset, 0));
    
    f.render_widget(messages_paragraph, chunks[0]);
    
    // Input area
    let input = Paragraph::new(ui_data.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input (Press Enter to execute)")
        );
    
    f.render_widget(input, chunks[1]);
    
    // Status bar
    let status_text = if ui_data.is_generating {
        "⏳ EXECUTING (ESC to cancel) | F1: Help | F10: Exit".to_string()
    } else {
        "F1: Help | F10: Exit".to_string()
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(status, chunks[2]);
}

fn render_help_ui(f: &mut Frame) {
    let area = f.area();
    let popup_area = centered_rect(80, 80, area);
    
    f.render_widget(Clear, popup_area);
    
    let help_text = vec![
        Line::from("AI Terminal Help"),
        Line::from(""),
        Line::from("Keyboard Shortcuts:"),
        Line::from("  F1      - Show/hide this help"),
        Line::from("  F10/Esc - Exit application"),
        Line::from(""),
        Line::from("Scrolling Controls:"),
        Line::from("  Page Up/Down    - Scroll chat by 5 lines"),
        Line::from("  Ctrl+Up/Down    - Scroll chat by 1 line"),
        Line::from("  Home            - Jump to top of chat"),
        Line::from("  End             - Jump to bottom of chat"),
        Line::from(""),
        Line::from("Command Execution:"),
        Line::from("  Type any shell command and press Enter to execute"),
        Line::from("  The output will be displayed in a block-based format"),
        Line::from(""),
        Line::from("Press Esc or F1 to close this help"),
    ];
    
    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(help, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}