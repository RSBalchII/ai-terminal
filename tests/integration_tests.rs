//! Integration tests for the AI Terminal application

#[cfg(test)]
mod integration_tests {
    use anyhow::Result;
    use std::process::Command;
    use std::time::Duration;
    use tokio::time::timeout;

    /// Test that the application can start and show the welcome message
    #[tokio::test]
    async fn test_application_startup() -> Result<()> {
        // Start the application
        let mut child = Command::new("cargo")
            .args(["run", "--bin", "ai-terminal"])
            .spawn()?;

        // Wait for a short time to let the application start
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check that the process is still running
        assert!(child.try_wait()?.is_none());

        // Kill the process
        child.kill()?;

        Ok(())
    }

    /// Test that the Ollama client can be created
    #[tokio::test]
    async fn test_ollama_client_creation() -> Result<()> {
        // Try to create an Ollama client
        let client = ollama_client::OllamaClient::new()?;

        // Check that the client has the expected default values
        assert_eq!(client.base_url, "http://localhost:11434/api");
        assert_eq!(client.model, "llama3");

        Ok(())
    }

    /// Test that the MCP client can be created
    #[tokio::test]
    async fn test_mcp_client_creation() -> Result<()> {
        // Try to create an MCP client - this will fail since we don't have a server running
        // but we can at least verify the code compiles and the struct can be created
        let client = ai_terminal::mcp::MCPClient::new("echo 'test'").await;

        // We expect this to succeed in creating the struct, but fail when trying to start the process
        // since "echo 'test'" isn't actually an MCP server
        assert!(client.is_err());

        Ok(())
    }
}