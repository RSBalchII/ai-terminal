use serde::{Deserialize, Serialize};

/// Request structure for the Ollama API
#[derive(Debug, Serialize, Clone)]
pub struct OllamaRequest {
    /// The model name to use for generation
    pub model: String,
    
    /// The prompt to send to the model
    pub prompt: String,
    
    /// Optional system message to guide the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    
    /// Optional template to use for the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    
    /// Optional context to continue a conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
    
    /// Optional stream flag (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// Optional raw mode flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,
}

impl OllamaRequest {
    /// Create a new OllamaRequest
    pub fn new(model: String, prompt: String) -> Self {
        Self {
            model,
            prompt,
            system: None,
            template: None,
            context: None,
            stream: Some(true), // Default to streaming
            raw: None,
        }
    }
    
    /// Set the system message
    pub fn with_system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }
    
    /// Set the context
    pub fn with_context(mut self, context: Vec<i32>) -> Self {
        self.context = Some(context);
        self
    }
}

/// Response structure from the Ollama API (for streaming)
#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    /// The model name that generated the response
    pub model: String,
    
    /// The time the response was created
    pub created_at: String,
    
    /// The response content
    pub response: String,
    
    /// Whether this is the final response
    pub done: bool,
    
    /// Context for continuing the conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
    
    /// Total duration of the request
    pub total_duration: Option<u64>,
    
    /// Duration of loading the model
    pub load_duration: Option<u64>,
    
    /// Number of evaluations
    pub eval_count: Option<u32>,
    
    /// Duration of evaluations
    pub eval_duration: Option<u64>,
}