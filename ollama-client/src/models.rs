//! # Ollama Data Models
//!
//! This module defines the data models used for requests and responses
//! when interacting with the Ollama API.

use serde::{Deserialize, Serialize};

/// A request to the Ollama API
#[derive(Debug, Serialize, Clone)]
pub struct OllamaRequest {
    /// The model to use for generation
    pub model: String,
    
    /// The prompt to send to the model
    pub prompt: String,
    
    /// Optional system message to guide the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    
    /// Optional context from previous interactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
    
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl OllamaRequest {
    /// Create a new OllamaRequest with default settings
    pub fn new(model: String, prompt: String) -> Self {
        Self {
            model,
            prompt,
            system: None,
            context: None,
            stream: Some(true), // Default to streaming
        }
    }
    
    /// Create a new OllamaRequest with a system message
    pub fn with_system(model: String, prompt: String, system: String) -> Self {
        Self {
            model,
            prompt,
            system: Some(system),
            context: None,
            stream: Some(true), // Default to streaming
        }
    }
    
    /// Create a new OllamaRequest with context
    pub fn with_context(model: String, prompt: String, context: Vec<i32>) -> Self {
        Self {
            model,
            prompt,
            system: None,
            context: Some(context),
            stream: Some(true), // Default to streaming
        }
    }
}

/// A response from the Ollama API
#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    /// The model used for generation
    pub model: String,
    
    /// The time the response was created
    pub created_at: String,
    
    /// The response content
    pub response: String,
    
    /// Whether this is the final response
    pub done: bool,
    
    /// Optional context for continuing the conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ollama_request_with_system_message() {
        let request = OllamaRequest::with_system(
            "llama3".to_string(),
            "Hello, world!".to_string(),
            "You are a helpful assistant.".to_string()
        );
        
        assert_eq!(request.model, "llama3");
        assert_eq!(request.prompt, "Hello, world!");
        assert_eq!(request.system, Some("You are a helpful assistant.".to_string()));
    }
}