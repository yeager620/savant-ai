//! Savant AI MCP (Model Context Protocol) Server
//! 
//! Standalone MCP server for LLM integration with Savant AI conversation database

use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;
use std::sync::Arc;
use savant_db::{TranscriptDatabase, MCPServer};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    let matches = Command::new("savant-mcp-server")
        .version("1.0.0")
        .author("Savant AI")
        .about("MCP server for Savant AI conversation database")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("Configuration file path")
        )
        .arg(
            Arg::new("database")
                .long("database")
                .short('d')
                .value_name("PATH")
                .help("Database file path")
        )
        .arg(
            Arg::new("transport")
                .long("transport")
                .short('t')
                .value_name("TYPE")
                .help("Transport type (stdio, http)")
                .default_value("stdio")
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .value_name("PORT")
                .help("HTTP server port (for http transport)")
                .default_value("8080")
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose logging")
        )
        .get_matches();
    
    // Parse database path
    let db_path = matches.get_one::<String>("database")
        .map(PathBuf::from);
    
    // Initialize database
    println!("Initializing Savant AI database...");
    let database = Arc::new(TranscriptDatabase::new(db_path).await?);
    
    // Create MCP server
    println!("Starting MCP server...");
    let mcp_server = MCPServer::new(database)?;
    
    // Start server based on transport type
    let transport = matches.get_one::<String>("transport").unwrap();
    
    match transport.as_str() {
        "stdio" => {
            println!("MCP server running on stdio transport");
            mcp_server.start_stdio_server().await?;
        }
        "http" => {
            let port = matches.get_one::<String>("port").unwrap()
                .parse::<u16>()
                .unwrap_or(8080);
            println!("HTTP transport not yet implemented. Using stdio.");
            mcp_server.start_stdio_server().await?;
        }
        _ => {
            eprintln!("Unknown transport type: {}", transport);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Print usage examples
#[allow(dead_code)]
fn print_usage_examples() {
    println!("Usage Examples:");
    println!("");
    println!("  # Start MCP server with default database");
    println!("  savant-mcp-server");
    println!("");
    println!("  # Start with custom database path");
    println!("  savant-mcp-server --database /path/to/custom.db");
    println!("");
    println!("  # Claude Desktop integration (add to config):");
    println!("  {{");
    println!("    \"mcpServers\": {{");
    println!("      \"savant-ai\": {{");
    println!("        \"command\": \"/path/to/savant-mcp-server\",");
    println!("        \"args\": [\"--database\", \"/path/to/transcripts.db\"]");
    println!("      }}");
    println!("    }}");
    println!("  }}");
    println!("");
    println!("  # Test MCP server manually:");
    println!("  echo '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{}}}}' | savant-mcp-server");
}