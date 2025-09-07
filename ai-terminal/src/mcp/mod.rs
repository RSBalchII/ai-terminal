//! # MCP Module for AI Terminal
//!
//! This module contains the MCP client implementation for connecting to MCP servers.

pub mod client;
pub mod ai_command_processor;

pub use client::MCPClient;
pub use ai_command_processor::process_ai_command;