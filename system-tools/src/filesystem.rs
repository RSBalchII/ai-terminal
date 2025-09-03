use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::warn;
use walkdir::WalkDir;
use regex::Regex;

use crate::ToolResult;

/// File system operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemTool {
    /// List directory contents (ls)
    List { 
        path: String, 
        recursive: bool,
        show_hidden: bool,
    },
    /// Read file contents (cat)
    Read { 
        path: String, 
        lines: Option<(usize, usize)>, // start_line, end_line
        max_size: Option<usize>,
    },
    /// Search in files (grep)
    Search { 
        path: String, 
        pattern: String, 
        recursive: bool,
        case_sensitive: bool,
    },
    /// Find files by name pattern (find)
    Find { 
        path: String, 
        name_pattern: String,
        file_type: Option<String>, // "file", "dir", "link"
    },
    /// Display file information (stat)
    Info { 
        path: String 
    },
    /// Copy file or directory
    Copy { 
        source: String, 
        destination: String 
    },
    /// Move file or directory
    Move { 
        source: String, 
        destination: String 
    },
    /// Write content to file
    Write { 
        path: String, 
        content: String, 
        append: bool 
    },
    /// Delete file or directory
    Delete { 
        path: String, 
        recursive: bool 
    },
}

pub async fn execute_filesystem_tool(tool: FileSystemTool, working_dir: &Path) -> Result<ToolResult> {
    match tool {
        FileSystemTool::List { path, recursive, show_hidden } => {
            list_directory(&path, recursive, show_hidden, working_dir).await
        }
        FileSystemTool::Read { path, lines, max_size } => {
            read_file(&path, lines, max_size, working_dir).await
        }
        FileSystemTool::Search { path, pattern, recursive, case_sensitive } => {
            search_in_files(&path, &pattern, recursive, case_sensitive, working_dir).await
        }
        FileSystemTool::Find { path, name_pattern, file_type } => {
            find_files(&path, &name_pattern, file_type.as_deref(), working_dir).await
        }
        FileSystemTool::Info { path } => {
            get_file_info(&path, working_dir).await
        }
        FileSystemTool::Copy { source, destination } => {
            copy_file_or_dir(&source, &destination, working_dir).await
        }
        FileSystemTool::Move { source, destination } => {
            move_file_or_dir(&source, &destination, working_dir).await
        }
        FileSystemTool::Write { path, content, append } => {
            write_file(&path, &content, append, working_dir).await
        }
        FileSystemTool::Delete { path, recursive } => {
            delete_file_or_dir(&path, recursive, working_dir).await
        }
    }
}

pub fn is_dangerous_filesystem_tool(tool: &FileSystemTool) -> bool {
    matches!(tool, 
        FileSystemTool::Write { .. } | 
        FileSystemTool::Delete { .. } | 
        FileSystemTool::Move { .. }
    )
}

pub fn describe_filesystem_tool(tool: &FileSystemTool) -> String {
    match tool {
        FileSystemTool::List { path, recursive, .. } => {
            if *recursive {
                format!("List contents of {} recursively", path)
            } else {
                format!("List contents of {}", path)
            }
        }
        FileSystemTool::Read { path, lines, .. } => {
            if let Some((start, end)) = lines {
                format!("Read lines {}-{} from {}", start, end, path)
            } else {
                format!("Read contents of {}", path)
            }
        }
        FileSystemTool::Search { path, pattern, recursive, .. } => {
            if *recursive {
                format!("Search for '{}' in {} and subdirectories", pattern, path)
            } else {
                format!("Search for '{}' in {}", pattern, path)
            }
        }
        FileSystemTool::Find { path, name_pattern, .. } => {
            format!("Find files matching '{}' in {}", name_pattern, path)
        }
        FileSystemTool::Info { path } => {
            format!("Get information about {}", path)
        }
        FileSystemTool::Copy { source, destination } => {
            format!("Copy {} to {}", source, destination)
        }
        FileSystemTool::Move { source, destination } => {
            format!("Move {} to {}", source, destination)
        }
        FileSystemTool::Write { path, append, .. } => {
            if *append {
                format!("Append content to {}", path)
            } else {
                format!("Write content to {}", path)
            }
        }
        FileSystemTool::Delete { path, recursive } => {
            if *recursive {
                format!("Delete {} recursively", path)
            } else {
                format!("Delete {}", path)
            }
        }
    }
}

async fn list_directory(
    path: &str, 
    recursive: bool, 
    show_hidden: bool, 
    working_dir: &Path
) -> Result<ToolResult> {
    let full_path = resolve_path(path, working_dir);
    
    if !full_path.exists() {
        return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("Path '{}' does not exist", path)),
            execution_time: std::time::Duration::ZERO,
            metadata: HashMap::new(),
        });
    }

    let mut output = String::new();
    let mut entries = Vec::new();

    if recursive {
        for entry in WalkDir::new(&full_path).min_depth(1) {
            match entry {
                Ok(entry) => {
                    let entry_name = entry.file_name().to_string_lossy();
                    if show_hidden || !entry_name.starts_with('.') {
                        let metadata = entry.metadata().ok();
                        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                        let is_dir = entry.file_type().is_dir();
                        
                        let relative_path = entry.path().strip_prefix(&full_path)
                            .unwrap_or(entry.path());
                        
                        entries.push(format!(
                            "{} {:>10} {}",
                            if is_dir { "d" } else { "-" },
                            if is_dir { "-".to_string() } else { format_size(size) },
                            relative_path.display()
                        ));
                    }
                }
                Err(e) => {
                    warn!("Error reading directory entry: {}", e);
                }
            }
        }
    } else {
        let entries_iter = fs::read_dir(&full_path)?;
        for entry in entries_iter {
            if let Ok(entry) = entry {
                let entry_name = entry.file_name().to_string_lossy().to_string();
                if show_hidden || !entry_name.starts_with('.') {
                    let metadata = entry.metadata().ok();
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                    
                    entries.push(format!(
                        "{} {:>10} {}",
                        if is_dir { "d" } else { "-" },
                        if is_dir { "-".to_string() } else { format_size(size) },
                        entry_name
                    ));
                }
            }
        }
    }

    entries.sort();
    output = entries.join("\n");

    let mut metadata = HashMap::new();
    metadata.insert("entries_count".to_string(), serde_json::json!(entries.len()));
    metadata.insert("path".to_string(), serde_json::json!(full_path.display().to_string()));

    Ok(ToolResult {
        success: true,
        output,
        error: None,
        execution_time: std::time::Duration::ZERO,
        metadata,
    })
}

async fn read_file(
    path: &str, 
    lines: Option<(usize, usize)>, 
    max_size: Option<usize>, 
    working_dir: &Path
) -> Result<ToolResult> {
    let full_path = resolve_path(path, working_dir);
    
    if !full_path.exists() {
        return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("File '{}' does not exist", path)),
            execution_time: std::time::Duration::ZERO,
            metadata: HashMap::new(),
        });
    }

    if full_path.is_dir() {
        return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("'{}' is a directory", path)),
            execution_time: std::time::Duration::ZERO,
            metadata: HashMap::new(),
        });
    }

    // Check file size
    let file_size = full_path.metadata()?.len();
    let max_allowed = max_size.unwrap_or(1024 * 1024); // Default 1MB limit
    
    if file_size > max_allowed as u64 {
        return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("File '{}' is too large ({} bytes, max: {})", 
                path, file_size, max_allowed)),
            execution_time: std::time::Duration::ZERO,
            metadata: HashMap::new(),
        });
    }

    let content = fs::read_to_string(&full_path)?;
    
    let output = if let Some((start_line, end_line)) = lines {
        let lines: Vec<&str> = content.lines().collect();
        let start = start_line.saturating_sub(1); // Convert to 0-based
        let end = end_line.min(lines.len());
        
        if start >= lines.len() {
            String::new()
        } else {
            lines[start..end].join("\n")
        }
    } else {
        content
    };

    let mut metadata = HashMap::new();
    metadata.insert("file_size".to_string(), serde_json::json!(file_size));
    metadata.insert("line_count".to_string(), serde_json::json!(output.lines().count()));

    Ok(ToolResult {
        success: true,
        output,
        error: None,
        execution_time: std::time::Duration::ZERO,
        metadata,
    })
}

async fn search_in_files(
    path: &str, 
    pattern: &str, 
    recursive: bool,
    case_sensitive: bool, 
    working_dir: &Path
) -> Result<ToolResult> {
    let full_path = resolve_path(path, working_dir);
    
    let regex = if case_sensitive {
        Regex::new(pattern)?
    } else {
        Regex::new(&format!("(?i){}", pattern))?
    };

    let mut matches = Vec::new();
    let mut files_searched = 0;

    let search_path = if full_path.is_file() {
        vec![full_path.clone()]
    } else if recursive {
        WalkDir::new(&full_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    } else {
        fs::read_dir(&full_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|e| e.path())
            .collect()
    };

    for file_path in search_path {
        files_searched += 1;
        
        // Skip binary files (basic check)
        if let Ok(mut file) = fs::File::open(&file_path) {
            let mut buffer = [0; 1024];
            if let Ok(bytes_read) = file.read(&mut buffer) {
                if buffer[..bytes_read].iter().any(|&b| b == 0) {
                    continue; // Skip binary files
                }
            }
        }
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    let relative_path = file_path.strip_prefix(&full_path)
                        .unwrap_or(&file_path);
                    matches.push(format!("{}:{}:{}", 
                        relative_path.display(), 
                        line_num + 1, 
                        line.trim()
                    ));
                }
            }
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("files_searched".to_string(), serde_json::json!(files_searched));
    metadata.insert("matches_found".to_string(), serde_json::json!(matches.len()));
    metadata.insert("pattern".to_string(), serde_json::json!(pattern));

    Ok(ToolResult {
        success: true,
        output: matches.join("\n"),
        error: None,
        execution_time: std::time::Duration::ZERO,
        metadata,
    })
}

async fn find_files(
    path: &str, 
    name_pattern: &str, 
    file_type: Option<&str>, 
    working_dir: &Path
) -> Result<ToolResult> {
    let full_path = resolve_path(path, working_dir);
    
    let pattern = name_pattern.replace("*", ".*").replace("?", ".");
    let regex = Regex::new(&format!("^{}$", pattern))?;
    
    let mut found_files = Vec::new();

    for entry in WalkDir::new(&full_path) {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().to_string_lossy();
            
            // Check name pattern
            if !regex.is_match(&file_name) {
                continue;
            }
            
            // Check file type
            let entry_type = entry.file_type();
            let matches_type = match file_type {
                Some("file") => entry_type.is_file(),
                Some("dir") => entry_type.is_dir(),
                Some("link") => entry_type.is_symlink(),
                None => true,
                _ => true,
            };
            
            if matches_type {
                let relative_path = entry.path().strip_prefix(&full_path)
                    .unwrap_or(entry.path());
                found_files.push(relative_path.display().to_string());
            }
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("files_found".to_string(), serde_json::json!(found_files.len()));
    metadata.insert("pattern".to_string(), serde_json::json!(name_pattern));

    Ok(ToolResult {
        success: true,
        output: found_files.join("\n"),
        error: None,
        execution_time: std::time::Duration::ZERO,
        metadata,
    })
}

async fn get_file_info(path: &str, working_dir: &Path) -> Result<ToolResult> {
    let full_path = resolve_path(path, working_dir);
    
    if !full_path.exists() {
        return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("Path '{}' does not exist", path)),
            execution_time: std::time::Duration::ZERO,
            metadata: HashMap::new(),
        });
    }

    let metadata = full_path.metadata()?;
    let mut info = Vec::new();
    
    info.push(format!("Path: {}", full_path.display()));
    info.push(format!("Type: {}", if metadata.is_dir() { "Directory" } 
                     else if metadata.is_symlink() { "Symbolic Link" } 
                     else { "File" }));
    info.push(format!("Size: {} bytes ({})", metadata.len(), format_size(metadata.len())));
    
    if let Ok(modified) = metadata.modified() {
        info.push(format!("Modified: {:?}", modified));
    }
    
    if let Ok(permissions) = std::fs::metadata(&full_path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = permissions.permissions().mode();
            info.push(format!("Permissions: {:o}", mode & 0o777));
        }
    }

    let mut result_metadata = HashMap::new();
    result_metadata.insert("size".to_string(), serde_json::json!(metadata.len()));
    result_metadata.insert("is_dir".to_string(), serde_json::json!(metadata.is_dir()));
    result_metadata.insert("is_file".to_string(), serde_json::json!(metadata.is_file()));

    Ok(ToolResult {
        success: true,
        output: info.join("\n"),
        error: None,
        execution_time: std::time::Duration::ZERO,
        metadata: result_metadata,
    })
}

// Placeholder implementations for dangerous operations
async fn copy_file_or_dir(_source: &str, _destination: &str, _working_dir: &Path) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        output: String::new(),
        error: Some("Copy operation not yet implemented".to_string()),
        execution_time: std::time::Duration::ZERO,
        metadata: HashMap::new(),
    })
}

async fn move_file_or_dir(_source: &str, _destination: &str, _working_dir: &Path) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        output: String::new(),
        error: Some("Move operation not yet implemented".to_string()),
        execution_time: std::time::Duration::ZERO,
        metadata: HashMap::new(),
    })
}

async fn write_file(_path: &str, _content: &str, _append: bool, _working_dir: &Path) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        output: String::new(),
        error: Some("Write operation not yet implemented".to_string()),
        execution_time: std::time::Duration::ZERO,
        metadata: HashMap::new(),
    })
}

async fn delete_file_or_dir(_path: &str, _recursive: bool, _working_dir: &Path) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        output: String::new(),
        error: Some("Delete operation not yet implemented".to_string()),
        execution_time: std::time::Duration::ZERO,
        metadata: HashMap::new(),
    })
}

// Utility functions
fn resolve_path(path: &str, working_dir: &Path) -> PathBuf {
    if path.starts_with('/') {
        PathBuf::from(path)
    } else {
        working_dir.join(path)
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1}KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
