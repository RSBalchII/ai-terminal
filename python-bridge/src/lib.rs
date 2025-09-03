use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub persona: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemToolRequest {
    pub tool_type: String,
    pub tool_name: String,
    pub args: serde_json::Value,
    pub security_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemToolResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Clone)]
pub struct PythonBridge {
    agent_pipeline: Vec<AgentConfig>,
    python_path: String,
    system_tools_executor: Option<Arc<tokio::sync::mpsc::UnboundedSender<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>>>,
}

impl PythonBridge {
    pub fn new() -> Result<Self> {
        // Initialize Python interpreter
        pyo3::prepare_freethreaded_python();
        
        Python::with_gil(|py| {
            let python_path = match std::env::current_dir()?
                .join("../cli-terminal-ai")
                .canonicalize() {
                    Ok(path) => path.to_string_lossy().to_string(),
                    Err(e) => {
                        info!("Python project path not found: {}, continuing without Python integration", e);
                        // Use current directory as fallback
                        std::env::current_dir()?.to_string_lossy().to_string()
                    }
                };
                
            info!("Initializing Python bridge with path: {}", python_path);
            
            // Try to add the Python project path to sys.path
            if let Ok(sys) = py.import_bound("sys") {
                if let Ok(path) = sys.getattr("path") {
                    let _ = path.call_method1("append", (&python_path,));
                }
            }
            
            Ok(Self {
                agent_pipeline: Vec::new(),
                python_path,
                system_tools_executor: None,
            })
        })
    }
    
    pub fn load_config(&mut self, config_path: &str) -> Result<()> {
        Python::with_gil(|py| {
            let _yaml_module = py.import_bound("yaml")?;
            
            // Read and parse the config file
            let config_content = std::fs::read_to_string(config_path)?;
            let config: serde_json::Value = serde_yaml::from_str(&config_content)
                .map_err(|e| anyhow!("Failed to parse YAML config: {}", e))?;
            
            // Extract agent pipeline
            if let Some(pipeline) = config.get("agent_pipeline") {
                if let Some(pipeline_array) = pipeline.as_array() {
                    self.agent_pipeline.clear();
                    for agent in pipeline_array {
                        if let (Some(name), Some(persona)) = (agent.get("name"), agent.get("persona")) {
                            self.agent_pipeline.push(AgentConfig {
                                name: name.as_str().unwrap_or("").to_string(),
                                persona: persona.as_str().unwrap_or("").to_string(),
                                model: agent.get("model").and_then(|m| m.as_str().map(|s| s.to_string())),
                            });
                        }
                    }
                }
            }
            
            info!("Loaded {} agents from config", self.agent_pipeline.len());
            Ok(())
        })
    }
    
    pub fn get_agent_pipeline(&self) -> &[AgentConfig] {
        &self.agent_pipeline
    }
    
    pub fn recognize_tool_intent(&self, prompt: &str) -> Result<Option<ToolCall>> {
        debug!("Python bridge: recognize_tool_intent called with: {}", prompt);
        // For now, return None to avoid complex type conversions
        // This will be implemented properly once we have the full integration working
        debug!("Python bridge: returning None for tool intent");
        Ok(None)
    }
    
    pub fn dispatch_tool_call(&self, _tool_call: &ToolCall) -> Result<serde_json::Value> {
        // For now, return a simple result to avoid complex type conversions
        // This will be implemented properly once we have the full integration working
        Ok(serde_json::json!({"result": "Tool executed successfully (placeholder)"}))
    }
    
    pub fn execute_agent_pipeline(&self, prompt: &str, _conversation_history: &[serde_json::Value]) -> Result<String> {
        Python::with_gil(|_py| {
            debug!("Executing agent pipeline with {} agents", self.agent_pipeline.len());
            
            let current_data = prompt.to_string();
            
            for (_i, agent) in self.agent_pipeline.iter().enumerate() {
                debug!("Processing with agent: {}", agent.name);
                
                // Here you would call your Python agent processing logic
                // For now, we'll simulate with a simple pass-through
                // In the full implementation, you'd integrate with your existing agent system
                
                // This would be replaced with actual agent processing:
                // current_data = self.process_agent(agent, &current_data, conversation_history)?;
            }
            
            Ok(current_data)
        })
    }
    
    pub fn is_python_available(&self) -> bool {
        Python::with_gil(|py| {
            py.import_bound("llm_cli.main").is_ok()
        })
    }
    
    /// Set the system tools executor channel
    pub fn set_system_tools_executor(
        &mut self, 
        executor: Arc<tokio::sync::mpsc::UnboundedSender<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>>
    ) {
        self.system_tools_executor = Some(executor);
    }
    
    /// Execute a system tool via the executor channel
    pub async fn execute_system_tool(&self, request: SystemToolRequest) -> Result<SystemToolResponse> {
        let executor = self.system_tools_executor.as_ref()
            .ok_or_else(|| anyhow!("System tools executor not initialized"))?;
            
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        executor.send((request, response_tx))
            .map_err(|e| anyhow!("Failed to send tool request: {}", e))?;
            
        response_rx.await
            .map_err(|e| anyhow!("Failed to receive tool response: {}", e))
    }
    
    /// Parse user input to detect system tool requests
    pub fn parse_system_tool_request(&self, input: &str) -> Option<SystemToolRequest> {
        // Simple pattern matching for tool requests
        // In a real implementation, this would be more sophisticated
        
        let input = input.trim();
        
        // Check for filesystem commands (exact matches only)
        if input.starts_with("ls ") && input.len() > 3 {
            let path = input.split_whitespace().nth(1).unwrap_or(".").to_string();
            return Some(SystemToolRequest {
                tool_type: "filesystem".to_string(),
                tool_name: "list_directory".to_string(),
                args: serde_json::json!({"path": path}),
                security_level: Some("medium".to_string()),
            });
        }
        
        if input.starts_with("cat ") && input.len() > 4 {
            let path = input.split_whitespace().nth(1).unwrap_or("").to_string();
            if !path.is_empty() && !path.contains(" ") {
                return Some(SystemToolRequest {
                    tool_type: "filesystem".to_string(),
                    tool_name: "read_file".to_string(),
                    args: serde_json::json!({"path": path}),
                    security_level: Some("medium".to_string()),
                });
            }
        }
        
        if input.starts_with("find ") || input.starts_with("search ") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() >= 3 {
                let path = parts[1].to_string();
                let pattern = parts[2].to_string();
                return Some(SystemToolRequest {
                    tool_type: "filesystem".to_string(),
                    tool_name: "find_files".to_string(),
                    args: serde_json::json!({"path": path, "pattern": pattern}),
                    security_level: Some("medium".to_string()),
                });
            }
        }
        
        // Check for network commands
        if input.starts_with("ping ") {
            let host = input.split_whitespace().nth(1).unwrap_or("").to_string();
            if !host.is_empty() {
                return Some(SystemToolRequest {
                    tool_type: "network".to_string(),
                    tool_name: "ping".to_string(),
                    args: serde_json::json!({"host": host, "count": 4}),
                    security_level: Some("medium".to_string()),
                });
            }
        }
        
        // Check for process commands
        if input == "ps" || input == "processes" {
            return Some(SystemToolRequest {
                tool_type: "process".to_string(),
                tool_name: "list_processes".to_string(),
                args: serde_json::json!({}),
                security_level: Some("low".to_string()),
            });
        }
        
        None
    }
    
    /// Get available system tools for the AI assistant
    pub fn get_system_tools_info(&self) -> serde_json::Value {
        serde_json::json!({
            "description": "System tools available for execution",
            "tools": {
                "filesystem": {
                    "list_directory": "List contents of a directory",
                    "read_file": "Read contents of a file",
                    "search_content": "Search for patterns in files",
                    "find_files": "Find files matching pattern",
                    "file_info": "Get file information"
                },
                "network": {
                    "ping": "Ping a host",
                    "resolve_dns": "Resolve DNS hostname",
                    "http_request": "Make HTTP request",
                    "port_scan": "Scan ports on a host",
                    "netcat": "Test network connectivity"
                },
                "process": {
                    "list_processes": "List running processes",
                    "process_info": "Get process information"
                }
            },
            "usage": {
                "filesystem": "Use commands like 'ls /path', 'cat file.txt', 'find /path pattern'",
                "network": "Use commands like 'ping hostname'",
                "process": "Use commands like 'ps' to list processes"
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_bridge_creation() {
        let bridge = PythonBridge::new();
        assert!(bridge.is_ok());
    }
}
