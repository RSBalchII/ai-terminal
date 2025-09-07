//! # MCP Client for AI Terminal
//!
//! This module implements an MCP client that can connect to MCP servers
//! and interact with their tools.

use anyhow::Result;
use jsonrpc_core::{Call, MethodCall, Params, Version};
use jsonrpc_core::serde_json::{self, Value};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tracing::{info, error};

/// MCP client for connecting to MCP servers
pub struct MCPClient {
    /// Child process for the MCP server
    child: Child,
    
    /// stdin handle for sending requests
    stdin: ChildStdin,
    
    /// stdout handle for reading responses
    stdout: BufReader<ChildStdout>,
    
    /// Request ID counter
    id_counter: AtomicU64,
}

/// Parameters for listing tools
#[derive(Debug, Serialize)]
pub struct ListToolsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Parameters for calling a tool
#[derive(Debug, Serialize)]
pub struct CallToolParams {
    pub name: String,
    pub arguments: Value,
}

/// Result of calling a tool
#[derive(Debug, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<TextContent>,
    pub is_error: bool,
}

/// Text content block
#[derive(Debug, Deserialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl MCPClient {
    /// Create a new MCP client
    pub async fn new(server_command: &str) -> Result<Self> {
        info!("Starting MCP server: {}", server_command);
        
        // Spawn the MCP server process
        let mut child = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(server_command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()?;
        
        // Get handles to stdin and stdout
        let stdin = child.stdin.take().ok_or(anyhow::anyhow!("Failed to get stdin"))?;
        let stdout = BufReader::new(child.stdout.take().ok_or(anyhow::anyhow!("Failed to get stdout"))?);
        
        Ok(Self {
            child,
            stdin,
            stdout,
            id_counter: AtomicU64::new(1),
        })
    }
    
    /// List available tools from the MCP server
    pub async fn list_tools(&mut self) -> Result<Value> {
        let params = ListToolsParams { cursor: None };
        let result = self.send_request("tools/list", Some(serde_json::to_value(params)?)).await?;
        Ok(result)
    }
    
    /// Call a specific tool on the MCP server
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<CallToolResult> {
        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };
        let result = self.send_request("tools/call", Some(serde_json::to_value(params)?)).await?;
        let tool_result: CallToolResult = serde_json::from_value(result)?;
        Ok(tool_result)
    }
    
    /// Send a JSON-RPC request to the MCP server
    async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        // Generate a unique request ID
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        
        // Create the JSON-RPC request
        let request = MethodCall {
            jsonrpc: Some(Version::V2),
            method: method.to_string(),
            params: params.map(|p| Params::from(p)).unwrap_or(Params::None),
            id: jsonrpc_core::Id::Num(id),
        };
        
        // Serialize the request to JSON
        let request_json = serde_json::to_string(&request)?;
        info!("Sending request: {}", request_json);
        
        // Send the request to the server
        self.stdin.write_all(request_json.as_bytes()).await?;
        self.stdin.write_all(b"\\n").await?;
        self.stdin.flush().await?;
        
        // Read the response from the server
        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line).await?;
        
        info!("Received response: {}", response_line);
        
        // Parse the response
        let response: jsonrpc_core::Output = serde_json::from_str(&response_line)?;
        
        match response {
            jsonrpc_core::Output::Success(success) => Ok(success.result),
            jsonrpc_core::Output::Failure(failure) => {
                error!("JSON-RPC error: {:?}", failure.error);
                Err(anyhow::anyhow!("JSON-RPC error: {:?}", failure.error))
            }
        }
    }
    
    /// Kill the MCP server process
    pub async fn kill(mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }
}