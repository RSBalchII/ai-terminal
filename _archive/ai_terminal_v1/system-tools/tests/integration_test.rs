use system_tools::*;
use anyhow::Result;

#[tokio::test]
async fn test_filesystem_tools() -> Result<()> {
    let executor = ToolExecutor::new();
    
    // Test listing current directory
    let list_tool = SystemTool::FileSystem(FileSystemTool::List {
        path: ".".to_string(),
        recursive: false,
        show_hidden: false,
    });
    
    let result = executor.execute(list_tool).await?;
    assert!(result.success, "List directory should succeed");
    assert!(!result.output.is_empty(), "Should have directory contents");
    
    // Test reading a file that exists (Cargo.toml)
    let read_tool = SystemTool::FileSystem(FileSystemTool::Read {
        path: "Cargo.toml".to_string(),
        lines: Some((1, 5)),
        max_size: None,
    });
    
    let result = executor.execute(read_tool).await?;
    assert!(result.success, "Read file should succeed");
    assert!(result.output.contains("[package]"), "Should read Cargo.toml content");
    
    Ok(())
}

#[tokio::test]
async fn test_process_tools() -> Result<()> {
    let executor = ToolExecutor::new();
    
    // Test listing processes
    let ps_tool = SystemTool::Process(ProcessTool::List {
        filter: None,
        show_all: false,
    });
    
    let result = executor.execute(ps_tool).await?;
    assert!(result.success, "Process list should succeed");
    assert!(result.output.contains("PID"), "Should have process header");
    
    Ok(())
}

#[tokio::test]
async fn test_network_tools() -> Result<()> {
    let executor = ToolExecutor::new();
    
    // Test ping to localhost
    let ping_tool = SystemTool::Network(NetworkTool::Ping {
        host: "127.0.0.1".to_string(),
        count: Some(1),
        timeout: Some(2),
    });
    
    let result = executor.execute(ping_tool).await?;
    assert!(result.success, "Ping to localhost should succeed");
    assert!(result.output.contains("bytes from"), "Should have ping response");
    
    Ok(())
}

#[tokio::test]
async fn test_security_restrictions() -> Result<()> {
    let executor = ToolExecutor::new();
    
    // Test that writing files is blocked by default
    let write_tool = SystemTool::FileSystem(FileSystemTool::Write {
        path: "/tmp/test_file.txt".to_string(),
        content: "test content".to_string(),
        append: false,
    });
    
    let result = executor.execute(write_tool).await;
    assert!(result.is_err(), "Write should be blocked by security");
    
    // Test that netcat is blocked by default
    let nc_tool = SystemTool::Network(NetworkTool::Netcat {
        host: "localhost".to_string(),
        port: 11434,
        mode: system_tools::network::NetcatMode::Test,
        timeout: Some(1),
    });
    
    let result = executor.execute(nc_tool).await;
    assert!(result.is_err(), "Netcat should be blocked by security");
    
    Ok(())
}

#[tokio::test]  
async fn test_search_functionality() -> Result<()> {
    let executor = ToolExecutor::new();
    
    // Search for a known pattern in the current directory
    let search_tool = SystemTool::FileSystem(FileSystemTool::Search {
        path: ".".to_string(),
        pattern: "system-tools".to_string(),  // The package name
        recursive: false,
        case_sensitive: false,
    });
    
    let result = executor.execute(search_tool).await?;
    assert!(result.success, "Search should succeed");
    // The pattern should be found in Cargo.toml
    assert!(!result.output.is_empty(), "Should find the pattern somewhere");
    
    Ok(())
}

#[tokio::test]
async fn test_tool_execution_completes() -> Result<()> {
    let executor = ToolExecutor::new();
    
    // Test that a simple command completes successfully
    let ping_tool = SystemTool::Network(NetworkTool::Ping {
        host: "127.0.0.1".to_string(),
        count: Some(1),
        timeout: Some(2),
    });
    
    let result = executor.execute(ping_tool).await?;
    assert!(result.success, "Simple ping should complete successfully");
    
    Ok(())
}
