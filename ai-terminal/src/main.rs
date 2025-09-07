use anyhow::Result;
use clap::Command;
use tracing::info;
use tracing_subscriber;

use terminal_ui::TerminalSession;

mod mcp;
mod config; // Add this line to import the config module

use mcp::{MCPClient, process_ai_command};
use config::Config; // Add this line to import the Config struct

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::load("config.toml")?;
    info!("Loaded configuration: {:?}", config);
    
    let _matches = Command::new("ai-terminal")
        .version("0.1.0")
        .about("AI-powered terminal")
        .get_matches();

    info!("Starting AI Terminal...");

    // Create and configure terminal session
    info!("About to create terminal session");
    let mut terminal_session = TerminalSession::new()?;
    info!("Terminal session created successfully");
    
    info!("About to start terminal application");
    
    // Setup terminal
    let mut terminal = terminal_session.setup_terminal()?;
    
    // Run the application
    let result = terminal_session.run().await;
    
    // Restore terminal
    terminal_session.restore_terminal(&mut terminal)?;
    
    result
}