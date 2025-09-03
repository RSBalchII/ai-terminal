use anyhow::Result;
use clap::{Arg, Command};
use tracing::info;
use tracing_subscriber;
use python_bridge::{SystemToolRequest, SystemToolResponse};

use ollama_client::OllamaClient;
use python_bridge::PythonBridge;
use terminal_ui::TerminalApp;

mod system_tools_integration;
use system_tools_integration::SystemToolsManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("ai-terminal")
        .version("0.1.0")
        .about("AI-powered terminal with Ollama integration")
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

    info!("Starting AI Terminal...");

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
    info!("About to spawn system tools executor task");
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
    
    // Create and configure terminal application
    info!("About to create terminal application");
    let mut terminal_app = TerminalApp::new(ollama_client, python_bridge)?;
    info!("Terminal application created successfully");
    terminal_app.set_system_tools_executor(tools_tx_arc);
    info!("System tools executor configured");
    
    if let Some(model_name) = model {
        info!("Setting model to: {}", model_name);
        terminal_app.set_model(model_name.clone()).await?;
    }
    
    terminal_app.set_offline_mode(offline_mode);
    info!("About to start terminal application");
    terminal_app.run().await?;

    Ok(())
}
