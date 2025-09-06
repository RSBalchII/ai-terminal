use system_tools::*;
use system_tools::network::NetcatMode;

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
    
    match executor.execute(list_tool).await {
        Ok(result) => {
            println!("✅ Success: {}", result.success);
            println!("Output:\n{}", result.output);
            if let Some(error) = result.error {
                println!("Error: {}", error);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test 2: Read a small file
    println!("\n📖 Test 2: Read Cargo.toml (first 10 lines)");
    let read_tool = SystemTool::FileSystem(FileSystemTool::Read {
        path: "../Cargo.toml".to_string(),
        lines: Some((1, 10)),
        max_size: None,
    });
    
    match executor.execute(read_tool).await {
        Ok(result) => {
            println!("✅ Success: {}", result.success);
            println!("Output:\n{}", result.output);
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test 3: Search for "rust" in current directory
    println!("\n🔍 Test 3: Search for 'tokio' in current directory");
    let search_tool = SystemTool::FileSystem(FileSystemTool::Search {
        path: ".".to_string(),
        pattern: "tokio".to_string(),
        recursive: true,
        case_sensitive: false,
    });
    
    match executor.execute(search_tool).await {
        Ok(result) => {
            println!("✅ Success: {}", result.success);
            println!("Output:\n{}", result.output);
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test 4: List running processes
    println!("\n🔄 Test 4: List processes containing 'cargo'");
    let ps_tool = SystemTool::Process(ProcessTool::List {
        filter: Some("cargo".to_string()),
        show_all: false,
    });
    
    match executor.execute(ps_tool).await {
        Ok(result) => {
            println!("✅ Success: {}", result.success);
            println!("Output:\n{}", result.output);
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test 5: Ping localhost
    println!("\n🌐 Test 5: Ping localhost");
    let ping_tool = SystemTool::Network(NetworkTool::Ping {
        host: "localhost".to_string(),
        count: Some(2),
        timeout: Some(2),
    });
    
    match executor.execute(ping_tool).await {
        Ok(result) => {
            println!("✅ Success: {}", result.success);
            if result.success {
                println!("Output:\n{}", result.output);
            } else if let Some(error) = result.error {
                println!("Error: {}", error);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test 6: Test connection to Ollama port
    println!("\n🔌 Test 6: Test connection to Ollama (port 11434)");
    let nc_tool = SystemTool::Network(NetworkTool::Netcat {
        host: "localhost".to_string(),
        port: 11434,
        mode: NetcatMode::Test,
        timeout: Some(3),
    });
    
    match executor.execute(nc_tool).await {
        Ok(result) => {
            println!("✅ Success: {}", result.success);
            if result.success {
                println!("Ollama is running! ✨");
            } else if let Some(error) = result.error {
                println!("Ollama connection failed: {}", error);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    println!("\n✅ System tools test completed!");
    println!("🎉 Your AI Terminal now has system access capabilities!");
    Ok(())
}
