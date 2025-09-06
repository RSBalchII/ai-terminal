use anyhow::Result;
use clap::{Arg, Command};
use tracing::info;
use tracing_subscriber;

use terminal_ui::TerminalSession;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("ai-terminal")
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