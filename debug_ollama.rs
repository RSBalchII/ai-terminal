use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Ollama connection directly...");
    
    // Test basic connection
    let client = reqwest::Client::new();
    
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        client.get("http://localhost:11434/api/tags").send()
    ).await??;
    
    if response.status().is_success() {
        println!("✅ Ollama API is responding");
    } else {
        println!("❌ Ollama API error: {}", response.status());
        return Ok(());
    }
    
    // Test generate request
    println!("🧠 Testing generate request...");
    
    let generate_request = serde_json::json!({
        "model": "nemotron-mini:4b-instruct-q8_0",
        "prompt": "Hello, just respond with 'Hi there!'",
        "stream": false
    });
    
    let response = tokio::time::timeout(
        Duration::from_secs(10),
        client.post("http://localhost:11434/api/generate")
            .json(&generate_request)
            .send()
    ).await??;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        if let Some(response_text) = result.get("response").and_then(|r| r.as_str()) {
            println!("✅ AI Response: {}", response_text);
        } else {
            println!("❌ No response text in result: {}", result);
        }
    } else {
        println!("❌ Generate request failed: {}", response.status());
    }
    
    Ok(())
}
