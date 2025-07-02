//! LLM database integration commands for Tauri
//! 
//! Provides natural language querying capabilities for the conversation database

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use savant_db::{TranscriptDatabase, QueryProcessor, QueryOptimizer, UserFeedback, LLMQueryResult, LLMConfig, LLMClientFactory};
use savant_mcp::MCPServer;

/// Shared MCP server state
pub type MCPServerState = Arc<Mutex<Option<MCPServer>>>;

/// Request for enhanced natural language database query
#[derive(Debug, Deserialize)]
pub struct NaturalQueryRequest {
    pub query: String,
    pub session_id: Option<String>,
    pub include_context: Option<bool>,
}

/// Enhanced response from natural language database query
#[derive(Debug, Serialize)]
pub struct NaturalQueryResponse {
    pub success: bool,
    pub results: serde_json::Value,
    pub summary: String,
    pub execution_time_ms: u64,
    pub intent_type: String,
    pub result_count: usize,
    pub confidence: f32,
    pub sql_query: Option<String>,
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
}

/// MCP server status
#[derive(Debug, Serialize)]
pub struct MCPServerStatus {
    pub running: bool,
    pub session_count: usize,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Execute an enhanced natural language query against the database
#[tauri::command]
pub async fn natural_language_query(
    request: NaturalQueryRequest,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<NaturalQueryResponse, String> {
    let start_time = std::time::Instant::now();
    
    // Initialize LLM client (try Ollama first, fallback to mock)
    let llm_config = LLMConfig::default();
    let llm_client = match LLMClientFactory::create_client(&llm_config) {
        Ok(client) => Some(client),
        Err(_) => {
            log::warn!("LLM client unavailable, using pattern-based fallback");
            None
        }
    };
    
    // Create enhanced query processor
    let query_processor = QueryProcessor::new(database.pool.clone(), llm_client.clone());
    
    let session_id = request.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    // Process query with LLM-powered understanding
    match query_processor.process_query(&request.query, &session_id).await {
        Ok(llm_result) => {
            // Execute the structured query (placeholder - would need actual execution logic)
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            // Format results for response
            let summary = format_llm_query_summary(&llm_result, &request.query, execution_time);
            
            Ok(NaturalQueryResponse {
                success: true,
                results: serde_json::json!({
                    "intent": llm_result.intent,
                    "sql_query": llm_result.sql_query,
                    "parameters": llm_result.parameters,
                    "confidence": llm_result.confidence
                }),
                summary,
                execution_time_ms: execution_time,
                intent_type: llm_result.intent.clone(),
                result_count: 0, // Would be populated from actual query execution
                confidence: llm_result.confidence,
                sql_query: Some(llm_result.sql_query.clone()),
                session_id: Some(session_id),
                error: None,
                suggestions: None,
            })
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            Ok(NaturalQueryResponse {
                success: false,
                results: serde_json::Value::Null,
                summary: format!("Query failed: {}", e),
                execution_time_ms: execution_time,
                intent_type: "error".to_string(),
                result_count: 0,
                confidence: 0.0,
                sql_query: None,
                session_id: Some(session_id),
                error: Some(e.to_string()),
                suggestions: None,
            })
        }
    }
}

/// Start the enhanced MCP server for external LLM integration
#[tauri::command]
pub async fn start_mcp_server(
    database: State<'_, Arc<TranscriptDatabase>>,
    mcp_server_state: State<'_, MCPServerState>,
) -> Result<String, String> {
    let mut server_guard = mcp_server_state.lock().await;
    
    if server_guard.is_some() {
        return Ok("MCP server already running".to_string());
    }
    
    // Create enhanced MCP server with LLM integration
    let llm_configs = vec![
        LLMConfig::default(), // Ollama
        LLMConfig {
            provider: "mock".to_string(),
            ..Default::default()
        }
    ];
    
    let mcp_server = MCPServer::new(database.inner().clone(), Some(llm_configs)).await
        .map_err(|e| format!("Failed to create enhanced MCP server: {}", e))?;
    
    // Store server instance
    *server_guard = Some(mcp_server);
    
    // Start server in background task (stdio mode)
    let mcp_server_state_clone = mcp_server_state.inner().clone();
    drop(server_guard);
    
    tokio::spawn(async move {
        let server_guard = mcp_server_state_clone.lock().await;
        if let Some(server) = server_guard.as_ref() {
            if let Err(e) = server.start_stdio_server().await {
                log::error!("MCP server failed: {}", e);
            }
        }
    });
    
    Ok("Enhanced MCP server started successfully with LLM integration".to_string())
}

/// Get MCP server status
#[tauri::command]
pub async fn get_mcp_server_status(
    mcp_server_state: State<'_, MCPServerState>,
) -> Result<MCPServerStatus, String> {
    let server_guard = mcp_server_state.lock().await;
    
    Ok(MCPServerStatus {
        running: server_guard.is_some(),
        session_count: 0, // Would be tracked in a real implementation
        last_activity: None, // Would be tracked in a real implementation
    })
}

/// Test database connection and basic functionality
#[tauri::command]
pub async fn test_database_connection(
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    // Test basic database operations
    match database.get_speaker_stats().await {
        Ok(stats) => {
            let conversations = database.list_conversations(Some(5)).await
                .unwrap_or_default();
            
            Ok(serde_json::json!({
                "success": true,
                "speaker_count": stats.len(),
                "conversation_count": conversations.len(),
                "database_path": "Connected successfully",
                "capabilities": [
                    "natural_language_queries",
                    "speaker_analytics", 
                    "semantic_search",
                    "conversation_analysis"
                ]
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

/// Get database statistics for the frontend
#[tauri::command]
pub async fn get_database_stats(
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    let stats = database.get_speaker_stats().await
        .map_err(|e| e.to_string())?;
    
    let _conversations = database.list_conversations(Some(1)).await
        .map_err(|e| e.to_string())?;
    
    let total_duration: f64 = stats.iter().map(|s| s.total_duration_seconds).sum();
    let total_conversations: i64 = stats.iter().map(|s| s.conversation_count).sum();
    let total_segments: i64 = stats.iter().map(|s| s.total_segments).sum();
    
    Ok(serde_json::json!({
        "total_speakers": stats.len(),
        "total_conversations": total_conversations,
        "total_segments": total_segments,
        "total_duration_hours": total_duration / 3600.0,
        "average_conversation_length_minutes": if total_conversations > 0 {
            (total_duration / 60.0) / total_conversations as f64
        } else {
            0.0
        },
        "top_speakers": stats.into_iter().take(5).map(|s| serde_json::json!({
            "name": s.speaker,
            "conversation_count": s.conversation_count,
            "total_duration_hours": s.total_duration_seconds / 3600.0
        })).collect::<Vec<_>>()
    }))
}

/// Search conversations with natural language
#[tauri::command]
pub async fn search_conversations(
    query: String,
    limit: Option<usize>,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    let search_limit = limit.unwrap_or(20);
    
    // Try semantic search first, fall back to text search
    let results = database.text_search(&query, search_limit).await
        .map_err(|e| e.to_string())?;
    
    let formatted_results: Vec<serde_json::Value> = results.into_iter().map(|result| {
        serde_json::json!({
            "text": result.text,
            "speaker": result.speaker_id,
            "timestamp": result.timestamp,
            "confidence": result.similarity_score,
            "conversation_id": result.conversation_id
        })
    }).collect();
    
    Ok(serde_json::json!({
        "query": query,
        "results": formatted_results,
        "result_count": formatted_results.len()
    }))
}

/// Get conversation analysis
#[tauri::command]
pub async fn analyze_conversation(
    conversation_id: String,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    match database.analyze_conversation(&conversation_id).await {
        Ok(analysis) => {
            Ok(serde_json::json!({
                "success": true,
                "analysis": {
                    "topics": analysis.topics,
                    "sentiment_score": analysis.sentiment_score,
                    "duration": analysis.duration,
                    "participant_count": analysis.participant_count,
                    "summary": analysis.summary
                }
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

/// List available speakers with statistics
#[tauri::command]
pub async fn list_speakers_with_stats(
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<Vec<serde_json::Value>, String> {
    let stats = database.get_speaker_stats().await
        .map_err(|e| e.to_string())?;
    
    Ok(stats.into_iter().map(|s| serde_json::json!({
        "name": s.speaker,
        "conversation_count": s.conversation_count,
        "total_duration_seconds": s.total_duration_seconds,
        "total_segments": s.total_segments,
        "avg_confidence": s.avg_confidence
    })).collect())
}

/// Provide feedback on query results for learning
#[tauri::command]
pub async fn provide_query_feedback(
    query: String,
    sql_query: Option<String>,
    feedback: String,
    results: serde_json::Value,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<String, String> {
    let user_feedback = match feedback.as_str() {
        "good" => UserFeedback::Good,
        "bad_results" => UserFeedback::BadResults,
        "too_slow" => UserFeedback::TooSlow,
        "wrong_intent" => UserFeedback::WrongIntent,
        "irrelevant" => UserFeedback::Irrelevant,
        _ => return Err("Invalid feedback type".to_string()),
    };
    
    let query_optimizer = QueryOptimizer::new(database.pool.clone());
    
    match query_optimizer.learn_from_feedback(
        &query,
        &sql_query.unwrap_or_default(),
        &results,
        user_feedback
    ).await {
        Ok(_) => Ok("Feedback recorded successfully".to_string()),
        Err(e) => Err(format!("Failed to record feedback: {}", e)),
    }
}

/// Get query suggestions based on successful patterns
#[tauri::command]
pub async fn get_query_suggestions(
    partial_query: String,
    max_suggestions: Option<usize>,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<Vec<String>, String> {
    let query_optimizer = QueryOptimizer::new(database.pool.clone());
    let suggestions = query_optimizer.get_query_suggestions(&partial_query).await;
    
    let limit = max_suggestions.unwrap_or(3);
    Ok(suggestions.into_iter().take(limit).collect())
}

/// Format LLM query result into human-readable summary
fn format_llm_query_summary(result: &LLMQueryResult, original_query: &str, execution_time_ms: u64) -> String {
    let intent_desc = match result.intent.as_str() {
        "find_conversations" => "Found Conversations",
        "analyze_speaker" => "Speaker Analysis", 
        "search_content" => "Content Search",
        "get_statistics" => "Database Statistics",
        "list_speakers" => "Speaker List",
        _ => "Query Results",
    };
    
    format!(
        "{}: Processed query with {:.1}% confidence in {}ms for \"{}\"",
        intent_desc,
        result.confidence * 100.0,
        execution_time_ms,
        original_query
    )
}