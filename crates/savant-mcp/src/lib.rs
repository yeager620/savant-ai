//! # Savant MCP Server
//!
//! Model Context Protocol server for Savant AI database integration.
//! Exposes conversation database to external LLMs via JSON-RPC 2.0.

pub mod mcp_server;
pub mod mcp_server_tools;
pub mod mcp_server_prompts;

pub use mcp_server::{MCPServer, MCPRequest, MCPResponse, MCPTransport, StdioTransport};

// Re-export commonly needed types
pub use savant_db::{TranscriptDatabase, LLMConfig};
pub use savant_core::types::*;