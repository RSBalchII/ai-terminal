use anyhow::Result;
use clap::{Arg, Command};
use eframe::egui;
use egui_ui::AiChatApp;
use ollama_client::OllamaClient;
use python_bridge::{PythonBridge, SystemToolRequest, SystemToolResponse};
use tracing::info;
use tracing_subscriber;

#[path = "system_tools_integration.rs"]
mod system_tools_integration;
use system_tools_integration::SystemToolsManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("ai-terminal-gui")
        .version("0.1.0")
        .about("AI-powered terminal GUI with Ollama integration")
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .help("Specify the Ollama model to use")
                .value_name("MODEL")
        )
        .arg(
            Arg::new("offline")
                .long("offline")
                .help("Run in offline mode without internet connectivity")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    info!("Starting AI Terminal GUI...");

    // Initialize components
    let model = matches.get_one::<String>("model");
    let offline_mode = matches.get_flag("offline");

    // Create Ollama client
    let ollama_client = OllamaClient::new("http://localhost:11434").await?;
    
    // Initialize Python bridge for agent pipeline
    let python_bridge = PythonBridge::new()?;
    info!("Python bridge initialized successfully");
    
    // Initialize system tools manager
    let system_tools_manager = SystemToolsManager::new();
    info!("System tools manager initialized");
    
    // Create system tools executor channel
    let (tools_tx, mut tools_rx) = tokio::sync::mpsc::unbounded_channel::<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>();
    let tools_tx_arc = std::sync::Arc::new(tools_tx);
    info!("System tools channel created");
    
    // Spawn system tools executor task
    let system_tools_manager_clone = system_tools_manager.clone();
    tokio::spawn(async move {
        info!("System tools executor task started");
        while let Some((request, response_tx)) = tools_rx.recv().await {
            info!("Received system tool request: {} - {}", request.tool_type, request.tool_name);
            
            // Convert from Python bridge type to integration type
            let tool_request = system_tools_integration::ToolRequest {
                tool_type: request.tool_type,
                tool_name: request.tool_name,
                args: request.args,
                security_level: request.security_level,
            };
            
            let response = system_tools_manager_clone.execute_tool_request(tool_request).await;
            info!("System tool executed, success: {}", response.success);
            
            // Convert back to Python bridge type
            let bridge_response = SystemToolResponse {
                success: response.success,
                output: response.output,
                error: response.error,
                execution_time_ms: response.execution_time_ms,
            };
            
            if let Err(_) = response_tx.send(bridge_response) {
                tracing::warn!("Failed to send system tool response: receiver dropped");
            }
        }
        info!("System tools executor task ended");
    });

    // Create the GUI application
    let mut app = AiChatApp::new(ollama_client, python_bridge).await?;
    info!("AI Chat GUI application created successfully");
    
    // Set system tools executor
    app.set_system_tools_executor(tools_tx_arc);
    info!("System tools executor configured");

    // Set model if specified
    if let Some(model_name) = model {
        info!("Setting model to: {}", model_name);
        // Note: We'll need to handle this after the GUI starts
    }

    // Configure native options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(
                // Load icon from bytes if you have one
                eframe::icon_data::from_png_bytes(&[]).unwrap_or_default(),
            ),
        default_theme: eframe::Theme::Dark, // Start with dark theme
        ..Default::default()
    };

    info!("Starting GUI application");
    
    // Run the GUI application
    eframe::run_native(
        "AI Terminal",
        options,
        Box::new(|cc| {
            // This gives us image support and better fonts:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            // Set up custom fonts if desired
            let fonts = egui::FontDefinitions::default();
            cc.egui_ctx.set_fonts(fonts);
            
            // Configure input handling for better WSL/X11 compatibility
            cc.egui_ctx.set_pixels_per_point(1.0);
            
            // Enable text input events explicitly
            cc.egui_ctx.options_mut(|o| {
                o.screen_reader = false;  // Disable screen reader which can interfere
                o.preload_font_glyphs = true;  // Preload font glyphs for better performance
            });
            
            // Storage will be handled by the app's save/load methods
            // The app will automatically restore its settings

            Ok(Box::new(app))
        }),
    ).map_err(|e| anyhow::anyhow!("GUI application error: {}", e))?;

    Ok(())
}
