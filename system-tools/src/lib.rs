use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

pub mod filesystem;
pub mod network;
pub mod process;
pub mod security;

pub use filesystem::FileSystemTool;
pub use network::NetworkTool;
pub use process::ProcessTool;
pub use security::{SecurityLevel, ToolSecurity};

/// Main system tool enum that encompasses all tool categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemTool {
    FileSystem(FileSystemTool),
    Network(NetworkTool),
    Process(ProcessTool),
}

/// Result of executing a system tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time: Duration,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool execution context and settings
#[derive(Debug, Clone)]
pub struct ToolExecutor {
    pub security: ToolSecurity,
    pub timeout: Duration,
    pub working_directory: PathBuf,
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            security: ToolSecurity::default(),
            timeout: Duration::from_secs(30),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        }
    }

    pub async fn execute(&self, tool: SystemTool) -> Result<ToolResult> {
        let start_time = SystemTime::now();
        
        // Security check
        self.security.check_permissions(&tool)?;
        
        debug!("Executing tool: {:?}", tool);
        
        // Execute with timeout
        let result = tokio::time::timeout(self.timeout, self.run_tool(tool)).await?;
        
        let execution_time = start_time.elapsed().unwrap_or(Duration::ZERO);
        
        match result {
            Ok(mut tool_result) => {
                tool_result.execution_time = execution_time;
                info!("Tool executed successfully in {:?}", execution_time);
                Ok(tool_result)
            }
            Err(e) => {
                warn!("Tool execution failed: {}", e);
                Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    execution_time,
                    metadata: HashMap::new(),
                })
            }
        }
    }

    async fn run_tool(&self, tool: SystemTool) -> Result<ToolResult> {
        match tool {
            SystemTool::FileSystem(fs_tool) => {
                filesystem::execute_filesystem_tool(fs_tool, &self.working_directory).await
            }
            SystemTool::Network(net_tool) => {
                network::execute_network_tool(net_tool).await
            }
            SystemTool::Process(proc_tool) => {
                process::execute_process_tool(proc_tool).await
            }
        }
    }

    pub fn is_dangerous(&self, tool: &SystemTool) -> bool {
        match tool {
            SystemTool::FileSystem(fs_tool) => filesystem::is_dangerous_filesystem_tool(fs_tool),
            SystemTool::Network(net_tool) => network::is_dangerous_network_tool(net_tool),
            SystemTool::Process(proc_tool) => process::is_dangerous_process_tool(proc_tool),
        }
    }

    pub fn get_description(&self, tool: &SystemTool) -> String {
        match tool {
            SystemTool::FileSystem(fs_tool) => filesystem::describe_filesystem_tool(fs_tool),
            SystemTool::Network(net_tool) => network::describe_network_tool(net_tool),
            SystemTool::Process(proc_tool) => process::describe_process_tool(proc_tool),
        }
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}
