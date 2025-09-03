use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub digest: String,
    pub modified_at: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: Option<bool>,
    pub options: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
    current_model: Option<String>,
}

impl OllamaClient {
    pub async fn new(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        let client = Client::new();
        
        let mut ollama_client = Self {
            base_url,
            client,
            current_model: None,
        };
        
        // Test connection and get default model
        match ollama_client.get_models().await {
            Ok(models) => {
                if let Some(first_model) = models.first() {
                    ollama_client.current_model = Some(first_model.name.clone());
                    info!("Connected to Ollama, using model: {}", first_model.name);
                }
            }
            Err(e) => {
                warn!("Could not connect to Ollama: {}", e);
            }
        }
        
        Ok(ollama_client)
    }
    
    pub async fn is_available(&self) -> bool {
        self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await
            .is_ok()
    }
    
    pub async fn get_models(&self) -> Result<Vec<OllamaModel>> {
        debug!("Fetching available models from Ollama");
        
        let response = self
            .client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch models: {}", response.status()));
        }
        
        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.models)
    }
    
    pub async fn set_model(&mut self, model_name: String) -> Result<()> {
        debug!("Setting model to: {}", model_name);
        
        let models = self.get_models().await?;
        if models.iter().any(|m| m.name == model_name) {
            self.current_model = Some(model_name.clone());
            info!("Model set to: {}", model_name);
            Ok(())
        } else {
            Err(anyhow!("Model '{}' not found", model_name))
        }
    }
    
    pub fn get_current_model(&self) -> Option<&String> {
        self.current_model.as_ref()
    }
    
    pub async fn generate(&self, prompt: String) -> Result<String> {
        let model = self
            .current_model
            .as_ref()
            .ok_or_else(|| anyhow!("No model selected"))?;
            
        debug!("Generating response with model: {}", model);
        
        let request = GenerateRequest {
            model: model.clone(),
            prompt,
            stream: Some(false),
            options: None,
        };
        
        let response = self
            .client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow!("Generation failed: {}", response.status()));
        }
        
        let generate_response: GenerateResponse = response.json().await?;
        Ok(generate_response.response)
    }
    
    pub async fn generate_stream(&self, prompt: String) -> Result<impl StreamExt<Item = Result<String>>> {
        let model = self
            .current_model
            .as_ref()
            .ok_or_else(|| anyhow!("No model selected"))?;
            
        debug!("Starting streaming generation with model: {}", model);
        
        let request = GenerateRequest {
            model: model.clone(),
            prompt,
            stream: Some(true),
            options: None,
        };
        
        let response = self
            .client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow!("Streaming generation failed: {}", response.status()));
        }
        
        let stream = response.bytes_stream().map(|chunk| {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    // Parse each line as JSON (Ollama streams NDJSON)
                    for line in text.lines() {
                        if !line.trim().is_empty() {
                            if let Ok(response) = serde_json::from_str::<GenerateResponse>(line) {
                                return Ok(response.response);
                            }
                        }
                    }
                    Ok(String::new())
                }
                Err(e) => Err(anyhow!("Stream error: {}", e)),
            }
        });
        
        Ok(stream)
    }
    
    pub async fn pull_model(&self, model_name: String) -> Result<()> {
        info!("Pulling model: {}", model_name);
        
        let request = serde_json::json!({
            "name": model_name
        });
        
        let response = self
            .client
            .post(&format!("{}/api/pull", self.base_url))
            .json(&request)
            .send()
            .await?;
            
        if response.status().is_success() {
            info!("Successfully pulled model: {}", model_name);
            Ok(())
        } else {
            Err(anyhow!("Failed to pull model: {}", response.status()))
        }
    }
    
    pub async fn delete_model(&self, model_name: String) -> Result<()> {
        info!("Deleting model: {}", model_name);
        
        let request = serde_json::json!({
            "name": model_name
        });
        
        let response = self
            .client
            .delete(&format!("{}/api/delete", self.base_url))
            .json(&request)
            .send()
            .await?;
            
        if response.status().is_success() {
            info!("Successfully deleted model: {}", model_name);
            Ok(())
        } else {
            Err(anyhow!("Failed to delete model: {}", response.status()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = OllamaClient::new("http://localhost:11434").await;
        assert!(client.is_ok());
    }
}
