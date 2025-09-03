use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing::debug;

use crate::ToolResult;

/// Network operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkTool {
    /// Test connectivity (ping)
    Ping { 
        host: String, 
        count: Option<u32>,
        timeout: Option<u32>, // seconds
    },
    /// Resolve hostname to IP
    Resolve { 
        hostname: String 
    },
    /// Make HTTP request (curl-like)
    Curl { 
        url: String, 
        method: Option<String>, 
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
        timeout: Option<u32>,
    },
    /// Netcat functionality (nc)
    Netcat { 
        host: String, 
        port: u16, 
        mode: NetcatMode,
        timeout: Option<u32>,
    },
    /// Port scanning
    PortScan { 
        host: String, 
        ports: Vec<u16>,
        timeout: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetcatMode {
    /// Test port connectivity
    Test,
    /// Send data and close
    Send(String),
    /// Listen mode (dangerous)
    Listen,
}

pub async fn execute_network_tool(tool: NetworkTool) -> Result<ToolResult> {
    match tool {
        NetworkTool::Ping { host, count, timeout } => {
            ping_host(&host, count, timeout).await
        }
        NetworkTool::Resolve { hostname } => {
            resolve_hostname(&hostname).await
        }
        NetworkTool::Curl { url, method, headers, body, timeout } => {
            curl_request(&url, method, headers, body, timeout).await
        }
        NetworkTool::Netcat { host, port, mode, timeout } => {
            netcat_operation(&host, port, mode, timeout).await
        }
        NetworkTool::PortScan { host, ports, timeout } => {
            port_scan(&host, &ports, timeout).await
        }
    }
}

pub fn is_dangerous_network_tool(tool: &NetworkTool) -> bool {
    matches!(tool, 
        NetworkTool::Netcat { mode: NetcatMode::Listen, .. } |
        NetworkTool::Netcat { mode: NetcatMode::Send(_), .. }
    )
}

pub fn describe_network_tool(tool: &NetworkTool) -> String {
    match tool {
        NetworkTool::Ping { host, count, .. } => {
            if let Some(c) = count {
                format!("Ping {} ({} times)", host, c)
            } else {
                format!("Ping {} (4 times)", host)
            }
        }
        NetworkTool::Resolve { hostname } => {
            format!("Resolve hostname {}", hostname)
        }
        NetworkTool::Curl { url, method, .. } => {
            let method = method.as_deref().unwrap_or("GET");
            format!("{} request to {}", method, url)
        }
        NetworkTool::Netcat { host, port, mode, .. } => {
            match mode {
                NetcatMode::Test => format!("Test connection to {}:{}", host, port),
                NetcatMode::Send(_) => format!("Send data to {}:{}", host, port),
                NetcatMode::Listen => format!("Listen on port {}", port),
            }
        }
        NetworkTool::PortScan { host, ports, .. } => {
            if ports.len() == 1 {
                format!("Scan port {} on {}", ports[0], host)
            } else {
                format!("Scan {} ports on {}", ports.len(), host)
            }
        }
    }
}

async fn ping_host(host: &str, count: Option<u32>, timeout: Option<u32>) -> Result<ToolResult> {
    let count = count.unwrap_or(4);
    let timeout = timeout.unwrap_or(5);
    
    debug!("Pinging {} {} times with {}s timeout", host, count, timeout);
    
    // Use system ping command for accurate results
    let output = Command::new("ping")
        .arg("-c")
        .arg(count.to_string())
        .arg("-W")
        .arg(timeout.to_string())
        .arg(host)
        .output()
        .await;

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            let mut metadata = HashMap::new();
            metadata.insert("host".to_string(), serde_json::json!(host));
            metadata.insert("count".to_string(), serde_json::json!(count));
            metadata.insert("exit_code".to_string(), serde_json::json!(output.status.code()));
            
            // Parse basic statistics from ping output
            if let Some(stats_line) = stdout.lines().find(|line| line.contains("packet loss")) {
                if let Some(loss_str) = stats_line.split_whitespace()
                    .find(|word| word.ends_with('%')) {
                    metadata.insert("packet_loss".to_string(), serde_json::json!(loss_str));
                }
            }

            Ok(ToolResult {
                success: output.status.success(),
                output: stdout.to_string(),
                error: if stderr.is_empty() { None } else { Some(stderr.to_string()) },
                execution_time: Duration::ZERO,
                metadata,
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute ping: {}", e)),
                execution_time: Duration::ZERO,
                metadata: HashMap::new(),
            })
        }
    }
}

async fn resolve_hostname(hostname: &str) -> Result<ToolResult> {
    debug!("Resolving hostname: {}", hostname);
    
    let start_time = Instant::now();
    
    // Try to resolve using std library
    match format!("{}:80", hostname).to_socket_addrs() {
        Ok(addrs) => {
            let ip_addresses: Vec<String> = addrs
                .map(|addr| addr.ip().to_string())
                .collect();
            
            let mut metadata = HashMap::new();
            metadata.insert("hostname".to_string(), serde_json::json!(hostname));
            metadata.insert("addresses_found".to_string(), serde_json::json!(ip_addresses.len()));
            
            let output = if ip_addresses.is_empty() {
                format!("No addresses found for {}", hostname)
            } else {
                format!("{} resolves to:\n{}", hostname, ip_addresses.join("\n"))
            };
            
            Ok(ToolResult {
                success: !ip_addresses.is_empty(),
                output,
                error: None,
                execution_time: start_time.elapsed(),
                metadata,
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to resolve {}: {}", hostname, e)),
                execution_time: start_time.elapsed(),
                metadata: HashMap::new(),
            })
        }
    }
}

async fn curl_request(
    url: &str, 
    method: Option<String>, 
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    timeout: Option<u32>
) -> Result<ToolResult> {
    let method = method.unwrap_or_else(|| "GET".to_string());
    let timeout = timeout.unwrap_or(30);
    
    debug!("Making {} request to {}", method, url);
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout as u64))
        .build()?;
    
    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        "PATCH" => client.patch(url),
        _ => return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("Unsupported HTTP method: {}", method)),
            execution_time: Duration::ZERO,
            metadata: HashMap::new(),
        }),
    };
    
    // Add headers if provided
    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.header(&key, &value);
        }
    }
    
    // Add body if provided
    if let Some(body) = body {
        request = request.body(body);
    }
    
    let start_time = Instant::now();
    
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let headers: HashMap<String, String> = response.headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            
            let body = response.text().await.unwrap_or_else(|_| "Failed to read response body".to_string());
            
            let mut metadata = HashMap::new();
            metadata.insert("status_code".to_string(), serde_json::json!(status.as_u16()));
            metadata.insert("headers".to_string(), serde_json::json!(headers));
            metadata.insert("method".to_string(), serde_json::json!(method));
            metadata.insert("url".to_string(), serde_json::json!(url));
            
            let output = format!("HTTP {} {}\n\nHeaders:\n{}\n\nBody:\n{}", 
                status.as_u16(), 
                status.canonical_reason().unwrap_or("Unknown"),
                headers.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n"),
                if body.len() > 1000 { 
                    format!("{}... (truncated, {} bytes total)", &body[..1000], body.len()) 
                } else { 
                    body 
                }
            );
            
            Ok(ToolResult {
                success: status.is_success(),
                output,
                error: if status.is_success() { None } else { Some(format!("HTTP {}", status)) },
                execution_time: start_time.elapsed(),
                metadata,
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Request failed: {}", e)),
                execution_time: start_time.elapsed(),
                metadata: HashMap::new(),
            })
        }
    }
}

async fn netcat_operation(
    host: &str, 
    port: u16, 
    mode: NetcatMode, 
    timeout: Option<u32>
) -> Result<ToolResult> {
    let timeout = timeout.unwrap_or(5);
    
    match mode {
        NetcatMode::Test => {
            debug!("Testing connection to {}:{}", host, port);
            
            let start_time = Instant::now();
            
            // Try to establish TCP connection
            match std::net::TcpStream::connect_timeout(
                &format!("{}:{}", host, port).parse::<SocketAddr>()?,
                Duration::from_secs(timeout as u64)
            ) {
                Ok(_) => {
                    let mut metadata = HashMap::new();
                    metadata.insert("host".to_string(), serde_json::json!(host));
                    metadata.insert("port".to_string(), serde_json::json!(port));
                    metadata.insert("status".to_string(), serde_json::json!("open"));
                    
                    Ok(ToolResult {
                        success: true,
                        output: format!("Connection to {}:{} succeeded", host, port),
                        error: None,
                        execution_time: start_time.elapsed(),
                        metadata,
                    })
                }
                Err(e) => {
                    let mut metadata = HashMap::new();
                    metadata.insert("host".to_string(), serde_json::json!(host));
                    metadata.insert("port".to_string(), serde_json::json!(port));
                    metadata.insert("status".to_string(), serde_json::json!("closed"));
                    
                    Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Connection to {}:{} failed: {}", host, port, e)),
                        execution_time: start_time.elapsed(),
                        metadata,
                    })
                }
            }
        }
        NetcatMode::Send(_data) => {
            // For security, we'll implement this as a placeholder for now
            Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Send mode not implemented for security reasons".to_string()),
                execution_time: Duration::ZERO,
                metadata: HashMap::new(),
            })
        }
        NetcatMode::Listen => {
            // Listening mode is dangerous and not implemented
            Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Listen mode not implemented for security reasons".to_string()),
                execution_time: Duration::ZERO,
                metadata: HashMap::new(),
            })
        }
    }
}

async fn port_scan(host: &str, ports: &[u16], timeout: Option<u32>) -> Result<ToolResult> {
    let timeout = timeout.unwrap_or(3);
    
    debug!("Scanning {} ports on {}", ports.len(), host);
    
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut open_ports = Vec::new();
    
    for &port in ports {
        let address = format!("{}:{}", host, port);
        
        match std::net::TcpStream::connect_timeout(
            &address.parse::<SocketAddr>()?,
            Duration::from_secs(timeout as u64)
        ) {
            Ok(_) => {
                results.push(format!("{}: open", port));
                open_ports.push(port);
            }
            Err(_) => {
                results.push(format!("{}: closed", port));
            }
        }
    }
    
    let mut metadata = HashMap::new();
    metadata.insert("host".to_string(), serde_json::json!(host));
    metadata.insert("ports_scanned".to_string(), serde_json::json!(ports.len()));
    metadata.insert("open_ports".to_string(), serde_json::json!(open_ports));
    metadata.insert("timeout".to_string(), serde_json::json!(timeout));
    
    Ok(ToolResult {
        success: true,
        output: results.join("\n"),
        error: None,
        execution_time: start_time.elapsed(),
        metadata,
    })
}
