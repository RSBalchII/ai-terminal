use thiserror::Error;

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("JSON serialization/deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid response from Ollama API: {0}")]
    InvalidResponse(String),
    
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
}