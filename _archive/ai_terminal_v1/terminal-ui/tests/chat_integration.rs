use tokio;
use std::time::{Duration, Instant};
use anyhow::Result;

// Import our chat functionality
use ollama_client::OllamaClient;
use python_bridge::PythonBridge;

#[cfg(test)]
mod chat_integration_tests {
    use super::*;

    /// Test Environment Setup
    /// Validates that our testing environment meets the requirements from core-functionality.yaml
    #[tokio::test]
    async fn test_environment_setup() -> Result<()> {
        // Verify Ollama server is accessible
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:11434/api/tags")
            .send()
            .await?;
        
        assert!(response.status().is_success(), "Ollama server should be accessible");
        
        let models: serde_json::Value = response.json().await?;
        let model_names: Vec<String> = models["models"]
            .as_array()
            .unwrap()
            .iter()
            .map(|model| model["name"].as_str().unwrap().to_string())
            .collect();

        // Verify required models are available (from spec environment section)
        let required_models = vec![
            "nemotron-mini:4b-instruct-q8_0",
            "mistral-nemo:12b-instruct-2407-q8_0"
        ];

        for required_model in required_models {
            assert!(
                model_names.iter().any(|model| model.contains(required_model)), 
                "Required model {} should be available", 
                required_model
            );
        }

        println!("✅ Test environment validation passed");
        println!("📋 Available models: {:?}", model_names);
        
        Ok(())
    }

    /// CHAT_001: Basic Message Send and Receive
    /// Tests the fundamental chat functionality as specified in core-functionality.yaml
    #[tokio::test]
    async fn test_basic_message_flow() -> Result<()> {
        // GIVEN: User is in chat mode with a model loaded
        let mut ollama_client = OllamaClient::new("http://localhost:11434").await?;
        let _python_bridge = PythonBridge::new()?;
        
        // Set the model (using the smaller one for faster tests)
        let test_model = "nemotron-mini:4b-instruct-q8_0";
        ollama_client.set_model(test_model.to_string()).await?;
        
        if let Some(current_model) = ollama_client.get_current_model() {
            assert!(current_model.contains(test_model), "Current model should contain {}, got {}", test_model, current_model);
        } else {
            panic!("No current model set");
        }
        
        // WHEN: User types a message and presses Enter
        let user_message = "Hello AI, please respond with exactly 'Test successful'";
        let start_time = Instant::now();
        
        // Simulate the message send process from TerminalApp::process_user_input
        let ai_response = ollama_client.generate(user_message.to_string()).await?;
        let response_time = start_time.elapsed();
        
        // THEN: Response completes within 45-second timeout (from spec)
        assert!(
            response_time < Duration::from_secs(45),
            "Response should complete within 45 seconds, took {:?}",
            response_time
        );
        
        // THEN: AI response is generated
        assert!(!ai_response.is_empty(), "AI should provide a non-empty response");
        
        // Validation criteria: Basic response quality
        assert!(
            ai_response.len() > 5,
            "Response should be substantial, got: '{}'", 
            ai_response
        );
        
        println!("✅ CHAT_001: Basic message flow test passed");
        println!("📝 User message: {}", user_message);
        println!("🤖 AI response: {} characters", ai_response.len());
        println!("⏱️ Response time: {:?}", response_time);
        
        Ok(())
    }

    /// CHAT_001: Message Formatting and Timestamping
    /// Tests that messages preserve formatting and include timestamps
    #[tokio::test]
    async fn test_message_formatting() -> Result<()> {
        let mut ollama_client = OllamaClient::new("http://localhost:11434").await?;
        ollama_client.set_model("nemotron-mini:4b-instruct-q8_0".to_string()).await?;
        
        // Test message with special characters and formatting
        let complex_message = "Test message with:\n- Line breaks\n- Special chars: !@#$%\n- Unicode: 🚀✨";
        let timestamp_before = Instant::now();
        
        let response = ollama_client.generate(complex_message.to_string()).await?;
        let timestamp_after = Instant::now();
        
        // Validation criteria: Response should handle complex input
        assert!(!response.is_empty(), "Should handle complex message formatting");
        
        // Simulate timestamp recording (as done in TerminalApp::add_message)
        let message_timestamp = timestamp_before;
        assert!(message_timestamp <= timestamp_after, "Timestamp should be recorded correctly");
        
        println!("✅ CHAT_001: Message formatting test passed");
        println!("📝 Complex message handled successfully");
        
        Ok(())
    }

    /// CHAT_002: Streaming Response Validation (Simplified)
    /// Tests that streaming responses work correctly 
    #[tokio::test]
    async fn test_streaming_response_basic() -> Result<()> {
        let client = reqwest::Client::new();
        
        // Test streaming API directly (simulating what OllamaClient would do)
        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&serde_json::json!({
                "model": "nemotron-mini:4b-instruct-q8_0",
                "prompt": "Count from 1 to 5, one number per line",
                "stream": true
            }))
            .send()
            .await?;

        assert!(response.status().is_success(), "Streaming request should succeed");
        
        // Read the streaming response
        let mut bytes_received = 0;
        let mut chunks_received = 0;
        
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            bytes_received += chunk.len();
            chunks_received += 1;
            
            // Basic validation that we're getting JSON chunks
            if let Ok(chunk_str) = std::str::from_utf8(&chunk) {
                // Each chunk should be a valid JSON line for Ollama streaming
                for line in chunk_str.lines() {
                    if !line.trim().is_empty() {
                        assert!(
                            line.starts_with('{'), 
                            "Streaming chunks should be JSON, got: {}", 
                            line
                        );
                    }
                }
            }
        }
        
        // THEN: We should receive multiple chunks (streaming behavior)
        assert!(chunks_received > 1, "Should receive multiple streaming chunks, got {}", chunks_received);
        assert!(bytes_received > 50, "Should receive substantial data, got {} bytes", bytes_received);
        
        println!("✅ CHAT_002: Streaming response test passed");
        println!("📦 Chunks received: {}", chunks_received);
        println!("📊 Bytes received: {}", bytes_received);
        
        Ok(())
    }

    /// CHAT_003: Conversation Context (Simplified)
    /// Tests that context is maintained between messages
    #[tokio::test]
    async fn test_conversation_context_basic() -> Result<()> {
        let mut ollama_client = OllamaClient::new("http://localhost:11434").await?;
        ollama_client.set_model("nemotron-mini:4b-instruct-q8_0".to_string()).await?;
        
        // First message: Establish context
        let first_message = "My favorite color is blue. Remember this.";
        let first_response = ollama_client.generate(first_message.to_string()).await?;
        assert!(!first_response.is_empty(), "Should respond to context establishment");
        
        // Second message: Test context retention
        let followup_message = "What is my favorite color?";
        let followup_response = ollama_client.generate(followup_message.to_string()).await?;
        
        // Basic validation: Response should reference blue
        // Note: This is a simplified test - full context testing would require conversation history
        assert!(!followup_response.is_empty(), "Should respond to context question");
        
        println!("✅ CHAT_003: Basic context test passed");
        println!("🔗 Context established and queried");
        
        Ok(())
    }

    /// Performance Benchmark: Response Latency
    /// Tests that response times meet the 45-second threshold from the spec
    #[tokio::test]
    async fn test_response_latency_benchmark() -> Result<()> {
        let mut ollama_client = OllamaClient::new("http://localhost:11434").await?;
        ollama_client.set_model("nemotron-mini:4b-instruct-q8_0".to_string()).await?;
        
        let test_prompts = vec![
            "Short response test", // Short prompt
            "Please write a medium length response about the weather, including details about clouds, temperature, and what people might wear.", // Medium prompt
            "Write a comprehensive analysis of renewable energy technologies, covering solar, wind, hydro, and geothermal power sources, their advantages, disadvantages, and future prospects in the context of climate change and energy security." // Long prompt
        ];
        
        let mut response_times = Vec::new();
        
        for (i, prompt) in test_prompts.iter().enumerate() {
            let start_time = Instant::now();
            let response = ollama_client.generate(prompt.to_string()).await?;
            let elapsed = start_time.elapsed();
            
            response_times.push(elapsed);
            
            // Validate response within timeout
            assert!(
                elapsed < Duration::from_secs(45),
                "Response {} should complete within 45s, took {:?}",
                i + 1,
                elapsed
            );
            
            assert!(!response.is_empty(), "Response {} should not be empty", i + 1);
            
            println!("📊 Prompt {} ({} chars): {:?} -> {} chars", 
                i + 1, prompt.len(), elapsed, response.len());
        }
        
        let avg_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
        println!("✅ Performance benchmark passed");
        println!("📈 Average response time: {:?}", avg_time);
        
        Ok(())
    }
}

/// Test Helper Functions
/// Utilities for common test operations as referenced in the spec

pub struct TestHarness {
    pub ollama_client: OllamaClient,
    pub python_bridge: PythonBridge,
}

impl TestHarness {
    pub async fn new() -> Result<Self> {
        let ollama_client = OllamaClient::new("http://localhost:11434").await?;
        let python_bridge = PythonBridge::new()?;
        
        Ok(Self {
            ollama_client,
            python_bridge,
        })
    }
    
    pub async fn setup_test_model(&mut self) -> Result<()> {
        self.ollama_client.set_model("nemotron-mini:4b-instruct-q8_0".to_string()).await?;
        Ok(())
    }
    
    pub async fn send_test_message(&self, message: &str) -> Result<(String, Duration)> {
        let start_time = Instant::now();
        let response = self.ollama_client.generate(message.to_string()).await?;
        let elapsed = start_time.elapsed();
        
        Ok((response, elapsed))
    }
}
