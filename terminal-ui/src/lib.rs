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
    io::{self, Stdout},
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};

use ollama_client::OllamaClient;
use python_bridge::PythonBridge;

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
}

impl TerminalApp {
    pub fn new(ollama_client: OllamaClient, python_bridge: PythonBridge) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
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
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key).await?;
                }
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
                    
                    // Add user message
                    self.add_message(Message {
                        role: "user".to_string(),
                        content: user_input.clone(),
                        timestamp: Instant::now(),
                    });
                    
                    // Process with AI
                    self.process_user_input(user_input).await?;
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
            KeyCode::Esc | KeyCode::F(10) => {
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
                    if let Err(e) = self.ollama_client.set_model(model_name.clone()).await {
                        self.add_message(Message {
                            role: "error".to_string(),
                            content: format!("Failed to set model: {}", e),
                            timestamp: Instant::now(),
                        });
                    } else {
                        self.add_message(Message {
                            role: "system".to_string(),
                            content: format!("Model changed to: {}", model_name),
                            timestamp: Instant::now(),
                        });
                    }
                }
                self.mode = AppMode::Chat;
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
        // Check for tool intent first
        if let Ok(Some(tool_call)) = self.python_bridge.recognize_tool_intent(&input) {
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
        if let Some(model) = self.ollama_client.get_current_model() {
            self.add_message(Message {
                role: "system".to_string(),
                content: format!("Generating response with {}...", model),
                timestamp: Instant::now(),
            });
            
            match self.ollama_client.generate(input).await {
                Ok(response) => {
                    self.add_message(Message {
                        role: "assistant".to_string(),
                        content: response,
                        timestamp: Instant::now(),
                    });
                }
                Err(e) => {
                    self.add_message(Message {
                        role: "error".to_string(),
                        content: format!("Error generating response: {}", e),
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
    
    let status_text = format!(
        "Model: {} | Offline: {} | F1: Help | F2: Models | F3: Toggle Offline | F10: Exit",
        current_model,
        if ui_data.offline_mode { "ON" } else { "OFF" }
    );
    
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
