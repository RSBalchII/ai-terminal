//! # AI Command Processor
//!
//! This module handles processing of AI commands (prefixed with '/') in the terminal.

use anyhow::Result;
use tracing::{info, error};

use ollama_client::{OllamaClient, OllamaRequest, OllamaResponse};
use terminal_emulator::CommandBlock;
use futures_util::StreamExt;

/// Process an AI command and return the response
pub async fn process_ai_command(client: &OllamaClient, command: &str) -> Result<String> {
    // Remove the leading '/' from the command
    let prompt = command[1..].trim();
    
    // Get the last context from history
    let context = client.history.get_last_context();
    
    // Create the request
    let request = if let Some(context) = context {
        OllamaRequest::with_context(client.model.clone(), prompt.to_string(), context)
    } else {
        OllamaRequest::new(client.model.clone(), prompt.to_string())
    };
    
    // Send the request and get the response
    info!("Sending AI request: {}", prompt);
    let response = client.send_request(request).await?;
    
    // Check if the response is successful
    if !response.status().is_success() {
        error!("Received non-success status code: {}", response.status());
        return Err(anyhow::anyhow!("Received non-success status code: {}", response.status()));
    }
    
    // Collect the full response
    let mut full_response = String::new();
    
    // Process the streaming response
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                // Try to parse the bytes as a JSON object
                match serde_json::from_slice::<OllamaResponse>(&bytes) {
                    Ok(response) => {
                        full_response.push_str(&response.response);
                    },
                    Err(e) => {
                        error!("Failed to parse response: {}", e);
                    }
                }
            },
            Err(e) => {
                error!("Error receiving response: {}", e);
                return Err(anyhow::anyhow!("Error receiving response: {}", e));
            }
        }
    }
    
    info!("Received AI response: {}", full_response);
    Ok(full_response)
}