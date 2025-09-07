//! # Ollama API Client
//!
//! This module provides the main client for interacting with the Ollama API.
//! It handles sending requests, managing streaming responses, and maintaining
//! conversation history.

use crate::{error::OllamaError, models::{OllamaRequest, OllamaResponse}, history::ConversationHistory};
use reqwest::{Client, Response};
use serde_json::Value;
use std::env;
use tracing::{info, error};
use futures_util::StreamExt;

/// The default base URL for the Ollama API
const DEFAULT_BASE_URL: &str = "http://localhost:11434/api";

/// The main client for interacting with the Ollama API
#[derive(Debug, Clone)]
pub struct OllamaClient {
    /// The HTTP client used for requests
    http_client: Client,
    
    /// The base URL for the Ollama API
    pub base_url: String,
    
    /// The model to use for requests
    pub model: String,
    
    /// The conversation history
    pub history: ConversationHistory,
}

impl OllamaClient {
    /// Create a new OllamaClient with default settings
    pub fn new() -> Result<Self, OllamaError> {
        let base_url = env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        
        let client = Client::new();
        let history = ConversationHistory::new(10); // Default to 10 messages of history
        
        Ok(Self {
            http_client: client,
            base_url,
            model: "llama3".to_string(), // Default model
            history,
        })
    }
    
    /// Create a new OllamaClient with a specific model
    pub fn with_model(model: String) -> Result<Self, OllamaError> {
        let mut client = Self::new()?;
        client.model = model;
        Ok(client)
    }
    
    /// Send a request to the Ollama API
    pub async fn send_request(&self, request: OllamaRequest) -> Result<Response, OllamaError> {
        let url = format!("{}/generate", self.base_url);
        info!("Sending request to: {}", url);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;
            
        Ok(response)
    }
    
    /// Send a request to the Ollama API and stream the response
    pub async fn stream_request(&self, request: OllamaRequest) -> Result<impl StreamExt<Item = Result<OllamaResponse, OllamaError>>, OllamaError> {
        let response = self.send_request(request).await?;
        
        // Check if the response is successful
        if !response.status().is_success() {
            return Err(OllamaError::RequestFailed(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }
        
        // Create a stream from the response bytes
        let stream = response.bytes_stream()
            .map(|result| {
                match result {
                    Ok(bytes) => {
                        // Try to parse the bytes as a JSON object
                        match serde_json::from_slice::<OllamaResponse>(&bytes) {
                            Ok(response) => Ok(response),
                            Err(e) => Err(OllamaError::JsonError(e)),
                        }
                    },
                    Err(e) => Err(OllamaError::RequestFailed(e)),
                }
            });
            
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ollama_client_with_config() {
        // This test would require a running Ollama instance to work properly
        // For now, we'll just test that we can create a client with a specific model
        let client = OllamaClient::with_model("llama3".to_string());
        assert!(client.is_ok());
    }
}