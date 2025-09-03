use anyhow::Result;
use clap::{Arg, Command};
use tracing::info;
use tracing_subscriber;

use ollama_client::OllamaClient;
use python_bridge::PythonBridge;
use terminal_ui::TerminalApp;

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
    
    // Create and run terminal application
    let mut terminal_app = TerminalApp::new(ollama_client, python_bridge)?;
    
    if let Some(model_name) = model {
        terminal_app.set_model(model_name.clone()).await?;
    }
    
    terminal_app.set_offline_mode(offline_mode);
    terminal_app.run().await?;

    Ok(())
}
