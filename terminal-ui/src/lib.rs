use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io::{self, IsTerminal, Stdout},
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};

use ollama_client::OllamaClient;
use python_bridge::{PythonBridge, SystemToolRequest, SystemToolResponse};
use tokio::sync::mpsc;
use std::sync::Arc;

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
}

#[derive(Debug, Clone)]
pub struct UIData {
    pub mode: AppMode,
    pub messages: Vec<Message>,
    pub input: String,
    pub current_model: Option<String>,
    pub available_models: Vec<String>,
    pub selected_model_index: usize,
    pub offline_mode: bool,
    pub is_generating: bool,
}

pub struct TerminalApp {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    ollama_client: OllamaClient,
    python_bridge: PythonBridge,
    messages: VecDeque<Message>,
    input: String,
    mode: AppMode,
    offline_mode: bool,
    available_models: Vec<String>,
    selected_model_index: usize,
    should_quit: bool,
    is_generating: bool,
    system_tools_tx: Option<Arc<mpsc::UnboundedSender<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>>>,
}

impl TerminalApp {
    pub fn new(ollama_client: OllamaClient, python_bridge: PythonBridge) -> Result<Self> {
        debug!("Creating TerminalApp - step 1: checking terminal environment");
        
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
        
        debug!("Creating TerminalApp - step 2: enabling raw mode");
        enable_raw_mode().map_err(|e| {
            anyhow::anyhow!("Failed to enable raw mode: {}", e)
        })?;
        
        debug!("Creating TerminalApp - step 2: setting up stdout");
        let mut stdout = io::stdout();
        debug!("Creating TerminalApp - step 3: executing terminal commands");
        
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
            anyhow::anyhow!("Failed to setup terminal (not in interactive environment?): {}", e)
        })?;
        
        debug!("Creating TerminalApp - step 4: creating backend");
        let backend = CrosstermBackend::new(stdout);
        debug!("Creating TerminalApp - step 5: creating terminal");
        let terminal = Terminal::new(backend)?;
        debug!("Creating TerminalApp - step 6: completed successfully");
        
        Ok(Self {
            terminal,
            ollama_client,
            python_bridge,
            messages: VecDeque::new(),
            input: String::new(),
            mode: AppMode::Chat,
            offline_mode: false,
            available_models: Vec::new(),
            selected_model_index: 0,
            should_quit: false,
            is_generating: false,
            system_tools_tx: None,
        })
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
            if event::poll(Duration::from_millis(100))? {
                debug!("Event available, reading...");
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
            } else {
                // No events available, continue the loop
                debug!("No events available, continuing...");
            }
        }
        
        self.restore_terminal()?;
        Ok(())
    }
    
    fn restore_terminal(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Chat => self.handle_chat_key(key).await?,
            AppMode::ModelSelector => self.handle_model_selector_key(key).await?,
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
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::F(10) => {
                self.should_quit = true;
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
                            if let Some(last_msg) = self.messages.back() {
                                if last_msg.content.starts_with("⏳ Switching to model:") {
                                    self.messages.pop_back();
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
                            if let Some(last_msg) = self.messages.back() {
                                if last_msg.content.starts_with("⏳ Switching to model:") {
                                    self.messages.pop_back();
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
                            if let Some(last_msg) = self.messages.back() {
                                if last_msg.content.starts_with("⏳ Switching to model:") {
                                    self.messages.pop_back();
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
        
        // Generate AI response
        debug!("About to generate AI response");
        if let Some(model) = self.ollama_client.get_current_model() {
            debug!("Using model: {}", model);
            self.add_message(Message {
                role: "system".to_string(),
                content: format!("Generating response with {}...", model),
                timestamp: Instant::now(),
            });
            
            // Set generating flag and add loading message
            self.is_generating = true;
            self.add_message(Message {
                role: "system".to_string(),
                content: "⏳ Generating response...".to_string(),
                timestamp: Instant::now(),
            });
            
            debug!("About to generate AI response directly");
            
            // Try direct generation with timeout
            match tokio::time::timeout(
                Duration::from_secs(15), // Shorter timeout
                self.ollama_client.generate(input.clone())
            ).await {
                Ok(Ok(response)) => {
                    debug!("Received response from Ollama: {} chars", response.len());
                    // Reset generating flag and remove loading message
                    self.is_generating = false;
                    if let Some(last_msg) = self.messages.back() {
                        if last_msg.content == "⏳ Generating response..." {
                            self.messages.pop_back();
                        }
                    }
                    self.add_message(Message {
                        role: "assistant".to_string(),
                        content: response,
                        timestamp: Instant::now(),
                    });
                }
                Ok(Err(e)) => {
                    // Reset generating flag and remove loading message
                    self.is_generating = false;
                    if let Some(last_msg) = self.messages.back() {
                        if last_msg.content == "⏳ Generating response..." {
                            self.messages.pop_back();
                        }
                    }
                    self.add_message(Message {
                        role: "error".to_string(),
                        content: format!("❌ Error generating response: {}", e),
                        timestamp: Instant::now(),
                    });
                }
                Err(_) => {
                    // Timeout error
                    self.is_generating = false;
                    if let Some(last_msg) = self.messages.back() {
                        if last_msg.content == "⏳ Generating response..." {
                            self.messages.pop_back();
                        }
                    }
                    self.add_message(Message {
                        role: "error".to_string(),
                        content: "❌ Request timed out after 15 seconds".to_string(),
                        timestamp: Instant::now(),
                    });
                }
            }
        } else {
            self.add_message(Message {
                role: "error".to_string(),
                content: "No model selected. Press F2 to select a model.".to_string(),
                timestamp: Instant::now(),
            });
        }
        
        Ok(())
    }
    
    fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        
        // Keep only last 100 messages to prevent memory issues
        if self.messages.len() > 100 {
            self.messages.pop_front();
        }
    }
    
    fn prepare_ui_data(&self) -> UIData {
        UIData {
            mode: self.mode.clone(),
            messages: self.messages.iter().cloned().collect(),
            input: self.input.clone(),
            current_model: self.ollama_client.get_current_model().map(|s| s.clone()),
            available_models: self.available_models.clone(),
            selected_model_index: self.selected_model_index,
            offline_mode: self.offline_mode,
            is_generating: self.is_generating,
        }
    }
    
    
    
    
}

fn render_ui(f: &mut Frame, ui_data: &UIData) {
    match ui_data.mode {
        AppMode::Chat => render_chat_ui(f, ui_data),
        AppMode::ModelSelector => render_model_selector(f, ui_data),
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
    
    // Messages area
    let messages: Vec<ListItem> = ui_data
        .messages
        .iter()
        .map(|m| {
            let style = match m.role.as_str() {
                "user" => Style::default().fg(Color::Cyan),
                "assistant" => Style::default().fg(Color::Green),
                "system" => Style::default().fg(Color::Yellow),
                "tool" => Style::default().fg(Color::Magenta),
                "error" => Style::default().fg(Color::Red),
                _ => Style::default(),
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}]", m.role), style.add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::raw(&m.content),
            ]))
        })
        .collect();
    
    let messages_list = List::new(messages)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("AI Terminal Chat")
        );
    
    f.render_widget(messages_list, chunks[0]);
    
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
