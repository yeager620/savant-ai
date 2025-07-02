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

use crate::llm_client::{LLMClient, LLMClientFactory, LLMConfig};
use crate::natural_query::{QueryProcessor, ConversationContextManager, QueryOptimizer, LLMQueryResult, LLMClientWrapper};
use crate::security::QuerySecurityManager;
use crate::TranscriptDatabase;

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
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
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
            "tools/call" => self.handle_call_tool(request.params).await,
            "prompts/list" => self.handle_list_prompts().await,
            "prompts/get" => self.handle_get_prompt(request.params).await,
            _ => Err(anyhow!("Method not found: {}", request.method)),
        };
        
        match result {
            Ok(value) => MCPResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(value),
                error: None,
            },
            Err(e) => {
                log::error!("Request failed: {}", e);
                MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    }),
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
        let uri = params
            .and_then(|p| p.get("uri"))
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow!("Missing uri parameter"))?;
        
        match uri {
            "conversations://list" => {
                let conversations = self.database.list_conversations(50).await
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
    
    /// Handle list tools request
    async fn handle_list_tools(&self) -> Result<Value> {
        // Import tools from mcp_server_tools module
        Ok(json!({
            "tools": [
                {
                    "name": "query_conversations",
                    "description": "Query conversations using natural language",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Natural language query to search conversations"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Optional session ID for context"
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "get_speaker_analytics",
                    "description": "Get analytics for speakers",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "speaker": {
                                "type": "string",
                                "description": "Optional specific speaker to analyze"
                            }
                        }
                    }
                },
                {
                    "name": "search_semantic",
                    "description": "Perform semantic search on conversation content",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results",
                                "default": 20
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "get_database_stats",
                    "description": "Get database statistics and performance metrics",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "include_performance": {
                                "type": "boolean",
                                "description": "Include performance metrics",
                                "default": false
                            }
                        }
                    }
                }
            ]
        }))
    }
    
    /// Handle initialize request
    async fn handle_initialize(&self, params: Option<Value>) -> Result<Value> {
        let session_id = Uuid::new_v4().to_string();
        
        let client_info = params.and_then(|p| p.get("clientInfo").cloned());
        
        let session = MCPSession {
            id: session_id.clone(),
            client_info,
            capabilities: vec![
                "resources".to_string(),
                "tools".to_string(),
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
                }
            },
            "serverInfo": {
                "name": "savant-ai-database",
                "version": "1.0.0",
                "description": "Savant AI conversation database MCP server"
            },
            "sessionId": session_id
        }))
    }
    
    /// Handle list resources request
    async fn handle_list_resources(&self) -> Result<Value> {
        let conversations = self.database.list_conversations(Some(10)).await?;
        let speakers = self.database.list_speakers().await?;
        
        let mut resources = Vec::new();
        
        // Add conversation resources
        for conversation in conversations {
            resources.push(MCPResource {
                uri: format!("conversation://{}", conversation.id),
                name: format!("Conversation: {}", 
                    conversation.title.unwrap_or_else(|| "Untitled".to_string())),
                description: format!("Conversation with {} participants ({})", 
                    conversation.participants.len(), 
                    conversation.participants.join(", ")),
                mime_type: "application/json".to_string(),
                annotations: Some(json!({
                    "participants": conversation.participants,
                    "start_time": conversation.start_time,
                    "segment_count": conversation.segment_count
                })),
            });
        }
        
        // Add speaker resources
        for speaker in speakers {
            resources.push(MCPResource {
                uri: format!("speaker://{}", speaker.id),
                name: format!("Speaker: {}", speaker.name.unwrap_or_else(|| "Unknown".to_string())),
                description: format!("Speaker profile with conversation history"),
                mime_type: "application/json".to_string(),
                annotations: Some(json!({
                    "conversation_count": speaker.conversation_count,
                    "total_duration": speaker.total_conversation_time
                })),
            });
        }
        
        // Add database overview resource
        resources.push(MCPResource {
            uri: "database://overview".to_string(),
            name: "Database Overview".to_string(),
            description: "Overall statistics and information about the conversation database".to_string(),
            mime_type: "application/json".to_string(),
            annotations: None,
        });
        
        Ok(json!({ "resources": resources }))
    }
    
    /// Handle read resource request
    async fn handle_read_resource(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow!("Missing params"))?;
        let uri = params.get("uri")
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow!("Missing URI"))?;
        
        if uri.starts_with("conversation://") {
            let conversation_id = uri.strip_prefix("conversation://").unwrap();
            let conversation_data = self.database.export_conversation(conversation_id).await?;
            
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&conversation_data)?
                }]
            }))
        } else if uri.starts_with("speaker://") {
            let speaker_id = uri.strip_prefix("speaker://").unwrap();
            let speakers = self.database.list_speakers().await?;
            let speaker = speakers.into_iter()
                .find(|s| s.id == speaker_id)
                .ok_or_else(|| anyhow!("Speaker not found"))?;
            
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&speaker)?
                }]
            }))
        } else if uri == "database://overview" {
            let stats = self.database.get_speaker_stats().await?;
            let conversations = self.database.list_conversations(Some(1)).await?;
            
            let overview = json!({
                "total_speakers": stats.len(),
                "total_conversations": conversations.len(),
                "speaker_statistics": stats,
                "recent_conversations": conversations
            });
            
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&overview)?
                }]
            }))
        } else {
            Err(anyhow!("Unknown resource URI: {}", uri))
        }
    }
    
    /// Handle list tools request
    async fn handle_list_tools(&self) -> Result<Value> {
        let tools = vec![
            MCPTool {
                name: "query_conversations".to_string(),
                description: "Query conversations using natural language".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query about conversations"
                        }
                    },
                    "required": ["query"]
                }),
            },
            MCPTool {
                name: "get_speaker_analytics".to_string(),
                description: "Get detailed analytics for a specific speaker".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "speaker": {
                            "type": "string",
                            "description": "Speaker name or ID to analyze"
                        }
                    },
                    "required": ["speaker"]
                }),
            },
            MCPTool {
                name: "search_semantic".to_string(),
                description: "Perform semantic search across conversation content".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "term": {
                            "type": "string",
                            "description": "Search term or phrase"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results",
                            "default": 10
                        }
                    },
                    "required": ["term"]
                }),
            },
            MCPTool {
                name: "get_database_stats".to_string(),
                description: "Get overall database statistics and summaries".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "analyze_conversation".to_string(),
                description: "Analyze a specific conversation for insights".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "conversation_id": {
                            "type": "string",
                            "description": "ID of conversation to analyze"
                        }
                    },
                    "required": ["conversation_id"]
                }),
            },
        ];
        
        Ok(json!({ "tools": tools }))
    }
    
    /// Handle tool call request
    async fn handle_call_tool(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow!("Missing params"))?;
        let tool_name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow!("Missing tool name"))?;
        let arguments = params.get("arguments")
            .ok_or_else(|| anyhow!("Missing arguments"))?;
        
        match tool_name {
            "query_conversations" => self.handle_query_conversations(arguments).await,
            "get_speaker_analytics" => self.handle_speaker_analytics(arguments).await,
            "search_semantic" => self.handle_semantic_search(arguments).await,
            "get_database_stats" => self.handle_database_stats().await,
            "analyze_conversation" => self.handle_analyze_conversation(arguments).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name)),
        }
    }
    
    /// Handle natural language query tool
    async fn handle_query_conversations(&self, arguments: &Value) -> Result<Value> {
        let query = arguments.get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| anyhow!("Missing query parameter"))?;
        
        // Validate natural language query
        let sanitized_query = self.security.validate_natural_query(query)
            .map_err(|e| anyhow!("Query validation failed: {}", e))?;
        
        // Execute natural language query
        let result = self.query_parser.execute_natural_query(&sanitized_query).await?;
        
        Ok(json!({
            "content": [{
                "type": "text",
                "text": self.format_query_response(&result)?
            }],
            "isError": false
        }))
    }
    
    /// Handle speaker analytics tool
    async fn handle_speaker_analytics(&self, arguments: &Value) -> Result<Value> {
        let speaker = arguments.get("speaker")
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow!("Missing speaker parameter"))?;
        
        let stats = self.database.get_speaker_stats().await?;
        let speaker_stats = stats.into_iter()
            .find(|s| s.speaker.to_lowercase() == speaker.to_lowercase())
            .ok_or_else(|| anyhow!("Speaker not found: {}", speaker))?;
        
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Speaker Analytics for '{}'\n\n\
                     • Total Conversations: {}\n\
                     • Total Duration: {:.1} minutes\n\
                     • Total Segments: {}\n\
                     • Average Confidence: {:.2}%",
                    speaker_stats.speaker,
                    speaker_stats.conversation_count,
                    speaker_stats.total_duration_seconds / 60.0,
                    speaker_stats.total_segments,
                    speaker_stats.avg_confidence * 100.0
                )
            }],
            "isError": false
        }))
    }
    
    /// Handle semantic search tool
    async fn handle_semantic_search(&self, arguments: &Value) -> Result<Value> {
        let term = arguments.get("term")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Missing term parameter"))?;
        
        let limit = arguments.get("limit")
            .and_then(|l| l.as_u64())
            .unwrap_or(10) as usize;
        
        let results = self.database.text_search(term, limit).await?;
        
        let formatted_results = results.into_iter()
            .map(|result| format!(
                "• {} ({}): {}",
                result.speaker.unwrap_or_else(|| "Unknown".to_string()),
                result.timestamp.format("%Y-%m-%d %H:%M"),
                result.text.chars().take(100).collect::<String>()
            ))
            .collect::<Vec<_>>()
            .join("\n");
        
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Search Results for '{}'\n\n{}", term, formatted_results)
            }],
            "isError": false
        }))
    }
    
    /// Handle database statistics tool
    async fn handle_database_stats(&self) -> Result<Value> {
        let stats = self.database.get_speaker_stats().await?;
        let conversations = self.database.list_conversations(Some(1)).await?;
        
        let total_duration: f64 = stats.iter().map(|s| s.total_duration_seconds).sum();
        let total_conversations: i64 = stats.iter().map(|s| s.conversation_count).sum();
        let total_segments: i64 = stats.iter().map(|s| s.total_segments).sum();
        
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Database Statistics\n\n\
                     • Total Speakers: {}\n\
                     • Total Conversations: {}\n\
                     • Total Segments: {}\n\
                     • Total Duration: {:.1} hours\n\
                     • Average Conversation Length: {:.1} minutes",
                    stats.len(),
                    total_conversations,
                    total_segments,
                    total_duration / 3600.0,
                    if total_conversations > 0 { 
                        (total_duration / 60.0) / total_conversations as f64 
                    } else { 
                        0.0 
                    }
                )
            }],
            "isError": false
        }))
    }
    
    /// Handle conversation analysis tool
    async fn handle_analyze_conversation(&self, arguments: &Value) -> Result<Value> {
        let conversation_id = arguments.get("conversation_id")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow!("Missing conversation_id parameter"))?;
        
        let analysis = self.database.analyze_conversation(conversation_id).await?;
        
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Conversation Analysis\n\n\
                     • Topics: {}\n\
                     • Sentiment: {:.2}\n\
                     • Duration: {:.1} minutes\n\
                     • Participants: {}\n\
                     • Summary: {}",
                    analysis.topics.join(", "),
                    analysis.sentiment_score,
                    analysis.total_duration / 60.0,
                    analysis.participants.join(", "),
                    analysis.summary.unwrap_or_else(|| "No summary available".to_string())
                )
            }],
            "isError": false
        }))
    }
    
    /// Format query response for human-readable output
    fn format_query_response(&self, result: &LLMQueryResult) -> Result<String> {
        let intent_desc = match result.intent.as_str() {
            "find_conversations" => "Found Conversations",
            "analyze_speaker" => "Speaker Analysis",
            "search_content" => "Content Search Results",
            "get_statistics" => "Database Statistics",
            _ => "Query Results",
        };
        
        let mut response = format!(
            "{}\n(SQL: \"{}\" | Confidence: {:.1}%)\n\n",
            intent_desc,
            result.sql_query,
            result.confidence * 100.0
        );
        
        // Show SQL query parameters if any
        if !result.parameters.is_empty() {
            response.push_str("Parameters:\n");
            for (key, value) in &result.parameters {
                response.push_str(&format!("  {}: {}\n", key, value));
            }
            response.push('\n');
        }
        
        Ok(response)
    }
    
    /// Handle list prompts request
    async fn handle_list_prompts(&self) -> Result<Value> {
        Ok(json!({
            "prompts": [
                {
                    "name": "conversation_summary",
                    "description": "Generate a summary of conversation data",
                    "arguments": [
                        {
                            "name": "conversation_id",
                            "description": "ID of conversation to summarize",
                            "required": true
                        }
                    ]
                },
                {
                    "name": "speaker_analysis",
                    "description": "Analyze speaker patterns and interactions",
                    "arguments": [
                        {
                            "name": "speaker_name",
                            "description": "Name of speaker to analyze",
                            "required": false
                        }
                    ]
                }
            ]
        }))
    }
    
    /// Handle get prompt request
    async fn handle_get_prompt(&self, params: Option<Value>) -> Result<Value> {
        let name = params
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow!("Prompt name is required"))?;
            
        match name {
            "conversation_summary" => {
                Ok(json!({
                    "description": "Generate a detailed summary of conversation data",
                    "prompt": "Based on the conversation data provided, generate a comprehensive summary including key topics discussed, main participants, sentiment analysis, and notable insights. Focus on actionable information and key takeaways."
                }))
            },
            "speaker_analysis" => {
                Ok(json!({
                    "description": "Analyze speaker patterns and communication style", 
                    "prompt": "Analyze the communication patterns of the specified speaker, including their speaking frequency, topics of interest, interaction patterns with other participants, and overall contribution to conversations. Provide insights into their communication style and engagement level."
                }))
            },
            _ => Err(anyhow!("Unknown prompt: {}", name))
        }
    }
}

impl MCPResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }
    
    pub fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(MCPError {
                code,
                message,
                data: None,
            }),
        }
    }
}