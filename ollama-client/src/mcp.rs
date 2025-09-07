//! # MCP Server for Ollama
//!
//! This module implements an MCP server that exposes Ollama functionality as MCP tools.

use anyhow::Result;
use jsonrpc_core::{IoHandler, Result as JsonRpcResult};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, error};

use crate::{OllamaClient, OllamaRequest};

/// MCP tool for Ollama chat functionality
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatTool {
    /// The model name to use for generation
    pub model: String,
    
    /// The prompt to send to the model
    pub prompt: String,
    
    /// Optional system message to guide the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

/// MCP tool result for Ollama chat functionality
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatResult {
    /// The model's response
    pub response: String,
}

/// Parameters for listing tools
#[derive(Debug, Deserialize, Serialize)]
pub struct ListToolsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Parameters for calling a tool
#[derive(Debug, Deserialize, Serialize)]
pub struct CallToolParams {
    pub name: String,
    pub arguments: Value,
}

/// Result of calling a tool
#[derive(Debug, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<TextContent>,
    pub is_error: bool,
}

/// Text content block
#[derive(Debug, Serialize, Deserialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Trait defining the MCP methods for the Ollama server
#[rpc]
pub trait OllamaMcp {
    /// List available tools
    #[rpc(name = "tools/list")]
    fn list_tools(&self, params: ListToolsParams) -> JsonRpcResult<Value>;
    
    /// Call a specific tool
    #[rpc(name = "tools/call")]
    fn call_tool(&self, params: CallToolParams) -> JsonRpcResult<CallToolResult>;
}

/// Implementation of the MCP server for Ollama
#[derive(Clone)]
pub struct OllamaMcpServer {
    /// The Ollama client
    client: OllamaClient,
}

impl OllamaMcpServer {
    /// Create a new MCP server
    pub fn new() -> Result<Self> {
        let client = OllamaClient::new()?;
        Ok(Self { client })
    }
    
    /// Get the JSON-RPC I/O handler for this server
    pub fn io_handler(self) -> IoHandler {
        let mut io = IoHandler::new();
        io.extend_with(OllamaMcp::to_delegate(self));
        io
    }
}

impl OllamaMcp for OllamaMcpServer {
    fn list_tools(&self, _params: ListToolsParams) -> JsonRpcResult<Value> {
        // For now, we'll return a simple list of tools
        // In a full implementation, this would dynamically list available Ollama models
        let tools = serde_json::json!({
            "tools": [
                {
                    "name": "ollama_chat",
                    "description": "Send a chat prompt to an Ollama model",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "model": {
                                "type": "string",
                                "description": "The Ollama model to use for generation"
                            },
                            "prompt": {
                                "type": "string",
                                "description": "The prompt to send to the model"
                            },
                            "system": {
                                "type": "string",
                                "description": "Optional system message to guide the model"
                            }
                        },
                        "required": ["model", "prompt"]
                    }
                }
            ]
        });
        
        Ok(tools)
    }
    
    fn call_tool(&self, params: CallToolParams) -> JsonRpcResult<CallToolResult> {
        match params.name.as_str() {
            "ollama_chat" => {
                // Parse the arguments for the ollama_chat tool
                let args: OllamaChatTool = serde_json::from_value(params.arguments)
                    .map_err(|e| jsonrpc_core::Error::invalid_params(format!("Invalid arguments: {}", e)))?;
                
                // For now, we'll return a mock response
                // In a full implementation, this would actually call the Ollama API
                let result = CallToolResult {
                    content: vec![
                        TextContent {
                            content_type: "text".to_string(),
                            text: format!("This is a mock response from Ollama for model '{}' with prompt '{}'", args.model, args.prompt),
                        }
                    ],
                    is_error: false,
                };
                
                Ok(result)
            }
            _ => {
                Ok(CallToolResult {
                    content: vec![
                        TextContent {
                            content_type: "text".to_string(),
                            text: format!("Unknown tool: {}", params.name),
                        }
                    ],
                    is_error: true,
                })
            }
        }
    }
}