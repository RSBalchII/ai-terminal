use anyhow::Result;
use serde::{Deserialize, Serialize};
use system_tools::{
    SystemTool, ToolExecutor, ToolResult,
    filesystem::FileSystemTool, network::NetworkTool, process::ProcessTool
};
use tracing::{debug, error, info};

/// Represents a tool execution request from the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool_type: String,
    pub tool_name: String,
    pub args: serde_json::Value,
    pub security_level: Option<String>,
}

/// Represents the result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// System tools integration manager
#[derive(Clone)]
pub struct SystemToolsManager {
    executor: ToolExecutor,
}

impl SystemToolsManager {
    /// Create a new system tools manager
    pub fn new() -> Self {
        Self {
            executor: ToolExecutor::new(),
        }
    }

    /// Execute a tool request and return the response
    pub async fn execute_tool_request(&self, request: ToolRequest) -> ToolResponse {
        info!(
            "Executing tool: {} - {} with args: {}", 
            request.tool_type, 
            request.tool_name,
            request.args
        );

        let result = self.execute_tool_internal(request).await;

        match result {
            Ok(tool_result) => {
                debug!("Tool execution successful: {}", tool_result.output);
                ToolResponse {
                    success: tool_result.success,
                    output: tool_result.output,
                    error: tool_result.error,
                    execution_time_ms: tool_result.execution_time.as_millis() as u64,
                }
            }
            Err(e) => {
                error!("Tool execution failed: {}", e);
                ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Execution error: {}", e)),
                    execution_time_ms: 0,
                }
            }
        }
    }

    /// Internal tool execution logic
    async fn execute_tool_internal(&self, request: ToolRequest) -> Result<ToolResult> {
        // Create the appropriate tool based on the request
        let tool = self.create_tool(&request)?;

        // Execute the tool using ToolExecutor
        let result = self.executor.execute(tool).await?;

        Ok(result)
    }

    /// Create a system tool from a tool request
    fn create_tool(&self, request: &ToolRequest) -> Result<SystemTool> {
        match request.tool_type.as_str() {
            "filesystem" => {
                let fs_tool = self.create_filesystem_tool(request)?;
                Ok(SystemTool::FileSystem(fs_tool))
            }
            "network" => {
                let net_tool = self.create_network_tool(request)?;
                Ok(SystemTool::Network(net_tool))
            }
            "process" => {
                let proc_tool = self.create_process_tool(request)?;
                Ok(SystemTool::Process(proc_tool))
            }
            _ => Err(anyhow::anyhow!("Unknown tool type: {}", request.tool_type))
        }
    }

    /// Create a filesystem tool from request
    fn create_filesystem_tool(&self, request: &ToolRequest) -> Result<FileSystemTool> {
        match request.tool_name.as_str() {
            "list_directory" => {
                let path = request.args.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
                Ok(FileSystemTool::List { 
                    path: path.to_string(),
                    recursive: false,
                    show_hidden: false,
                })
            }
            "read_file" => {
                let path = request.args.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
                let lines = request.args.get("lines")
                    .and_then(|v| v.as_u64())
                    .map(|l| (1, l as usize));
                Ok(FileSystemTool::Read { 
                    path: path.to_string(), 
                    lines,
                    max_size: Some(1024 * 1024), // 1MB limit
                })
            }
            "search_content" => {
                let path = request.args.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
                let pattern = request.args.get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'pattern' argument"))?;
                Ok(FileSystemTool::Search { 
                    path: path.to_string(), 
                    pattern: pattern.to_string(),
                    recursive: true,
                    case_sensitive: false,
                })
            }
            "find_files" => {
                let path = request.args.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
                let pattern = request.args.get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'pattern' argument"))?;
                Ok(FileSystemTool::Find { 
                    path: path.to_string(), 
                    name_pattern: pattern.to_string(),
                    file_type: None,
                })
            }
            "file_info" => {
                let path = request.args.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
                Ok(FileSystemTool::Info { path: path.to_string() })
            }
            _ => Err(anyhow::anyhow!("Unknown filesystem tool: {}", request.tool_name))
        }
    }

    /// Create a network tool from request
    fn create_network_tool(&self, request: &ToolRequest) -> Result<NetworkTool> {
        match request.tool_name.as_str() {
            "ping" => {
                let host = request.args.get("host")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'host' argument"))?;
                let count = request.args.get("count")
                    .and_then(|v| v.as_u64())
                    .map(|c| c as u32);
                Ok(NetworkTool::Ping { 
                    host: host.to_string(), 
                    count,
                    timeout: Some(5),
                })
            }
            "resolve_dns" => {
                let hostname = request.args.get("hostname")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'hostname' argument"))?;
                Ok(NetworkTool::Resolve { hostname: hostname.to_string() })
            }
            "http_request" => {
                let url = request.args.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'url' argument"))?;
                let method = request.args.get("method")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Ok(NetworkTool::Curl { 
                    url: url.to_string(), 
                    method,
                    headers: None,
                    body: None,
                    timeout: Some(10),
                })
            }
            "port_scan" => {
                let host = request.args.get("host")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'host' argument"))?;
                let ports = request.args.get("ports")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|p| p.as_u64().map(|u| u as u16)).collect())
                    .unwrap_or_else(|| vec![80, 443, 22, 21, 25]);
                Ok(NetworkTool::PortScan { 
                    host: host.to_string(), 
                    ports,
                    timeout: Some(5),
                })
            }
            "netcat" => {
                let host = request.args.get("host")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'host' argument"))?;
                let port = request.args.get("port")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'port' argument"))?;
                Ok(NetworkTool::Netcat { 
                    host: host.to_string(), 
                    port: port as u16, 
                    mode: system_tools::network::NetcatMode::Test,
                    timeout: Some(5),
                })
            }
            _ => Err(anyhow::anyhow!("Unknown network tool: {}", request.tool_name))
        }
    }

    /// Create a process tool from request
    fn create_process_tool(&self, request: &ToolRequest) -> Result<ProcessTool> {
        match request.tool_name.as_str() {
            "list_processes" => {
                let filter = request.args.get("filter")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Ok(ProcessTool::List { 
                    filter, 
                    show_all: true 
                })
            }
            "process_info" => {
                let pid = request.args.get("pid")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'pid' argument"))?;
                Ok(ProcessTool::Info { pid: pid as u32 })
            }
            _ => Err(anyhow::anyhow!("Unknown process tool: {}", request.tool_name))
        }
    }


    /// Get available tools as JSON for the AI assistant
    pub fn get_available_tools(&self) -> serde_json::Value {
        serde_json::json!({
            "filesystem": {
                "list_directory": {
                    "description": "List contents of a directory",
                    "args": {
                        "path": {"type": "string", "required": true, "description": "Directory path to list"}
                    }
                },
                "read_file": {
                    "description": "Read contents of a file",
                    "args": {
                        "path": {"type": "string", "required": true, "description": "File path to read"},
                        "lines": {"type": "number", "required": false, "description": "Number of lines to read"}
                    }
                },
                "search_content": {
                    "description": "Search for pattern in files",
                    "args": {
                        "path": {"type": "string", "required": true, "description": "Directory path to search in"},
                        "pattern": {"type": "string", "required": true, "description": "Pattern to search for"}
                    }
                },
                "find_files": {
                    "description": "Find files matching pattern",
                    "args": {
                        "path": {"type": "string", "required": true, "description": "Directory path to search in"},
                        "pattern": {"type": "string", "required": true, "description": "File name pattern"}
                    }
                },
                "file_info": {
                    "description": "Get information about a file",
                    "args": {
                        "path": {"type": "string", "required": true, "description": "File path"}
                    }
                }
            },
            "network": {
                "ping": {
                    "description": "Ping a host",
                    "args": {
                        "host": {"type": "string", "required": true, "description": "Host to ping"},
                        "count": {"type": "number", "required": false, "description": "Number of pings"}
                    }
                },
                "resolve_dns": {
                    "description": "Resolve DNS hostname",
                    "args": {
                        "hostname": {"type": "string", "required": true, "description": "Hostname to resolve"}
                    }
                },
                "http_request": {
                    "description": "Make HTTP request",
                    "args": {
                        "url": {"type": "string", "required": true, "description": "URL to request"},
                        "method": {"type": "string", "required": false, "description": "HTTP method (GET, POST, etc.)"}
                    }
                },
                "port_scan": {
                    "description": "Scan ports on a host",
                    "args": {
                        "host": {"type": "string", "required": true, "description": "Host to scan"},
                        "ports": {"type": "array", "required": false, "description": "List of ports to scan"}
                    }
                },
                "netcat": {
                    "description": "Test network connectivity",
                    "args": {
                        "host": {"type": "string", "required": true, "description": "Host to connect to"},
                        "port": {"type": "number", "required": true, "description": "Port to connect to"}
                    }
                }
            },
            "process": {
                "list_processes": {
                    "description": "List running processes",
                    "args": {}
                },
                "process_info": {
                    "description": "Get information about a process",
                    "args": {
                        "pid": {"type": "number", "required": true, "description": "Process ID"}
                    }
                }
            }
        })
    }
}

impl Default for SystemToolsManager {
    fn default() -> Self {
        Self::new()
    }
}
