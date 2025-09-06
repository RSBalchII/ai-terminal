use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{System, Pid};
use tracing::debug;

use crate::ToolResult;

/// Process management operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessTool {
    /// List running processes (ps)
    List { 
        filter: Option<String>, 
        show_all: bool,
    },
    /// Get detailed process information
    Info { 
        pid: u32 
    },
    /// Kill a process
    Kill { 
        pid: u32, 
        signal: Option<String>, // TERM, KILL, HUP, etc.
        force: bool,
    },
    /// Send signal to process
    Signal { 
        pid: u32, 
        signal: String 
    },
}

pub async fn execute_process_tool(tool: ProcessTool) -> Result<ToolResult> {
    match tool {
        ProcessTool::List { filter, show_all } => {
            list_processes(filter, show_all).await
        }
        ProcessTool::Info { pid } => {
            get_process_info(pid).await
        }
        ProcessTool::Kill { pid, signal, force } => {
            kill_process(pid, signal, force).await
        }
        ProcessTool::Signal { pid, signal } => {
            send_signal(pid, &signal).await
        }
    }
}

pub fn is_dangerous_process_tool(tool: &ProcessTool) -> bool {
    matches!(tool, 
        ProcessTool::Kill { .. } | 
        ProcessTool::Signal { .. }
    )
}

pub fn describe_process_tool(tool: &ProcessTool) -> String {
    match tool {
        ProcessTool::List { filter, show_all } => {
            if let Some(filter) = filter {
                format!("List processes matching '{}'", filter)
            } else if *show_all {
                "List all processes".to_string()
            } else {
                "List user processes".to_string()
            }
        }
        ProcessTool::Info { pid } => {
            format!("Get information for process {}", pid)
        }
        ProcessTool::Kill { pid, signal, force } => {
            let sig = signal.as_deref().unwrap_or("TERM");
            if *force {
                format!("Force kill process {} with {}", pid, sig)
            } else {
                format!("Kill process {} with {}", pid, sig)
            }
        }
        ProcessTool::Signal { pid, signal } => {
            format!("Send {} signal to process {}", signal, pid)
        }
    }
}

async fn list_processes(filter: Option<String>, show_all: bool) -> Result<ToolResult> {
    debug!("Listing processes (filter: {:?}, show_all: {})", filter, show_all);
    
    let mut system = System::new();
    system.refresh_all();
    
    let mut processes = Vec::new();
    let _current_uid = unsafe { libc::getuid() };
    
    for (pid, process) in system.processes() {
        // For now, show all processes (user filtering can be added later)
        // if !show_all {
        //     // User filtering logic would go here
        // }
        
        let name = process.name();
        let cmd = process.cmd().join(" ");
        
        // Apply filter if provided
        if let Some(ref filter_str) = filter {
            if !name.to_lowercase().contains(&filter_str.to_lowercase()) &&
               !cmd.to_lowercase().contains(&filter_str.to_lowercase()) {
                continue;
            }
        }
        
        let cpu_usage = process.cpu_usage();
        let memory = process.memory();
        let status = format!("{:?}", process.status());
        
        processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name: name.to_string(),
            cmd: if cmd.is_empty() { name.to_string() } else { cmd },
            cpu_usage,
            memory_kb: memory / 1024, // Convert to KB
            status,
        });
    }
    
    // Sort by CPU usage (descending) then by memory usage
    processes.sort_by(|a, b| {
        b.cpu_usage.partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.memory_kb.cmp(&a.memory_kb))
    });
    
    // Format output
    let mut output = format!("{:>8} {:>6} {:>8} {:>10} {:>8} {}\n",
        "PID", "CPU%", "MEM(KB)", "STATUS", "NAME", "COMMAND");
    output.push_str(&"-".repeat(80));
    output.push('\n');
    
    for proc in processes.iter().take(50) { // Limit to top 50 processes
        output.push_str(&format!("{:>8} {:>6.1} {:>8} {:>10} {:>8} {}\n",
            proc.pid,
            proc.cpu_usage,
            proc.memory_kb,
            proc.status,
            proc.name,
            if proc.cmd.len() > 40 { 
                format!("{}...", &proc.cmd[..37]) 
            } else { 
                proc.cmd.clone() 
            }
        ));
    }
    
    let mut metadata = HashMap::new();
    metadata.insert("total_processes".to_string(), serde_json::json!(processes.len()));
    metadata.insert("displayed".to_string(), serde_json::json!(processes.len().min(50)));
    metadata.insert("filter".to_string(), serde_json::json!(filter));
    metadata.insert("show_all".to_string(), serde_json::json!(show_all));
    
    Ok(ToolResult {
        success: true,
        output,
        error: None,
        execution_time: std::time::Duration::ZERO,
        metadata,
    })
}

async fn get_process_info(pid: u32) -> Result<ToolResult> {
    debug!("Getting process info for PID: {}", pid);
    
    let mut system = System::new();
    system.refresh_process(Pid::from_u32(pid));
    
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        let mut info = Vec::new();
        
        info.push(format!("PID: {}", pid));
        info.push(format!("Name: {}", process.name()));
        info.push(format!("Status: {:?}", process.status()));
        info.push(format!("CPU Usage: {:.1}%", process.cpu_usage()));
        info.push(format!("Memory: {} KB", process.memory() / 1024));
        info.push(format!("Virtual Memory: {} KB", process.virtual_memory() / 1024));
        
        if let Some(parent) = process.parent() {
            info.push(format!("Parent PID: {}", parent.as_u32()));
        }
        
        let start_time = process.start_time();
        info.push(format!("Start Time: {} seconds since epoch", start_time));
        
        let cmd = process.cmd();
        if !cmd.is_empty() {
            info.push(format!("Command: {}", cmd.join(" ")));
        }
        
        let env = process.environ();
        if !env.is_empty() && env.len() < 20 { // Only show env if reasonable size
            info.push("Environment:".to_string());
            for var in env.iter().take(10) {
                info.push(format!("  {}", var));
            }
        }
        
        let mut metadata = HashMap::new();
        metadata.insert("pid".to_string(), serde_json::json!(pid));
        metadata.insert("name".to_string(), serde_json::json!(process.name()));
        metadata.insert("cpu_usage".to_string(), serde_json::json!(process.cpu_usage()));
        metadata.insert("memory_kb".to_string(), serde_json::json!(process.memory() / 1024));
        
        Ok(ToolResult {
            success: true,
            output: info.join("\n"),
            error: None,
            execution_time: std::time::Duration::ZERO,
            metadata,
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("Process with PID {} not found", pid)),
            execution_time: std::time::Duration::ZERO,
            metadata: HashMap::new(),
        })
    }
}

async fn kill_process(pid: u32, signal: Option<String>, force: bool) -> Result<ToolResult> {
    let signal = if force {
        "KILL".to_string()
    } else {
        signal.unwrap_or_else(|| "TERM".to_string())
    };
    
    debug!("Killing process {} with signal {}", pid, signal);
    
    // For safety, we'll implement this as a placeholder for now
    // In a real implementation, you'd want proper permission checking
    Ok(ToolResult {
        success: false,
        output: String::new(),
        error: Some("Kill operation disabled for security reasons".to_string()),
        execution_time: std::time::Duration::ZERO,
        metadata: HashMap::new(),
    })
}

async fn send_signal(pid: u32, signal: &str) -> Result<ToolResult> {
    debug!("Sending {} signal to process {}", signal, pid);
    
    // For safety, we'll implement this as a placeholder for now
    Ok(ToolResult {
        success: false,
        output: String::new(),
        error: Some("Signal sending disabled for security reasons".to_string()),
        execution_time: std::time::Duration::ZERO,
        metadata: HashMap::new(),
    })
}

// Helper struct for process information
#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cmd: String,
    cpu_usage: f32,
    memory_kb: u64,
    status: String,
}
