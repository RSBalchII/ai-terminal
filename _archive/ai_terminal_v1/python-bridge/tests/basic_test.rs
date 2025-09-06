use python_bridge::{PythonBridge, SystemToolRequest};
use anyhow::Result;

#[test]
fn test_python_bridge_initialization() {
    let result = PythonBridge::new();
    assert!(result.is_ok(), "Python bridge should initialize");
}

#[test]
fn test_parse_ls_command() {
    let bridge = PythonBridge::new().unwrap();
    let request = bridge.parse_system_tool_request("ls .");
    
    assert!(request.is_some(), "Should parse 'ls .' command");
    let request = request.unwrap();
    assert_eq!(request.tool_type, "filesystem");
    assert_eq!(request.tool_name, "list_directory");
}

#[test]
fn test_parse_cat_command() {
    let bridge = PythonBridge::new().unwrap();
    let request = bridge.parse_system_tool_request("cat README.md");
    
    assert!(request.is_some(), "Should parse 'cat' command");
    let request = request.unwrap();
    assert_eq!(request.tool_type, "filesystem");
    assert_eq!(request.tool_name, "read_file");
}

#[test]
fn test_parse_ps_command() {
    let bridge = PythonBridge::new().unwrap();
    let request = bridge.parse_system_tool_request("ps");
    
    assert!(request.is_some(), "Should parse 'ps' command");
    let request = request.unwrap();
    assert_eq!(request.tool_type, "process");
    assert_eq!(request.tool_name, "list_processes");
    
    // Also test "processes" alias
    let request2 = bridge.parse_system_tool_request("processes");
    assert!(request2.is_some(), "Should parse 'processes' command");
}

#[test]
fn test_parse_invalid_command() {
    let bridge = PythonBridge::new().unwrap();
    let request = bridge.parse_system_tool_request("hello world");
    
    assert!(request.is_none(), "Should not parse random text as command");
}

#[test]
fn test_agent_pipeline_empty() {
    let bridge = PythonBridge::new().unwrap();
    let pipeline = bridge.get_agent_pipeline();
    
    // Pipeline starts empty without config
    assert_eq!(pipeline.len(), 0, "Pipeline should be empty initially");
}

#[test]
fn test_tool_intent_recognition() -> Result<()> {
    let bridge = PythonBridge::new()?;
    
    // Currently returns None as it's a placeholder
    let intent = bridge.recognize_tool_intent("Show me the files")?;
    assert!(intent.is_none(), "Tool intent is currently a placeholder");
    
    Ok(())
}
