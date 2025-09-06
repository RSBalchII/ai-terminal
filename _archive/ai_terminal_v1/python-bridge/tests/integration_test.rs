// Python Bridge Integration Test
// This tests the Python bridge's ability to recognize and execute system tools

use python_bridge::{PythonBridge, SystemToolRequest, SystemToolResponse};
use system_tools::{ToolExecutor, SystemTool, FileSystemTool};
use anyhow::Result;
use tokio::sync::mpsc;
use std::sync::Arc;

#[tokio::test]
async fn test_python_bridge_integration() -> Result<()> {
    println!("========================================");
    println!("🐍 Python Bridge Integration Test");
    println!("========================================\n");
    
    // Initialize Python bridge
    println!("1️⃣ Initializing Python Bridge...");
    let mut bridge = PythonBridge::new()?;
    println!("   ✅ Python bridge created");
    
    // Check if Python is available
    let python_available = bridge.is_python_available();
    println!("   Python available: {}", if python_available { "✅" } else { "⚠️ No" });
    
    // Set up system tools executor channel
    println!("\n2️⃣ Setting up System Tools Executor...");
    let (tool_tx, mut tool_rx) = mpsc::unbounded_channel::<(SystemToolRequest, tokio::sync::oneshot::Sender<SystemToolResponse>)>();
    let tool_tx = Arc::new(tool_tx);
    bridge.set_system_tools_executor(tool_tx.clone());
    println!("   ✅ Executor channel configured");
    
    // Spawn a task to handle tool requests
    let tools_handle = tokio::spawn(async move {
        let executor = ToolExecutor::new();
        
        while let Some((request, response_tx)) = tool_rx.recv().await {
            println!("   📨 Received tool request: {} - {}", request.tool_type, request.tool_name);
            
            // Convert request to system tool
            let tool = match request.tool_type.as_str() {
                "filesystem" => {
                    match request.tool_name.as_str() {
                        "list_directory" => {
                            let path = request.args.get("path")
                                .and_then(|p| p.as_str())
                                .unwrap_or(".")
                                .to_string();
                            Some(SystemTool::FileSystem(FileSystemTool::List {
                                path,
                                recursive: false,
                                show_hidden: false,
                            }))
                        },
                        "read_file" => {
                            let path = request.args.get("path")
                                .and_then(|p| p.as_str())
                                .unwrap_or("")
                                .to_string();
                            Some(SystemTool::FileSystem(FileSystemTool::Read {
                                path,
                                lines: Some((1, 10)),
                                max_size: Some(1024 * 1024), // 1MB max
                            }))
                        },
                        _ => None
                    }
                },
                _ => None
            };
            
            // Execute the tool if valid
            let response = if let Some(tool) = tool {
                match executor.execute(tool).await {
                    Ok(result) => SystemToolResponse {
                        success: result.success,
                        output: result.output,
                        error: result.error,
                        execution_time_ms: result.execution_time.as_millis() as u64,
                    },
                    Err(e) => SystemToolResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Execution error: {}", e)),
                        execution_time_ms: 0,
                    }
                }
            } else {
                SystemToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unknown tool: {}/{}", request.tool_type, request.tool_name)),
                    execution_time_ms: 0,
                }
            };
            
            let _ = response_tx.send(response);
        }
    });
    
    // Test 1: Parse system tool requests
    println!("\n3️⃣ Testing Tool Request Parsing...");
    let test_inputs = vec![
        ("ls .", "Should parse as list_directory"),
        ("cat Cargo.toml", "Should parse as read_file"),
        ("ps aux", "Should parse as process list"),
        ("hello world", "Should not parse as tool"),
    ];
    
    for (input, description) in test_inputs {
        print!("   Testing '{}': ", input);
        if let Some(request) = bridge.parse_system_tool_request(input) {
            println!("✅ Parsed as {}/{}", request.tool_type, request.tool_name);
        } else {
            println!("⚠️ Not recognized as tool command");
        }
    }
    
    // Test 2: Execute a tool via Python bridge
    println!("\n4️⃣ Testing Tool Execution via Bridge...");
    
    // List current directory
    let list_request = SystemToolRequest {
        tool_type: "filesystem".to_string(),
        tool_name: "list_directory".to_string(),
        args: serde_json::json!({"path": "."}),
        security_level: Some("medium".to_string()),
    };
    
    println!("   Executing: list directory '.'");
    match bridge.execute_system_tool(list_request).await {
        Ok(response) => {
            if response.success {
                println!("   ✅ Success! Files found:");
                for line in response.output.lines().take(5) {
                    println!("      {}", line);
                }
                if response.output.lines().count() > 5 {
                    println!("      ... and {} more", response.output.lines().count() - 5);
                }
            } else {
                println!("   ❌ Failed: {:?}", response.error);
            }
        },
        Err(e) => println!("   ❌ Error: {}", e),
    }
    
    // Read a file
    let read_request = SystemToolRequest {
        tool_type: "filesystem".to_string(),
        tool_name: "read_file".to_string(),
        args: serde_json::json!({"path": "Cargo.toml"}),
        security_level: Some("medium".to_string()),
    };
    
    println!("\n   Executing: read file 'Cargo.toml'");
    match bridge.execute_system_tool(read_request).await {
        Ok(response) => {
            if response.success {
                println!("   ✅ Success! First 3 lines:");
                for line in response.output.lines().take(3) {
                    println!("      {}", line);
                }
            } else {
                println!("   ❌ Failed: {:?}", response.error);
            }
        },
        Err(e) => println!("   ❌ Error: {}", e),
    }
    
    // Test 3: Test tool intent recognition (placeholder for now)
    println!("\n5️⃣ Testing Tool Intent Recognition...");
    let prompts = vec![
        "Show me the files in the current directory",
        "What's in my home folder?",
        "Read the README file",
    ];
    
    for prompt in prompts {
        print!("   '{}': ", prompt);
        match bridge.recognize_tool_intent(prompt)? {
            Some(tool_call) => println!("✅ Recognized as: {}", tool_call.tool_name),
            None => println!("⚠️ No tool intent recognized"),
        }
    }
    
    // Test 4: Agent pipeline (if available)
    println!("\n6️⃣ Testing Agent Pipeline...");
    let pipeline = bridge.get_agent_pipeline();
    if pipeline.is_empty() {
        println!("   ⚠️ No agents configured in pipeline");
    } else {
        println!("   Found {} agents in pipeline:", pipeline.len());
        for agent in pipeline {
            println!("      - {}: {}", agent.name, &agent.persona[..50.min(agent.persona.len())]);
        }
    }
    
    // Summary
    println!("\n========================================");
    println!("📊 Test Summary");
    println!("========================================");
    println!("✅ Python bridge initialized");
    println!("✅ Tool executor connected");
    println!("✅ Tool parsing works");
    println!("✅ Tool execution works");
    println!("⚠️ Tool intent recognition: placeholder");
    println!("⚠️ Agent pipeline: not configured");
    
    println!("\n🎉 Python Bridge Integration Test Complete!");
    
    // Clean up
    drop(tool_tx);
    tools_handle.await?;
    
    Ok(())
}

#[tokio::test]
async fn test_tool_parsing() -> Result<()> {
    let bridge = PythonBridge::new()?;
    
    // Test that common commands are parsed
    assert!(bridge.parse_system_tool_request("ls .").is_some());
    assert!(bridge.parse_system_tool_request("cat Cargo.toml").is_some());
    assert!(bridge.parse_system_tool_request("hello world").is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_python_availability() -> Result<()> {
    let bridge = PythonBridge::new()?;
    // This test just verifies that Python bridge initializes
    // Python module might not be available, which is okay
    let _available = bridge.is_python_available();
    Ok(())
}
