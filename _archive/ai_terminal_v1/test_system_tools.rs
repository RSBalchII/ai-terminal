use system_tools::*;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Testing AI Terminal System Tools");
    println!("=====================================");
    
    let executor = ToolExecutor::new();
    
    // Test 1: List current directory
    println!("\n📁 Test 1: List current directory");
    let list_tool = SystemTool::FileSystem(FileSystemTool::List {
        path: ".".to_string(),
        recursive: false,
        show_hidden: false,
    });
    
    let result = executor.execute(list_tool).await?;
    println!("Success: {}", result.success);
    println!("Output:\n{}", result.output);
    if let Some(error) = result.error {
        println!("Error: {}", error);
    }
    
    // Test 2: Read a file
    println!("\n📖 Test 2: Read Cargo.toml");
    let read_tool = SystemTool::FileSystem(FileSystemTool::Read {
        path: "Cargo.toml".to_string(),
        lines: Some((1, 10)),
        max_size: None,
    });
    
    let result = executor.execute(read_tool).await?;
    println!("Success: {}", result.success);
    println!("Output:\n{}", result.output);
    
    // Test 3: List running processes
    println!("\n🔄 Test 3: List processes");
    let ps_tool = SystemTool::Process(ProcessTool::List {
        filter: Some("rust".to_string()),
        show_all: false,
    });
    
    let result = executor.execute(ps_tool).await?;
    println!("Success: {}", result.success);
    println!("Output:\n{}", result.output);
    
    // Test 4: Ping localhost
    println!("\n🌐 Test 4: Ping localhost");
    let ping_tool = SystemTool::Network(NetworkTool::Ping {
        host: "localhost".to_string(),
        count: Some(2),
        timeout: Some(1),
    });
    
    let result = executor.execute(ping_tool).await?;
    println!("Success: {}", result.success);
    println!("Output:\n{}", result.output);
    
    // Test 5: Port scan
    println!("\n🔍 Test 5: Port scan localhost");
    let scan_tool = SystemTool::Network(NetworkTool::PortScan {
        host: "localhost".to_string(),
        ports: vec![22, 80, 443, 11434], // SSH, HTTP, HTTPS, Ollama
        timeout: Some(1),
    });
    
    let result = executor.execute(scan_tool).await?;
    println!("Success: {}", result.success);
    println!("Output:\n{}", result.output);
    
    println!("\n✅ System tools test completed!");
    Ok(())
}
