//! Model Context Protocol (MCP) server implementation for LLM database integration
//! 
//! Provides standardized interface for LLMs to query the conversation database

use anyhow::{anyhow, Result};
use sqlx::Row;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use uuid::Uuid;

use savant_db::{
    TranscriptDatabase, 
    LLMClientFactory, LLMConfig,
    QueryProcessor, ConversationContextManager, QueryOptimizer,
    QuerySecurityManager
};
use savant_db::natural_query::LLMClientWrapper;

/// MCP JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// MCP JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MCPResponse {
    ToolsList {
        jsonrpc: String,
        id: Option<Value>,
        result: Value,
    },
    ToolResult {
        jsonrpc: String,
        id: Option<Value>,
        result: Value,
        content: Vec<ContentItem>,
    },
    ResourcesList {
        jsonrpc: String,
        id: Option<Value>,
        result: Value,
        resources: Vec<MCPResource>,
    },
    Error {
        jsonrpc: String,
        id: Option<Value>,
        error: MCPError,
    },
}

#[derive(Debug, Serialize)]
pub struct ContentItem {
    pub r#type: String,
    pub text: String,
}

/// MCP error object
#[derive(Debug, Serialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP resource definition
#[derive(Debug, Serialize)]
pub struct MCPResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Value>,
}

/// MCP tool definition
#[derive(Debug, Serialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP server session state
#[derive(Debug)]
pub struct MCPSession {
    pub id: String,
    pub client_info: Option<Value>,
    pub capabilities: Vec<String>,
    pub query_count: u64,
}

/// Transport abstraction for MCP communication
#[async_trait::async_trait]
pub trait MCPTransport: Send + Sync {
    async fn receive(&mut self) -> Result<MCPRequest>;
    async fn send(&mut self, response: MCPResponse) -> Result<()>;
}

/// Stdio transport implementation
pub struct StdioTransport {
    reader: BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
}

/// Enhanced MCP server implementation with LLM integration
pub struct MCPServer {
    pub database: Arc<TranscriptDatabase>,
    pub security: QuerySecurityManager,
    pub query_processor: QueryProcessor,
    pub context_manager: Arc<ConversationContextManager>,
    pub query_optimizer: QueryOptimizer,
    pub llm_client: Option<LLMClientWrapper>,
    pub sessions: Arc<Mutex<HashMap<String, MCPSession>>>,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
        }
    }
}

#[async_trait::async_trait]
impl MCPTransport for StdioTransport {
    async fn receive(&mut self) -> Result<MCPRequest> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await
            .map_err(|e| anyhow!("Failed to read from stdin: {}", e))?;

        if line.trim().is_empty() {
            return Err(anyhow!("Empty request"));
        }

        serde_json::from_str(line.trim())
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e))
    }

    async fn send(&mut self, response: MCPResponse) -> Result<()> {
        let json = serde_json::to_string(&response)
            .map_err(|e| anyhow!("Failed to serialize response: {}", e))?;

        self.writer.write_all(json.as_bytes()).await
            .map_err(|e| anyhow!("Failed to write to stdout: {}", e))?;
        self.writer.write_all(b"\n").await
            .map_err(|e| anyhow!("Failed to write newline: {}", e))?;
        self.writer.flush().await
            .map_err(|e| anyhow!("Failed to flush stdout: {}", e))?;

        Ok(())
    }
}

impl MCPServer {
    /// Create a new enhanced MCP server instance
    pub async fn new(database: Arc<TranscriptDatabase>, llm_configs: Option<Vec<LLMConfig>>) -> Result<Self> {
        let pool = database.pool.clone();

        // Initialize LLM client
        let llm_client = if let Some(configs) = llm_configs {
            Some(LLMClientFactory::create_best_available(configs).await?)
        } else {
            // Try default Ollama config
            let default_config = LLMConfig::default();
            match LLMClientFactory::create_client(&default_config) {
                Ok(client) => Some(client),
                Err(_) => {
                    log::warn!("No LLM client available, using pattern-based fallback");
                    None
                }
            }
        };

        let query_processor = QueryProcessor::new(pool.clone(), llm_client.clone());
        let context_manager = Arc::new(ConversationContextManager::new());
        let query_optimizer = QueryOptimizer::new(pool);
        let security = QuerySecurityManager::read_only();

        Ok(Self {
            database,
            security,
            query_processor,
            context_manager,
            query_optimizer,
            llm_client,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create server with explicit LLM client
    pub fn with_llm_client(database: Arc<TranscriptDatabase>, llm_client: LLMClientWrapper) -> Result<Self> {
        let pool = database.pool.clone();
        let query_processor = QueryProcessor::new(pool.clone(), Some(llm_client.clone()));
        let context_manager = Arc::new(ConversationContextManager::new());
        let query_optimizer = QueryOptimizer::new(pool);
        let security = QuerySecurityManager::read_only();

        Ok(Self {
            database,
            security,
            query_processor,
            context_manager,
            query_optimizer,
            llm_client: Some(llm_client),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start the MCP server with specified transport
    pub async fn start_server<T: MCPTransport>(&self, mut transport: T) -> Result<()> {
        log::info!("Starting MCP server with enhanced query processing");

        loop {
            match transport.receive().await {
                Ok(request) => {
                    let response = self.handle_request(request).await;
                    if let Err(e) = transport.send(response).await {
                        log::error!("Failed to send response: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Failed to receive request: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Start the MCP server with stdio transport (convenience method)
    pub async fn start_stdio_server(&self) -> Result<()> {
        let transport = StdioTransport::new();
        self.start_server(transport).await
    }

    /// Handle incoming MCP requests with enhanced processing
    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        let id = request.id.clone();

        // Log request for debugging
        log::debug!("Handling MCP request: {} {}", request.method, 
                   request.params.as_ref().map(|p| p.to_string()).unwrap_or_default());

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "resources/list" => self.handle_list_resources().await,
            "resources/read" => self.handle_read_resource(request.params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_tool_call(request.params).await,
            "prompts/list" => self.handle_list_prompts().await,
            "prompts/get" => self.handle_get_prompt(request.params).await,
            _ => Err(anyhow!("Method not found: {}", request.method)),
        };

        match result {
            Ok(value) => {
                match request.method.as_str() {
                    "tools/list" => {
                        let tools = value.get("tools").cloned().unwrap_or(json!([]));
                        MCPResponse::ToolsList {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: value,
                        }
                    },
                    "tools/call" => {
                        let content_text = value.get("content")
                            .and_then(|c| c.as_array())
                            .and_then(|arr| arr.get(0))
                            .and_then(|item| item.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("{}");

                        let content = vec![ContentItem {
                            r#type: "text".to_string(),
                            text: content_text.to_string(),
                        }];

                        MCPResponse::ToolResult {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: value,
                            content,
                        }
                    },
                    "resources/list" => {
                        let resources_value = value.get("resources").cloned().unwrap_or(json!([]));
                        let empty_vec = Vec::new(); // Create a longer-lived empty vector
                        let resources_array = resources_value.as_array().unwrap_or(&empty_vec);

                        let resources = resources_array.iter()
                            .map(|r| MCPResource {
                                uri: r.get("uri").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                                name: r.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                                description: r.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string(),
                                mime_type: r.get("mime_type").and_then(|m| m.as_str()).unwrap_or("").to_string(),
                                annotations: r.get("annotations").cloned(),
                            })
                            .collect();

                        MCPResponse::ResourcesList {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: value,
                            resources,
                        }
                    },
                    _ => {
                        // For other methods, use ToolResult as a fallback
                        let value_clone = value.clone(); // Clone value before moving it
                        MCPResponse::ToolResult {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: value,
                            content: vec![ContentItem {
                                r#type: "text".to_string(),
                                text: value_clone.to_string(),
                            }],
                        }
                    }
                }
            },
            Err(e) => {
                log::error!("Request failed: {}", e);
                MCPResponse::Error {
                    jsonrpc: "2.0".to_string(),
                    id,
                    error: MCPError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    },
                }
            }
        }
    }

    /// Handle initialization request
    async fn handle_initialize(&self, params: Option<Value>) -> Result<Value> {
        let client_info = params.unwrap_or(json!({}));
        let session_id = Uuid::new_v4().to_string();

        let session = MCPSession {
            id: session_id.clone(),
            client_info: Some(client_info),
            capabilities: vec![
                "resources".to_string(),
                "tools".to_string(),
                "prompts".to_string(),
            ],
            query_count: 0,
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session);

        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "resources": {
                    "subscribe": false,
                    "listChanged": false
                },
                "tools": {
                    "listChanged": false
                },
                "prompts": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "savant-ai-database",
                "version": "1.0.0"
            },
            "sessionId": session_id
        }))
    }

    /// Handle list resources request
    async fn handle_list_resources(&self) -> Result<Value> {
        let resources = vec![
            MCPResource {
                uri: "conversations://list".to_string(),
                name: "All Conversations".to_string(),
                description: "List of all conversation transcripts".to_string(),
                mime_type: "application/json".to_string(),
                annotations: None,
            },
            MCPResource {
                uri: "speakers://list".to_string(),
                name: "All Speakers".to_string(),
                description: "List of all speakers in the database".to_string(),
                mime_type: "application/json".to_string(),
                annotations: None,
            },
            MCPResource {
                uri: "schema://database".to_string(),
                name: "Database Schema".to_string(),
                description: "Database schema and table definitions".to_string(),
                mime_type: "application/json".to_string(),
                annotations: None,
            },
        ];

        Ok(json!({ "resources": resources }))
    }

    /// Handle read resource request
    async fn handle_read_resource(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow!("Missing parameters"))?;
        let uri = params.get("uri")
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow!("Missing uri parameter"))?;

        match uri {
            "conversations://list" => {
                let conversations = self.database.list_conversations(Some(50)).await
                    .map_err(|e| anyhow!("Failed to list conversations: {}", e))?;

                Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&conversations)?
                    }]
                }))
            }
            "speakers://list" => {
                let speakers = sqlx::query("SELECT DISTINCT speaker FROM segments WHERE speaker IS NOT NULL")
                    .fetch_all(&self.database.pool)
                    .await
                    .map_err(|e| anyhow!("Failed to list speakers: {}", e))?;

                let speaker_names: Vec<String> = speakers
                    .into_iter()
                    .map(|row| row.try_get::<String, _>("speaker").unwrap_or_default())
                    .filter(|s| !s.is_empty())
                    .collect();

                Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&speaker_names)?
                    }]
                }))
            }
            "schema://database" => {
                let schema = json!({
                    "conversations": {
                        "columns": ["id", "title", "start_time", "end_time", "context"],
                        "description": "Main conversation records"
                    },
                    "segments": {
                        "columns": ["id", "conversation_id", "speaker", "text", "processed_text", "timestamp", "confidence", "start_time", "end_time"],
                        "description": "Individual transcript segments"
                    }
                });

                Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&schema)?
                    }]
                }))
            }
            _ => Err(anyhow!("Unknown resource URI: {}", uri))
        }
    }
}
