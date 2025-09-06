use std::env;

use futures_util::StreamExt;
use reqwest::{Client, Response};
use tracing::{debug, error, info};

use crate::error::OllamaError;
use crate::models::{OllamaRequest, OllamaResponse};

/// The base URL for the Ollama API
const OLLAMA_API_BASE: &str = "http://localhost:11434/api";

/// A client for interacting with the Ollama API
pub struct OllamaClient {
    /// The HTTP client
    client: Client,
    
    /// The base URL for the Ollama API
    base_url: String,
}

impl OllamaClient {
    /// Create a new OllamaClient
    pub fn new() -> Result<Self, OllamaError> {
        // Check if OLLAMA_HOST is set in environment variables
        let base_url = env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| OLLAMA_API_BASE.to_string());
        
        info!("Initializing Ollama client with base URL: {}", base_url);
        
        Ok(Self {
            client: Client::new(),
            base_url,
        })
    }
    
    /// Send a request to the Ollama API and return a stream of responses
    pub async fn chat(
        &self,
        request: OllamaRequest,
    ) -> Result<impl futures_util::Stream<Item = Result<OllamaResponse, OllamaError>> + Unpin, OllamaError> {
        let url = format!("{}/generate", self.base_url);
        debug!("Sending request to: {}", url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(OllamaError::RequestFailed)?;
        
        // Check if the response status is successful
        if !response.status().is_success() {
            error!("Request failed with status: {}", response.status());
            return Err(OllamaError::InvalidResponse(format!(
                "Request failed with status: {}",
                response.status()
            )));
        }
        
        // Create a stream from the response
        Ok(Box::pin(self.stream_response(response)))
    }
    
    /// Stream the response from the Ollama API
    fn stream_response(
        &self,
        response: Response,
    ) -> impl futures_util::Stream<Item = Result<OllamaResponse, OllamaError>> {
        async_stream::stream! {
            let mut stream = response.bytes_stream();
            
            // Buffer for incomplete JSON lines
            let mut buffer = String::new();
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        // Convert bytes to string and append to buffer
                        let chunk_str = match std::str::from_utf8(&bytes) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Failed to convert bytes to string: {}", e);
                                yield Err(OllamaError::IoError(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Invalid UTF-8 sequence"
                                )));
                                continue;
                            }
                        };
                        
                        buffer.push_str(chunk_str);
                        
                        // Process complete lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            // Extract the line (including the newline)
                            let line = buffer[..=newline_pos].to_string();
                            // Remove the processed line from the buffer
                            buffer = buffer[newline_pos + 1..].to_string();
                            
                            // Skip empty lines
                            if line.trim().is_empty() {
                                continue;
                            }
                            
                            // Try to deserialize the line
                            match serde_json::from_str::<OllamaResponse>(&line) {
                                Ok(response) => {
                                    yield Ok(response);
                                }
                                Err(e) => {
                                    error!("Failed to deserialize response line: {}", e);
                                    error!("Problematic line: {:?}", line);
                                    yield Err(OllamaError::JsonError(e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read response chunk: {}", e);
                        yield Err(OllamaError::RequestFailed(e));
                    }
                }
            }
            
            // Process any remaining data in the buffer
            if !buffer.is_empty() {
                match serde_json::from_str::<OllamaResponse>(&buffer) {
                    Ok(response) => {
                        yield Ok(response);
                    }
                    Err(e) => {
                        error!("Failed to deserialize remaining buffer: {}", e);
                        error!("Remaining buffer: {:?}", buffer);
                        yield Err(OllamaError::JsonError(e));
                    }
                }
            }
        }
    }
}