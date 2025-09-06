use anyhow::Result;
use chrono::{DateTime, Local};
use egui::{Color32, FontId, RichText, ScrollArea, TextEdit};
use eframe::egui;
use ollama_client::OllamaClient;
use python_bridge::{PythonBridge, SystemToolRequest, SystemToolResponse};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Local>,
}

impl Message {
    pub fn new(role: String, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: Local::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMode {
    Chat,
    Settings,
    ModelSelector,
}

pub struct AiChatApp {
    // Core components
    ollama_client: OllamaClient,
    python_bridge: PythonBridge,
    
    // UI state
    messages: VecDeque<Message>,
    input_text: String,
    mode: AppMode,
    
    // Model management
    available_models: Vec<String>,
    selected_model_index: usize,
    
    // Status flags
    is_generating: bool,
    offline_mode: bool,
    
    // System tools
    system_tools_tx: Option<Arc<mpsc::UnboundedSender<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>>>,
    
    // UI settings
    show_timestamps: bool,
    word_wrap: bool,
    font_size: f32,
    
    // Window state
    scroll_to_bottom: bool,
}

impl AiChatApp {
    pub async fn new(
        ollama_client: OllamaClient,
        python_bridge: PythonBridge,
    ) -> Result<Self> {
        let mut app = Self {
            ollama_client,
            python_bridge,
            messages: VecDeque::new(),
            input_text: String::new(),
            mode: AppMode::Chat,
            available_models: Vec::new(),
            selected_model_index: 0,
            is_generating: false,
            offline_mode: false,
            system_tools_tx: None,
            show_timestamps: false,
            word_wrap: true,
            font_size: 14.0,
            scroll_to_bottom: false,
        };

        // Add welcome message
        app.add_message(Message::new(
            "system".to_string(),
            "Welcome to AI Terminal GUI! 🎉\nType your message below and press Enter to chat.".to_string(),
        ));

        // Refresh models if possible
        if let Err(e) = app.refresh_models().await {
            warn!("Could not refresh models: {}", e);
        }

        Ok(app)
    }

    pub fn set_system_tools_executor(
        &mut self,
        tx: Arc<mpsc::UnboundedSender<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>>,
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

    fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        self.scroll_to_bottom = true;

        // Keep only last 1000 messages to prevent memory issues
        if self.messages.len() > 1000 {
            self.messages.pop_front();
        }
    }

    async fn process_user_input(&mut self, input: String) -> Result<()> {
        debug!("Processing user input: {}", input);

        // Check for system tool requests first
        if let Some(tool_request) = self.python_bridge.parse_system_tool_request(&input) {
            self.add_message(Message::new(
                "system".to_string(),
                format!("🔧 Executing {} tool: {}", tool_request.tool_type, tool_request.tool_name),
            ));

            match self.python_bridge.execute_system_tool(tool_request).await {
                Ok(response) => {
                    let content = if response.success {
                        format!("✅ Tool executed successfully ({}ms):\n{}", response.execution_time_ms, response.output)
                    } else {
                        format!("❌ Tool execution failed ({}ms):\n{}", 
                               response.execution_time_ms, 
                               response.error.unwrap_or("Unknown error".to_string()))
                    };
                    
                    self.add_message(Message::new("tool".to_string(), content));
                    return Ok(());
                }
                Err(e) => {
                    self.add_message(Message::new(
                        "error".to_string(),
                        format!("❌ System tool error: {}", e),
                    ));
                    return Ok(());
                }
            }
        }

        // Generate AI response
        if let Some(_model) = self.ollama_client.get_current_model() {
            self.is_generating = true;
            self.add_message(Message::new(
                "system".to_string(),
                "⏳ Generating response...".to_string(),
            ));

            // Clone what we need for the async operation
            let ollama_client = self.ollama_client.clone();
            let input_clone = input.clone();

            // Spawn async task for AI generation
            let generation_task = tokio::spawn(async move {
                tokio::time::timeout(
                    std::time::Duration::from_secs(45),
                    ollama_client.generate(input_clone)
                ).await
            });

            match generation_task.await {
                Ok(Ok(Ok(response))) => {
                    // Remove loading message
                    if let Some(last_msg) = self.messages.back() {
                        if last_msg.content == "⏳ Generating response..." {
                            self.messages.pop_back();
                        }
                    }
                    
                    self.add_message(Message::new("assistant".to_string(), response));
                }
                Ok(Ok(Err(e))) => {
                    // Remove loading message and add error
                    if let Some(last_msg) = self.messages.back() {
                        if last_msg.content == "⏳ Generating response..." {
                            self.messages.pop_back();
                        }
                    }
                    self.add_message(Message::new(
                        "error".to_string(),
                        format!("❌ Error generating response: {}", e),
                    ));
                }
                _ => {
                    // Timeout or task error
                    if let Some(last_msg) = self.messages.back() {
                        if last_msg.content == "⏳ Generating response..." {
                            self.messages.pop_back();
                        }
                    }
                    self.add_message(Message::new(
                        "error".to_string(),
                        "❌ Request timed out after 45 seconds".to_string(),
                    ));
                }
            }

            self.is_generating = false;
        } else {
            self.add_message(Message::new(
                "error".to_string(),
                "❌ No model selected. Please select a model first.".to_string(),
            ));
        }

        Ok(())
    }
}

impl eframe::App for AiChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle async operations
        ctx.request_repaint();
        
        // Important: Ensure keyboard events are being processed
        // This helps with input focus issues in some environments
        ctx.set_pixels_per_point(ctx.pixels_per_point());

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::Chat => self.render_chat_ui(ui),
                AppMode::Settings => self.render_settings_ui(ui),
                AppMode::ModelSelector => self.render_model_selector_ui(ui),
            }
        });

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Chat", |ui| {
                    if ui.button("New Conversation").clicked() {
                        self.messages.clear();
                        self.add_message(Message::new(
                            "system".to_string(),
                            "New conversation started!".to_string(),
                        ));
                    }
                    
                    if ui.button("Toggle Offline Mode").clicked() {
                        self.offline_mode = !self.offline_mode;
                        self.add_message(Message::new(
                            "system".to_string(),
                            format!("Offline mode: {}", if self.offline_mode { "ON" } else { "OFF" }),
                        ));
                    }
                });

                ui.menu_button("Models", |ui| {
                    if ui.button("Select Model").clicked() {
                        self.mode = AppMode::ModelSelector;
                    }
                    
                    if ui.button("Refresh Models").clicked() {
                        let rt = tokio::runtime::Handle::current();
                        let mut app_clone = self.clone();
                        rt.spawn(async move {
                            let _ = app_clone.refresh_models().await;
                        });
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_timestamps, "Show Timestamps");
                    ui.checkbox(&mut self.word_wrap, "Word Wrap");
                    
                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0));
                    });
                    
                    if ui.button("Settings").clicked() {
                        self.mode = AppMode::Settings;
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.add_message(Message::new(
                            "system".to_string(),
                            "AI Terminal GUI v0.1.0\nBuilt with Rust + egui\n\nFeatures:\n• Chat with AI models\n• System tools integration\n• Word wrap and scrolling\n• Model selection".to_string(),
                        ));
                    }
                });
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "show_timestamps", &self.show_timestamps);
        eframe::set_value(storage, "word_wrap", &self.word_wrap);
        eframe::set_value(storage, "font_size", &self.font_size);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }
}

impl AiChatApp {
    fn render_chat_ui(&mut self, ui: &mut egui::Ui) {
        // Chat messages area
        let available_height = ui.available_height() - 100.0; // Reserve space for input

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(available_height)
            .stick_to_bottom(self.scroll_to_bottom)
            .show(ui, |ui| {
                for message in &self.messages {
                    self.render_message(ui, message);
                    ui.add_space(5.0);
                }
            });

        if self.scroll_to_bottom {
            self.scroll_to_bottom = false;
        }

        // Input area
        ui.separator();
        
        // Store whether we should send the message
        let mut should_send = false;
        
        ui.horizontal(|ui| {
            // Create a properly focused text edit with an ID for persistence
            let text_edit = TextEdit::multiline(&mut self.input_text)
                .id(egui::Id::new("main_input"))
                .desired_rows(3)
                .desired_width(ui.available_width() - 80.0)
                .hint_text("Type your message here... (Ctrl+Enter to send)")
                .lock_focus(false)  // Don't lock focus
                .interactive(!self.is_generating);  // Make non-interactive while generating
            
            let response = ui.add_enabled(!self.is_generating, text_edit);
            
            // Debug keyboard events in WSL
            #[cfg(debug_assertions)]
            if response.has_focus() {
                ui.ctx().input(|i| {
                    if !i.events.is_empty() {
                        for event in &i.events {
                            debug!("Input event: {:?}", event);
                        }
                    }
                });
            }
            
            // Check for Enter key (without modifiers for single line, Ctrl+Enter for multiline send)
            if response.has_focus() {
                ui.input(|i| {
                    // Allow normal Enter for newlines, Ctrl+Enter to send
                    if i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl {
                        should_send = true;
                    }
                });
            }
            
            // Alternative: Check if text changed and Enter was pressed
            if response.changed() {
                // Text was modified, ensure we're capturing input
                debug!("Text input changed: {}", self.input_text);
            }
            
            // Request focus on the text field if nothing else has focus
            if !self.is_generating {
                // Always try to maintain focus on input field when not generating
                response.request_focus();
            }
            
            // Send button - make it more prominent
            let send_button = ui.add_sized(
                [70.0, 30.0],
                egui::Button::new("📤 Send")
            );
            
            if send_button.clicked() {
                should_send = true;
            }
        });
        
        // Process send action outside of the UI building to avoid borrow conflicts
        if should_send && !self.input_text.trim().is_empty() && !self.is_generating {
            let user_input = self.input_text.trim().to_string();
            self.input_text.clear();
            
            // Add user message
            self.add_message(Message::new("user".to_string(), user_input.clone()));
            
            // Process with AI (async)
            let rt = tokio::runtime::Handle::current();
            let mut app_clone = self.clone();
            rt.spawn(async move {
                let _ = app_clone.process_user_input(user_input).await;
            });
        }
    }

    fn render_message(&self, ui: &mut egui::Ui, message: &Message) {
        let (color, prefix) = match message.role.as_str() {
            "user" => (Color32::LIGHT_BLUE, "👤"),
            "assistant" => (Color32::LIGHT_GREEN, "🤖"),
            "system" => (Color32::YELLOW, "ℹ️"),
            "tool" => (Color32::LIGHT_RED, "🔧"),
            "error" => (Color32::RED, "❌"),
            _ => (Color32::GRAY, "📝"),
        };

        ui.horizontal_wrapped(|ui| {
            // Role indicator
            ui.label(RichText::new(format!("{} [{}]", prefix, message.role))
                .color(color)
                .font(FontId::proportional(self.font_size))
                .strong());

            // Timestamp (if enabled)
            if self.show_timestamps {
                ui.label(RichText::new(format!("({})", message.timestamp.format("%H:%M:%S")))
                    .color(Color32::GRAY)
                    .font(FontId::proportional(self.font_size * 0.8)));
            }
        });

        // Message content
        if self.word_wrap {
            ui.label(RichText::new(&message.content)
                .color(color)
                .font(FontId::proportional(self.font_size)));
        } else {
            ui.monospace(&message.content);
        }

        ui.separator();
    }

    fn render_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        
        ui.horizontal(|ui| {
            if ui.button("← Back to Chat").clicked() {
                self.mode = AppMode::Chat;
            }
        });

        ui.separator();

        egui::Grid::new("settings_grid").show(ui, |ui| {
            ui.label("Show Timestamps:");
            ui.checkbox(&mut self.show_timestamps, "");
            ui.end_row();

            ui.label("Word Wrap:");
            ui.checkbox(&mut self.word_wrap, "");
            ui.end_row();

            ui.label("Font Size:");
            ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0));
            ui.end_row();

            ui.label("Offline Mode:");
            ui.checkbox(&mut self.offline_mode, "");
            ui.end_row();
        });
    }

    fn render_model_selector_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Select AI Model");
        
        ui.horizontal(|ui| {
            if ui.button("← Back to Chat").clicked() {
                self.mode = AppMode::Chat;
            }
            
            if ui.button("🔄 Refresh Models").clicked() {
                let rt = tokio::runtime::Handle::current();
                let mut app_clone = self.clone();
                rt.spawn(async move {
                    let _ = app_clone.refresh_models().await;
                });
            }
        });

        ui.separator();

        if self.available_models.is_empty() {
            ui.label("No models available. Make sure Ollama is running and try refreshing.");
        } else {
            ui.label(format!("Current model: {}", 
                self.ollama_client.get_current_model().map_or("None", |v| v)));
            
            ui.separator();
            
            let available_models = self.available_models.clone();
            let current_model = self.ollama_client.get_current_model().map(|s| s.clone());
            
            ScrollArea::vertical().show(ui, |ui| {
                for (_i, model_name) in available_models.iter().enumerate() {
                    let is_current = current_model
                        .as_ref()
                        .map(|current| current == model_name)
                        .unwrap_or(false);
                    
                    let text = if is_current {
                        RichText::new(format!("✅ {}", model_name)).strong()
                    } else {
                        RichText::new(model_name)
                    };
                    
                    if ui.selectable_label(is_current, text).clicked() && !is_current {
                        // Switch model
                        let rt = tokio::runtime::Handle::current();
                        let mut ollama_client = self.ollama_client.clone();
                        let model_name_clone = model_name.clone();
                        
                        rt.spawn(async move {
                            let _ = ollama_client.set_model(model_name_clone).await;
                        });
                        
                        self.mode = AppMode::Chat;
                        self.add_message(Message::new(
                            "system".to_string(),
                            format!("🔄 Switched to model: {}", model_name),
                        ));
                    }
                }
            });
        }
    }
}

// Make AiChatApp cloneable for async operations
impl Clone for AiChatApp {
    fn clone(&self) -> Self {
        Self {
            ollama_client: self.ollama_client.clone(),
            python_bridge: self.python_bridge.clone(),
            messages: self.messages.clone(),
            input_text: self.input_text.clone(),
            mode: self.mode.clone(),
            available_models: self.available_models.clone(),
            selected_model_index: self.selected_model_index,
            is_generating: self.is_generating,
            offline_mode: self.offline_mode,
            system_tools_tx: self.system_tools_tx.clone(),
            show_timestamps: self.show_timestamps,
            word_wrap: self.word_wrap,
            font_size: self.font_size,
            scroll_to_bottom: self.scroll_to_bottom,
        }
    }
}
