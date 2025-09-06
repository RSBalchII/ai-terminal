//! # Ollama Client
//!
//! A Rust client for interacting with the Ollama API.
//!
//! This crate provides functionality to send requests to the Ollama API
 //! and handle streaming responses, with support for conversational history.
//!
//! ## Usage
//!
//! ```rust
//! // Add example usage here once implemented
//! ```

/// Module for handling API requests and responses
pub mod api;

/// Module for defining data models
pub mod models;

/// Module for managing conversational history
pub mod history;

/// Module containing error types for the client
pub mod error;

/// Re-export the main client struct and models
pub use api::OllamaClient;
pub use models::{OllamaRequest, OllamaResponse};