use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use crate::SystemTool;

/// Security levels for tool execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum SecurityLevel {
    /// Safe operations that only read data
    Safe,
    /// Operations that might modify user data
    Moderate,
    /// Operations that can modify system state or be destructive
    Dangerous,
}

/// Security configuration for tool execution
#[derive(Debug, Clone)]
pub struct ToolSecurity {
    /// Maximum security level allowed without confirmation
    pub max_auto_level: SecurityLevel,
    /// Set of explicitly allowed tools
    pub allowed_tools: HashSet<String>,
    /// Set of explicitly blocked tools
    pub blocked_tools: HashSet<String>,
    /// Whether to require confirmation for dangerous operations
    pub require_confirmation: bool,
    /// Paths that are off-limits for file operations
    pub restricted_paths: HashSet<String>,
}

impl Default for ToolSecurity {
    fn default() -> Self {
        let mut restricted_paths = HashSet::new();
        restricted_paths.insert("/etc/passwd".to_string());
        restricted_paths.insert("/etc/shadow".to_string());
        restricted_paths.insert("/boot".to_string());
        restricted_paths.insert("/sys".to_string());
        restricted_paths.insert("/proc".to_string());

        Self {
            max_auto_level: SecurityLevel::Moderate,
            allowed_tools: HashSet::new(),
            blocked_tools: HashSet::new(),
            require_confirmation: true,
            restricted_paths,
        }
    }
}

impl ToolSecurity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn permissive() -> Self {
        Self {
            max_auto_level: SecurityLevel::Dangerous,
            allowed_tools: HashSet::new(),
            blocked_tools: HashSet::new(),
            require_confirmation: false,
            restricted_paths: HashSet::new(),
        }
    }

    pub fn restrictive() -> Self {
        Self {
            max_auto_level: SecurityLevel::Safe,
            allowed_tools: HashSet::new(),
            blocked_tools: HashSet::new(),
            require_confirmation: true,
            restricted_paths: Self::default().restricted_paths,
        }
    }

    pub fn check_permissions(&self, tool: &SystemTool) -> Result<()> {
        let tool_name = self.get_tool_name(tool);
        let security_level = self.get_security_level(tool);

        // Check if tool is explicitly blocked
        if self.blocked_tools.contains(&tool_name) {
            return Err(anyhow!("Tool '{}' is blocked by security policy", tool_name));
        }

        // Check if tool is explicitly allowed
        if self.allowed_tools.contains(&tool_name) {
            return Ok(());
        }

        // Check security level
        match security_level {
            SecurityLevel::Safe => Ok(()),
            SecurityLevel::Moderate => {
                if self.max_auto_level >= SecurityLevel::Moderate {
                    Ok(())
                } else {
                    Err(anyhow!("Tool '{}' requires Moderate security level", tool_name))
                }
            }
            SecurityLevel::Dangerous => {
                if self.max_auto_level >= SecurityLevel::Dangerous {
                    Ok(())
                } else {
                    Err(anyhow!("Tool '{}' requires Dangerous security level and confirmation", tool_name))
                }
            }
        }
    }

    pub fn check_path_access(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();
        
        for restricted in &self.restricted_paths {
            if path_str.starts_with(restricted) {
                return Err(anyhow!("Access to path '{}' is restricted", path_str));
            }
        }
        
        Ok(())
    }

    fn get_tool_name(&self, tool: &SystemTool) -> String {
        match tool {
            SystemTool::FileSystem(fs_tool) => format!("fs_{:?}", fs_tool).to_lowercase(),
            SystemTool::Network(net_tool) => format!("net_{:?}", net_tool).to_lowercase(),
            SystemTool::Process(proc_tool) => format!("proc_{:?}", proc_tool).to_lowercase(),
        }
    }

    fn get_security_level(&self, tool: &SystemTool) -> SecurityLevel {
        match tool {
            SystemTool::FileSystem(fs_tool) => {
                use crate::filesystem::FileSystemTool;
                match fs_tool {
                    FileSystemTool::List { .. } | 
                    FileSystemTool::Read { .. } | 
                    FileSystemTool::Info { .. } => SecurityLevel::Safe,
                    
                    FileSystemTool::Search { .. } | 
                    FileSystemTool::Find { .. } => SecurityLevel::Safe,
                    
                    FileSystemTool::Write { .. } | 
                    FileSystemTool::Delete { .. } => SecurityLevel::Dangerous,
                    
                    FileSystemTool::Copy { .. } | 
                    FileSystemTool::Move { .. } => SecurityLevel::Moderate,
                }
            }
            SystemTool::Network(net_tool) => {
                use crate::network::NetworkTool;
                match net_tool {
                    NetworkTool::Ping { .. } | 
                    NetworkTool::Resolve { .. } => SecurityLevel::Safe,
                    
                    NetworkTool::Curl { .. } | 
                    NetworkTool::PortScan { .. } => SecurityLevel::Moderate,
                    
                    NetworkTool::Netcat { .. } => SecurityLevel::Dangerous,
                }
            }
            SystemTool::Process(proc_tool) => {
                use crate::process::ProcessTool;
                match proc_tool {
                    ProcessTool::List { .. } | 
                    ProcessTool::Info { .. } => SecurityLevel::Safe,
                    
                    ProcessTool::Kill { .. } | 
                    ProcessTool::Signal { .. } => SecurityLevel::Dangerous,
                }
            }
        }
    }
}
