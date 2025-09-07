use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

/// The main configuration structure, representing the TOML format
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Ollama-specific configuration
    pub ollama: OllamaConfig,
    
    /// Custom prompts that can be referenced by name
    #[serde(default)]
    pub custom_prompts: std::collections::HashMap<String, String>,
}

/// Configuration for Ollama integration
#[derive(Debug, Deserialize, Clone)]
pub struct OllamaConfig {
    /// The default model to use for Ollama requests
    pub model: String,
    
    /// Optional system prompt to guide the model's behavior
    #[serde(rename = "system_prompt")]
    pub system_prompt: Option<String>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
    
    /// Get a custom prompt by name
    pub fn get_custom_prompt(&self, name: &str) -> Option<&String> {
        self.custom_prompts.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        // Create a temporary TOML file with test data
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[ollama]
model = "test-model"
system_prompt = "Test system prompt"

[custom_prompts]
test_prompt = "This is a test prompt"
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        // Load the configuration
        let config = Config::load(temp_file.path()).unwrap();

        // Verify the loaded configuration
        assert_eq!(config.ollama.model, "test-model");
        assert_eq!(config.ollama.system_prompt, Some("Test system prompt".to_string()));
        assert_eq!(config.custom_prompts.get("test_prompt"), Some(&"This is a test prompt".to_string()));
    }

    #[test]
    fn test_config_loading_without_system_prompt() {
        // Create a temporary TOML file with test data (no system prompt)
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[ollama]
model = "test-model"
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        // Load the configuration
        let config = Config::load(temp_file.path()).unwrap();

        // Verify the loaded configuration
        assert_eq!(config.ollama.model, "test-model");
        assert_eq!(config.ollama.system_prompt, None);
    }

    #[test]
    fn test_get_custom_prompt() {
        // Create a temporary TOML file with test data
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[ollama]
model = "test-model"

[custom_prompts]
prompt1 = "First test prompt"
prompt2 = "Second test prompt"
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        // Load the configuration
        let config = Config::load(temp_file.path()).unwrap();

        // Verify getting custom prompts
        assert_eq!(config.get_custom_prompt("prompt1"), Some(&"First test prompt".to_string()));
        assert_eq!(config.get_custom_prompt("prompt2"), Some(&"Second test prompt".to_string()));
        assert_eq!(config.get_custom_prompt("nonexistent"), None);
    }
}