use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap, Tabs},
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io::{self, IsTerminal, Stdout},
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};
use chrono::Local;

use ollama_client::OllamaClient;
use python_bridge::{PythonBridge, SystemToolRequest, SystemToolResponse};
use tokio::sync::mpsc;
use std::sync::Arc;
use terminal_emulator::{PtyExecutor, ExecutionEvent, CommandBlock, BlockState};

mod workspace;
use workspace::Tab;

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Input(String),
    ModelChange(String),
    ToggleOfflineMode,
    Exit,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum AppMode {
    Chat,
    ModelSelector,
    Help,
    CommandPalette,
}

#[derive(Debug, Clone)]
pub struct UIData {
    pub mode: AppMode,
    pub command_blocks: Vec<CommandBlock>,
    pub input: String,
    pub current_model: Option<String>,
    pub available_models: Vec<String>,
    pub selected_model_index: usize,
    pub offline_mode: bool,
    pub is_generating: bool,
    pub scroll_offset: u16,
}

pub struct App {
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new(initial_session: TerminalSession) -> Self {
        Self {
            tabs: vec![Tab::new(initial_session, "Session 1".to_string())],
            active_tab_index: 0,
            should_quit: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        let mut terminal = self.tabs[self.active_tab_index].app.setup_terminal()?;

        while !self.should_quit {
            // Draw the UI
            self.draw(&mut terminal)?;

            // Handle events
            tokio::select! {
                res = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(100))) => {
                    match res? {
                        Ok(true) => {
                            match event::read()? {
                                Event::Key(key) => {
                                    self.handle_key_event(key).await?;
                                }
                                Event::Mouse(mouse_event) => {
                                    self.handle_mouse_event(mouse_event).await?;
                                }
                                Event::Resize(width, height) => {
                                    // Handle terminal resize for all sessions
                                    for tab in &mut self.tabs {
                                        tab.app.handle_resize(width, height);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(false) => {
                            // No events available, continue the loop
                        }
                        Err(e) => {
                            warn!("Error polling for events: {}", e);
                        }
                    }
                }
                // Poll for command execution events from the active session
                Some(event) = self.tabs[self.active_tab_index].app.command_events_rx.recv() => {
                    self.tabs[self.active_tab_index].app.handle_command_event(event).await?;
                }
            }
        }

        self.tabs[self.active_tab_index].app.restore_terminal(&mut terminal)?;
        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Tab bar
                    Constraint::Min(0),    // Main content
                ])
                .split(f.size());

            // Render tab bar
            let titles: Vec<Line> = self.tabs
                .iter()
                .enumerate()
                .map(|(i, tab)|
                    Line::from(Span::styled(
                        format!(" {} ", tab.title),
                        Style::default()
                            .fg(Color::White)
                            .bg(if i == self.active_tab_index { Color::DarkGray } else { Color::Black })
                    ))
                )
                .collect();

            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::BOTTOM))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(self.active_tab_index);

            f.render_widget(tabs, chunks[0]);

            // Render active session
            let active_session_ui_data = self.tabs[self.active_tab_index].app.prepare_ui_data();
            let active_session = &self.tabs[self.active_tab_index].app;
            render_ui(f, &active_session_ui_data, active_session);
        })?;
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('t') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Ctrl+T to open new tab
                let new_session = TerminalSession::new(
                    self.tabs[self.active_tab_index].app.ollama_client.clone(),
                    self.tabs[self.active_tab_index].app.python_bridge.clone(),
                )?;
                let new_tab_index = self.tabs.len();
                self.tabs.push(Tab::new(new_session, format!("Session {}", new_tab_index + 1)));
                self.active_tab_index = new_tab_index;
            }
            KeyCode::PageUp if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Ctrl+PageUp to switch to previous tab
                if self.active_tab_index > 0 {
                    self.active_tab_index -= 1;
                }
            }
            KeyCode::PageDown if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Ctrl+PageDown to switch to next tab
                if self.active_tab_index < self.tabs.len() - 1 {
                    self.active_tab_index += 1;
                }
            }
            KeyCode::F(10) => {
                self.should_quit = true;
            }
            KeyCode::Char('p') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                // Ctrl+Shift+P to open command palette
                self.tabs[self.active_tab_index].app.mode = AppMode::CommandPalette;
                self.tabs[self.active_tab_index].app.command_palette_input.clear();
                self.tabs[self.active_tab_index].app.filtered_commands = self.tabs[self.active_tab_index].app.get_all_commands(); // Populate initial commands
                self.tabs[self.active_tab_index].app.selected_command_index = 0;
            }
            _ => {
                // Delegate to active session's key event handler
                self.tabs[self.active_tab_index].app.handle_key_event(key).await?;
            }
        }
        Ok(())
    }

    async fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        // Delegate to active session's mouse event handler
        self.tabs[self.active_tab_index].app.handle_mouse_event(mouse_event).await?;
        Ok(())
    }

    terminal: Terminal<CrosstermBackend<Stdout>>,
    ollama_client: OllamaClient,
    python_bridge: PythonBridge,
    pty_executor: PtyExecutor,
    command_blocks: VecDeque<CommandBlock>,
    input: String,
    mode: AppMode,
    offline_mode: bool,
    available_models: Vec<String>,
    selected_model_index: usize,
    should_quit: bool,
    is_generating: bool,
    scroll_offset: u16,
    system_tools_tx: Option<Arc<mpsc::UnboundedSender<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>>>,
    current_command_block: Option<CommandBlock>,
    command_events_rx: mpsc::UnboundedReceiver<ExecutionEvent>,
    command_events_tx: mpsc::UnboundedSender<ExecutionEvent>,
    command_palette_input: String,
    filtered_commands: Vec<String>,
    selected_command_index: usize,
}

impl TerminalSession {
    pub fn new(ollama_client: OllamaClient, python_bridge: PythonBridge) -> Result<Self> {
        let (command_events_tx, command_events_rx) = mpsc::unbounded_channel();
        
        let mut session = Self {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
            ollama_client,
            python_bridge,
            pty_executor: PtyExecutor::new()?,
            command_blocks: VecDeque::new(),
            input: String::new(),
            mode: AppMode::Chat,
            offline_mode: false,
            available_models: Vec::new(),
            selected_model_index: 0,
            should_quit: false,
            is_generating: false,
            scroll_offset: 0,
            system_tools_tx: None,
            current_command_block: None,
            command_events_rx,
            command_events_tx,
            command_palette_input: String::new(),
            filtered_commands: Vec::new(),
            selected_command_index: 0,
        };

        // Add welcome message
        session.add_message(Message {
            role: "system".to_string(),
            content: "Welcome to AI Terminal! Type your message and press Enter to chat. Press F1 for help.".to_string(),
            timestamp: Instant::now(),
        });

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

    pub fn command_events_receiver(&mut self) -> &mut mpsc::UnboundedReceiver<ExecutionEvent> {
        &mut self.command_events_rx
    }

    pub async fn handle_command_event(&mut self, event: ExecutionEvent) -> Result<()> {
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
                if let Some(block) = &mut self.current_command_block {
                    block.complete(exit_code, duration);
                    self.add_message(Message {
                        role: "system".to_string(),
                        content: format!("Command finished with exit code {} in {:?}", exit_code, duration),
                        timestamp: Instant::now(),
                    });
                    self.current_command_block = None; // Command finished
                }
            }
            ExecutionEvent::Failed(error) => {
                if let Some(block) = &mut self.current_command_block {
                    block.state = BlockState::Failed;
                    self.add_message(Message {
                        role: "error".to_string(),
                        content: format!("Command failed: {}", error),
                        timestamp: Instant::now(),
                    });
                    self.current_command_block = None; // Command finished
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        // This method can be used to adjust internal state based on resize
        // For now, we don't have any specific internal state that needs adjusting
        // beyond what ratatui handles automatically.
        debug!("TerminalSession received resize: {}x{}", width, height);
    }

    fn get_all_commands(&self) -> Vec<String> {
        vec![
            "chat: Send message to AI".to_string(),
            "model: Select AI model".to_string(),
            "help: Show help".to_string(),
            "offline: Toggle offline mode".to_string(),
            "exit: Exit application".to_string(),
            "shell: Execute shell command".to_string(),
            // Add more commands here as they are implemented
        ]
    }
    
    pub async fn set_model(&mut self, model_name: String) -> Result<()> {
        self.ollama_client.set_model(model_name).await?;
        self.refresh_models().await?;
        Ok(())
    }
    
    pub fn set_offline_mode(&mut self, offline: bool) {
        self.offline_mode = offline;
        info!("Offline mode: {}", offline);
    }
    
    pub fn set_system_tools_executor(
        &mut self, 
        tx: Arc<mpsc::UnboundedSender<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>>
    ) {
        self.system_tools_tx = Some(tx.clone());
        self.python_bridge.set_system_tools_executor(tx);
    }
    
    async fn refresh_models(&mut self) -> Result<()> {
        if !self.offline_mode && self.ollama_client.is_available().await {
            let models = self.ollama_client.get_models().await?;
            self.available_models = models.into_iter().map(|m| m.name).collect();
            debug!("Available models: {:?}", self.available_models);
        }
        Ok(())
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting terminal UI...");
        
        // Refresh models on startup
        self.refresh_models().await.unwrap_or_else(|e| {
            warn!("Could not refresh models: {}", e);
        });
        
        // Add welcome message
        self.add_message(Message {
            role: "system".to_string(),
            content: "Welcome to AI Terminal! Type your message and press Enter to chat. Press F1 for help.".to_string(),
            timestamp: Instant::now(),
        });
        
        while !self.should_quit {
            // Create UI components based on current state
            let ui_data = self.prepare_ui_data();
            
            self.terminal.draw(|f| {
                render_ui(f, &ui_data);
            })?;
            
            debug!("Polling for events...");
            // Check for terminal events (key, mouse, resize)
            if event::poll(Duration::from_millis(100))? {
                debug!("Event available, reading...");
                match event::read()? {
                    Event::Key(key) => {
                        debug!("Key event received: {:?}", key);
                        self.handle_key_event(key).await?;
                    }
                    Event::Mouse(mouse_event) => {
                        debug!("Mouse event received: {:?}", mouse_event);
                        self.handle_mouse_event(mouse_event).await?;
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
            tokio::select! {
                // Poll for terminal events (key, mouse, resize)
                res = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(100))) => {
                    match res? {
                        Ok(true) => {
                            match event::read()? {
                                Event::Key(key) => {
                                    debug!("Key event received: {:?}", key);
                                    self.handle_key_event(key).await?;
                                }
                                Event::Mouse(mouse_event) => {
                                    debug!("Mouse event received: {:?}", mouse_event);
                                    self.handle_mouse_event(mouse_event).await?;
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
                        Ok(false) => {
                            // No events available, continue the loop
                        }
                        Err(e) => {
                            warn!("Error polling for events: {}", e);
                        }
                    }
                }
                // Poll for command execution events
                Some(event) = self.command_events_rx.recv() => {
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
                            if let Some(block) = &mut self.current_command_block {
                                block.complete(exit_code, duration);
                                self.add_message(Message {
                                    role: "system".to_string(),
                                    content: format!("Command finished with exit code {} in {:?}", exit_code, duration),
                                    timestamp: Instant::now(),
                                });
                                self.current_command_block = None; // Command finished
                            }
                        }
                        ExecutionEvent::Failed(error) => {
                            if let Some(block) = &mut self.current_command_block {
                                block.state = BlockState::Failed;
                                self.add_message(Message {
                                    role: "error".to_string(),
                                    content: format!("Command failed: {}", error),
                                    timestamp: Instant::now(),
                                });
                                self.current_command_block = None; // Command finished
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        
        Ok(())
    }
    
    
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Chat => self.handle_chat_key(key).await?,
            AppMode::ModelSelector => self.handle_model_selector_key(key).await?,
            AppMode::Help => self.handle_help_key(key)?,
            AppMode::CommandPalette => self.handle_command_palette_key(key).await?,
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
                    
                    // Add user message
                    self.add_message(Message {
                        role: "user".to_string(),
                        content: user_input.clone(),
                        timestamp: Instant::now(),
                    });
                    
                    debug!("About to call process_user_input");
                    // Process with AI
                    self.process_user_input(user_input).await?;
                    debug!("Finished process_user_input");
                }
            }
            KeyCode::F(1) => {
                self.mode = AppMode::Help;
            }
            KeyCode::F(2) => {
                self.mode = AppMode::ModelSelector;
            }
            KeyCode::F(3) => {
                self.offline_mode = !self.offline_mode;
                self.add_message(Message {
                    role: "system".to_string(),
                    content: format!("Offline mode: {}", if self.offline_mode { "ON" } else { "OFF" }),
                    timestamp: Instant::now(),
                });
            }
            KeyCode::Esc => {
                if self.is_generating {
                    // Cancel generation
                    self.is_generating = false;
                    self.add_message(Message {
                        role: "system".to_string(),
                        content: "❌ Generation cancelled by user".to_string(),
                        timestamp: Instant::now(),
                    });
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
    
    async fn handle_model_selector_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                if self.selected_model_index > 0 {
                    self.selected_model_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_model_index < self.available_models.len().saturating_sub(1) {
                    self.selected_model_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(model_name) = self.available_models.get(self.selected_model_index) {
                    // Clone the model name first to avoid borrow issues
                    let model_name_clone = model_name.clone();
                    
                    // Add loading message immediately
                    self.add_message(Message {
                        role: "system".to_string(),
                        content: format!("⏳ Switching to model: {}...", model_name_clone),
                        timestamp: Instant::now(),
                    });
                    
                    // Switch to chat mode immediately for better UX
                    self.mode = AppMode::Chat;
                    
                    // Clone data for background task
                    let mut ollama_client = self.ollama_client.clone();
                    let model_name_for_task = model_name_clone.clone();
                    
                    // Spawn background task for model switching
                    let model_switch_task = tokio::spawn(async move {
                        ollama_client.set_model(model_name_for_task).await
                    });
                    
                    // Handle the result
                    match model_switch_task.await {
                        Ok(Ok(())) => {
                            // Remove loading message and add success message
                            if let Some(last_block) = self.command_blocks.back() {
                                if last_block.output.contains("⏳ Switching to model:") {
                                    self.command_blocks.pop_back();
                                }
                            }
                            self.add_message(Message {
                                role: "system".to_string(),
                                content: format!("✅ Model changed to: {}", model_name_clone),
                                timestamp: Instant::now(),
                            });
                        }
                        Ok(Err(e)) => {
                            // Remove loading message and add error
                            if let Some(last_block) = self.command_blocks.back() {
                                if last_block.output.contains("⏳ Switching to model:") {
                                    self.command_blocks.pop_back();
                                }
                            }
                            self.add_message(Message {
                                role: "error".to_string(),
                                content: format!("❌ Failed to set model: {}", e),
                                timestamp: Instant::now(),
                            });
                        }
                        Err(e) => {
                            // Remove loading message and add task error
                            if let Some(last_block) = self.command_blocks.back() {
                                if last_block.output.contains("⏳ Switching to model:") {
                                    self.command_blocks.pop_back();
                                }
                            }
                            self.add_message(Message {
                                role: "error".to_string(),
                                content: format!("❌ Model switch task error: {}", e),
                                timestamp: Instant::now(),
                            });
                        }
                    }
                } else {
                    self.mode = AppMode::Chat;
                }
            }
            KeyCode::Esc => {
                self.mode = AppMode::Chat;
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
    
    async fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        match self.mode {
            AppMode::Chat => {
                match mouse_event.kind {
                    MouseEventKind::ScrollUp => {
                        self.scroll_offset = self.scroll_offset.saturating_sub(3);
                        debug!("Mouse scroll up, offset: {}", self.scroll_offset);
                    }
                    MouseEventKind::ScrollDown => {
                        self.scroll_offset = self.scroll_offset.saturating_add(3);
                        debug!("Mouse scroll down, offset: {}", self.scroll_offset);
                    }
                    _ => {
                        // Handle other mouse events like clicks if needed in the future
                        debug!("Unhandled mouse event: {:?}", mouse_event.kind);
                    }
                }
            }
            _ => {
                // Mouse events in other modes (model selector, help) can be handled here
                debug!("Mouse event in mode {:?}: {:?}", self.mode, mouse_event.kind);
            }
        }
        Ok(())
    }

    async fn handle_command_palette_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(c) => {
                self.command_palette_input.push(c);
                self.filter_commands();
            }
            KeyCode::Backspace => {
                self.command_palette_input.pop();
                self.filter_commands();
            }
            KeyCode::Up => {
                if self.selected_command_index > 0 {
                    self.selected_command_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_command_index < self.filtered_commands.len().saturating_sub(1) {
                    self.selected_command_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(command) = self.filtered_commands.get(self.selected_command_index) {
                    self.mode = AppMode::Chat; // Exit command palette
                    self.execute_command_from_palette(command.clone()).await?;
                }
            }
            KeyCode::Esc => {
                self.mode = AppMode::Chat; // Exit command palette
            }
            _ => {}
        }
        Ok(())
    }

    fn filter_commands(&mut self) {
        let all_commands = self.get_all_commands();
        self.filtered_commands = all_commands
            .into_iter()
            .filter(|cmd| cmd.to_lowercase().contains(&self.command_palette_input.to_lowercase()))
            .collect();
        // Reset selection if filtered list is empty or selection is out of bounds
        if self.selected_command_index >= self.filtered_commands.len() {
            self.selected_command_index = self.filtered_commands.len().saturating_sub(1);
        }
    }

    async fn execute_command_from_palette(&mut self, command: String) -> Result<()> {
        // This is where we map command palette commands to actions
        match command.as_str() {
            "chat: Send message to AI" => {
                // No direct action, just return to chat mode
            }
            "model: Select AI model" => {
                self.mode = AppMode::ModelSelector;
            }
            "help: Show help" => {
                self.mode = AppMode::Help;
            }
            "offline: Toggle offline mode" => {
                self.offline_mode = !self.offline_mode;
                self.add_message(Message {
                    role: "system".to_string(),
                    content: format!("Offline mode: {}", if self.offline_mode { "ON" } else { "OFF" }),
                    timestamp: Instant::now(),
                });
            }
            "exit: Exit application" => {
                self.should_quit = true;
            }
            cmd if cmd.starts_with("shell:") => {
                let shell_command = cmd.trim_start_matches("shell:").trim();
                self.process_user_input(shell_command.to_string()).await?;
            }
            _ => {
                self.add_message(Message {
                    role: "error".to_string(),
                    content: format!("Unknown command: {}", command),
                    timestamp: Instant::now(),
                });
            }
        }
        Ok(())
    }

    fn get_all_commands(&self) -> Vec<String> {
        vec![
            "chat: Send message to AI".to_string(),
            "model: Select AI model".to_string(),
            "help: Show help".to_string(),
            "offline: Toggle offline mode".to_string(),
            "exit: Exit application".to_string(),
            "shell: Execute shell command".to_string(),
            // Add more commands here as they are implemented
        ]
    }
    
    async fn process_user_input(&mut self, input: String) -> Result<()> {
        debug!("Starting process_user_input with: {}", input);
        
        // Check for system tool requests first
        if let Some(tool_request) = self.python_bridge.parse_system_tool_request(&input) {
            debug!("System tool request detected: {:?}", tool_request);
            
            self.add_message(Message {
                role: "system".to_string(),
                content: format!("🔧 Executing {} tool: {}", tool_request.tool_type, tool_request.tool_name),
                timestamp: Instant::now(),
            });
            
            // Execute system tool
            match self.python_bridge.execute_system_tool(tool_request).await {
                Ok(response) => {
                    let content = if response.success {
                        format!("✅ Tool executed successfully ({}ms):\n{}", response.execution_time_ms, response.output)
                    } else {
                        format!("❌ Tool execution failed ({}ms):\n{}", 
                               response.execution_time_ms, 
                               response.error.unwrap_or("Unknown error".to_string()))
                    };
                    
                    self.add_message(Message {
                        role: "tool".to_string(),
                        content,
                        timestamp: Instant::now(),
                    });
                    return Ok(());
                }
                Err(e) => {
                    self.add_message(Message {
                        role: "error".to_string(),
                        content: format!("❌ System tool error: {}", e),
                        timestamp: Instant::now(),
                    });
                    return Ok(());
                }
            }
        }
        
        // Check for legacy tool intent (keeping for compatibility)
        debug!("Checking legacy tool intent via Python bridge");
        if let Ok(Some(tool_call)) = self.python_bridge.recognize_tool_intent(&input) {
            debug!("Legacy tool intent recognized: {:?}", tool_call);
            self.add_message(Message {
                role: "system".to_string(),
                content: format!("Executing tool: {}", tool_call.tool_name),
                timestamp: Instant::now(),
            });
            
            if let Ok(tool_result) = self.python_bridge.dispatch_tool_call(&tool_call) {
                self.add_message(Message {
                    role: "tool".to_string(),
                    content: format!("Tool result: {}", tool_result),
                    timestamp: Instant::now(),
                });
                return Ok(());
            }
        }
        
        // Check for legacy tool intent (keeping for compatibility)
        debug!("Checking legacy tool intent via Python bridge");
        if let Ok(Some(tool_call)) = self.python_bridge.recognize_tool_intent(&input) {
            debug!("Legacy tool intent recognized: {:?}", tool_call);
            self.add_message(Message {
                role: "system".to_string(),
                content: format!("Executing tool: {}", tool_call.tool_name),
                timestamp: Instant::now(),
            });
            
            if let Ok(tool_result) = self.python_bridge.dispatch_tool_call(&tool_call) {
                self.add_message(Message {
                    role: "tool".to_string(),
                    content: format!("Tool result: {}", tool_result),
                    timestamp: Instant::now(),
                });
                return Ok(());
            }
        }
        
        // If not a system tool or legacy tool, treat as a shell command
        debug!("Treating input as shell command: {}", input);
        let mut command_block = CommandBlock::new(input.clone(), self.pty_executor.working_dir().to_string());
        self.add_message(Message {
            role: "system".to_string(),
            content: format!("Executing shell command: `{}`", input),
            timestamp: Instant::now(),
        });
        
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
                    command_events_tx_clone.send(ExecutionEvent::Completed { exit_code: block_to_execute.exit_code.unwrap_or(0), duration: block_to_execute.duration.unwrap_or_default() }).unwrap();
                }
                Err(e) => {
                    command_events_tx_clone.send(ExecutionEvent::Failed(format!("Execution error: {}", e))).unwrap();
                }
            }
        });
        
        Ok(())
    }
    
    fn add_message(&mut self, message: Message) {
        let mut block = CommandBlock::new(String::new(), self.pty_executor.working_dir().to_string());
        block.append_output(&format!("[{}] {}", message.role, message.content), false);
        block.timestamp = Local::now();
        
        // For system/error/tool messages, we can mark them as completed immediately
        block.complete(0, Duration::new(0, 0)); // Assuming success for messages
        
        self.command_blocks.push_back(block);
        
        // Keep only last 100 messages to prevent memory issues
        if self.command_blocks.len() > 100 {
            self.command_blocks.pop_front();
        }
        
        // Auto-scroll to bottom when new messages arrive
        // Only if user isn't actively scrolling (scroll_offset close to 0)
        if self.scroll_offset < 3 {
            self.scroll_offset = 0;
        }
    }
    
    fn prepare_ui_data(&self) -> UIData {
        UIData {
            mode: self.mode.clone(),
            command_blocks: self.command_blocks.iter().cloned().collect(),
            input: self.input.clone(),
            current_model: self.ollama_client.get_current_model().map(|s| s.clone()),
            available_models: self.available_models.clone(),
            selected_model_index: self.selected_model_index,
            offline_mode: self.offline_mode,
            is_generating: self.is_generating,
            scroll_offset: self.scroll_offset,
        }
    }
    
    
    
    
}

fn render_ui(f: &mut Frame, ui_data: &UIData, session: &TerminalSession) {
    match ui_data.mode {
        AppMode::Chat => render_chat_ui(f, ui_data),
        AppMode::ModelSelector => render_model_selector(f, ui_data),
        AppMode::Help => render_help_ui(f),
        AppMode::CommandPalette => render_command_palette(f, session),
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
                Span::styled(block.visible_output(), Style::default().fg(Color::White)),
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
                .title("AI Terminal Chat")
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
                .title("Input (Press Enter to send)")
        );
    
    f.render_widget(input, chunks[1]);
    
    // Status bar
    let current_model = ui_data.current_model
        .as_ref()
        .map(|m| m.as_str())
        .unwrap_or("No model");
    
    let status_text = if ui_data.is_generating {
        format!(
            "Model: {} | Offline: {} | ⏳ GENERATING (ESC to cancel) | F1: Help | F10: Exit",
            current_model,
            if ui_data.offline_mode { "ON" } else { "OFF" }
        )
    } else {
        format!(
            "Model: {} | Offline: {} | F1: Help | F2: Models | F3: Toggle Offline | F10: Exit",
            current_model,
            if ui_data.offline_mode { "ON" } else { "OFF" }
        )
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(status, chunks[2]);
}

fn render_model_selector(f: &mut Frame, ui_data: &UIData) {
    let area = f.area();
    let popup_area = centered_rect(60, 70, area);
    
    f.render_widget(Clear, popup_area);
    
    let models: Vec<ListItem> = ui_data
        .available_models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let style = if i == ui_data.selected_model_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            
            ListItem::new(model.as_str()).style(style)
        })
        .collect();
    
    let models_list = List::new(models)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Model (↑↓ to navigate, Enter to select, Esc to cancel)")
        );
    
    f.render_widget(models_list, popup_area);
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
        Line::from("  F2      - Open model selector"),
        Line::from("  F3      - Toggle offline mode"),
        Line::from("  F10/Esc - Exit application"),
        Line::from(""),
        Line::from("Scrolling Controls:"),
        Line::from("  Page Up/Down    - Scroll chat by 5 lines"),
        Line::from("  Ctrl+Up/Down    - Scroll chat by 1 line"),
        Line::from("  Mouse Wheel     - Scroll chat by 3 lines"),
        Line::from("  Home            - Jump to top of chat"),
        Line::from("  End             - Jump to bottom of chat"),
        Line::from(""),
        Line::from("Chat Commands:"),
        Line::from("  Type any message and press Enter to send"),
        Line::from("  The AI will respond using the selected model"),
        Line::from(""),
        Line::from("Tool Integration:"),
        Line::from("  File operations: 'list files', 'read file.txt', etc."),
        Line::from("  Web search: 'search for rust tutorials'"),
        Line::from("  Weather: 'weather in London'"),
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn render_command_palette(f: &mut Frame, session: &TerminalSession) {
    let area = f.area();
    let popup_area = centered_rect(80, 50, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Min(0),    // Commands list
        ])
        .split(popup_area);

    // Input area
    let input = Paragraph::new(session.command_palette_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Palette (Type to filter)")
        );
    f.render_widget(input, chunks[0]);

    // Commands list
    let commands: Vec<ListItem> = session
        .filtered_commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let style = if i == session.selected_command_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(cmd.as_str()).style(style)
        })
        .collect();

    let commands_list = List::new(commands)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(commands_list, chunks[1]);
}
