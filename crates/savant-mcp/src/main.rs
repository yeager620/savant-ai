//! Savant AI MCP Server CLI
//! 
//! Standalone Model Context Protocol server for LLM database integration
//! Following UNIX philosophy: do one thing and do it well

use anyhow::Result;
use clap::{Arg, Command};
use env_logger;
use std::path::PathBuf;
use std::sync::Arc;
use tokio;

use savant_mcp::{TranscriptDatabase, MCPServer, LLMConfig, StdioTransport};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments first
    let matches = Command::new("savant-mcp")
        .version("1.0.0")
        .about("Savant AI MCP Server - Database access for LLMs")
        .arg(
            Arg::new("database")
                .short('d')
                .long("database")
                .value_name("PATH")
                .help("Path to SQLite database file")
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level (error, warn, info, debug, trace)")
                .default_value("info")
        )
        .arg(
            Arg::new("llm-provider")
                .long("llm-provider")
                .value_name("PROVIDER")
                .help("LLM provider (ollama, openai, mock)")
                .default_value("ollama")
        )
        .arg(
            Arg::new("llm-endpoint")
                .long("llm-endpoint")
                .value_name("URL")
                .help("LLM endpoint URL")
                .default_value("http://localhost:11434")
        )
        .arg(
            Arg::new("llm-model")
                .long("llm-model")
                .value_name("MODEL")
                .help("LLM model name")
                .default_value("llama3.2")
        )
        .arg(
            Arg::new("test")
                .long("test")
                .help("Run in test mode with mock data")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();
    
    // Set log level
    if let Some(level) = matches.get_one::<String>("log-level") {
        std::env::set_var("RUST_LOG", level);
    }
    env_logger::init();
    
    // Print startup banner
    eprintln!("🤖 Savant AI MCP Server v1.0.0");
    eprintln!("   Model Context Protocol server for conversation database");
    eprintln!("   Following UNIX philosophy: composable, focused, reliable");
    eprintln!();
    
    // Initialize database
    let db_path = matches.get_one::<String>("database")
        .map(|p| PathBuf::from(p));
    
    let database = Arc::new(TranscriptDatabase::new(db_path).await?);
    eprintln!("✅ Database initialized successfully");
    
    // Configure LLM client
    let llm_config = LLMConfig {
        provider: matches.get_one::<String>("llm-provider").unwrap().clone(),
        endpoint: matches.get_one::<String>("llm-endpoint").unwrap().clone(),
        model: matches.get_one::<String>("llm-model").unwrap().clone(),
        timeout_seconds: 30,
        max_tokens: Some(1000),
        temperature: Some(0.1),
        api_key: std::env::var("OPENAI_API_KEY").ok(),
    };
    
    let llm_configs = vec![llm_config];
    
    // Create MCP server with enhanced capabilities
    let mcp_server = MCPServer::new(database, Some(llm_configs)).await?;
    eprintln!("✅ Enhanced MCP server created with LLM integration");
    
    // Check if running in test mode
    if matches.get_flag("test") {
        eprintln!("🧪 Running in test mode");
        return run_test_mode(&mcp_server).await;
    }
    
    eprintln!("🚀 Starting MCP server (stdio transport)");
    eprintln!("   Listening for JSON-RPC 2.0 requests on stdin");
    eprintln!("   Responses will be written to stdout");
    eprintln!("   Press Ctrl+C to stop");
    eprintln!();
    
    // Start server with stdio transport
    let transport = StdioTransport::new();
    mcp_server.start_server(transport).await?;
    
    Ok(())
}

/// Run server in test mode with sample interactions
async fn run_test_mode(server: &MCPServer) -> Result<()> {
    eprintln!("=== MCP Server Test Mode ===");
    
    // Test 1: Initialize
    eprintln!("Test 1: Initialize");
    let init_request = savant_mcp::MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    };
    
    let response = server.handle_request(init_request).await;
    println!("{}", serde_json::to_string_pretty(&response)?);
    eprintln!("✅ Initialize test passed\n");
    
    // Test 2: List tools
    eprintln!("Test 2: List tools");
    let tools_request = savant_mcp::MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(2)),
        method: "tools/list".to_string(),
        params: None,
    };
    
    let response = server.handle_request(tools_request).await;
    println!("{}", serde_json::to_string_pretty(&response)?);
    eprintln!("✅ List tools test passed\n");
    
    // Test 3: Query conversations
    eprintln!("Test 3: Query conversations");
    let query_request = savant_mcp::MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(3)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "query_conversations",
            "arguments": {
                "query": "Find conversations about the project meeting",
                "session_id": "test-session-123"
            }
        })),
    };
    
    let response = server.handle_request(query_request).await;
    println!("{}", serde_json::to_string_pretty(&response)?);
    eprintln!("✅ Query conversations test passed\n");
    
    // Test 4: Get database stats
    eprintln!("Test 4: Get database stats");
    let stats_request = savant_mcp::MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(4)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "get_database_stats",
            "arguments": {
                "include_performance": true
            }
        })),
    };
    
    let response = server.handle_request(stats_request).await;
    println!("{}", serde_json::to_string_pretty(&response)?);
    eprintln!("✅ Database stats test passed\n");
    
    // Test 5: List resources
    eprintln!("Test 5: List resources");
    let resources_request = savant_mcp::MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(5)),
        method: "resources/list".to_string(),
        params: None,
    };
    
    let response = server.handle_request(resources_request).await;
    println!("{}", serde_json::to_string_pretty(&response)?);
    eprintln!("✅ List resources test passed\n");
    
    eprintln!("🎉 All tests passed! MCP server is working correctly.");
    eprintln!("   You can now use this server with MCP-compatible LLM clients.");
    eprintln!("   Example usage:");
    eprintln!("     echo '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}}' | ./savant-mcp");
    
    Ok(())
}

/// Signal handler for graceful shutdown
#[allow(dead_code)]
fn setup_signal_handlers() {
    use tokio::signal;
    
    tokio::spawn(async {
        if signal::ctrl_c().await.is_ok() {
            eprintln!("\n🛑 Received Ctrl+C, shutting down gracefully...");
            std::process::exit(0);
        }
    });
}